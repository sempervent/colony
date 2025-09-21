# Guide: How to Build a Mod

This guide will walk you through the complete process of building a mod for the Colony Simulator, from initial setup to final deployment.

## Overview

Building a mod involves several steps:

1. **Setup**: Create the mod structure and dependencies
2. **Development**: Write the mod code and logic
3. **Testing**: Test the mod functionality
4. **Building**: Compile and package the mod
5. **Validation**: Validate the mod for correctness
6. **Signing**: Sign the mod for authenticity
7. **Deployment**: Deploy the mod to the simulation

## Prerequisites

### Required Tools

- **Rust**: Latest stable version
- **Cargo**: Rust package manager
- **wasm-pack**: For building WASM modules
- **colony-mod-cli**: Colony Mod CLI tools
- **Git**: Version control

### Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Install colony-mod-cli
cargo install colony-mod-cli

# Verify installations
rustc --version
cargo --version
wasm-pack --version
colony-mod --version
```

## Step 1: Create Mod Structure

### Initialize New Mod

```bash
# Create new mod
colony-mod new my-awesome-mod

# Navigate to mod directory
cd my-awesome-mod

# Verify structure
ls -la
```

### Mod Structure

The CLI creates this structure:

```
my-awesome-mod/
├── mod.toml              # Mod manifest
├── Cargo.toml           # Rust dependencies
├── src/                  # Rust source code
│   ├── lib.rs           # Main library
│   └── ops/             # WASM operations
├── lua/                  # Lua scripts
│   └── main.lua         # Main Lua script
├── assets/               # Mod assets
├── tests/                # Test files
├── docs/                 # Documentation
└── README.md            # Mod documentation
```

### Mod Manifest

The `mod.toml` file defines your mod:

```toml
[mod]
name = "my-awesome-mod"
version = "0.1.0"
description = "An awesome mod for Colony Simulator"
author = "Your Name"
email = "your@email.com"
license = "MIT"
repository = "https://github.com/your-username/my-awesome-mod"

[mod.capabilities]
capabilities = [
    "sim_time",
    "sim_state",
    "enqueue_job",
    "event_register"
]

[mod.dependencies]
# Add dependencies here

[mod.build]
target = "wasm32-unknown-unknown"
features = ["default"]
```

## Step 2: Develop Mod Code

### WASM Operations

Create WASM operations in `src/ops/`:

```rust
// src/ops/data_processing.rs
use colony_modsdk::ops::WasmOpSpec;
use colony_modsdk::types::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DataProcessingParams {
    pub algorithm: String,
    pub iterations: u32,
    pub threshold: f32,
}

pub struct DataProcessingOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
}

impl DataProcessingOp {
    pub fn new() -> Self {
        Self {
            name: "Data Processing".to_string(),
            description: "Processes data using various algorithms".to_string(),
            resource_cost: OpResourceCost {
                cpu: 15.0,
                gpu: 0.0,
                io: 8.0,
                time: 12.0,
                memory: 2048,
                bandwidth: 1024,
            },
        }
    }
}

impl WasmOpSpec for DataProcessingOp {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    
    fn get_description(&self) -> String {
        self.description.clone()
    }
    
    fn get_resource_cost(&self) -> OpResourceCost {
        self.resource_cost
    }
    
    fn execute(&self, context: &mut WasmOpContext) -> OpResult {
        // Get input data
        let input_data = context.get_input_data()?;
        
        // Parse parameters
        let params: DataProcessingParams = serde_json::from_slice(&input_data)
            .map_err(|e| OpError::DeserializationError(e.to_string()))?;
        
        // Process data
        let result = self.process_data(&params)?;
        
        // Set output data
        context.set_output_data(&result)?;
        
        OpResult::Success
    }
}

impl DataProcessingOp {
    fn process_data(&self, params: &DataProcessingParams) -> Result<Vec<u8>, OpError> {
        match params.algorithm.as_str() {
            "sort" => self.sort_data(params),
            "filter" => self.filter_data(params),
            "transform" => self.transform_data(params),
            _ => Err(OpError::InvalidParameter("Unknown algorithm".to_string())),
        }
    }
    
    fn sort_data(&self, params: &DataProcessingParams) -> Result<Vec<u8>, OpError> {
        // Sorting implementation
        Ok(vec![])
    }
    
