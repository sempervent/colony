# Modding: Introduction

Welcome to the Colony Simulator modding system! This guide will introduce you to the powerful modding capabilities that allow you to extend and customize the simulation.

## 🎯 What is Modding?

Modding in the Colony Simulator allows you to:

- **Create Custom Operations**: Add new data processing operations
- **Script Events**: Respond to simulation events with custom logic
- **Modify Behavior**: Change how the simulation behaves
- **Add Content**: Introduce new scenarios, technologies, and features
- **Share Creations**: Distribute your mods to the community

## 🔧 Modding Technologies

### WebAssembly (WASM)

WebAssembly provides high-performance, sandboxed execution for custom operations:

- **Performance**: Near-native execution speed
- **Safety**: Sandboxed execution environment
- **Language Support**: Write in Rust, C++, or other WASM-compatible languages
- **Memory Management**: Automatic memory management with limits

### Lua Scripting

Lua provides flexible, event-driven scripting:

- **Simplicity**: Easy-to-learn scripting language
- **Event-Driven**: Respond to simulation events
- **Rapid Development**: Quick iteration and testing
- **Integration**: Seamless integration with the simulation

## 🏗️ Mod Architecture

### Mod Structure

Every mod follows a standard structure:

```
com.example.mymod/
├── mod.toml                  # Mod manifest
├── src/
│   └── lib.rs               # WASM operations (optional)
├── scripts/
│   ├── on_tick.lua          # Lua event handlers
│   ├── on_init.lua          # Initialization script
│   └── on_fault.lua         # Fault handling script
├── assets/                  # Mod assets (optional)
│   ├── textures/
│   └── sounds/
└── README.md               # Mod documentation
```

### Mod Manifest

The `mod.toml` file defines your mod's metadata and capabilities:

```toml
id = "com.example.mymod"
name = "My Awesome Mod"
version = "1.0.0"
authors = ["Your Name"]
description = "A mod that does amazing things"

[entrypoints]
wasm_ops = ["Op_MyCustom", "Op_AnotherOp"]
lua_events = ["on_tick.lua", "on_init.lua"]

[capabilities]
sim_time = true
log_debug = true
enqueue_job = false
```

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.70+ (for WASM mods)
- **Lua** 5.4+ (for Lua scripts)
- **Colony Mod CLI**: `cargo install colony-mod`

### Creating Your First Mod

1. **Create a new mod**:
   ```bash
   colony-mod new com.example.mymod
   cd com.example.mymod
   ```

2. **Edit the manifest**:
   ```bash
   vim mod.toml
   ```

3. **Add your code**:
   ```bash
   # For WASM operations
   vim src/lib.rs
   
   # For Lua events
   vim scripts/on_tick.lua
   ```

4. **Validate your mod**:
   ```bash
   colony-mod validate .
   ```

5. **Test your mod**:
   ```bash
   colony-mod test .
   ```

## 🔧 Mod Development Workflow

### 1. Planning

Before you start coding:

- **Define Goals**: What do you want your mod to do?
- **Choose Technology**: WASM for performance, Lua for flexibility
- **Plan Capabilities**: What permissions does your mod need?
- **Design Interface**: How will your mod interact with the simulation?

### 2. Development

During development:

- **Start Simple**: Begin with basic functionality
- **Test Frequently**: Validate and test your mod regularly
- **Use Documentation**: Reference the API documentation
- **Follow Best Practices**: Use proper error handling and logging

### 3. Testing

Before releasing:

- **Unit Tests**: Test individual functions
- **Integration Tests**: Test with the simulation
- **Performance Tests**: Ensure your mod doesn't impact performance
- **Security Tests**: Verify your mod is secure

### 4. Release

When ready to share:

- **Package**: Create a distributable package
- **Document**: Write clear documentation
- **Version**: Use semantic versioning
- **Share**: Distribute to the community

## 🛡️ Security & Sandboxing

### Capability System

Mods are granted specific capabilities through the manifest:

- **sim_time**: Access to simulation time
- **log_debug**: Ability to log debug messages
- **enqueue_job**: Ability to create new jobs
- **modify_worker**: Ability to modify worker properties
- **access_resources**: Ability to read resource information

### Execution Limits

All mods run within strict limits:

#### WASM Limits
- **Fuel Limit**: 5,000,000 units (prevents infinite loops)
- **Memory Limit**: 64 MB (prevents memory exhaustion)
- **Execution Time**: 100ms per operation (prevents blocking)

#### Lua Limits
- **Instruction Budget**: 200,000 instructions per event
- **Memory Limit**: 32 MB (prevents memory exhaustion)
- **Execution Time**: 50ms per event (prevents blocking)

