# Compute Colony Simulator

An asynchronous colony simulator built in Rust with Bevy ECS and Tokio. Features chill automation pacing with adjustable time scales, soft degradation leading to catastrophic spirals, real I/O parsers, and a modding system.

## Features

- **Chill Automation**: Adjustable tick scale from real-time to years per tick
- **Soft Degradation**: Systems degrade gracefully until Black Swan events trigger catastrophic failures
- **Real I/O Parsers**: UDP, CAN, TCP/HTTP, and Modbus parsers with simulators
- **Dual Mode**: Desktop (Bevy) and Headless (Axum web UI) applications
- **Modding System**: Data-driven content in TOML/RON format
- **Save/Load**: RON-based save system
- **Resource Management**: Power caps, thermal throttling, bandwidth limits
- **Maintenance Jobs**: Cool yards and reduce corruption with maintenance operations
- **Live Tuning**: Adjust resource parameters in real-time via UI sliders
- **Real I/O Processing**: UDP and HTTP simulators with realistic traffic patterns
- **Pipeline Processing**: Data-driven pipelines with UdpDemux, Decode, Kalman, Export, HttpParse, HttpExport
- **Bandwidth Integration**: I/O traffic feeds into M1 bandwidth utilization and latency tails
- **Corruption System**: Global and per-worker corruption that rises with stress and injects soft faults
- **Fault Taxonomy**: Transient, DataSkew, StickyConfig, and QueueDrop faults with different recovery mechanisms
- **Advanced Schedulers**: SJF (Shortest Job First) and EDF (Earliest Deadline First) alongside FCFS
- **Deadline Tracking**: KPI monitoring for deadline hit rates and queue health
- **GPU Farm**: VRAM management, PCIe transfer modeling, micro-batching with configurable timeouts
- **GPU Operations**: GpuPreprocess, Yolo, GpuExport with realistic cost models and VRAM requirements
- **CAN/Modbus I/O**: Simulators for fieldbus protocols with arbitration errors and loss modeling
- **Mixed Precision**: GPU performance optimization with configurable speedup multipliers
- **Black Swan Events**: Data-driven catastrophic events triggered by stress combinations
- **Pipeline Mutations**: Self-healing topology changes with heritable gene tags
- **Research/Tech Tree**: Unlock cures, mitigations, and advanced capabilities
- **Debt System**: Layered debuffs affecting power, heat, bandwidth, and fault rates
- **Rituals/Maintenance**: Time-boxed cures that consume resources and downtime
- **Game Setup Wizard**: Scenario selection, difficulty scaling, and configuration
- **Victory/Loss Conditions**: Configurable win/lose rules with SLA tracking and scoring
- **Session Management**: Pause/resume, fast-forward, autosave, and manual save/load
- **Replay System**: Deterministic replay from event logs and seeds
- **Save System**: Versioned save schema with migration support
- **WASM Operations**: Sandboxed, deterministic custom operations with fuel limits
- **Lua Event Scripts**: Fast-iteration scripting with instruction budgets and sandboxing
- **Hot Reload**: Atomic mod updates with shadow world validation and KPI monitoring
- **Mod Console**: In-game mod management with logs, docs, and dry-run testing
- **CLI Tools**: `colony-mod` CLI for scaffolding, validation, signing, and documentation
- **Security**: Capability-based permissions, resource limits, and deterministic execution

## Architecture

```
crates/
  colony-core/         # ECS types, scheduling, time scaling
  colony-io/           # I/O parsers and simulators
  colony-sim/          # Black Swan engine, thermal/corruption systems
  colony-desktop/      # Bevy desktop application
  colony-headless/     # Axum web server
  colony-mod/          # Mod loader and schema validation
  colony-content/      # Built-in vanilla content
```

## Quick Start

### Desktop Application
```bash
cargo run --bin colony-desktop
```

### Headless Server
```bash
cargo run --bin colony-headless
```