    fn filter_data(&self, params: &DataProcessingParams) -> Result<Vec<u8>, OpError> {
        // Filtering implementation
        Ok(vec![])
    }
    
    fn transform_data(&self, params: &DataProcessingParams) -> Result<Vec<u8>, OpError> {
        // Transformation implementation
        Ok(vec![])
    }
}
```

### Lua Scripts

Create Lua scripts in `lua/`:

```lua
-- lua/main.lua
local function on_mod_loaded(event_data)
    print("My Awesome Mod loaded!")
    
    -- Check capabilities
    if colony.capabilities.has("sim_time") then
        print("Mod has sim_time capability")
    end
    
    -- Register event handlers
    colony.events.register("tick_start", on_tick_start)
    colony.events.register("job_created", on_job_created)
end

local function on_tick_start(event_data)
    local current_tick = colony.time.get_current_tick()
    
    -- Perform tick-based operations
    if current_tick % 100 == 0 then
        print("Reached tick milestone: " .. current_tick)
        
        -- Create periodic job
        if colony.capabilities.has("enqueue_job") then
            local job = {
                id = "periodic_job_" .. current_tick,
                pipeline = "data_processing",
                priority = 5,
                deadline = current_tick + 1000
            }
            
            colony.jobs.enqueue(job)
            print("Enqueued periodic job: " .. job.id)
        end
    end
end

local function on_job_created(event_data)
    local job = event_data.job
    
    -- Log job creation
    print("Job created: " .. job.id)
    print("Pipeline: " .. job.pipeline)
    print("Priority: " .. job.priority)
    
    -- Apply custom logic
    if job.pipeline == "data_processing" then
        -- Boost priority for data processing jobs
        job.priority = math.min(job.priority + 1, 10)
        print("Boosted priority for data processing job")
    end
end

-- Register mod loaded handler
colony.events.register("mod_loaded", on_mod_loaded)
```

### Main Library

Update `src/lib.rs`:

```rust
// src/lib.rs
use wasm_bindgen::prelude::*;
use colony_modsdk::ops::WasmOpSpec;

mod ops;

// Export operations
pub use ops::data_processing::DataProcessingOp;

// Export operation creation function
#[wasm_bindgen]
pub fn create_data_processing_operation() -> JsValue {
    let op = DataProcessingOp::new();
    serde_wasm_bindgen::to_value(&op).unwrap()
}

// Export operation metadata
#[wasm_bindgen]
pub fn get_operation_metadata() -> JsValue {
    let metadata = OperationMetadata {
        name: "Data Processing".to_string(),
        description: "Processes data using various algorithms".to_string(),
        version: "1.0.0".to_string(),
        author: "Your Name".to_string(),
        resource_cost: OpResourceCost {
            cpu: 15.0,
            gpu: 0.0,
            io: 8.0,
            time: 12.0,
            memory: 2048,
            bandwidth: 1024,
        },
    };
    
    serde_wasm_bindgen::to_value(&metadata).unwrap()
}
```

## Step 3: Configure Dependencies

### Cargo.toml

Update `Cargo.toml` with required dependencies:

```toml
[package]
name = "my-awesome-mod"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
colony-modsdk = "0.9.0"
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
serde_json = "1.0"

[dependencies.web-sys]
version = "0.3"
features = [
    "console",
    "js-sys",
]

[dev-dependencies]
colony-modsdk-testing = "0.9.0"
```

### Mod Manifest

Update `mod.toml` with operation definitions:

```toml
[mod]
name = "my-awesome-mod"
version = "0.1.0"
description = "An awesome mod for Colony Simulator"
author = "Your Name"
email = "your@email.com"
license = "MIT"
repository = "https://github.com/your-username/my-awesome-mod"

[mod.capabilities]
capabilities = [
    "sim_time",
    "sim_state",
    "enqueue_job",
    "event_register",
    "event_emit"
]

[mod.operations]
operations = [
    { name = "data_processing", type = "wasm", file = "src/ops/data_processing.rs" }
]

[mod.scripts]
scripts = [
    { name = "main", file = "lua/main.lua" }
]

[mod.dependencies]
# Add dependencies here

[mod.build]
target = "wasm32-unknown-unknown"
features = ["default"]
```

## Step 4: Write Tests

### Unit Tests

Create unit tests in `tests/`:

```rust
// tests/unit_tests.rs
use colony_modsdk_testing::*;
use my_awesome_mod::DataProcessingOp;

