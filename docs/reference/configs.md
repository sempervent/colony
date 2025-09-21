# Reference: Configuration Files

This document provides comprehensive reference for all configuration files used in the Asynchronous Colony Simulator.

## Overview

The colony simulator uses a hierarchical configuration system based on TOML files. Configuration is loaded from multiple sources with the following precedence (highest to lowest):

1. Command-line arguments
2. Environment variables
3. User configuration file (`~/.config/colony/config.toml`)
4. System configuration file (`/etc/colony/config.toml`)
5. Default configuration (built into the binary)

## Main Configuration File

### `config.toml`

The main configuration file containing global settings.

```toml
# Colony Simulator Configuration

[simulation]
# Simulation timing and performance
tick_scale = 1.0                    # Time acceleration factor
max_ticks_per_frame = 100           # Maximum ticks to process per frame
enable_deterministic_mode = true    # Ensure deterministic execution
random_seed = 42                    # Seed for deterministic RNG (0 = random)

[logging]
# Logging configuration
level = "info"                      # Log level: trace, debug, info, warn, error
format = "json"                     # Log format: text, json
output = "stdout"                   # Output: stdout, stderr, file path
file_rotation = "daily"             # File rotation: daily, weekly, monthly, never
max_file_size = "100MB"             # Maximum log file size

[performance]
# Performance tuning
worker_threads = 0                  # Number of worker threads (0 = auto)
enable_profiling = false            # Enable performance profiling
memory_limit = "1GB"                # Memory limit for simulation
gc_threshold = 0.8                  # Garbage collection threshold

[mods]
# Modding system configuration
mod_directory = "mods"              # Directory containing mods
enable_hot_reload = true            # Enable hot reloading of mods
mod_validation = true               # Validate mods before loading
sandbox_timeout = 1000              # WASM/Lua execution timeout (ms)
max_mod_memory = "64MB"             # Maximum memory per mod

[headless]
# Headless server configuration
bind_address = "127.0.0.1"          # Server bind address
port = 8080                         # Server port
enable_cors = true                  # Enable CORS for web clients
api_rate_limit = 1000               # API requests per minute
websocket_buffer_size = 1024        # WebSocket buffer size

[desktop]
# Desktop application configuration
window_width = 1920                 # Initial window width
window_height = 1080                # Initial window height
fullscreen = false                  # Start in fullscreen mode
vsync = true                        # Enable vertical sync
ui_scale = 1.0                      # UI scaling factor
theme = "dark"                      # UI theme: light, dark, auto
```

## Scenario Configuration

### `scenarios/*.toml`

Scenario files define specific game scenarios with custom rules and objectives.

```toml
# Example scenario: "survival_challenge.toml"

[scenario]
id = "survival_challenge"
name = "Survival Challenge"
description = "Survive for 30 days with limited resources"
version = "1.0.0"
author = "Colony Team"
difficulty = "hard"

[objectives]
# Victory conditions
[victory_conditions]
uptime_days = 30                    # Maintain uptime for 30 days
min_deadline_hit_rate = 0.85       # Maintain 85% deadline hit rate
max_corruption_field = 0.5         # Keep corruption below 50%

# Loss conditions
[loss_conditions]
max_power_deficit_ticks = 100       # Max consecutive power deficit ticks
max_sticky_workers_percent = 0.3   # Max 30% of workers stuck
black_swan_chain_length = 3         # Max 3 consecutive Black Swans

[initial_state]
# Starting resources
power_capacity = 5000.0             # Initial power capacity (kW)
heat_dissipation = 1000.0           # Heat dissipation rate
bandwidth_capacity = 10.0           # Bandwidth capacity (Gbps)

# Starting infrastructure
cpu_workyards = 2                   # Number of CPU workyards
gpu_workyards = 1                   # Number of GPU workyards
io_workyards = 1                    # Number of I/O workyards
workers_per_workyard = 8            # Workers per workyard

# Starting research
initial_research_points = 100       # Starting research points
unlocked_technologies = ["basic_power", "basic_cooling"]

[events]
# Event configuration
black_swan_probability = 0.1        # Base Black Swan probability
corruption_growth_rate = 0.01       # Corruption field growth rate
fault_probability_multiplier = 1.5  # Fault probability multiplier

[scoring]
# Scoring system
base_score = 1000                   # Base score for completion
uptime_multiplier = 10              # Points per day of uptime
efficiency_bonus = 0.1              # Bonus for high efficiency
research_bonus = 5                  # Points per technology unlocked
```

## Operation Specifications

### `ops/*.toml`

Operation specification files define the available operations in the simulation.