The headless server runs on `http://localhost:8080` with REST endpoints:
- `GET /state/summary` - Get colony status
- `PUT /clock/scale` - Adjust time scale
- `POST /job` - Submit new job
- `PUT /scheduler` - Change scheduler policy
- `PUT /io/udp/sim` - Configure UDP simulator
- `PUT /io/http/sim` - Configure HTTP simulator
- `POST /pipeline/{id}/enqueue` - Enqueue pipeline job
- `GET /metrics/io` - Get I/O metrics and performance data
- `PUT /sched/policy` - Change scheduler policy (FCFS, SJF, EDF)
- `GET /metrics/faults` - Get fault statistics and KPIs
- `PUT /corruption/tunables` - Configure corruption parameters
- `POST /workers/{id}/reimage` - Reset worker corruption and clear sticky faults
- `PUT /io/can/sim` - Configure CAN bus simulator
- `PUT /io/modbus/sim` - Configure Modbus simulator
- `GET /metrics/gpu` - Get GPU utilization, VRAM, and batch metrics
- `PUT /gpu/tunables` - Configure GPU batching and performance parameters
- `PUT /gpu/flags` - Toggle GPU features like mixed precision
- `GET /events` - Get Black Swan event status and eligible events
- `POST /events/{id}/fire` - Force-fire a Black Swan event (debug)
- `GET /debts` - Get active debt effects and their durations
- `GET /research` - Get research state and available techs
- `POST /research/unlock/{tech_id}` - Unlock a research technology
- `POST /rituals/{id}/start` - Start a ritual cure
- `POST /session/start` - Start a new game session
- `POST /session/pause` - Pause the current session
- `POST /session/resume` - Resume the paused session
- `PUT /session/ffwd` - Set fast-forward mode
- `GET /session/status` - Get session status and metrics
- `PUT /session/autosave` - Set autosave interval
- `POST /save/manual` - Save to a manual slot
- `POST /load/manual` - Load from a manual slot
- `POST /replay/start` - Start replay from a save
- `POST /replay/stop` - Stop current replay
- `GET /metrics/summary` - Get comprehensive metrics summary
- `GET /mods` - Get installed mods and their status
- `POST /mods/reload` - Hot reload a specific mod
- `POST /mods/enable` - Enable/disable a mod
- `POST /mods/dryrun` - Run dry-run validation for a mod
- `GET /mods/docs` - Get modding API documentation

## Controls

### Desktop
- **Time Scale Button**: Click to cycle through time scales (Real Time → 1s → 10s → 1d → 7d → 1y)
- **1-4 Keys**: Switch scheduler (1=FCFS, 2=SJF, 3=EDF, 4=HeteroAware)
- **M Key**: Schedule maintenance job to cool yards and reduce corruption
- **S Key**: Save game to `save.ron`
- **L Key**: Load game from `save.ron`

