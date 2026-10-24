use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StaminaSnapshot {
    current: Option<u32>,
    maximum: Option<u32>,
}

impl StaminaSnapshot {
    pub(crate) fn new(current: Option<u32>, maximum: Option<u32>) -> Self {
        Self { current, maximum }
    }

    pub(crate) fn current(&self) -> Option<u32> {
        self.current
    }

    pub(crate) fn maximum(&self) -> Option<u32> {
        self.maximum
    }

    pub(crate) fn observe(&mut self, stamina: &Self) {
        self.current = stamina.current.or(self.current);
        self.maximum = stamina.maximum.or(self.maximum);
    }
}
