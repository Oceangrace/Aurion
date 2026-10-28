use super::trigger_state::TriggerState;
use crate::core::game::GameSnapshot;
use crate::core::game::SourceStatus;
use crate::modules::interfaces::evaluation::PreviewVisual;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveFrame {
    status: SourceStatus,
    in_world: bool,
    character_data_complete: bool,
    /// Snapshot time; for a live capture it is Unix time, so the client can subtract frame age.
    at_us: Option<u64>,
    error: Option<String>,
    visuals: HashMap<String, PreviewVisual>,
    states: HashMap<String, TriggerState>,
    document_errors: BTreeMap<String, String>,
}

impl LiveFrame {
    pub(super) fn new(status: SourceStatus, error: Option<String>) -> Self {
        Self {
            status,
            in_world: false,
            character_data_complete: false,
            at_us: None,
            error,
            visuals: HashMap::new(),
            states: HashMap::new(),
            document_errors: BTreeMap::new(),
        }
    }

    pub(super) fn absorb(&mut self, name: &str, part: Result<Self, String>) {
        match part {
            Ok(part) => {
                self.visuals.extend(part.visuals);
                self.states.extend(part.states);
            }
            Err(error) => {
                self.document_errors.insert(name.to_owned(), error);
            }
        }
    }

    pub(crate) fn states(&self) -> &HashMap<String, TriggerState> {
        &self.states
    }

    pub(super) fn observe_world(&mut self, game: &GameSnapshot) {
        self.at_us = Some(game.header().at_us());
        self.in_world = game.player_in_world();
        self.character_data_complete = game.player_data_complete();
    }

    pub(crate) fn visuals(&self) -> &HashMap<String, PreviewVisual> {
        &self.visuals
    }

    pub(super) fn insert(
        &mut self,
        id: String,
        state: TriggerState,
        visuals: HashMap<String, PreviewVisual>,
    ) {
        self.states.insert(id, state);
        self.visuals.extend(visuals);
    }
}
