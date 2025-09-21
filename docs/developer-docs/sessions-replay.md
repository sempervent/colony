# Developer Docs: Sessions & Replay

The sessions and replay system provides comprehensive session management, deterministic replay capabilities, and save/load functionality. This document explains how these systems work and how to implement them.

## Overview

The sessions and replay system provides:

- **Session Management**: Control simulation sessions (pause, resume, save, load)
- **Deterministic Replay**: Record and playback simulation sessions
- **Save/Load System**: Persist game state and restore it
- **Session Control**: Pause, resume, fast-forward, and rewind
- **Replay Analysis**: Analyze replay data for debugging and optimization
- **State Serialization**: Efficient serialization of game state

## Session Management

### Session State

A session represents a complete simulation run:

```rust
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub current_tick: u64,
    pub status: SessionStatus,
    pub configuration: SessionConfiguration,
    pub state: GameState,
    pub metadata: SessionMetadata,
}

pub enum SessionStatus {
    NotStarted,                  // Session created but not started
    Running,                     // Session is running
    Paused,                      // Session is paused
    Completed,                   // Session completed successfully
    Failed,                      // Session failed
    Cancelled,                   // Session was cancelled
}

pub struct SessionConfiguration {
    pub scenario: ScenarioId,
    pub difficulty: DifficultyLevel,
    pub victory_conditions: Vec<VictoryConditionSpec>,
    pub loss_conditions: Vec<LossConditionSpec>,
    pub mods: Vec<ModId>,
    pub settings: GameSettings,
}
```

### Session Control

```rust
pub struct SessionController {
    pub session: Session,
    pub replay_log: ReplayLog,
    pub state_manager: StateManager,
    pub control_state: ControlState,
}

pub struct ControlState {
    pub is_paused: bool,
    pub tick_scale: TickScale,
    pub target_tick: Option<u64>,
    pub playback_speed: f32,
    pub is_recording: bool,
    pub is_playback: bool,
}

pub enum TickScale {
    RealTime,                    // Real-time simulation
    FastForward(f32),            // Fast-forward by factor
    SlowMotion(f32),             // Slow-motion by factor
    StepByStep,                  // Step-by-step execution
    Custom(f32),                 // Custom scale factor
}
```

### Session Operations

```rust
impl SessionController {
    pub fn start(&mut self) -> Result<(), SessionError> {
        if self.session.status != SessionStatus::NotStarted {
            return Err(SessionError::InvalidState);
        }
        
        self.session.status = SessionStatus::Running;
        self.control_state.is_recording = true;
        self.replay_log.start_recording();
        
        Ok(())
    }
    
    pub fn pause(&mut self) -> Result<(), SessionError> {
        if self.session.status != SessionStatus::Running {
            return Err(SessionError::InvalidState);
        }
        
        self.session.status = SessionStatus::Paused;
        self.control_state.is_paused = true;
        
        Ok(())
    }
    
    pub fn resume(&mut self) -> Result<(), SessionError> {
        if self.session.status != SessionStatus::Paused {
            return Err(SessionError::InvalidState);
        }
        
        self.session.status = SessionStatus::Running;
        self.control_state.is_paused = false;
        
        Ok(())
    }
    
    pub fn fast_forward(&mut self, factor: f32) -> Result<(), SessionError> {
        self.control_state.tick_scale = TickScale::FastForward(factor);
        Ok(())
    }
    
    pub fn step(&mut self) -> Result<(), SessionError> {
        self.control_state.tick_scale = TickScale::StepByStep;
        self.control_state.is_paused = false;
        Ok(())
    }
}
```

## Deterministic Replay

### Replay Log

The replay log records all events and state changes:

