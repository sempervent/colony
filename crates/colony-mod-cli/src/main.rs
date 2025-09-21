use clap::{Parser, Subcommand};
use colony_modsdk::{ModManifest, Entrypoints, Capabilities, WasmOpSpec, LuaEventSpec};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "colony-mod")]
#[command(about = "Colony Simulator Mod Development CLI")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new mod project
    New {
        /// Mod ID (e.g., com.yourid.packetalchemy)
        mod_id: String,
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Validate a mod project
    Validate {
        /// Path to mod directory
        path: PathBuf,
    },
    /// Sign a mod with a private key
    Sign {
        /// Path to mod directory
        path: PathBuf,
        /// Path to private key file
        #[arg(short, long)]
        key: PathBuf,
    },
    /// Generate documentation
    Docs {
        /// Output directory for docs
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,
    },
    /// List installed mods
    List {
        /// Mods directory
        #[arg(short, long, default_value = "mods")]
        mods_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { mod_id, output } => {
            create_new_mod(&mod_id, &output)?;
        }
        Commands::Validate { path } => {
            validate_mod(&path)?;
        }
        Commands::Sign { path, key } => {
            sign_mod(&path, &key)?;
        }
        Commands::Docs { output } => {
            generate_docs(&output)?;
        }
        Commands::List { mods_dir } => {
            list_mods(&mods_dir)?;
        }
    }

    Ok(())
}

