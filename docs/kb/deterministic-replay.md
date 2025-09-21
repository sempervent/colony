# Knowledge Base: Deterministic Replay

This article provides comprehensive guidance on deterministic replay in the Colony Simulator, covering how it works, how to use it, and how to ensure deterministic behavior.

## Overview

Deterministic replay allows you to:

- **Record Simulation Sessions**: Record complete simulation sessions
- **Playback Sessions**: Playback recorded sessions exactly
- **Debug Issues**: Debug issues by replaying sessions
- **Analyze Performance**: Analyze performance by replaying sessions
- **Test Modifications**: Test modifications by replaying sessions
- **Share Sessions**: Share sessions with others for analysis

## How Deterministic Replay Works

### Core Principles

Deterministic replay is based on these principles:

1. **Deterministic Execution**: The simulation must execute deterministically
2. **Event Recording**: All events must be recorded
3. **State Snapshots**: State snapshots must be taken at regular intervals
4. **Checksum Validation**: Checksums must be used to validate state
5. **Replay Accuracy**: Replay must be identical to original execution

### Recording Process

The recording process involves:

```rust
// Recording process
pub struct ReplayRecorder {
    pub event_log: EventLog,
    pub state_snapshots: StateSnapshots,
    pub checksums: Checksums,
    pub metadata: ReplayMetadata,
}

impl ReplayRecorder {
    pub fn start_recording(&mut self) {
        // Initialize recording
        self.event_log.clear();
        self.state_snapshots.clear();
        self.checksums.clear();
        
        // Record initial state
        let initial_state = self.get_current_state();
        self.record_state_snapshot(0, &initial_state);
        
        // Start event recording
        self.start_event_recording();
    }
    
    pub fn record_event(&mut self, event: &Event) {
        // Record event
        self.event_log.record(event);
        
        // Take state snapshot if needed
        if self.should_take_snapshot(event) {
            let state = self.get_current_state();
            self.record_state_snapshot(event.tick, &state);
        }
    }
    
    fn should_take_snapshot(&self, event: &Event) -> bool {
        // Take snapshot every N ticks
        event.tick % self.snapshot_interval == 0
    }
}
```

### Playback Process

The playback process involves:

```rust
// Playback process
pub struct ReplayPlayer {
    pub event_log: EventLog,
    pub state_snapshots: StateSnapshots,
    pub current_tick: u64,
    pub current_state: GameState,
}

impl ReplayPlayer {
    pub fn load_replay(&mut self, replay_data: &ReplayData) {
        // Load replay data
        self.event_log = replay_data.event_log.clone();
        self.state_snapshots = replay_data.state_snapshots.clone();
        
        // Initialize state
        self.current_tick = 0;
        self.current_state = self.get_initial_state();
    }
    
    pub fn play(&mut self) -> Result<(), ReplayError> {
        // Play events in order
        for event in &self.event_log.events {
            // Apply event
            self.apply_event(event)?;
            
            // Update tick
            self.current_tick = event.tick;
            
            // Check for state snapshot
            if let Some(snapshot) = self.get_snapshot_at_tick(event.tick) {
                self.current_state = snapshot.state.clone();
            }
        }
        
        Ok(())
    }
    
    fn apply_event(&mut self, event: &Event) -> Result<(), ReplayError> {
        // Apply event to current state
        match event.event_type {
            EventType::SystemEvent => self.apply_system_event(event),
            EventType::UserEvent => self.apply_user_event(event),
            EventType::RandomEvent => self.apply_random_event(event),
            EventType::ExternalEvent => self.apply_external_event(event),
        }
    }
}
```

## Ensuring Deterministic Behavior

### Random Number Generation

Ensure deterministic random number generation:

```rust
// Deterministic random number generation
pub struct DeterministicRng {
    pub seed: u64,
    pub state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { seed, state: seed }
    }
    
    pub fn next(&mut self) -> u64 {
        // Use deterministic algorithm
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }
    
    pub fn next_f32(&mut self) -> f32 {
        (self.next() as f32) / (u64::MAX as f32)
    }
    
    pub fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
}

// Global deterministic RNG
thread_local! {
    static DETERMINISTIC_RNG: RefCell<DeterministicRng> = RefCell::new(DeterministicRng::new(0));
}

pub fn set_random_seed(seed: u64) {
    DETERMINISTIC_RNG.with(|rng| {
        rng.borrow_mut().seed = seed;
        rng.borrow_mut().state = seed;
    });
}

pub fn random() -> f32 {
    DETERMINISTIC_RNG.with(|rng| rng.borrow_mut().next_f32())
}
```