```rust
pub struct ReplayLog {
    pub session_id: SessionId,
    pub start_tick: u64,
    pub end_tick: Option<u64>,
    pub events: Vec<ReplayEvent>,
    pub state_snapshots: Vec<StateSnapshot>,
    pub checksums: Vec<Checksum>,
    pub metadata: ReplayMetadata,
}

pub enum ReplayEvent {
    Input(InputEvent),           // User input
    System(SystemEvent),         // System events
    StateChange(StateChangeEvent), // State changes
    Random(RandomEvent),         // Random number generation
    External(ExternalEvent),     // External events
}

pub struct StateSnapshot {
    pub tick: u64,
    pub state: GameState,
    pub checksum: u64,
    pub compression: CompressionInfo,
}
```

### Replay Recording

```rust
impl ReplayLog {
    pub fn start_recording(&mut self) {
        self.start_tick = 0;
        self.end_tick = None;
        self.events.clear();
        self.state_snapshots.clear();
        self.checksums.clear();
    }
    
    pub fn record_event(&mut self, event: ReplayEvent) {
        self.events.push(event);
    }
    
    pub fn record_state_snapshot(&mut self, tick: u64, state: &GameState) {
        let checksum = self.calculate_checksum(state);
        let compressed_state = self.compress_state(state);
        
        let snapshot = StateSnapshot {
            tick,
            state: compressed_state,
            checksum,
            compression: CompressionInfo::new(),
        };
        
        self.state_snapshots.push(snapshot);
        self.checksums.push(Checksum { tick, checksum });
    }
    
    pub fn stop_recording(&mut self) {
        self.end_tick = Some(self.get_current_tick());
    }
}
```

### Replay Playback

```rust
pub struct ReplayPlayer {
    pub replay_log: ReplayLog,
    pub current_tick: u64,
    pub playback_speed: f32,
    pub is_playing: bool,
    pub state: GameState,
    pub event_index: usize,
    pub snapshot_index: usize,
}

impl ReplayPlayer {
    pub fn load_replay(&mut self, replay_log: ReplayLog) -> Result<(), ReplayError> {
        self.replay_log = replay_log;
        self.current_tick = 0;
        self.event_index = 0;
        self.snapshot_index = 0;
        
        // Load initial state
        if let Some(snapshot) = self.replay_log.state_snapshots.first() {
            self.state = self.decompress_state(&snapshot.state);
        }
        
        Ok(())
    }
    
    pub fn play(&mut self) -> Result<(), ReplayError> {
        self.is_playing = true;
        
        while self.is_playing && self.current_tick < self.replay_log.end_tick.unwrap_or(u64::MAX) {
            self.step()?;
        }
        
        Ok(())
    }
    
    pub fn step(&mut self) -> Result<(), ReplayError> {
        // Apply events for current tick
        while self.event_index < self.replay_log.events.len() {
            let event = &self.replay_log.events[self.event_index];
            if event.get_tick() > self.current_tick {
                break;
            }
            
            self.apply_event(event)?;
            self.event_index += 1;
        }
        
        // Check for state snapshot
        if self.snapshot_index < self.replay_log.state_snapshots.len() {
            let snapshot = &self.replay_log.state_snapshots[self.snapshot_index];
            if snapshot.tick == self.current_tick {
                self.state = self.decompress_state(&snapshot.state);
                self.snapshot_index += 1;
            }
        }
        
        self.current_tick += 1;
        Ok(())
    }
    
    pub fn seek(&mut self, target_tick: u64) -> Result<(), ReplayError> {
        // Find closest state snapshot
        let snapshot = self.find_closest_snapshot(target_tick);
        if let Some(snapshot) = snapshot {
            self.state = self.decompress_state(&snapshot.state);
            self.current_tick = snapshot.tick;
            self.event_index = self.find_event_index(target_tick);
            self.snapshot_index = self.find_snapshot_index(target_tick);
        }
        
        // Fast-forward to target tick
        while self.current_tick < target_tick {
            self.step()?;
        }
        
        Ok(())
    }
}
```

## Save/Load System

### State Serialization

