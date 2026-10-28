use super::boss_recovery::BossRecovery;
use super::source_recovery::SourceRecovery;
use super::source_request::SourceRequest;
use crate::config::CaptureConfiguration;
use crate::core::game::GameEvent;
use crate::core::game::GameSnapshot;
use crate::core::game::SourceFrame;
use crate::core::game::SourceStatus;
use crate::integrations::aion2::PacketPipeline;
use crate::integrations::aion2::StreamReset;
use crate::integrations::aion2::capture::CaptureProcess;
use crate::integrations::aion2::capture::CaptureRead;
use crate::integrations::aion2::capture::CaptureTransport;
use crate::integrations::aion2::capture::CapturedPacket;
use crate::integrations::aion2::state::GameState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub(super) struct SourceWorker {
    configuration: CaptureConfiguration,
    pipeline: PacketPipeline,
    state: GameState,
    last_message_us: Option<u64>,
    recovery: SourceRecovery,
    bosses: BossRecovery,
}

impl SourceWorker {
    pub(super) fn new(
        configuration: CaptureConfiguration,
        pipeline: PacketPipeline,
        combat_idle_ms: u64,
        recovery_path: PathBuf,
    ) -> Self {
        let state = GameState::new(
            uuid::Uuid::new_v4().to_string(),
            pipeline.capabilities().clone(),
            pipeline.catalog().clone(),
            combat_idle_ms,
        );

        Self {
            configuration,
            pipeline,
            state,
            last_message_us: None,
            bosses: BossRecovery::new(recovery_path.with_file_name("world-boss-cache.json")),
            recovery: SourceRecovery::new(recovery_path),
        }
    }

    pub(super) fn run(mut self, stop: Arc<AtomicBool>, latest: Arc<Mutex<Arc<SourceFrame>>>) {
        let result = self.capture(&stop, &latest);

        if let Err(error) = result {
            eprintln!("AION 2 source stopped: {error}");
            self.reset();
            self.publish(&latest, 0, SourceStatus::Offline, Some(error));
        }
    }

    fn capture(
        &mut self,
        stop: &AtomicBool,
        latest: &Mutex<Arc<SourceFrame>>,
    ) -> Result<(), String> {
        let mut transport = SourceRequest::environment()?.transport(self.configuration.clone())?;

        transport.start()?;

        if transport.mode() == SourceStatus::Live {
            self.recovery.load();
        }

        let result = self.pump(&mut *transport, stop, latest);

        let now = transport.now_us();

        self.recovery.flush(
            &self.state.snapshot(now, self.status(now, transport.mode())),
            transport.process(),
            self.pipeline.active_stream(),
        );
        self.bosses.flush(self.state.boss_cache());
        transport.stop();

        result
    }

    fn pump(
        &mut self,
        transport: &mut dyn CaptureTransport,
        stop: &AtomicBool,
        latest: &Mutex<Arc<SourceFrame>>,
    ) -> Result<(), String> {
        let mut published = Instant::now();

        let mut error = None;

        while !stop.load(Ordering::Acquire) {
            for _ in 0..2048 {
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                match transport.poll() {
                    Ok(CaptureRead::Packet(packet)) => {
                        match self.packet(
                            &packet,
                            transport.server_ports(),
                            transport.process().as_ref(),
                        ) {
                            Ok(true) => error = None,
                            Ok(false) => {}
                            Err(failure) => {
                                error = Some(failure);

                                break;
                            }
                        }
                    }

                    Ok(CaptureRead::Reset) => self.reset_transport(transport),
                    Ok(CaptureRead::Pending | CaptureRead::Finished) => break,
                    Err(failure) => {
                        self.reset_transport(transport);
                        error = Some(failure);

                        break;
                    }
                }
            }

            if published.elapsed() >= Duration::from_millis(self.configuration.publish_ms()) {
                let now = transport.now_us();

                let status = self.status(now, transport.mode());

                let snapshot = self.state.snapshot(now, status);

                self.bosses.save(self.state.boss_cache());

                self.recovery.save(
                    &snapshot,
                    transport.process(),
                    self.pipeline.active_stream(),
                );
                Self::publish_snapshot(latest, snapshot, error.clone());
                published = Instant::now();
            }

            std::thread::sleep(Duration::from_millis(self.configuration.poll_ms()));
        }

        Ok(())
    }

    fn packet(
        &mut self,
        packet: &CapturedPacket,
        server_ports: &[u16],
        process: Option<&CaptureProcess>,
    ) -> Result<bool, String> {
        if let Some(process) = process
            && let Some(cache) = self.bosses.synchronize(process, packet.timestamp_us())
        {
            self.reset();
            self.state.set_boss_cache(cache);
        }

        let update = self.pipeline.push(packet, server_ports);

        let closing = update.reset().is_some();

        // Events of a closing connection belong to the observation that is ending.
        if closing {
            self.apply(update.events(), packet.timestamp_us(), process)?;
        }

        if let Some(reset) = update.reset() {
            if matches!(reset, StreamReset::Closed) && process.is_some() {
                self.recovery.clear();
            }

            self.state.recover(reset);
            self.last_message_us = None;

            if let StreamReset::Interrupted(error) = reset {
                if process.is_some() {
                    self.recovery.load();
                }

                return Err(format!("Game TCP stream interrupted: {error}"));
            }
        }

        if !closing
            && !update.events().is_empty()
            && self.recovery.restore(
                &mut self.state,
                process,
                self.pipeline.observed_stream(),
                packet.timestamp_us(),
            )?
        {
            self.pipeline.resume_observed_stream();
        }

        if !closing {
            self.apply(update.events(), packet.timestamp_us(), process)?;
        }

        if !update.events().is_empty() {
            self.last_message_us = Some(packet.timestamp_us());
        }

        Ok(!update.events().is_empty())
    }

    fn apply(
        &mut self,
        events: &[GameEvent],
        at_us: u64,
        process: Option<&CaptureProcess>,
    ) -> Result<(), String> {
        for event in events {
            if let Err(failure) = self.state.apply(event, at_us) {
                self.reset();

                // A bounded-state failure invalidates observations, not the saved
                // character. Confirm its process and world socket again on fresh traffic.
                if process.is_some() {
                    self.recovery.load();
                }

                return Err(failure);
            }
        }

        Ok(())
    }

    fn status(&self, now_us: u64, mode: SourceStatus) -> SourceStatus {
        let Some(last) = self.last_message_us else {
            return SourceStatus::Offline;
        };

        if mode == SourceStatus::Replay {
            return mode;
        }

        if now_us.saturating_sub(last) > self.configuration.stale_ms().saturating_mul(1000) {
            return SourceStatus::Stale;
        }

        mode
    }

    fn reset(&mut self) {
        self.pipeline.reset();
        self.state.reset(uuid::Uuid::new_v4().to_string());
        self.last_message_us = None;
    }

    fn reset_transport(&mut self, transport: &dyn CaptureTransport) {
        self.reset();

        if transport.mode() == SourceStatus::Live {
            self.recovery.load();
        }
    }

    fn publish(
        &mut self,
        latest: &Mutex<Arc<SourceFrame>>,
        now: u64,
        status: SourceStatus,
        error: Option<String>,
    ) {
        Self::publish_snapshot(latest, self.state.snapshot(now, status), error);
    }

    fn publish_snapshot(
        latest: &Mutex<Arc<SourceFrame>>,
        snapshot: GameSnapshot,
        error: Option<String>,
    ) {
        let frame = Arc::new(SourceFrame::new(snapshot, error));

        if let Ok(mut latest) = latest.lock() {
            *latest = frame;
        }
    }
}