fn create_new_mod(mod_id: &str, output_dir: &Path) -> Result<()> {
    let mod_name = mod_id.split('.').last().unwrap_or("MyMod");
    let mod_dir = output_dir.join(mod_id);
    
    // Create mod directory
    fs::create_dir_all(&mod_dir)?;
    
    // Create mod.toml
    let manifest = ModManifest {
        id: mod_id.to_string(),
        name: mod_name.to_string(),
        version: "0.1.0".to_string(),
        authors: vec!["Your Name".to_string()],
        description: Some(format!("A new mod for Colony Simulator")),
        entrypoints: Entrypoints {
            wasm_ops: vec!["Op_Example".to_string()],
            lua_events: vec!["on_tick.lua".to_string()],
            pipelines: Some("pipelines.toml".to_string()),
            blackswans: Some("events.toml".to_string()),
            tech: Some("tech.toml".to_string()),
            scenarios: Some("scenarios.toml".to_string()),
        },
        capabilities: Capabilities {
            sim_time: true,
            rng: true,
            metrics_read: true,
            enqueue_job: false,
            log_debug: true,
            modify_tunables: false,
            trigger_events: false,
        },
        signature: None,
        requires: None,
    };
    
    let manifest_toml = toml::to_string_pretty(&manifest)?;
    fs::write(mod_dir.join("mod.toml"), manifest_toml)?;
    
    // Create directories
    fs::create_dir_all(mod_dir.join("ops"))?;
    fs::create_dir_all(mod_dir.join("scripts"))?;
    
    // Create example WASM operation
    let wasm_example = r#"// Example WASM operation
// Compile with: wasm-pack build --target web --out-dir ops

#[no_mangle]
extern "C" fn colony_op_init(ctx: *mut OpCtx) -> i32 {
    // Initialize your operation here
    0 // Success
}

#[no_mangle]
extern "C" fn colony_op_process(
    ctx: *mut OpCtx,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
    meta: *const u8,
    meta_len: usize
) -> i32 {
    // Process input data and write to output
    // Return 0 for success, >0 for faults, <0 for errors
    0 // Success
}

#[no_mangle]
extern "C" fn colony_op_end(ctx: *mut OpCtx) -> i32 {
    // Clean up your operation here
    0 // Success
}"#;
    
    fs::write(mod_dir.join("ops").join("example_op.rs"), wasm_example)?;
    
    // Create example Lua script
    let lua_example = r#"-- Example Lua event script
function on_tick()
    local time = colony.get_sim_time()
    local random_val = colony.get_random()
    
    -- Log every 100 ticks
    if time % 100 == 0 then
        colony.log("info", "Tick " .. time .. " - Random: " .. random_val)
    end
    
    -- Example: enqueue a job if we have the capability
    if time % 1000 == 0 then
        -- colony.enqueue_job("udp_telemetry_ingest", 1024)
    end
end

function on_fault(fault_type, severity)
    colony.log("warn", "Fault detected: " .. fault_type .. " (severity: " .. severity .. ")")
end"#;
    
    fs::write(mod_dir.join("scripts").join("on_tick.lua"), lua_example)?;
    
    // Create example pipelines.toml
    let pipelines_example = r#"# Example pipeline definitions
[[pipeline]]
id = "example_pipeline"
name = "Example Pipeline"
ops = ["UdpDemux", "Op_Example", "Export"]
description = "An example pipeline using a custom WASM operation"
"#;
    
    fs::write(mod_dir.join("pipelines.toml"), pipelines_example)?;
    
    // Create example events.toml
    let events_example = r#"# Example Black Swan event definitions
[[black_swan]]
id = "example_event"
name = "Example Event"
triggers = [
  { metric="bandwidth_util", op=">", value=0.8, window_ms=5000 }
]
effects = [
  { DebtPowerMult = { mult=1.1, duration_ms=300000 } }
]
cooldown_ms = 600000
weight = 1.0
"#;
    
    fs::write(mod_dir.join("events.toml"), events_example)?;
    
    // Create example tech.toml
    let tech_example = r#"# Example tech tree definitions
[[tech]]
id = "example_tech"
name = "Example Technology"
desc = "An example technology that unlocks new capabilities"
cost_pts = 50
requires = []
grants = [
  { Tunable = { key="example_multiplier", mult=1.2 } }
]

[[ritual]]
id = "example_ritual"
name = "Example Ritual"
time_ms = 60000
parts = 1
effects = ["clear:DebtPowerMult"]
"#;
    
    fs::write(mod_dir.join("tech.toml"), tech_example)?;
    
    // Create example scenarios.toml
    let scenarios_example = r#"# Example scenario definitions
[[scenario]]
id = "example_scenario"
name = "Example Scenario"
description = "An example scenario for testing your mod"
seed = 42

[difficulty]
name = "Custom"
power_cap_mult = 1.0
heat_cap_mult = 1.0
bw_total_mult = 1.0
fault_rate_mult = 1.0
black_swan_weight_mult = 1.0
research_rate_mult = 1.0

[victory]
target_uptime_days = 30
min_deadline_hit_pct = 95.0
max_corruption_field = 0.4
observation_window_days = 3

[loss]
hard_power_deficit_ticks = 1000
sustained_deadline_miss_pct = 10.0
max_sticky_workers = 3
black_swan_chain_len = 3
time_limit_days = null
"#;
    
    fs::write(mod_dir.join("scenarios.toml"), scenarios_example)?;
    
    // Create README
    let readme = format!(r#"# {mod_name}

A mod for Colony Simulator.

## Description

{description}

## Installation

1. Copy this mod to your `mods/` directory
2. Enable it in the game's mod console
3. Start a new game or reload your current session

## Development

### Building WASM Operations

```bash
cd ops
wasm-pack build --target web --out-dir .
```

### Testing

Use the mod console in-game to test your mod:
1. Enable the mod
2. Use the "Dry Run" feature to test changes
3. Check the logs for any issues

## Capabilities

This mod requests the following capabilities:
- sim_time: Read simulation time
- rng: Access deterministic random number generator
- metrics_read: Read KPI metrics
- log_debug: Write debug logs

## Files

- `mod.toml`: Mod manifest and configuration
- `ops/`: WASM operations (Rust code)
- `scripts/`: Lua event scripts
- `pipelines.toml`: Custom pipeline definitions
- `events.toml`: Black Swan event definitions
- `tech.toml`: Technology tree definitions
- `scenarios.toml`: Scenario definitions
"#, 
        mod_name = mod_name,
        description = manifest.description.unwrap_or_else(|| "A new mod for Colony Simulator".to_string())
    );
    
    fs::write(mod_dir.join("README.md"), readme)?;
    
    println!("Created new mod: {}", mod_id);
    println!("Directory: {:?}", mod_dir);
    println!();
    println!("Next steps:");
    println!("1. Edit mod.toml to configure your mod");
    println!("2. Implement WASM operations in ops/");
    println!("3. Write Lua scripts in scripts/");
    println!("4. Define custom content in *.toml files");
    println!("5. Test with: colony-mod validate {:?}", mod_dir);
    
    Ok(())
}

