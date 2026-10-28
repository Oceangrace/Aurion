use crate::core::game::ComboAvailability;
use crate::core::game::CooldownSnapshot;
use crate::core::game::GameSnapshot;
use crate::integrations::aion2::catalog::CatalogScope;
use crate::integrations::aion2::catalog::CatalogStore;
use crate::modules::interfaces::rules::CatalogSelection;

#[derive(Clone)]
pub(super) struct SelectionResolver<'a> {
    catalog: &'a CatalogStore,
}

impl<'a> SelectionResolver<'a> {
    pub(super) fn npc_name(&self, id: u32, locale: &str) -> Option<String> {
        self.catalog.npc_name(id, locale)
    }

    pub(super) fn new(catalog: &'a CatalogStore) -> Self {
        Self { catalog }
    }

    /// IDs of the entry in the current catalog; `None` when it is no longer there.
    pub(super) fn ids(&self, selection: &CatalogSelection) -> Option<Vec<u32>> {
        self.catalog
            .find(
                &selection.scope,
                selection.context_key.as_deref(),
                &selection.entry_key,
            )
            .filter(|item| {
                item.forms
                    .iter()
                    .any(|form| form.key == selection.choice_key && !form.match_ids.is_empty())
            })
            .map(|item| item.match_ids())
    }

    pub(super) fn matches_skill(&self, ids: &[u32], skill_id: u32) -> bool {
        if ids.contains(&skill_id) {
            return true;
        }

        self.catalog
            .runtime()
            .skill(skill_id)
            .and_then(|skill| skill.charge_skill())
            .is_some_and(|base_id| ids.contains(&base_id))
    }

    pub(super) fn actor_ids(&self, key: &str) -> Option<Vec<u32>> {
        Some(
            self.catalog
                .find(&CatalogScope::Enemies, None, key)?
                .forms
                .iter()
                .flat_map(|form| form.match_ids.clone())
                .collect(),
        )
    }

    pub(super) fn icon(&self, effect: Option<u32>, skill: Option<u32>) -> Option<&str> {
        effect
            .and_then(|id| self.catalog.effect_icon(id))
            .or_else(|| skill.and_then(|id| self.catalog.skill_icon(id)))
    }

    pub(super) fn cooldown<'g>(
        &self,
        ids: &[u32],
        game: &'g GameSnapshot,
    ) -> Option<&'g CooldownSnapshot> {
        game.cooldowns()
            .iter()
            .filter(|cooldown| {
                ids.iter().any(|id| {
                    self.catalog
                        .runtime()
                        .skill(*id)
                        .and_then(|skill| skill.cooldown_group())
                        == Some(cooldown.group_id().get())
                })
            })
            .max_by_key(|cooldown| cooldown.remaining_ms())
    }

    pub(super) fn combo<'g>(
        &self,
        ids: &[u32],
        game: &'g GameSnapshot,
    ) -> Option<&'g ComboAvailability> {
        game.combos()
            .iter()
            .filter(|combo| combo.available() && self.in_chain(combo.skill_id().get(), ids))
            .max_by_key(|combo| combo.at_us())
    }

    fn in_chain(&self, mut id: u32, ids: &[u32]) -> bool {
        for _ in 0..16 {
            if ids.contains(&id) {
                return true;
            }

            let Some(previous) = self
                .catalog
                .runtime()
                .skill(id)
                .and_then(|skill| skill.chain_previous())
            else {
                return false;
            };

            id = previous;
        }

        false
    }

    pub(super) fn combo_available(&self, ids: &[u32], game: &GameSnapshot) -> Option<bool> {
        let mut observed = game
            .combos()
            .iter()
            .filter(|combo| self.in_chain(combo.skill_id().get(), ids));
        let first = observed.next()?;

        Some(first.available() || observed.any(|combo| combo.available()))
    }

    pub(super) fn shock_condition(&self, ids: &[u32], game: &GameSnapshot) -> Option<bool> {
        if !ids.iter().any(|id| {
            self.catalog
                .runtime()
                .skill(*id)
                .is_some_and(|skill| skill.requires_shock())
        }) {
            return None;
        }

        let player = game
            .player_id()
            .and_then(|id| game.entities().get(&id))
            .filter(|unit| unit.present())?;
        let active = player.effects().items().iter().any(|effect| {
            self.catalog
                .runtime()
                .effect(effect.effect().effect_id().get())
                .is_some_and(|metadata| metadata.enables_shock_removal())
        });

        (active || player.effects().complete()).then_some(active)
    }
}
