use crate::core::game::EntityId;

pub(super) struct SelfTracker {
    id: Option<EntityId>,
    hint: Option<EntityId>,
    hint_count: u8,
    server: Option<u16>,
}

impl SelfTracker {
    pub(super) fn new() -> Self {
        Self {
            id: None,
            hint: None,
            hint_count: 0,
            server: None,
        }
    }

    pub(super) fn id(&self) -> Option<EntityId> {
        self.id
    }

    /// Server of the character for this session. It outlives the own unit, which is rebuilt on
    /// travel, recovered spawns and interrupted streams.
    pub(super) fn server(&self) -> Option<u16> {
        self.server
    }

    pub(super) fn locate(&mut self, server: u16) {
        self.server = Some(server);
    }

    pub(super) fn identify(&mut self, id: EntityId, authoritative: bool) {
        if authoritative {
            self.id = Some(id);

            return;
        }

        if self.id.is_some() {
            return;
        }

        self.hint_count = if self.hint == Some(id) {
            self.hint_count.saturating_add(1)
        } else {
            1
        };
        self.hint = Some(id);

        if self.hint_count >= 3 {
            self.id = Some(id);
        }
    }
}