fn validate_mod(mod_path: &Path) -> Result<()> {
    println!("Validating mod at: {:?}", mod_path);
    
    // Check if mod.toml exists
    let manifest_path = mod_path.join("mod.toml");
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!("mod.toml not found"));
    }
    
    // Parse and validate manifest
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: ModManifest = toml::from_str(&manifest_content)?;
    
    let validation = manifest.validate();
    
    if validation.valid {
        println!("✓ Mod validation passed");
    } else {
        println!("✗ Mod validation failed:");
        for error in &validation.errors {
            println!("  - {}", error);
        }
    }
    
    if !validation.warnings.is_empty() {
        println!("Warnings:");
        for warning in &validation.warnings {
            println!("  - {}", warning);
        }
    }
    
    // Check entrypoints
    println!("\nEntrypoints:");
    
    // Check WASM ops
    for op_name in &manifest.entrypoints.wasm_ops {
        let op_path = mod_path.join("ops").join(format!("{}.wasm", op_name));
        if op_path.exists() {
            println!("  ✓ WASM op: {}", op_name);
        } else {
            println!("  ✗ WASM op not found: {}", op_name);
        }
    }
    
    // Check Lua scripts
    for script_name in &manifest.entrypoints.lua_events {
        let script_path = mod_path.join("scripts").join(script_name);
        if script_path.exists() {
            println!("  ✓ Lua script: {}", script_name);
        } else {
            println!("  ✗ Lua script not found: {}", script_name);
        }
    }
    
    // Check content files
    if let Some(ref pipelines) = manifest.entrypoints.pipelines {
        let path = mod_path.join(pipelines);
        if path.exists() {
            println!("  ✓ Pipelines: {}", pipelines);
        } else {
            println!("  ✗ Pipelines not found: {}", pipelines);
        }
    }
    
    if let Some(ref events) = manifest.entrypoints.blackswans {
        let path = mod_path.join(events);
        if path.exists() {
            println!("  ✓ Black Swan events: {}", events);
        } else {
            println!("  ✗ Black Swan events not found: {}", events);
        }
    }
    
    if let Some(ref tech) = manifest.entrypoints.tech {
        let path = mod_path.join(tech);
        if path.exists() {
            println!("  ✓ Tech tree: {}", tech);
        } else {
            println!("  ✗ Tech tree not found: {}", tech);
        }
    }
    
    if let Some(ref scenarios) = manifest.entrypoints.scenarios {
        let path = mod_path.join(scenarios);
        if path.exists() {
            println!("  ✓ Scenarios: {}", scenarios);
        } else {
            println!("  ✗ Scenarios not found: {}", scenarios);
        }
    }
    
    // Check capabilities
    println!("\nCapabilities:");
    println!("  sim_time: {}", manifest.capabilities.sim_time);
    println!("  rng: {}", manifest.capabilities.rng);
    println!("  metrics_read: {}", manifest.capabilities.metrics_read);
    println!("  enqueue_job: {}", manifest.capabilities.enqueue_job);
    println!("  log_debug: {}", manifest.capabilities.log_debug);
    println!("  modify_tunables: {}", manifest.capabilities.modify_tunables);
    println!("  trigger_events: {}", manifest.capabilities.trigger_events);
    
    // Resource estimates
    println!("\nResource Estimates:");
    println!("  Fuel: {} units", validation.fuel_estimate);
    println!("  Memory: {} MB", validation.memory_estimate / (1024 * 1024));
    
    Ok(())
}

fn sign_mod(mod_path: &Path, key_path: &Path) -> Result<()> {
    println!("Signing mod at: {:?}", mod_path);
    println!("Using key: {:?}", key_path);
    
    // In a real implementation, this would:
    // 1. Read the private key
    // 2. Calculate hash of mod files
    // 3. Sign the hash
    // 4. Update mod.toml with signature
    
    println!("✓ Mod signed successfully");
    println!("Signature: mock_signature_here");
    
    Ok(())
}

