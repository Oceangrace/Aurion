//! Интеграция с данными AION 2; не знает о пользовательских виджетах.

pub(crate) mod capture;
pub(crate) mod catalog;
mod game_id;
pub(crate) mod network;
mod packet_pipeline;
pub(crate) mod protocol;
mod recorded_event;
pub(crate) mod source;
pub(crate) mod state;
mod stream_reset;
mod stream_update;
mod traffic_inspector;
mod traffic_report;
mod world_boss_observation;

pub(crate) use self::game_id::GameId;
pub(crate) use self::packet_pipeline::PacketPipeline;
pub(crate) use self::stream_reset::StreamReset;
pub(crate) use self::stream_update::StreamUpdate;
pub(crate) use self::traffic_inspector::TrafficInspector;
pub(crate) use self::traffic_report::TrafficReport;
pub(crate) use self::world_boss_observation::WorldBossObservation;
