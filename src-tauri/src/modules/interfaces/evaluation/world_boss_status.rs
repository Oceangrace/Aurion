use crate::integrations::aion2::WorldBossObservation;
use serde::Serialize;
use serde::Serializer;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum WorldBossStatus {
    Waiting,
    Countdown,
    ObservedAlive,
    LastSeenAlive,
    Expected,
    Refresh,
}

impl WorldBossStatus {
    pub(super) fn resolve(observation: &WorldBossObservation<'_>) -> Self {
        if observation.observed_alive() == Some(true) {
            return if observation.fresh_alive() == Some(true) {
                Self::ObservedAlive
            } else {
                Self::LastSeenAlive
            };
        }

        if observation.remaining().is_some() {
            return Self::Countdown;
        }

        if observation.expected() {
            return Self::Expected;
        }

        if observation.known() {
            return Self::Refresh;
        }

        Self::Waiting
    }

    /// Status shape of 0.1.4 kept for `row.status`: a boss that was seen alive or whose
    /// respawn time has passed is alive.
    pub(super) fn compatible(self) -> Self {
        match self {
            Self::LastSeenAlive | Self::Expected => Self::ObservedAlive,
            status => status,
        }
    }

    pub(super) fn rank(self) -> u8 {
        match self {
            Self::ObservedAlive | Self::LastSeenAlive => 0,
            Self::Countdown => 1,
            Self::Expected | Self::Refresh => 2,
            Self::Waiting => 3,
        }
    }

    pub(super) const ALL: [Self; 6] = [
        Self::Waiting,
        Self::Countdown,
        Self::ObservedAlive,
        Self::LastSeenAlive,
        Self::Expected,
        Self::Refresh,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Countdown => "countdown",
            Self::ObservedAlive => "observed-alive",
            Self::LastSeenAlive => "last-seen-alive",
            Self::Expected => "expected",
            Self::Refresh => "refresh",
        }
    }
}

impl Serialize for WorldBossStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