### Sandboxing

Mods run in isolated environments:

- **No File System Access**: Cannot read/write files
- **No Network Access**: Cannot make network requests
- **No System Calls**: Cannot execute system commands
- **Controlled API**: Only approved functions are available

## 📚 Modding Concepts

### Operations

Operations are the building blocks of data processing:

```rust
// WASM operation example
#[wasm_bindgen]
pub fn op_my_custom(input: &[u8]) -> Result<Vec<u8>, String> {
    // Process input data
    let result = process_data(input);
    Ok(result)
}
```

### Events

Events allow mods to respond to simulation changes:

```lua
-- Lua event example
function on_tick()
    local power_usage = colony.get_power_usage()
    if power_usage > 0.8 then
        colony.log_debug("High power usage detected: " .. power_usage)
    end
end
```

### Jobs

Jobs represent work to be done:

```lua
-- Create a new job
function create_custom_job()
    local job = {
        pipeline = "MyCustomPipeline",
        payload = "custom_data",
        priority = "high"
    }
    colony.enqueue_job(job)
end
```

### Resources

Resources provide access to simulation state:

```lua
-- Access resource information
function check_resources()
    local power = colony.get_power_usage()
    local heat = colony.get_heat_level()
    local corruption = colony.get_corruption_level()
    
    colony.log_debug(string.format("Power: %.2f, Heat: %.2f, Corruption: %.2f", 
        power, heat, corruption))
end
```

## 🎯 Mod Types

### Performance Mods

Enhance simulation performance:

- **Optimization Operations**: More efficient data processing
- **Caching Systems**: Reduce redundant computations
- **Load Balancing**: Distribute work more evenly
- **Resource Management**: Better resource utilization

### Content Mods

Add new content to the simulation:

- **New Operations**: Additional data processing capabilities
- **Custom Scenarios**: New simulation scenarios
- **Visual Enhancements**: Improved graphics and UI
- **Audio**: Sound effects and music

### Gameplay Mods

Modify gameplay mechanics:

- **New Victory Conditions**: Custom win/loss conditions
- **Research Trees**: Additional technology paths
- **Fault Systems**: New types of failures
- **Black Swan Events**: Custom unpredictable events

### Utility Mods

Provide helpful tools:

- **Debugging Tools**: Enhanced debugging capabilities
- **Performance Monitors**: Advanced performance tracking
- **Configuration Tools**: Easy configuration management
- **Data Export**: Export simulation data

## 🔄 Hot Reload

The Colony Simulator supports hot reloading of mods:

### How It Works

1. **File Watching**: The system monitors mod files for changes
2. **Change Detection**: When changes are detected, the mod is queued for reload
3. **Atomic Update**: The mod is replaced atomically
4. **State Preservation**: Simulation state is preserved during reload

### Benefits

- **Rapid Development**: Test changes without restarting
- **Live Debugging**: Debug issues in real-time
- **Iterative Development**: Quickly iterate on mod features
- **Seamless Updates**: Update mods without interrupting gameplay

### Limitations

- **State Dependencies**: Some state changes may require restart
- **Resource Limits**: Hot reload has resource constraints
- **Cooldown Period**: Prevents excessive reloading
- **Compatibility**: Some mods may not support hot reload

## 📖 Documentation

### API Reference

- **[WASM ABI](wasm-abi.md)**: WebAssembly interface reference
- **[Lua API](lua-api.md)**: Lua scripting interface reference
- **[Configuration](configs.md)**: Configuration file reference

### Guides

- **[WASM Operations](wasm-ops.md)**: Creating WASM operations
- **[Lua Events](lua-events.md)**: Writing Lua event handlers
- **[Hot Reload](hot-reload.md)**: Using hot reload effectively
- **[Capabilities](capabilities.md)**: Understanding the capability system
- **[CLI Tools](cli.md)**: Using the mod development CLI

### Examples

- **Basic Mod**: Simple mod with Lua events
- **WASM Operation**: Custom data processing operation
- **Complex Mod**: Advanced mod with multiple features
- **Performance Mod**: Optimization-focused mod

## 🎯 Next Steps

Now that you understand modding basics:

1. **Create Your First Mod**: Follow the getting started guide
2. **Explore Examples**: Study existing mods
3. **Read Documentation**: Learn the APIs and interfaces
4. **Join the Community**: Share your creations and get help

The modding system is designed to be powerful yet safe. Whether you're creating simple utility mods or complex gameplay modifications, the system provides the tools and safety guarantees you need.

---

**Ready to create your first mod?** 🔧⚡