### Headless API
```bash
# Get colony status
curl http://localhost:8080/state/summary

# Set time scale to 1 second per tick
curl -X PUT http://localhost:8080/clock/scale \
  -H "Content-Type: application/json" \
  -d '{"scale": "seconds", "value": 1}'

# Submit a job
curl -X POST http://localhost:8080/job \
  -H "Content-Type: application/json" \
  -d '{
    "pipeline": ["UdpDemux", "Decode", "Kalman"],
    "qos": "Balanced",
    "deadline_ms": 50,
    "payload_sz": 4096
  }'

# Change scheduler to SJF
curl -X PUT http://localhost:8080/scheduler \
  -H "Content-Type: application/json" \
  -d '{"scheduler": "SJF"}'

# Configure UDP simulator
curl -X PUT http://localhost:8080/io/udp/sim \
  -H "Content-Type: application/json" \
  -d '{
    "rate_hz": 200.0,
    "jitter_ms": 10,
    "burstiness": 0.2,
    "loss": 0.05,
    "payload_bytes": 2048,
    "http_paths": []
  }'

# Enqueue UDP pipeline job
curl -X POST http://localhost:8080/pipeline/udp_telemetry_ingest/enqueue \
  -H "Content-Type: application/json" \
  -d '{"payload_sz": 4096}'

# Get I/O metrics
curl http://localhost:8080/metrics/io

# Change scheduler policy to EDF
curl -X PUT http://localhost:8080/sched/policy \
  -H "Content-Type: application/json" \
  -d '{"policy": "edf"}'

# Get fault metrics
curl http://localhost:8080/metrics/faults

# Configure corruption tunables
curl -X PUT http://localhost:8080/corruption/tunables \
  -H "Content-Type: application/json" \
  -d '{
    "base_fault_rate": 0.005,
    "heat_weight": 0.8,
    "bw_weight": 0.6,
    "starvation_weight": 0.4,
    "decay_per_tick": 0.002,
    "worker_decay_per_tick": 0.005,
    "recover_boost": 0.01,
    "retry_backoff_ms": 10,
    "max_retries": 3
  }'

# Reimage worker 1
curl -X POST http://localhost:8080/workers/1/reimage

# Configure CAN simulator
curl -X PUT http://localhost:8080/io/can/sim \
  -H "Content-Type: application/json" \
  -d '{
    "rate_hz": 100.0,
    "jitter_ms": 2,
    "burstiness": 0.1,
    "error_rate": 0.01,
    "id_space": [256, 2047]
  }'

# Configure Modbus simulator
curl -X PUT http://localhost:8080/io/modbus/sim \
  -H "Content-Type: application/json" \
  -d '{
    "rate_hz": 20.0,
    "loss": 0.02,
    "jitter_ms": 5,
    "fcodes": [3, 4, 6, 16],
    "payload_bytes": 512
  }'

# Get GPU metrics
curl http://localhost:8080/metrics/gpu

# Configure GPU tunables
curl -X PUT http://localhost:8080/gpu/tunables \
  -H "Content-Type: application/json" \
  -d '{
    "vram_gb": 24.0,
    "pcie_gbps": 16.0,
    "kernel_launch_ms": 0,
    "batch_max": 64,
    "batch_timeout_ms": 10,
    "mixed_precision_speedup": 1.5,
    "warmup_ms": 50
  }'

# Enable mixed precision
curl -X PUT http://localhost:8080/gpu/flags \
  -H "Content-Type: application/json" \
  -d '{"mixed_precision": true}'

# Get Black Swan events
curl http://localhost:8080/events

# Force-fire a Black Swan event (debug)
curl -X POST http://localhost:8080/events/vram_ecc_propagation/fire

# Get active debts
curl http://localhost:8080/debts

# Get research state
curl http://localhost:8080/research

# Unlock a technology
curl -X POST http://localhost:8080/research/unlock/truth_beacon

# Start a ritual cure
curl -X POST http://localhost:8080/rituals/ecc_scrub/start

# Start a new session
curl -X POST http://localhost:8080/session/start \
  -H "Content-Type: application/json" \
  -d '{
    "scenario": {
      "id": "factory_horizon_nominal",
      "name": "Factory Horizon (Nominal)",
      "description": "Standard industrial operation",
      "seed": 123,
      "difficulty": {
        "name": "Nominal",
        "power_cap_mult": 1.0,
        "heat_cap_mult": 1.0,
        "bw_total_mult": 1.0,
        "fault_rate_mult": 1.0,
        "black_swan_weight_mult": 1.0,
        "research_rate_mult": 1.0
      },
      "victory": {
        "target_uptime_days": 365,
        "min_deadline_hit_pct": 99.5,
        "max_corruption_field": 0.35,
        "observation_window_days": 7
      },
      "loss": {
        "hard_power_deficit_ticks": 1000,
        "sustained_deadline_miss_pct": 5.0,
        "max_sticky_workers": 3,
        "black_swan_chain_len": 3,
        "time_limit_days": null
      }
    },
    "mods": ["vanilla"],
    "tick_scale": "RealTime"
  }'

# Get session status
curl http://localhost:8080/session/status

# Pause session
curl -X POST http://localhost:8080/session/pause

# Set fast forward
curl -X PUT "http://localhost:8080/session/ffwd?on=true"

# Set autosave interval
curl -X PUT "http://localhost:8080/session/autosave?minutes=10"

# Save manually
curl -X POST "http://localhost:8080/save/manual?slot=my_save"

# Load manually
curl -X POST "http://localhost:8080/load/manual?slot=my_save"

# Start replay
curl -X POST http://localhost:8080/replay/start \
  -H "Content-Type: application/json" \
  -d '{"path": "my_save"}'

# Get comprehensive metrics
curl http://localhost:8080/metrics/summary

# Get installed mods
curl http://localhost:8080/mods

# Hot reload a mod
curl -X POST "http://localhost:8080/mods/reload?id=com.example.packetalchemy"

# Enable a mod
curl -X POST "http://localhost:8080/mods/enable?id=com.example.thermalboost&on=true"

# Run dry-run validation
curl -X POST "http://localhost:8080/mods/dryrun?id=com.example.packetalchemy&ticks=120"

# Get modding API docs
curl http://localhost:8080/mods/docs
```