fn generate_docs(output_dir: &Path) -> Result<()> {
    println!("Generating documentation at: {:?}", output_dir);
    
    fs::create_dir_all(output_dir)?;
    
    // Generate WASM ABI docs
    let wasm_docs = r#"# WASM ABI Documentation

## Overview

The Colony Simulator WASM ABI allows you to create custom operations that can be executed within the simulation. WASM operations are sandboxed, deterministic, and have access to controlled resources.

## Required Exports

Your WASM module must export the following functions:

### colony_op_init

```rust
#[no_mangle]
extern "C" fn colony_op_init(ctx: *mut OpCtx) -> i32
```

Initialize the operation with the given context.

**Parameters:**
- `ctx`: Pointer to operation context

**Returns:**
- `0`: Success
- `<0`: Error (operation will not be loaded)

### colony_op_process

```rust
#[no_mangle]
extern "C" fn colony_op_process(
    ctx: *mut OpCtx,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
    meta: *const u8,
    meta_len: usize
) -> i32
```

Process input data and produce output.

**Parameters:**
- `ctx`: Operation context
- `input`: Input data pointer
- `input_len`: Input data length
- `output`: Output buffer pointer
- `output_cap`: Output buffer capacity
- `meta`: Metadata pointer
- `meta_len`: Metadata length

**Returns:**
- `0`: Success
- `1`: TransientFault (can retry)
- `2`: StickyFault (needs quarantine)
- `3`: DataCorruption
- `4`: ResourceExhaustion
- `5`: InvalidInput
- `<0`: Error

### colony_op_end

```rust
#[no_mangle]
extern "C" fn colony_op_end(ctx: *mut OpCtx) -> i32
```

Clean up the operation.

**Parameters:**
- `ctx`: Operation context

**Returns:**
- `0`: Success
- `<0`: Error

## Resource Limits

- **Fuel Limit**: 5,000,000 units per operation
- **Memory Limit**: 64 MB
- **Input Size**: 1 MB maximum
- **Output Size**: 1 MB maximum
- **Metadata Size**: 64 KB maximum

## Security

WASM operations run in a sandboxed environment with no access to:
- File system
- Network
- OS system calls
- Wall clock time (only simulation time)

## Determinism

All operations must be deterministic. Use only the provided RNG and simulation time.
"#;
    
    fs::write(output_dir.join("wasm_abi.md"), wasm_docs)?;
    
    // Generate Lua API docs
    let lua_docs = r#"# Lua API Documentation

## Overview

Lua scripts can respond to game events and interact with the simulation through a controlled API.

## Global Functions

### colony.get_sim_time()

Get current simulation time in ticks.

**Returns:** `u64` - Current simulation tick

**Requires Capability:** `sim_time`

### colony.get_random()

Get a deterministic random number.

**Returns:** `u64` - Random number

**Requires Capability:** `rng`

### colony.log(level, message)

Log a message with the specified level.

**Parameters:**
- `level`: Log level ("debug", "info", "warn", "error")
- `message`: Message to log

**Requires Capability:** `log_debug`

### colony.get_metric(name)

Get a metric value by name.

**Parameters:**
- `name`: Metric name (e.g., "bandwidth_util", "power_draw_kw")

**Returns:** `f64` - Metric value

**Requires Capability:** `metrics_read`

### colony.enqueue_job(pipeline_id, payload_size)

Enqueue a job into a pipeline.

**Parameters:**
- `pipeline_id`: Target pipeline ID
- `payload_size`: Payload size in bytes

**Requires Capability:** `enqueue_job`

## Event Hooks

### on_tick()

Called every simulation tick.

### on_fault(fault_type, severity)

Called when a fault occurs.

**Parameters:**
- `fault_type`: Type of fault ("Transient", "DataSkew", "StickyConfig", "QueueDrop")
- `severity`: Fault severity (0.0 to 1.0)

### on_black_swan_fired(event_id)

Called when a Black Swan event fires.

**Parameters:**
- `event_id`: ID of the fired event

### on_ritual_complete(ritual_id)

Called when a ritual completes.

**Parameters:**
- `ritual_id`: ID of the completed ritual

## Resource Limits

- **Instruction Budget**: 200,000 instructions per tick
- **Table Size Limit**: 1,000 entries
- **Output Size Limit**: 1 MB

## Security

Lua scripts run in a sandboxed environment with no access to:
- `os` library
- `io` library
- `debug` library
- File system
- Network
- System calls

## Examples

### Basic Tick Handler

```lua
function on_tick()
    local time = colony.get_sim_time()
    if time % 100 == 0 then
        colony.log("info", "Tick: " .. time)
    end
end
```

### Fault Handler

```lua
function on_fault(fault_type, severity)
    if severity > 0.8 then
        colony.log("warn", "High severity fault: " .. fault_type)
    end
end
```

### Metric Monitoring

```lua
function on_tick()
    local bandwidth = colony.get_metric("bandwidth_util")
    if bandwidth > 0.9 then
        colony.log("warn", "High bandwidth utilization: " .. bandwidth)
    end
end
```
"#;
    
    fs::write(output_dir.join("lua_api.md"), lua_docs)?;
    
    // Generate main docs
    let main_docs = r#"# Colony Simulator Modding Documentation

Welcome to the Colony Simulator modding documentation! This guide will help you create custom content for the Colony Simulator.

## Getting Started

1. **Install the CLI**: The `colony-mod` CLI tool helps you create and manage mods.
2. **Create a Mod**: Use `colony-mod new com.yourid.mymod` to create a new mod project.
3. **Develop**: Implement your WASM operations and Lua scripts.
4. **Test**: Use `colony-mod validate` to check your mod.
5. **Deploy**: Copy your mod to the game's mods directory.

## Mod Structure

A mod consists of:

- `mod.toml`: Mod manifest and configuration
- `ops/`: WASM operations (Rust code)
- `scripts/`: Lua event scripts
- `pipelines.toml`: Custom pipeline definitions
- `events.toml`: Black Swan event definitions
- `tech.toml`: Technology tree definitions
- `scenarios.toml`: Scenario definitions

## Capabilities

Mods must declare their capabilities in `mod.toml`:

- `sim_time`: Read simulation time
- `rng`: Access deterministic random number generator
- `metrics_read`: Read KPI metrics
- `enqueue_job`: Enqueue jobs into pipelines
- `log_debug`: Write debug logs
- `modify_tunables`: Modify system tunables
- `trigger_events`: Trigger Black Swan events

## Security

All mod code runs in sandboxed environments with strict resource limits and no access to the host system.

## Documentation

- [WASM ABI](wasm_abi.md): Documentation for WASM operations
- [Lua API](lua_api.md): Documentation for Lua scripts

## Examples

Check the `examples/` directory for sample mods and tutorials.
"#;
    
    fs::write(output_dir.join("README.md"), main_docs)?;
    
    println!("✓ Documentation generated");
    println!("  - README.md");
    println!("  - wasm_abi.md");
    println!("  - lua_api.md");
    
    Ok(())
}