```rust
pub trait Serializable {
    fn serialize(&self) -> Result<Vec<u8>, SerializationError>;
    fn deserialize(data: &[u8]) -> Result<Self, SerializationError> where Self: Sized;
}

pub struct StateSerializer {
    pub compression: CompressionAlgorithm,
    pub checksum: ChecksumAlgorithm,
    pub format: SerializationFormat,
}

pub enum SerializationFormat {
    Binary,                      // Binary format
    Json,                        // JSON format
    MessagePack,                 // MessagePack format
    Bincode,                     // Bincode format
}

impl StateSerializer {
    pub fn serialize_state(&self, state: &GameState) -> Result<Vec<u8>, SerializationError> {
        let serialized = match self.format {
            SerializationFormat::Binary => self.serialize_binary(state)?,
            SerializationFormat::Json => self.serialize_json(state)?,
            SerializationFormat::MessagePack => self.serialize_messagepack(state)?,
            SerializationFormat::Bincode => self.serialize_bincode(state)?,
        };
        
        let compressed = self.compress(&serialized)?;
        let checksum = self.calculate_checksum(&compressed);
        
        let mut result = Vec::new();
        result.extend_from_slice(&checksum.to_le_bytes());
        result.extend_from_slice(&compressed);
        
        Ok(result)
    }
    
    pub fn deserialize_state(&self, data: &[u8]) -> Result<GameState, SerializationError> {
        if data.len() < 8 {
            return Err(SerializationError::InvalidData);
        }
        
        let expected_checksum = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        let compressed_data = &data[8..];
        
        let actual_checksum = self.calculate_checksum(compressed_data);
        if expected_checksum != actual_checksum {
            return Err(SerializationError::ChecksumMismatch);
        }
        
        let decompressed = self.decompress(compressed_data)?;
        
        match self.format {
            SerializationFormat::Binary => self.deserialize_binary(&decompressed),
            SerializationFormat::Json => self.deserialize_json(&decompressed),
            SerializationFormat::MessagePack => self.deserialize_messagepack(&decompressed),
            SerializationFormat::Bincode => self.deserialize_bincode(&decompressed),
        }
    }
}
```

### Save System

```rust
pub struct SaveSystem {
    pub save_directory: PathBuf,
    pub serializer: StateSerializer,
    pub save_manager: SaveManager,
}

pub struct SaveManager {
    pub saves: HashMap<SaveId, SaveInfo>,
    pub auto_saves: Vec<AutoSave>,
    pub save_slots: Vec<SaveSlot>,
}

pub struct SaveInfo {
    pub id: SaveId,
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub session_id: SessionId,
    pub tick: u64,
    pub file_size: u64,
    pub checksum: u64,
    pub metadata: SaveMetadata,
}

impl SaveSystem {
    pub fn save_session(&mut self, session: &Session, save_name: String) -> Result<SaveId, SaveError> {
        let save_id = SaveId::new();
        let save_path = self.save_directory.join(format!("{}.save", save_id));
        
        // Serialize session state
        let serialized = self.serializer.serialize_state(&session.state)?;
        
        // Write to file
        std::fs::write(&save_path, &serialized)?;
        
        // Create save info
        let save_info = SaveInfo {
            id: save_id,
            name: save_name,
            description: session.description.clone(),
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            session_id: session.id,
            tick: session.current_tick,
            file_size: serialized.len() as u64,
            checksum: self.serializer.calculate_checksum(&serialized),
            metadata: SaveMetadata::new(),
        };
        
        self.save_manager.saves.insert(save_id, save_info);
        
        Ok(save_id)
    }
    
    pub fn load_session(&mut self, save_id: SaveId) -> Result<Session, SaveError> {
        let save_info = self.save_manager.saves.get(&save_id)
            .ok_or(SaveError::SaveNotFound)?;
        
        let save_path = self.save_directory.join(format!("{}.save", save_id));
        let data = std::fs::read(&save_path)?;
        
        // Verify checksum
        let expected_checksum = save_info.checksum;
        let actual_checksum = self.serializer.calculate_checksum(&data);
        if expected_checksum != actual_checksum {
            return Err(SaveError::ChecksumMismatch);
        }
        
        // Deserialize state
        let state = self.serializer.deserialize_state(&data)?;
        
        // Reconstruct session
        let session = Session {
            id: save_info.session_id,
            name: save_info.name.clone(),
            description: save_info.description.clone(),
            created_at: save_info.created_at,
            last_modified: save_info.created_at,
            current_tick: save_info.tick,
            status: SessionStatus::Paused,
            configuration: SessionConfiguration::default(),
            state,
            metadata: SessionMetadata::new(),
        };
        
        Ok(session)
    }
}
```