#[test]
fn test_data_processing_operation() {
    let op = DataProcessingOp::new();
    let mut context = create_test_wasm_context();
    
    // Set test input data
    let test_params = DataProcessingParams {
        algorithm: "sort".to_string(),
        iterations: 1,
        threshold: 0.5,
    };
    
    let input_bytes = serde_json::to_vec(&test_params).unwrap();
    context.set_input_data(&input_bytes);
    
    // Execute operation
    let result = op.execute(&mut context);
    assert!(result.is_ok());
    
    // Verify output
    let output_data = context.get_output_data().unwrap();
    assert!(!output_data.is_empty());
}

#[test]
fn test_operation_metadata() {
    let op = DataProcessingOp::new();
    
    assert_eq!(op.get_name(), "Data Processing");
    assert_eq!(op.get_description(), "Processes data using various algorithms");
    
    let cost = op.get_resource_cost();
    assert_eq!(cost.cpu, 15.0);
    assert_eq!(cost.gpu, 0.0);
    assert_eq!(cost.io, 8.0);
    assert_eq!(cost.time, 12.0);
    assert_eq!(cost.memory, 2048);
    assert_eq!(cost.bandwidth, 1024);
}
```

### Integration Tests

Create integration tests:

```rust
// tests/integration_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_mod_integration() {
    let mut mod_loader = create_test_mod_loader();
    let mod_path = ".";
    
    // Load mod
    mod_loader.load_mod(mod_path).unwrap();
    
    // Verify mod is loaded
    assert!(mod_loader.is_mod_loaded("my-awesome-mod"));
    
    // Test operation
    let op = mod_loader.get_operation("data_processing").unwrap();
    let mut context = create_test_wasm_context();
    
    let result = op.execute(&mut context);
    assert!(result.is_ok());
}

#[test]
fn test_lua_script_integration() {
    let mut lua_host = create_test_lua_host();
    let script_path = "lua/main.lua";
    
    // Load script
    lua_host.load_script(script_path).unwrap();
    
    // Test script execution
    let result = lua_host.execute_script("main");
    assert!(result.is_ok());
}
```

## Step 5: Build the Mod

### Build WASM Module

```bash
# Build WASM module
wasm-pack build --target web --out-dir pkg

# Verify build output
ls -la pkg/
```

### Build with CLI

```bash
# Build using colony-mod CLI
colony-mod build

# Build with release optimizations
colony-mod build --release

# Build with specific features
colony-mod build --features "gpu,network"
```

### Build Output

The build process creates:

```
pkg/
├── my_awesome_mod.js        # JavaScript bindings
├── my_awesome_mod_bg.wasm   # WASM module
├── my_awesome_mod.d.ts      # TypeScript definitions
└── package.json             # Package metadata
```

## Step 6: Test the Mod

### Run Tests

```bash
# Run unit tests
colony-mod test --test unit_tests

# Run integration tests
colony-mod test --test integration_tests

# Run all tests
colony-mod test

# Run tests with coverage
colony-mod test --coverage
```

### Test Output

```bash
$ colony-mod test
Testing mod: my-awesome-mod
✓ Unit tests passed (5/5)
✓ Integration tests passed (3/3)
✓ WASM tests passed (2/2)
✓ Lua tests passed (4/4)

Test coverage: 95.2%
All tests passed successfully!
```

## Step 7: Validate the Mod

### Validate Mod Structure

```bash
# Validate mod
colony-mod validate

# Validate with verbose output
colony-mod validate --verbose

# Validate with strict checks
colony-mod validate --strict
```

### Validation Output

```bash
$ colony-mod validate
Validating mod: my-awesome-mod
✓ Manifest validation passed
✓ Dependency validation passed
✓ Capability validation passed
✓ File structure validation passed
✓ Code validation passed
✓ Asset validation passed
✓ Security validation passed

Mod validation completed successfully!
```

## Step 8: Sign the Mod

### Generate Signing Key

```bash
# Generate private key
openssl genrsa -out private.key 2048

# Generate public key
openssl rsa -in private.key -pubout -out public.key

# Generate certificate
openssl req -new -x509 -key private.key -out certificate.crt -days 365
```

### Sign the Mod

```bash
# Sign mod
colony-mod sign --key private.key --certificate certificate.crt

# Sign with passphrase
colony-mod sign --key private.key --passphrase "your-passphrase"

