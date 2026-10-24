//! Mutable source state; snapshots expose observations without mutable fields.

mod combat_accumulator;
mod combat_ledger;
mod combat_session;
mod combat_tracker;
mod cooldown_state;
mod effect_state;
mod entity_tracker;
mod event_journal;
mod game_state;
mod npc_recovery;
mod party_tracker;
mod player_recovery;
mod self_tracker;
mod server_clock;
mod skill_tracker;
mod target_tracker;
mod unit_state;
mod world_boss_cache;
mod world_boss_tracker;
mod world_tracker;

pub(crate) use self::game_state::GameState;
pub(crate) use self::npc_recovery::NpcRecovery;
pub(crate) use self::player_recovery::PlayerRecovery;
pub(crate) use self::world_boss_cache::WorldBossCache;