## Modding

The Colony Simulator supports extensive modding through WASM operations, Lua scripts, and content definitions.

### CLI Tools

Use the `colony-mod` CLI tool for mod development:

```bash
# Create a new mod
colony-mod new com.yourid.packetalchemy

# Validate a mod
colony-mod validate ./mods/packetalchemy

# Sign a mod
colony-mod sign ./mods/packetalchemy --key private.pem

# Generate documentation
colony-mod docs

# List installed mods
colony-mod list
```

### Mod Structure

Create mods in the `mods/` directory:

```
mods/
  mymod/
    pipelines.toml    # Define processing pipelines
    events.toml       # Black Swan events and triggers
    tech.toml         # Technology tree
```

### Example Pipeline
```toml
[[pipeline]]
id = "udp_telemetry_ingest"
ops = ["UdpDemux", "Decode", "Kalman", "Export"]
qos = "Balanced"
deadline_ms = 50
payload_sz = 4096
```

### Example Black Swan Event
```toml
[[black_swan]]
id = "vram_ecc_propagation"
name = "VRAM: Snow of Ash"
triggers = [
  "bandwidth_util>0.95,window=5",
  "gpu_thermal_events>=3,window=3600s",
  "corruption_field>0.6"
]
effects = [
  "pipeline.insert=CRC:all_outbound",
  "debt.power_mult=1.08,duration=7d",
  "ui.illusion=temperature,-5C,12h"
]
cure = "maintenance.run=memtest_vram,parts=3,time=8h"
weight = 1.0
```

## Development Roadmap

- **M0** ✅: Basic skeleton with FCFS scheduler and time controls
- **M1** ✅: Heat integration, power throttling, thermal UI, maintenance jobs
- **M2** ✅: UDP/HTTP parsers, I/O simulators, pipeline processing
- **M3** ✅: Corruption system, soft failures, advanced schedulers (SJF, EDF)
- **M4** ✅: GPU farm with batching, CAN/Modbus I/O, VRAM management
- **M5** ✅: Black Swan events, pipeline mutations, research/tech tree
- **M6** ✅: Game setup, victory/loss conditions, sessions, replay system
- **M7** ✅: Full modding & scripting SDK, hot reload, secure sandboxing

## Building

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Build with optimizations
cargo build --release
```

## Dependencies

- **Bevy 0.14**: ECS framework for desktop app
- **Tokio**: Async runtime for I/O and headless server
- **Axum**: Web framework for headless mode
- **RON**: Save/load serialization
- **TOML**: Mod configuration format

## License

MIT License - see LICENSE file for details.
A colony simulation game written in Rust