```toml
# Example: "cpu_ops.toml"

[[op_specs]]
id = "cpu_decode"
name = "CPU Decode"
description = "Decode compressed data using CPU"
workyard_affinity = "cpu"
cpu_cycles = 100
duration_ticks = 5
fault_probability = 0.01
power_draw = 50.0
heat_generation = 25.0

[[op_specs]]
id = "cpu_encrypt"
name = "CPU Encryption"
description = "Encrypt data using CPU"
workyard_affinity = "cpu"
cpu_cycles = 200
duration_ticks = 8
fault_probability = 0.015
power_draw = 75.0
heat_generation = 40.0

# GPU operations
[[op_specs]]
id = "gpu_render"
name = "GPU Render"
description = "Render graphics using GPU"
workyard_affinity = "gpu"
gpu_cycles = 500
vram_mb = 128
duration_ticks = 10
fault_probability = 0.02
power_draw = 200.0
heat_generation = 100.0

# I/O operations
[[op_specs]]
id = "io_read"
name = "I/O Read"
description = "Read data from storage"
workyard_affinity = "io"
io_bytes = 1024
duration_ticks = 3
fault_probability = 0.005
bandwidth_usage = 0.1
```

## Pipeline Specifications

### `pipelines/*.toml`

Pipeline specification files define job workflows.

```toml
# Example: "video_processing.toml"

[[pipeline_specs]]
id = "video_decode_render"
name = "Video Decode and Render"
description = "Decode video stream and render frames"
qos_target = "latency"
base_deadline_ticks = 100
priority_weight = 1.0

[[pipeline_specs.ops]]
op_id = "io_fetch_video"
order = 1
timeout_ticks = 20

[[pipeline_specs.ops]]
op_id = "cpu_decode"
order = 2
timeout_ticks = 30

[[pipeline_specs.ops]]
op_id = "gpu_render"
order = 3
timeout_ticks = 50

[[pipeline_specs]]
id = "data_encryption"
name = "Data Encryption Pipeline"
description = "Encrypt sensitive data"
qos_target = "throughput"
base_deadline_ticks = 200
priority_weight = 0.8

[[pipeline_specs.ops]]
op_id = "io_read"
order = 1
timeout_ticks = 10

[[pipeline_specs.ops]]
op_id = "cpu_encrypt"
order = 2
timeout_ticks = 50

[[pipeline_specs.ops]]
op_id = "io_write"
order = 3
timeout_ticks = 10
```

## Technology Tree

### `tech_tree.toml`

Technology tree configuration defining research progression.

```toml
# Technology Tree Configuration

[[technologies]]
id = "basic_power"
name = "Basic Power Management"
description = "Improve power efficiency by 10%"
research_cost = 50
prerequisites = []
effects = [
    { type = "power_efficiency", value = 0.1 }
]

[[technologies]]
id = "advanced_cooling"
name = "Advanced Cooling Systems"
description = "Reduce heat generation by 15%"
research_cost = 100
prerequisites = ["basic_power"]
effects = [
    { type = "heat_reduction", value = 0.15 }
]

[[technologies]]
id = "fault_tolerance"
name = "Fault Tolerance"
description = "Reduce fault probability by 20%"
research_cost = 150
prerequisites = ["advanced_cooling"]
effects = [
    { type = "fault_reduction", value = 0.2 }
]

[[technologies]]
id = "corruption_mitigation"
name = "Corruption Mitigation"
description = "Reduce corruption field growth"
research_cost = 200
prerequisites = ["fault_tolerance"]
effects = [
    { type = "corruption_reduction", value = 0.25 }
]

[[technologies]]
id = "black_swan_prediction"
name = "Black Swan Prediction"
description = "Predict Black Swan events"
research_cost = 300
prerequisites = ["corruption_mitigation"]
effects = [
    { type = "black_swan_prediction", value = true }
]
```

## Black Swan Events

### `black_swans.toml`

Black Swan event definitions.

```toml
# Black Swan Events Configuration

[[black_swan_specs]]
id = "power_surge"
name = "Power Surge"
description = "Sudden increase in power consumption"
severity = "medium"
probability = 0.05
triggers = [
    { condition = "power_usage > 0.8", weight = 2.0 },
    { condition = "corruption_field > 0.6", weight = 1.5 }
]
effects = [
    { type = "power_draw_multiplier", value = 2.0, duration_ticks = 100 },
    { type = "heat_generation_multiplier", value = 1.5, duration_ticks = 100 }
]

[[black_swan_specs]]
id = "massive_fault_cascade"
name = "Massive Fault Cascade"
description = "Cascade of faults affecting multiple workers"
severity = "high"
probability = 0.02
triggers = [
    { condition = "sticky_workers > 0.2", weight = 3.0 },
    { condition = "corruption_field > 0.8", weight = 2.0 }
]
effects = [
    { type = "fault_probability_multiplier", value = 5.0, duration_ticks = 200 },
    { type = "worker_stick_probability", value = 0.3, duration_ticks = 200 }
]

[[black_swan_specs]]
id = "thermal_meltdown"
name = "Thermal Meltdown"
description = "Critical overheating of all systems"
severity = "critical"
probability = 0.01
triggers = [
    { condition = "heat > 90", weight = 4.0 },
    { condition = "cooling_efficiency < 0.5", weight = 2.0 }
]
effects = [
    { type = "performance_degradation", value = 0.8, duration_ticks = 500 },
    { type = "fault_probability_multiplier", value = 10.0, duration_ticks = 500 }
]
```