# Sign with timestamp
colony-mod sign --timestamp
```

### Signing Output

```bash
$ colony-mod sign
Signing mod: my-awesome-mod
✓ Generating signature
✓ Creating signature file
✓ Validating signature
✓ Updating manifest

Mod signed successfully!
Signature: my-awesome-mod.sig
```

## Step 9: Package the Mod

### Create Mod Package

```bash
# Package mod
colony-mod package

# Package with specific format
colony-mod package --format tar.gz

# Package with compression
colony-mod package --compress gzip
```

### Package Contents

The package includes:

```
my-awesome-mod-0.1.0.tar.gz
├── mod.toml              # Mod manifest
├── pkg/                  # WASM modules
│   ├── my_awesome_mod.js
│   ├── my_awesome_mod_bg.wasm
│   └── my_awesome_mod.d.ts
├── lua/                  # Lua scripts
│   └── main.lua
├── assets/               # Mod assets
├── docs/                 # Documentation
├── tests/                # Test files
├── my-awesome-mod.sig    # Digital signature
└── checksums.txt         # File checksums
```

## Step 10: Deploy the Mod

### Install the Mod

```bash
# Install mod
colony-mod install my-awesome-mod-0.1.0.tar.gz

# Install with verification
colony-mod install --verify my-awesome-mod-0.1.0.tar.gz

# Install with dependencies
colony-mod install --with-dependencies my-awesome-mod-0.1.0.tar.gz
```

### Verify Installation

```bash
# List installed mods
colony-mod list

# Show mod information
colony-mod info my-awesome-mod

# Show mod status
colony-mod status my-awesome-mod
```

## Step 11: Test in Simulation

### Load Mod in Simulation

```bash
# Start simulation with mod
colony-simulator --mod my-awesome-mod

# Start with specific scenario
colony-simulator --scenario test_scenario --mod my-awesome-mod

# Start with debug mode
colony-simulator --debug --mod my-awesome-mod
```

### Monitor Mod Performance

```bash
# Monitor mod performance
colony-simulator --monitor --mod my-awesome-mod

# Monitor with metrics
colony-simulator --metrics --mod my-awesome-mod

# Monitor with logging
colony-simulator --log-level debug --mod my-awesome-mod
```

## Troubleshooting

### Common Build Issues

1. **Rust Toolchain**: Ensure latest stable Rust is installed
2. **Dependencies**: Check all dependencies are available
3. **WASM Target**: Ensure wasm32-unknown-unknown target is installed
4. **Permissions**: Check file permissions and access rights

### Common Test Issues

1. **Test Environment**: Ensure test environment is properly configured
2. **Dependencies**: Check test dependencies are available
3. **Configuration**: Verify test configuration is correct
4. **Resources**: Ensure sufficient resources for testing

### Common Validation Issues

1. **Manifest**: Check mod.toml syntax and content
2. **Capabilities**: Verify capability declarations
3. **Dependencies**: Check dependency specifications
4. **File Structure**: Verify file structure matches requirements

## Best Practices

### Development

1. **Version Control**: Use Git for version control
2. **Documentation**: Document your mod thoroughly
3. **Testing**: Write comprehensive tests
4. **Code Quality**: Follow Rust best practices
5. **Security**: Implement proper security measures

### Building

1. **Incremental Builds**: Use incremental builds for development
2. **Release Builds**: Use release builds for production
3. **Optimization**: Optimize for performance
4. **Validation**: Always validate before building
5. **Signing**: Sign mods for authenticity

### Deployment

1. **Testing**: Test thoroughly before deployment
2. **Documentation**: Provide clear documentation
3. **Support**: Provide support for users
4. **Updates**: Plan for future updates
5. **Monitoring**: Monitor mod performance

## Conclusion

Building a mod for the Colony Simulator involves several steps:

1. **Setup**: Create mod structure and dependencies
2. **Development**: Write mod code and logic
3. **Testing**: Test mod functionality
4. **Building**: Compile and package the mod
5. **Validation**: Validate the mod for correctness
6. **Signing**: Sign the mod for authenticity
7. **Deployment**: Deploy the mod to the simulation

By following these steps and best practices, you can create effective, reliable mods that enhance the Colony Simulator's capabilities.

---

**Building mods is a rewarding process that allows you to extend and customize the Colony Simulator.** 🏭🛠️