## State Management

### State Snapshots

```rust
pub struct StateManager {
    pub current_state: GameState,
    pub state_history: Vec<StateSnapshot>,
    pub max_history_size: usize,
    pub snapshot_interval: u64,
    pub last_snapshot_tick: u64,
}

impl StateManager {
    pub fn create_snapshot(&mut self, tick: u64) -> StateSnapshot {
        let snapshot = StateSnapshot {
            tick,
            state: self.current_state.clone(),
            checksum: self.calculate_checksum(&self.current_state),
            compression: CompressionInfo::new(),
        };
        
        self.state_history.push(snapshot.clone());
        
        // Limit history size
        if self.state_history.len() > self.max_history_size {
            self.state_history.remove(0);
        }
        
        self.last_snapshot_tick = tick;
        snapshot
    }
    
    pub fn restore_snapshot(&mut self, snapshot: &StateSnapshot) {
        self.current_state = snapshot.state.clone();
    }
    
    pub fn find_snapshot(&self, tick: u64) -> Option<&StateSnapshot> {
        self.state_history.iter()
            .find(|snapshot| snapshot.tick == tick)
    }
    
    pub fn find_closest_snapshot(&self, tick: u64) -> Option<&StateSnapshot> {
        self.state_history.iter()
            .filter(|snapshot| snapshot.tick <= tick)
            .max_by_key(|snapshot| snapshot.tick)
    }
}
```

### State Validation

```rust
pub struct StateValidator {
    pub validation_rules: Vec<ValidationRule>,
    pub checksum_algorithm: ChecksumAlgorithm,
}

pub enum ValidationRule {
    StateConsistency,            // Check state consistency
    ResourceBounds,              // Check resource bounds
    EntityIntegrity,             // Check entity integrity
    ComponentValidity,           // Check component validity
    SystemState,                 // Check system state
}

impl StateValidator {
    pub fn validate_state(&self, state: &GameState) -> Result<(), ValidationError> {
        for rule in &self.validation_rules {
            match rule {
                ValidationRule::StateConsistency => self.validate_consistency(state)?,
                ValidationRule::ResourceBounds => self.validate_resource_bounds(state)?,
                ValidationRule::EntityIntegrity => self.validate_entity_integrity(state)?,
                ValidationRule::ComponentValidity => self.validate_component_validity(state)?,
                ValidationRule::SystemState => self.validate_system_state(state)?,
            }
        }
        
        Ok(())
    }
    
    fn validate_consistency(&self, state: &GameState) -> Result<(), ValidationError> {
        // Check that all entities have valid components
        // Check that all components reference valid entities
        // Check that all systems are in valid states
        Ok(())
    }
    
    fn validate_resource_bounds(&self, state: &GameState) -> Result<(), ValidationError> {
        // Check that resources are within valid bounds
        // Check that resource consumption doesn't exceed generation
        // Check that resource allocation is valid
        Ok(())
    }
}
```

## Configuration

### Session Configuration

