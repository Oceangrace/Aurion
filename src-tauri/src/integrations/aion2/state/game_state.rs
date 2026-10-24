use super::combat_session::CombatSession;
use super::entity_tracker::EntityTracker;
use super::event_journal::EventJournal;
use super::npc_recovery::NpcRecovery;
use super::party_tracker::PartyTracker;
use super::player_recovery::PlayerRecovery;
use super::server_clock::ServerClock;
use super::skill_tracker::SkillTracker;
use super::world_boss_cache::WorldBossCache;
use super::world_tracker::WorldTracker;
use crate::core::game::GameCapabilities;
use crate::core::game::GameEvent;
use crate::core::game::GameSnapshot;
use crate::core::game::MapId;
use crate::core::game::SnapshotHeader;
use crate::core::game::SourceStatus;
use crate::core::game::WorldBossList;
use crate::integrations::aion2::StreamReset;
use crate::integrations::aion2::catalog::RuntimeCatalog;
use std::sync::Arc;

pub(crate) struct GameState {
    catalog: Arc<RuntimeCatalog>,
    observation_id: String,
    capabilities: GameCapabilities,
    revision: u64,
    clock: ServerClock,
    entities: EntityTracker,
    skills: SkillTracker,
    combat: CombatSession,
    party: PartyTracker,
    world: WorldTracker,
    journal: EventJournal,
    interrupted: bool,
    map_id: Option<MapId>,
}

impl GameState {
    pub(crate) fn new(
        observation_id: String,
        capabilities: GameCapabilities,
        catalog: Arc<RuntimeCatalog>,
        combat_idle_ms: u64,
    ) -> Self {
        Self {
            observation_id,
            catalog,
            capabilities,
            revision: 0,
            clock: ServerClock::new(),
            entities: EntityTracker::new(),
            skills: SkillTracker::default(),
            combat: CombatSession::new(combat_idle_ms),
            party: PartyTracker::default(),
            world: WorldTracker::default(),
            journal: EventJournal::default(),
            interrupted: false,
            map_id: None,
        }
    }

    pub(crate) fn reset(&mut self, observation_id: String) {
        self.reset_session(observation_id, false);
    }

    fn reset_session(&mut self, observation_id: String, interrupted: bool) {
        self.observation_id = observation_id;
        self.revision = 0;
        self.clock = ServerClock::new();
        self.entities.reset(interrupted);
        self.skills.reset(interrupted);
        self.combat.reset();
        self.party = PartyTracker::default();
        self.world.reset();
        self.journal = EventJournal::default();
        self.interrupted = interrupted;
        self.map_id = None;
    }

    pub(crate) fn boss_cache(&self) -> &WorldBossCache {
        self.world.boss_cache()
    }

    pub(crate) fn set_boss_cache(&mut self, cache: WorldBossCache) {
        self.world.replace_boss_cache(cache);
    }

    pub(crate) fn restore_player(&mut self, recovery: &PlayerRecovery) -> Result<(), String> {
        self.apply(&recovery.identity(), 0)?;
        self.skills.restore_book(recovery.skills().clone());
        self.entities.restore_player(recovery)
    }

    pub(crate) fn restore_bosses(&mut self, lists: &[WorldBossList]) {
        self.world.restore_bosses(lists, self.entities.server());
    }

    pub(crate) fn restore_npcs(
        &mut self,
        map_id: Option<MapId>,
        npcs: &[NpcRecovery],
        now_us: u64,
    ) -> Result<(), String> {
        if self.map_id.is_some() && self.map_id != map_id {
            return Ok(());
        }

        self.map_id = self.map_id.or(map_id);
        self.entities.restore_npcs(npcs, now_us)
    }

    pub(crate) fn has_player(&self) -> bool {
        self.entities.has_player()
    }

    pub(crate) fn recover(&mut self, reset: &StreamReset) {
        self.reset_session(
            uuid::Uuid::new_v4().to_string(),
            matches!(reset, StreamReset::Interrupted(_)),
        );
    }

    pub(crate) fn apply(&mut self, event: &GameEvent, at_us: u64) -> Result<(), String> {
        if self.entities.identity_changed(event) {
            self.reset(uuid::Uuid::new_v4().to_string());
        }

        if matches!(event, GameEvent::SelfRestoration { .. }) {
            if let Some(healing) = self.entities.restoration(event) {
                self.apply(&healing, at_us)?;
            }

            return Ok(());
        }

        let observed = self.journal.prepare(event, &mut self.entities)?;

        self.entities.observe(event, at_us, &self.catalog)?;
        self.world.observe(event, self.entities.server());
        self.skills
            .observe(event, at_us, self.entities.player(), &self.catalog)?;

        match event {
            GameEvent::Party { roster } => self.party.observe(roster.clone()),
            GameEvent::Clock { server_ms } => self.clock.sync(*server_ms, at_us),
            GameEvent::MapChanged { map_id } => {
                self.map_id = Some(*map_id);
                self.journal.clear();
                self.combat.reset();
                self.observation_id = uuid::Uuid::new_v4().to_string();
                self.interrupted = false;
            }
            _ => {}
        }

        for record in self
            .combat
            .observe(event, at_us, &self.entities, &self.party, &self.catalog)
        {
            self.journal.append(record.event().clone(), record.at_us());
        }

        if let Some(observed) = observed {
            self.journal.append(observed, at_us);
        }

        Ok(())
    }

    pub(crate) fn snapshot(&mut self, now_us: u64, status: SourceStatus) -> GameSnapshot {
        let entities = self.entities.snapshot(&self.clock, now_us, &self.catalog);

        let player = self.entities.player();

        for record in self.combat.tick(now_us, player) {
            self.journal.append(record.event().clone(), record.at_us());
        }

        self.revision += 1;

        let header =
            SnapshotHeader::new(self.observation_id.clone(), self.revision, now_us, status)
                .with_interrupted(self.interrupted);

        let party = self.party.snapshot(&entities);

        GameSnapshot::new(
            header,
            player,
            self.entities.target(),
            self.map_id,
            entities,
            self.journal.snapshot(now_us),
            self.capabilities.clone(),
        )
        .with_world(self.world.snapshot(now_us))
        .with_party(party)
        .with_skills(
            self.skills.cooldowns(now_us),
            self.skills.combos(now_us, &self.catalog),
            self.skills.book().clone(),
        )
        .with_combat(self.combat.snapshot(now_us, player))
        .with_statistics(self.combat.statistics())
    }
}