### System Ordering

Ensure deterministic system ordering:

```rust
// Deterministic system ordering
pub struct DeterministicScheduler {
    pub system_order: Vec<SystemId>,
    pub system_groups: Vec<SystemGroup>,
}

impl DeterministicScheduler {
    pub fn schedule_systems(&mut self, systems: &[System]) {
        // Sort systems deterministically
        let mut sorted_systems = systems.to_vec();
        sorted_systems.sort_by(|a, b| a.id.cmp(&b.id));
        
        // Group systems by dependencies
        let groups = self.group_systems_by_dependencies(&sorted_systems);
        
        // Order groups deterministically
        self.system_groups = groups;
        self.system_order = self.create_execution_order();
    }
    
    fn group_systems_by_dependencies(&self, systems: &[System]) -> Vec<SystemGroup> {
        let mut groups = Vec::new();
        let mut visited = HashSet::new();
        
        for system in systems {
            if !visited.contains(&system.id) {
                let group = self.create_system_group(system, &mut visited);
                groups.push(group);
            }
        }
        
        groups
    }
}
```

### State Management

Ensure deterministic state management:

```rust
// Deterministic state management
pub struct DeterministicStateManager {
    pub state: GameState,
    pub state_history: Vec<StateSnapshot>,
    pub checksums: Vec<Checksum>,
}

impl DeterministicStateManager {
    pub fn update_state(&mut self, delta: StateDelta) {
        // Apply state changes deterministically
        self.apply_state_delta(&delta);
        
        // Record state snapshot
        let snapshot = self.create_state_snapshot();
        self.state_history.push(snapshot);
        
        // Calculate checksum
        let checksum = self.calculate_checksum();
        self.checksums.push(checksum);
    }
    
    fn apply_state_delta(&mut self, delta: &StateDelta) {
        // Apply changes in deterministic order
        for change in &delta.changes {
            match change {
                StateChange::EntityAdded(entity) => self.add_entity(entity),
                StateChange::EntityRemoved(entity_id) => self.remove_entity(*entity_id),
                StateChange::ComponentAdded(entity_id, component) => {
                    self.add_component(*entity_id, component);
                },
                StateChange::ComponentRemoved(entity_id, component_type) => {
                    self.remove_component(*entity_id, *component_type);
                },
                StateChange::ResourceChanged(resource_id, value) => {
                    self.set_resource(*resource_id, *value);
                },
            }
        }
    }
}
```

## Using Deterministic Replay

### Recording a Session

Record a simulation session:

```rust
// Recording a session
pub struct SessionRecorder {
    pub recorder: ReplayRecorder,
    pub session: Session,
}

impl SessionRecorder {
    pub fn start_recording(&mut self, session_name: String) {
        // Create session
        self.session = Session::new(session_name);
        
        // Start recording
        self.recorder.start_recording();
        
        // Record session metadata
        self.record_session_metadata();
    }
    
    pub fn stop_recording(&mut self) -> ReplayData {
        // Stop recording
        self.recorder.stop_recording();
        
        // Create replay data
        ReplayData {
            session: self.session.clone(),
            event_log: self.recorder.event_log.clone(),
            state_snapshots: self.recorder.state_snapshots.clone(),
            checksums: self.recorder.checksums.clone(),
            metadata: self.recorder.metadata.clone(),
        }
    }
    
    fn record_session_metadata(&mut self) {
        self.recorder.metadata.session_name = self.session.name.clone();
        self.recorder.metadata.start_time = std::time::SystemTime::now();
        self.recorder.metadata.version = env!("CARGO_PKG_VERSION").to_string();
        self.recorder.metadata.platform = std::env::consts::OS.to_string();
    }
}
```

### Playing Back a Session

Play back a recorded session:

```rust
// Playing back a session
pub struct SessionPlayer {
    pub player: ReplayPlayer,
    pub session: Session,
}

impl SessionPlayer {
    pub fn load_session(&mut self, replay_data: &ReplayData) {
        // Load replay data
        self.player.load_replay(replay_data);
        
        // Set session
        self.session = replay_data.session.clone();
    }
    
    pub fn play_session(&mut self) -> Result<(), ReplayError> {
        // Play the session
        self.player.play()?;
        
        // Validate replay
        self.validate_replay()?;
        
        Ok(())
    }
    
    fn validate_replay(&self) -> Result<(), ReplayError> {
        // Validate checksums
        for (i, checksum) in self.player.checksums.iter().enumerate() {
            let expected = self.calculate_checksum_at_tick(i as u64);
            if *checksum != expected {
                return Err(ReplayError::ChecksumMismatch);
            }
        }
        
        Ok(())
    }
}
```

