use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(bevy::prelude::Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct SessionCtl {
    pub running: bool,
    pub fast_forward: bool,
    pub autosave_every_min: u32,
    pub next_autosave_tick: u64,
    pub slot_name: Option<String>,
}

impl SessionCtl {
    pub fn new() -> Self {
        Self {
            running: true,
            fast_forward: false,
            autosave_every_min: 5, // Default 5 minutes
            next_autosave_tick: 0,
            slot_name: None,
        }
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn resume(&mut self) {
        self.running = true;
    }

    pub fn toggle_fast_forward(&mut self) {
        self.fast_forward = !self.fast_forward;
    }

    pub fn set_autosave_interval(&mut self, minutes: u32) {
        self.autosave_every_min = minutes;
    }

    pub fn should_autosave(&self, current_tick: u64) -> bool {
        current_tick >= self.next_autosave_tick
    }

    pub fn update_autosave_tick(&mut self, current_tick: u64) {
        let ticks_per_minute = 60000 / 16; // 1 minute in 16ms ticks
        self.next_autosave_tick = current_tick + (self.autosave_every_min as u64 * ticks_per_minute);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayEvent {
    Tick { n: u64 },
    EnqueueJob { pipeline_id: String, payload: usize },
    PolicyChange { policy: String },
    TunableChange { key: String, value: f32 },
    SimStart { seed: u64, scenario_id: String },
    RitualStarted { id: String },
    EventFired { swan_id: String },
    MutationApplied { pipeline_id: String, kind: String },
}

#[derive(bevy::prelude::Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct ReplayLog {
    pub events: Vec<ReplayEvent>, // append-only; flush in chunks
    pub mode: ReplayMode,         // Off | Record | Playback
    pub max_events: usize,        // bounded buffer size
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayMode { 
    Off, 
    Record, 
    Playback 
}

impl Default for ReplayMode {
    fn default() -> Self {
        Self::Off
    }
}

impl ReplayLog {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            mode: ReplayMode::Off,
            max_events: 10000, // Keep last 10k events
        }
    }

    pub fn start_recording(&mut self) {
        self.mode = ReplayMode::Record;
        self.events.clear();
    }

    pub fn start_playback(&mut self) {
        self.mode = ReplayMode::Playback;
    }

    pub fn stop(&mut self) {
        self.mode = ReplayMode::Off;
    }

    pub fn record_event(&mut self, event: ReplayEvent) {
        if self.mode == ReplayMode::Record {
            self.events.push(event);
            
            // Keep buffer bounded
            if self.events.len() > self.max_events {
                self.events.remove(0);
            }
        }
    }

    pub fn get_next_event(&mut self) -> Option<ReplayEvent> {
        if self.mode == ReplayMode::Playback && !self.events.is_empty() {
            Some(self.events.remove(0))
        } else {
            None
        }
    }

    pub fn is_recording(&self) -> bool {
        self.mode == ReplayMode::Record
    }

    pub fn is_playback(&self) -> bool {
        self.mode == ReplayMode::Playback
    }
}

pub fn record_event(event: ReplayEvent, log: &mut ReplayLog) {
    log.record_event(event);
}

pub fn save_autosnap(
    session_ctl: &SessionCtl,
    replay_log: &ReplayLog,
    // TODO: Add other resources to serialize
) -> anyhow::Result<()> {
    // In a real implementation, this would serialize the game state
    println!("Autosaving to slot: {:?}", session_ctl.slot_name);
    println!("Replay events: {}", replay_log.events.len());
    Ok(())
}

pub fn load_snapshot(
    slot_name: &str,
    // TODO: Add resources to deserialize into
) -> anyhow::Result<()> {
    // In a real implementation, this would deserialize the game state
    println!("Loading snapshot from slot: {}", slot_name);
    Ok(())
}

pub fn session_control_system(
    mut session_ctl: ResMut<SessionCtl>,
    mut replay_log: ResMut<ReplayLog>,
    clock: Res<super::SimClock>,
    // TODO: Add event readers for session control commands
) {
    let current_tick = clock.now.timestamp_millis() as u64 / 16;

    // Check for autosave
    if session_ctl.should_autosave(current_tick) {
        if let Err(e) = save_autosnap(&session_ctl, &replay_log) {
            println!("Autosave failed: {}", e);
        }
        session_ctl.update_autosave_tick(current_tick);
    }

    // Record tick event if recording
    if replay_log.is_recording() {
        record_event(ReplayEvent::Tick { n: current_tick }, &mut replay_log);
    }

    // Handle playback events
    if replay_log.is_playback() {
        if let Some(event) = replay_log.get_next_event() {
            match event {
                ReplayEvent::Tick { n } => {
                    // TODO: Handle tick replay
                    println!("Replaying tick: {}", n);
                }
                ReplayEvent::EnqueueJob { pipeline_id, payload } => {
                    // TODO: Replay job enqueue
                    println!("Replaying job enqueue: {} ({} bytes)", pipeline_id, payload);
                }
                ReplayEvent::PolicyChange { policy } => {
                    // TODO: Replay policy change
                    println!("Replaying policy change: {}", policy);
                }
                ReplayEvent::TunableChange { key, value } => {
                    // TODO: Replay tunable change
                    println!("Replaying tunable change: {} = {}", key, value);
                }
                ReplayEvent::SimStart { seed, scenario_id } => {
                    // TODO: Replay sim start
                    println!("Replaying sim start: seed={}, scenario={}", seed, scenario_id);
                }
                ReplayEvent::RitualStarted { id } => {
                    // TODO: Replay ritual start
                    println!("Replaying ritual start: {}", id);
                }
                ReplayEvent::EventFired { swan_id } => {
                    // TODO: Replay Black Swan event
                    println!("Replaying Black Swan event: {}", swan_id);
                }
                ReplayEvent::MutationApplied { pipeline_id, kind } => {
                    // TODO: Replay mutation
                    println!("Replaying mutation: {} on {}", kind, pipeline_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_control() {
        let mut session = SessionCtl::new();
        assert!(session.running);
        assert!(!session.fast_forward);

        session.pause();
        assert!(!session.running);

        session.resume();
        assert!(session.running);

        session.toggle_fast_forward();
        assert!(session.fast_forward);
    }

    #[test]
    fn test_autosave_timing() {
        let mut session = SessionCtl::new();
        session.set_autosave_interval(1); // 1 minute
        session.update_autosave_tick(1000);

        assert!(!session.should_autosave(1000));
        assert!(session.should_autosave(5000)); // 4 seconds later
    }

    #[test]
    fn test_replay_log() {
        let mut log = ReplayLog::new();
        assert_eq!(log.mode, ReplayMode::Off);

        log.start_recording();
        assert_eq!(log.mode, ReplayMode::Record);
        assert!(log.is_recording());

        log.record_event(ReplayEvent::Tick { n: 100 });
        assert_eq!(log.events.len(), 1);

        log.start_playback();
        assert_eq!(log.mode, ReplayMode::Playback);
        assert!(log.is_playback());

        let event = log.get_next_event();
        assert!(event.is_some());
        assert_eq!(log.events.len(), 0);
    }

    #[test]
    fn test_replay_event_serialization() {
        let event = ReplayEvent::SimStart { 
            seed: 42, 
            scenario_id: "test".to_string() 
        };
        
        // Test that it can be serialized/deserialized
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ReplayEvent = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            ReplayEvent::SimStart { seed, scenario_id } => {
                assert_eq!(seed, 42);
                assert_eq!(scenario_id, "test");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