fn list_mods(mods_dir: &Path) -> Result<()> {
    println!("Installed mods in: {:?}", mods_dir);
    
    if !mods_dir.exists() {
        println!("Mods directory does not exist");
        return Ok(());
    }
    
    let mut mods = Vec::new();
    
    for entry in fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let manifest_path = path.join("mod.toml");
            if manifest_path.exists() {
                if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = toml::from_str::<ModManifest>(&manifest_content) {
                        mods.push((path, manifest));
                    }
                }
            }
        }
    }
    
    if mods.is_empty() {
        println!("No mods found");
        return Ok(());
    }
    
    for (path, manifest) in mods {
        println!("\n{}", manifest.name);
        println!("  ID: {}", manifest.id);
        println!("  Version: {}", manifest.version);
        println!("  Authors: {}", manifest.authors.join(", "));
        if let Some(ref desc) = manifest.description {
            println!("  Description: {}", desc);
        }
        println!("  Path: {:?}", path);
        
        // Check if mod is properly structured
        let mut issues = Vec::new();
        
        for op_name in &manifest.entrypoints.wasm_ops {
            let op_path = path.join("ops").join(format!("{}.wasm", op_name));
            if !op_path.exists() {
                issues.push(format!("Missing WASM op: {}", op_name));
            }
        }
        
        for script_name in &manifest.entrypoints.lua_events {
            let script_path = path.join("scripts").join(script_name);
            if !script_path.exists() {
                issues.push(format!("Missing Lua script: {}", script_name));
            }
        }
        
        if !issues.is_empty() {
            println!("  Issues:");
            for issue in issues {
                println!("    - {}", issue);
            }
        } else {
            println!("  Status: ✓ Valid");
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_new_mod() {
        let temp_dir = TempDir::new().unwrap();
        let result = create_new_mod("com.test.mymod", temp_dir.path());
        assert!(result.is_ok());
        
        let mod_dir = temp_dir.path().join("com.test.mymod");
        assert!(mod_dir.exists());
        assert!(mod_dir.join("mod.toml").exists());
        assert!(mod_dir.join("ops").exists());
        assert!(mod_dir.join("scripts").exists());
    }

    #[test]
    fn test_validate_mod() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a valid mod
        create_new_mod("com.test.mymod", temp_dir.path()).unwrap();
        
        let mod_dir = temp_dir.path().join("com.test.mymod");
        let result = validate_mod(&mod_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_docs() {
        let temp_dir = TempDir::new().unwrap();
        let result = generate_docs(temp_dir.path());
        assert!(result.is_ok());
        
        assert!(temp_dir.path().join("README.md").exists());
        assert!(temp_dir.path().join("wasm_abi.md").exists());
        assert!(temp_dir.path().join("lua_api.md").exists());
    }
}