### Debugging with Replay

Use replay for debugging:

```rust
// Debugging with replay
pub struct ReplayDebugger {
    pub player: ReplayPlayer,
    pub debugger: Debugger,
    pub breakpoints: Vec<Breakpoint>,
}

impl ReplayDebugger {
    pub fn debug_replay(&mut self, replay_data: &ReplayData) {
        // Load replay
        self.player.load_replay(replay_data);
        
        // Set breakpoints
        self.set_debug_breakpoints();
        
        // Play with debugging
        self.play_with_debugging();
    }
    
    fn set_debug_breakpoints(&mut self) {
        // Set breakpoints at specific ticks
        self.breakpoints.push(Breakpoint {
            tick: 1000,
            condition: None,
            action: BreakpointAction::Pause,
        });
        
        // Set breakpoints on specific events
        self.breakpoints.push(Breakpoint {
            tick: 0, // Any tick
            condition: Some(DebugCondition::EventType(EventType::FaultOccurred)),
            action: BreakpointAction::Pause,
        });
    }
    
    fn play_with_debugging(&mut self) {
        for event in &self.player.event_log.events {
            // Check breakpoints
            if self.should_break(event) {
                self.debugger.pause_at_event(event);
            }
            
            // Apply event
            self.player.apply_event(event).unwrap();
        }
    }
}
```

## Replay Validation

### Checksum Validation

Validate replay using checksums:

```rust
// Checksum validation
pub struct ChecksumValidator {
    pub checksums: Vec<Checksum>,
    pub algorithm: ChecksumAlgorithm,
}

impl ChecksumValidator {
    pub fn validate_replay(&self, replay_data: &ReplayData) -> Result<(), ValidationError> {
        // Validate event log checksum
        let event_checksum = self.calculate_checksum(&replay_data.event_log);
        if event_checksum != replay_data.metadata.event_checksum {
            return Err(ValidationError::EventChecksumMismatch);
        }
        
        // Validate state snapshot checksums
        for (i, snapshot) in replay_data.state_snapshots.iter().enumerate() {
            let snapshot_checksum = self.calculate_checksum(&snapshot);
            if snapshot_checksum != replay_data.checksums[i] {
                return Err(ValidationError::StateChecksumMismatch);
            }
        }
        
        Ok(())
    }
    
    fn calculate_checksum(&self, data: &dyn Serialize) -> Checksum {
        match self.algorithm {
            ChecksumAlgorithm::CRC32 => self.calculate_crc32(data),
            ChecksumAlgorithm::CRC64 => self.calculate_crc64(data),
            ChecksumAlgorithm::SHA256 => self.calculate_sha256(data),
        }
    }
}
```

### State Validation

Validate replay state:

```rust
// State validation
pub struct StateValidator {
    pub validation_rules: Vec<ValidationRule>,
}

impl StateValidator {
    pub fn validate_state(&self, state: &GameState) -> Result<(), ValidationError> {
        for rule in &self.validation_rules {
            match rule {
                ValidationRule::EntityIntegrity => self.validate_entity_integrity(state)?,
                ValidationRule::ComponentConsistency => self.validate_component_consistency(state)?,
                ValidationRule::ResourceBounds => self.validate_resource_bounds(state)?,
                ValidationRule::SystemState => self.validate_system_state(state)?,
            }
        }
        
        Ok(())
    }
    
    fn validate_entity_integrity(&self, state: &GameState) -> Result<(), ValidationError> {
        // Check that all entities have valid IDs
        for entity in &state.entities {
            if entity.id.is_nil() {
                return Err(ValidationError::InvalidEntityId);
            }
        }
        
        // Check that all components reference valid entities
        for (entity_id, components) in &state.components {
            if !state.entities.iter().any(|e| e.id == *entity_id) {
                return Err(ValidationError::OrphanedComponent);
            }
        }
        
        Ok(())
    }
}
```

## Replay Analysis

### Performance Analysis

Analyze replay performance:

