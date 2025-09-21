# Reference: Rust Crates

This section provides comprehensive API documentation for all Rust crates in the Asynchronous Colony Simulator project.

## Core Crates

### `colony-core`

The foundational crate containing the core simulation logic, ECS systems, and game state management.

**Key Modules:**
- `components/` - ECS components (Worker, Job, Workyard, etc.)
- `resources/` - Global resources (Colony, SimClock, JobQueue, etc.)
- `systems/` - Core simulation systems
- `specs/` - Data specifications (OpSpec, PipelineSpec, etc.)

**Main API:**
```rust
// Core simulation loop
pub fn run_simulation() -> Result<(), SimulationError>

// Resource access
pub struct Colony { /* ... */ }
pub struct SimClock { /* ... */ }
pub struct JobQueue { /* ... */ }

// Component definitions
#[derive(Component)]
pub struct Worker { /* ... */ }

#[derive(Component)]
pub struct Job { /* ... */ }
```

### `colony-headless`

Headless server implementation for running the simulation without a graphical interface.

**Key Features:**
- REST API for remote control
- WebSocket support for real-time updates
- Configuration management
- Session persistence

**API Endpoints:**
```rust
// REST API
GET /api/status
POST /api/start
POST /api/pause
POST /api/save
POST /api/load

// WebSocket events
pub enum SimulationEvent {
    TickUpdate(TickData),
    JobCompleted(JobId),
    BlackSwanTriggered(BlackSwanType),
}
```

### `colony-desktop`

Desktop application with graphical user interface built on Bevy.

**Key Features:**
- Real-time simulation visualization
- Interactive controls
- Performance monitoring
- Mod management UI

### `colony-modsdk`

SDK for creating mods that integrate with the simulation.

**Key Types:**
```rust
// WASM Op interface
pub trait WasmOp {
    fn execute(&mut self, context: &OpContext) -> Result<OpResult, OpError>;
}

// Lua API bindings
pub struct ColonyApi {
    pub fn get_current_tick(&self) -> u64;
    pub fn enqueue_job(&self, pipeline_id: &str) -> Result<JobId, ApiError>;
    pub fn log_message(&self, level: LogLevel, message: &str);
}

// Capability system
pub enum Capability {
    SimTime,
    EnqueueJob,
    ReadKpis,
    LogMessage,
    // ... more capabilities
}
```

### `colony-mod-cli`

Command-line tool for mod development and management.

**Commands:**
```rust
// Mod scaffolding
pub fn new_mod(mod_id: &str) -> Result<(), CliError>

// Validation
pub fn validate_mod(mod_path: &Path) -> Result<ValidationReport, CliError>

// WASM compilation
pub fn build_wasm(project_path: &Path) -> Result<(), BuildError>

// Mod signing (future)
pub fn sign_mod(mod_path: &Path, key: &PrivateKey) -> Result<Signature, SignError>
```

## Utility Crates

### `xtask`

Build automation and development tools.

**Key Commands:**
```rust
// Verification suite
pub fn verify() -> Result<VerificationReport, VerifyError>

// Release candidate generation
pub fn build_rc() -> Result<RcBundle, RcError>

// Development setup
pub fn setup_dev_env() -> Result<(), SetupError>
```

## Configuration Crates

### `colony-config`

Configuration management and validation.

**Key Types:**
```rust
// Game configuration
#[derive(Deserialize)]
pub struct GameConfig {
    pub scenarios: Vec<Scenario>,
    pub tech_tree: TechTree,
    pub black_swans: Vec<BlackSwanSpec>,
}

// Mod manifest
#[derive(Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub wasm_ops: Vec<WasmOpSpec>,
    pub lua_scripts: Vec<LuaScriptSpec>,
}
```

## Testing Crates

### `colony-test-utils`

Utilities for testing and benchmarking.

**Key Features:**
- Deterministic test scenarios
- Performance benchmarking
- Mock implementations
- Test data generation

```rust
// Test scenario builder
pub struct TestScenarioBuilder {
    pub fn with_workers(count: usize) -> Self;
    pub fn with_power_capacity(capacity: f64) -> Self;
    pub fn with_black_swan(swan: BlackSwanSpec) -> Self;
    pub fn build(self) -> TestScenario;
}

// Benchmark utilities
pub fn benchmark_simulation_ticks(ticks: u64) -> BenchmarkResult;
```

## Documentation Generation

API documentation is automatically generated using `cargo doc` and published as part of the documentation site. To generate locally:

```bash
# Generate docs for all crates
cargo doc --workspace --all-features --open

# Generate docs for specific crate
cargo doc -p colony-core --open
```

The generated documentation includes:
- Complete API reference for all public types and functions
- Code examples and usage patterns
- Feature flags and conditional compilation
- Cross-references between related types
- Search functionality

## Version Compatibility

All crates follow semantic versioning (SemVer):
- **Major version**: Breaking API changes
- **Minor version**: New features, backward compatible
- **Patch version**: Bug fixes, backward compatible

The workspace uses a unified versioning scheme where all crates share the same version number for consistency.