```toml
# In game configuration
[sessions]
default_save_directory = "saves"
max_save_files = 100
auto_save_interval = 300         # 300 ticks
auto_save_count = 5              # Keep 5 auto-saves
save_compression = "gzip"        # Compression algorithm
save_format = "bincode"          # Serialization format

[sessions.replay]
snapshot_interval = 100          # Snapshot every 100 ticks
max_snapshot_count = 1000        # Keep 1000 snapshots
event_buffer_size = 10000        # Event buffer size
compression_level = 6            # Compression level
checksum_algorithm = "crc64"     # Checksum algorithm

[sessions.control]
default_tick_scale = "realtime"  # Default tick scale
max_fast_forward = 10.0          # Maximum fast-forward factor
min_slow_motion = 0.1            # Minimum slow-motion factor
step_size = 1                    # Step size for step-by-step
```

### Save Configuration

```toml
[saves]
auto_save_enabled = true
auto_save_interval = 300         # 300 ticks
auto_save_count = 5              # Keep 5 auto-saves
manual_save_slots = 10           # 10 manual save slots
save_metadata = true             # Include metadata
save_checksums = true            # Include checksums
save_compression = true          # Enable compression
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let session = Session::new("Test Session", "Test Description");
        
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.status, SessionStatus::NotStarted);
        assert_eq!(session.current_tick, 0);
    }
    
    #[test]
    fn test_replay_recording() {
        let mut replay_log = ReplayLog::new();
        replay_log.start_recording();
        
        let event = ReplayEvent::Input(InputEvent::new());
        replay_log.record_event(event);
        
        assert_eq!(replay_log.events.len(), 1);
    }
    
    #[test]
    fn test_state_serialization() {
        let serializer = StateSerializer::new();
        let state = create_test_game_state();
        
        let serialized = serializer.serialize_state(&state).unwrap();
        let deserialized = serializer.deserialize_state(&serialized).unwrap();
        
        assert_eq!(state, deserialized);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_session_save_load() {
        let mut save_system = SaveSystem::new();
        let session = create_test_session();
        
        let save_id = save_system.save_session(&session, "Test Save".to_string()).unwrap();
        let loaded_session = save_system.load_session(save_id).unwrap();
        
        assert_eq!(session.id, loaded_session.id);
        assert_eq!(session.current_tick, loaded_session.current_tick);
    }
    
    #[test]
    fn test_replay_playback() {
        let mut replay_log = create_test_replay_log();
        let mut player = ReplayPlayer::new();
        
        player.load_replay(replay_log).unwrap();
        player.play().unwrap();
        
        assert_eq!(player.current_tick, player.replay_log.end_tick.unwrap());
    }
}
```

## Best Practices

### Design Guidelines

1. **Determinism**: Ensure replay is deterministic
2. **Efficiency**: Optimize serialization and compression
3. **Reliability**: Implement robust error handling
4. **Performance**: Balance snapshot frequency with performance
5. **Compatibility**: Maintain save file compatibility

### Performance Considerations

1. **Snapshot Frequency**: Balance snapshot frequency with memory usage
2. **Compression**: Use efficient compression algorithms
3. **Caching**: Cache frequently accessed data
4. **Lazy Loading**: Load data as needed
5. **Memory Management**: Manage memory usage efficiently

## Troubleshooting

### Common Issues

1. **Save Corruption**: Corrupted save files
2. **Replay Desync**: Replay not matching original
3. **Performance Issues**: Slow save/load operations
4. **Memory Usage**: High memory usage during replay
5. **Compatibility Issues**: Save file compatibility problems

### Debug Tools

- **Save Validator**: Validate save file integrity
- **Replay Analyzer**: Analyze replay data
- **State Inspector**: Inspect game state
- **Performance Profiler**: Profile save/load performance
- **Checksum Calculator**: Calculate and verify checksums

---

**The sessions and replay system provides comprehensive session management and deterministic replay capabilities. Understanding these systems is key to building robust and debuggable simulations.** 🏭🎮