```rust
// Performance analysis
pub struct ReplayAnalyzer {
    pub performance_metrics: PerformanceMetrics,
    pub bottleneck_analyzer: BottleneckAnalyzer,
}

impl ReplayAnalyzer {
    pub fn analyze_performance(&mut self, replay_data: &ReplayData) -> PerformanceAnalysis {
        // Analyze tick performance
        let tick_analysis = self.analyze_tick_performance(&replay_data.event_log);
        
        // Analyze system performance
        let system_analysis = self.analyze_system_performance(&replay_data.event_log);
        
        // Analyze resource usage
        let resource_analysis = self.analyze_resource_usage(&replay_data.state_snapshots);
        
        // Identify bottlenecks
        let bottlenecks = self.bottleneck_analyzer.analyze(&tick_analysis, &system_analysis, &resource_analysis);
        
        PerformanceAnalysis {
            tick_analysis,
            system_analysis,
            resource_analysis,
            bottlenecks,
        }
    }
    
    fn analyze_tick_performance(&self, event_log: &EventLog) -> TickAnalysis {
        let mut tick_times = Vec::new();
        
        for event in &event_log.events {
            if event.event_type == EventType::TickStart {
                tick_times.push(event.timestamp);
            }
        }
        
        TickAnalysis {
            average_tick_time: self.calculate_average_tick_time(&tick_times),
            min_tick_time: self.calculate_min_tick_time(&tick_times),
            max_tick_time: self.calculate_max_tick_time(&tick_times),
            tick_time_variance: self.calculate_tick_time_variance(&tick_times),
        }
    }
}
```

### Event Analysis

Analyze replay events:

```rust
// Event analysis
pub struct EventAnalyzer {
    pub event_statistics: EventStatistics,
    pub event_patterns: EventPatterns,
}

impl EventAnalyzer {
    pub fn analyze_events(&mut self, event_log: &EventLog) -> EventAnalysis {
        // Analyze event statistics
        let statistics = self.analyze_event_statistics(event_log);
        
        // Analyze event patterns
        let patterns = self.analyze_event_patterns(event_log);
        
        // Analyze event correlations
        let correlations = self.analyze_event_correlations(event_log);
        
        EventAnalysis {
            statistics,
            patterns,
            correlations,
        }
    }
    
    fn analyze_event_statistics(&self, event_log: &EventLog) -> EventStatistics {
        let mut event_counts = HashMap::new();
        
        for event in &event_log.events {
            *event_counts.entry(event.event_type).or_insert(0) += 1;
        }
        
        EventStatistics {
            total_events: event_log.events.len(),
            event_counts,
            average_events_per_tick: event_log.events.len() as f32 / event_log.max_tick as f32,
        }
    }
}
```

## Best Practices

### Recording Best Practices

1. **Use Deterministic Seeds**: Always use deterministic random seeds
2. **Record Everything**: Record all events and state changes
3. **Take Regular Snapshots**: Take state snapshots at regular intervals
4. **Validate Checksums**: Use checksums to validate replay integrity
5. **Compress Data**: Compress replay data to save space

### Playback Best Practices

1. **Validate Replay**: Always validate replay before playback
2. **Use Same Version**: Use the same version for recording and playback
3. **Check Dependencies**: Ensure all dependencies are available
4. **Monitor Performance**: Monitor performance during playback
5. **Handle Errors**: Handle errors gracefully during playback

### Debugging Best Practices

1. **Use Breakpoints**: Use breakpoints to pause at specific points
2. **Inspect State**: Inspect state at breakpoints
3. **Compare States**: Compare states between different runs
4. **Analyze Events**: Analyze event patterns and correlations
5. **Document Issues**: Document issues found during debugging

## Troubleshooting

### Common Issues

1. **Non-deterministic Behavior**: Random number generation or system ordering
2. **Checksum Mismatches**: State corruption or version differences
3. **Playback Failures**: Missing dependencies or invalid data
4. **Performance Issues**: Slow playback or high memory usage
5. **State Corruption**: Invalid state during playback

### Debug Techniques

1. **Compare States**: Compare states between recording and playback
2. **Analyze Events**: Analyze event sequences for differences
3. **Check Checksums**: Verify checksums at each step
4. **Monitor Resources**: Monitor resource usage during playback
5. **Use Debugging Tools**: Use debugging tools to identify issues

## Conclusion

Deterministic replay is a powerful tool for debugging, analysis, and testing. By following best practices and using the right techniques, you can effectively use deterministic replay to improve the Colony Simulator.

Key points to remember:

- **Ensure Determinism**: Make sure the simulation is deterministic
- **Record Everything**: Record all events and state changes
- **Validate Replay**: Always validate replay integrity
- **Use for Debugging**: Use replay for debugging and analysis
- **Follow Best Practices**: Follow best practices for recording and playback

---

**Deterministic replay is essential for reliable debugging and analysis.** 🏭🎬