## Workyard Configurations

### `workyards/*.toml`

Workyard type definitions and configurations.

```toml
# Workyard Configurations

[[workyard_specs]]
id = "cpu_array_standard"
name = "Standard CPU Array"
type = "cpu"
capacity = 8
power_draw = 200.0
heat_generation = 100.0
bandwidth_usage = 0.5
efficiency = 1.0
fault_tolerance = 0.95

[[workyard_specs]]
id = "gpu_yard_high_end"
name = "High-End GPU Yard"
type = "gpu"
capacity = 4
power_draw = 500.0
heat_generation = 300.0
bandwidth_usage = 2.0
vram_capacity_mb = 8192
pcie_bandwidth_gbps = 16.0
efficiency = 1.2
fault_tolerance = 0.90

[[workyard_specs]]
id = "io_hub_enterprise"
name = "Enterprise I/O Hub"
type = "io"
capacity = 6
power_draw = 150.0
heat_generation = 75.0
bandwidth_usage = 5.0
storage_capacity_gb = 1000
network_latency_ms = 1.0
efficiency = 1.1
fault_tolerance = 0.98
```

## Scheduler Configurations

### `schedulers.toml`

Job scheduler policy definitions.

```toml
# Scheduler Configurations

[[scheduler_specs]]
id = "shortest_job_first"
name = "Shortest Job First"
description = "Prioritize jobs with fewer operations"
policy_type = "sjf"
parameters = [
    { name = "weight_factor", value = 1.0 },
    { name = "deadline_bonus", value = 0.5 }
]

[[scheduler_specs]]
id = "earliest_deadline_first"
name = "Earliest Deadline First"
description = "Prioritize jobs with closest deadlines"
policy_type = "edf"
parameters = [
    { name = "urgency_threshold", value = 0.8 },
    { name = "slack_factor", value = 0.2 }
]

[[scheduler_specs]]
id = "least_slack_first"
name = "Least Slack First"
description = "Prioritize jobs with least slack time"
policy_type = "lsf"
parameters = [
    { name = "slack_weight", value = 2.0 },
    { name = "priority_bonus", value = 0.3 }
]
```

## Mod Configuration

### `mod.toml`

Mod manifest files (see Modding documentation for details).

```toml
# Mod Manifest Example

[mod]
id = "example_mod"
version = "1.0.0"
name = "Example Mod"
description = "An example mod demonstrating capabilities"
author = "Mod Author"
license = "MIT"
homepage = "https://example.com/mod"

[dependencies]
colony_core = ">=1.0.0"

[[wasm_ops]]
op_id = "custom_decode"
wasm_module = "custom_ops.wasm"
wasm_function = "decode_entrypoint"
workyard_affinity = "cpu"
cpu_cycles = 150
duration_ticks = 7
fault_probability = 0.01
capabilities = ["sim_time", "log_message"]

[[lua_scripts]]
path = "scripts/event_handler.lua"
capabilities = [
    "sim_time",
    "log_message",
    "enqueue_job",
    "read_kpis",
    "listen_events"
]
```

## Environment Variables

Configuration can be overridden using environment variables with the `COLONY_` prefix:

```bash
# Override configuration values
export COLONY_SIMULATION_TICK_SCALE=2.0
export COLONY_LOGGING_LEVEL=debug
export COLONY_HEADLESS_PORT=9090
export COLONY_MODS_ENABLE_HOT_RELOAD=false
```

## Command Line Arguments

Configuration can also be overridden using command line arguments:

```bash
# Override specific settings
colony-desktop --config simulation.tick_scale=2.0 --config logging.level=debug

# Use custom config file
colony-desktop --config-file /path/to/custom/config.toml

# Override multiple settings
colony-headless --config headless.port=9090 --config mods.enable_hot_reload=false
```

## Configuration Validation

All configuration files are validated on startup. Invalid configurations will cause the application to exit with an error message indicating the specific validation failure.

### Validation Rules

- All required fields must be present
- Numeric values must be within valid ranges
- String values must match allowed patterns
- File paths must exist and be accessible
- Dependencies between configuration sections must be satisfied

### Error Reporting

Configuration errors are reported with:
- File name and line number
- Field name and current value
- Expected value or constraint
- Suggested fix

This comprehensive configuration system provides flexibility while maintaining consistency and validation across all components of the colony simulator.
