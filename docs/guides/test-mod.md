# Guide: How to Test a Mod

This guide will walk you through the complete process of testing a mod for the Colony Simulator, from unit tests to integration tests and performance testing.

## Overview

Testing a mod involves several levels:

1. **Unit Tests**: Test individual components
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows
4. **Performance Tests**: Test performance and resource usage
5. **Security Tests**: Test security and sandboxing
6. **Compatibility Tests**: Test compatibility with different environments

## Testing Framework

### Colony Mod SDK Testing

The Colony Mod SDK provides comprehensive testing tools:

```rust
use colony_modsdk_testing::*;

// Test utilities
let mut context = create_test_wasm_context();
let mut simulation = create_test_simulation();
let mut mod_loader = create_test_mod_loader();
```

### Test Structure

```
tests/
├── unit_tests.rs          # Unit tests
├── integration_tests.rs   # Integration tests
├── e2e_tests.rs          # End-to-end tests
├── performance_tests.rs  # Performance tests
├── security_tests.rs     # Security tests
└── fixtures/             # Test fixtures
    ├── test_data.json
    ├── test_config.toml
    └── test_assets/
```

## Unit Tests

### Testing WASM Operations

```rust
// tests/unit_tests.rs
use colony_modsdk_testing::*;
use my_mod::DataProcessingOp;

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
fn test_operation_resource_cost() {
    let op = DataProcessingOp::new();
    let cost = op.get_resource_cost();
    
    assert_eq!(cost.cpu, 15.0);
    assert_eq!(cost.gpu, 0.0);
    assert_eq!(cost.io, 8.0);
    assert_eq!(cost.time, 12.0);
    assert_eq!(cost.memory, 2048);
    assert_eq!(cost.bandwidth, 1024);
}

#[test]
fn test_operation_metadata() {
    let op = DataProcessingOp::new();
    
    assert_eq!(op.get_name(), "Data Processing");
    assert_eq!(op.get_description(), "Processes data using various algorithms");
}

#[test]
fn test_operation_error_handling() {
    let op = DataProcessingOp::new();
    let mut context = create_test_wasm_context();
    
    // Test with invalid input
    context.set_input_data(b"invalid json");
    
    let result = op.execute(&mut context);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        OpError::DeserializationError(_) => {
            // Expected error
        },
        _ => panic!("Unexpected error type"),
    }
}

#[test]
fn test_operation_with_different_algorithms() {
    let op = DataProcessingOp::new();
    let mut context = create_test_wasm_context();
    
    let algorithms = vec!["sort", "filter", "transform"];
    
    for algorithm in algorithms {
        let test_params = DataProcessingParams {
            algorithm: algorithm.to_string(),
            iterations: 1,
            threshold: 0.5,
        };
        
        let input_bytes = serde_json::to_vec(&test_params).unwrap();
        context.set_input_data(&input_bytes);
        
        let result = op.execute(&mut context);
        assert!(result.is_ok(), "Failed for algorithm: {}", algorithm);
    }
}
```

### Testing Lua Scripts

```rust
// tests/unit_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_lua_script_loading() {
    let mut lua_host = create_test_lua_host();
    let script_path = "lua/main.lua";
    
    // Load script
    let result = lua_host.load_script(script_path);
    assert!(result.is_ok());
    
    // Verify script is loaded
    assert!(lua_host.is_script_loaded("main"));
}

#[test]
fn test_lua_script_execution() {
    let mut lua_host = create_test_lua_host();
    let script_path = "lua/main.lua";
    
    // Load script
    lua_host.load_script(script_path).unwrap();
    
    // Execute script
    let result = lua_host.execute_script("main");
    assert!(result.is_ok());
}

#[test]
fn test_lua_script_error_handling() {
    let mut lua_host = create_test_lua_host();
    
    // Test with invalid script
    let result = lua_host.load_script("nonexistent.lua");
    assert!(result.is_err());
}

#[test]
fn test_lua_script_capabilities() {
    let mut lua_host = create_test_lua_host();
    let script_path = "lua/main.lua";
    
    // Load script
    lua_host.load_script(script_path).unwrap();
    
    // Test capability checks
    assert!(lua_host.has_capability("sim_time"));
    assert!(lua_host.has_capability("event_register"));
    assert!(!lua_host.has_capability("system_control"));
}
```

## Integration Tests

### Testing Mod Loading

```rust
// tests/integration_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_mod_loading() {
    let mut mod_loader = create_test_mod_loader();
    let mod_path = ".";
    
    // Load mod
    let result = mod_loader.load_mod(mod_path);
    assert!(result.is_ok());
    
    // Verify mod is loaded
    assert!(mod_loader.is_mod_loaded("my-awesome-mod"));
    
    // Verify mod metadata
    let mod_info = mod_loader.get_mod_info("my-awesome-mod").unwrap();
    assert_eq!(mod_info.name, "my-awesome-mod");
    assert_eq!(mod_info.version, "0.1.0");
}

#[test]
fn test_mod_operations() {
    let mut mod_loader = create_test_mod_loader();
    let mod_path = ".";
    
    // Load mod
    mod_loader.load_mod(mod_path).unwrap();
    
    // Test operation loading
    let op = mod_loader.get_operation("data_processing");
    assert!(op.is_some());
    
    // Test operation execution
    let mut context = create_test_wasm_context();
    let result = op.unwrap().execute(&mut context);
    assert!(result.is_ok());
}

#[test]
fn test_mod_scripts() {
    let mut mod_loader = create_test_mod_loader();
    let mod_path = ".";
    
    // Load mod
    mod_loader.load_mod(mod_path).unwrap();
    
    // Test script loading
    let script = mod_loader.get_script("main");
    assert!(script.is_some());
    
    // Test script execution
    let result = script.unwrap().execute();
    assert!(result.is_ok());
}

#[test]
fn test_mod_capabilities() {
    let mut mod_loader = create_test_mod_loader();
    let mod_path = ".";
    
    // Load mod
    mod_loader.load_mod(mod_path).unwrap();
    
    // Test capability validation
    let mod_info = mod_loader.get_mod_info("my-awesome-mod").unwrap();
    assert!(mod_info.capabilities.contains(&"sim_time".to_string()));
    assert!(mod_info.capabilities.contains(&"event_register".to_string()));
}
```

### Testing Mod Integration

```rust
// tests/integration_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_mod_in_simulation() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod into simulation
    simulation.load_mod(mod_path).unwrap();
    
    // Verify mod is active
    assert!(simulation.is_mod_active("my-awesome-mod"));
    
    // Run simulation for several ticks
    for _ in 0..100 {
        simulation.tick();
    }
    
    // Verify mod is still active
    assert!(simulation.is_mod_active("my-awesome-mod"));
}

#[test]
fn test_mod_event_handling() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Trigger events
    simulation.trigger_event("tick_start");
    simulation.trigger_event("job_created");
    
    // Verify events are handled
    let event_log = simulation.get_event_log();
    assert!(event_log.contains("tick_start"));
    assert!(event_log.contains("job_created"));
}

#[test]
fn test_mod_resource_usage() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Monitor resource usage
    let initial_resources = simulation.get_resource_usage();
    
    // Run simulation
    for _ in 0..1000 {
        simulation.tick();
    }
    
    let final_resources = simulation.get_resource_usage();
    
    // Verify resource usage is reasonable
    assert!(final_resources.cpu_usage < 100.0);
    assert!(final_resources.memory_usage < 1024 * 1024); // 1MB
}
```

## End-to-End Tests

### Testing Complete Workflows

```rust
// tests/e2e_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_complete_mod_workflow() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Create test scenario
    let scenario = create_test_scenario();
    simulation.set_scenario(scenario);
    
    // Run complete simulation
    let result = simulation.run_until_completion();
    assert!(result.is_ok());
    
    // Verify mod contributed to simulation
    let mod_contributions = simulation.get_mod_contributions("my-awesome-mod");
    assert!(mod_contributions.jobs_created > 0);
    assert!(mod_contributions.events_handled > 0);
}

#[test]
fn test_mod_with_different_scenarios() {
    let scenarios = vec![
        "basic_scenario",
        "stress_scenario",
        "fault_scenario",
        "resource_scenario",
    ];
    
    for scenario_name in scenarios {
        let mut simulation = create_test_simulation();
        let mod_path = ".";
        
        // Load mod
        simulation.load_mod(mod_path).unwrap();
        
        // Set scenario
        let scenario = create_scenario(scenario_name);
        simulation.set_scenario(scenario);
        
        // Run simulation
        let result = simulation.run_for_ticks(1000);
        assert!(result.is_ok(), "Failed for scenario: {}", scenario_name);
    }
}

#[test]
fn test_mod_with_other_mods() {
    let mut simulation = create_test_simulation();
    
    // Load multiple mods
    simulation.load_mod(".").unwrap(); // Our mod
    simulation.load_mod("../other-mod").unwrap(); // Another mod
    
    // Verify both mods are active
    assert!(simulation.is_mod_active("my-awesome-mod"));
    assert!(simulation.is_mod_active("other-mod"));
    
    // Run simulation
    let result = simulation.run_for_ticks(1000);
    assert!(result.is_ok());
    
    // Verify no conflicts
    let conflicts = simulation.get_mod_conflicts();
    assert!(conflicts.is_empty());
}
```

## Performance Tests

### Testing Performance Metrics

```rust
// tests/performance_tests.rs
use colony_modsdk_testing::*;
use std::time::Instant;

#[test]
fn test_operation_performance() {
    let op = DataProcessingOp::new();
    let mut context = create_test_wasm_context();
    
    // Set test data
    let test_params = DataProcessingParams {
        algorithm: "sort".to_string(),
        iterations: 1000,
        threshold: 0.5,
    };
    
    let input_bytes = serde_json::to_vec(&test_params).unwrap();
    context.set_input_data(&input_bytes);
    
    // Measure execution time
    let start = Instant::now();
    let result = op.execute(&mut context);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000); // Should complete in under 1 second
}

#[test]
fn test_mod_memory_usage() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Measure initial memory usage
    let initial_memory = simulation.get_memory_usage();
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Measure memory usage after loading
    let loaded_memory = simulation.get_memory_usage();
    let memory_increase = loaded_memory - initial_memory;
    
    // Verify memory usage is reasonable
    assert!(memory_increase < 10 * 1024 * 1024); // Less than 10MB
    
    // Run simulation
    for _ in 0..1000 {
        simulation.tick();
    }
    
    // Measure memory usage after running
    let final_memory = simulation.get_memory_usage();
    let total_memory_increase = final_memory - initial_memory;
    
    // Verify no memory leaks
    assert!(total_memory_increase < 20 * 1024 * 1024); // Less than 20MB
}

#[test]
fn test_mod_cpu_usage() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Monitor CPU usage
    let mut cpu_samples = Vec::new();
    
    for _ in 0..100 {
        let start = Instant::now();
        simulation.tick();
        let duration = start.elapsed();
        cpu_samples.push(duration);
    }
    
    // Calculate average CPU usage
    let total_time: u128 = cpu_samples.iter().map(|d| d.as_micros()).sum();
    let average_time = total_time / cpu_samples.len() as u128;
    
    // Verify CPU usage is reasonable
    assert!(average_time < 10000); // Less than 10ms per tick
}

#[test]
fn test_mod_throughput() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Measure throughput
    let start = Instant::now();
    let mut operations_completed = 0;
    
    for _ in 0..1000 {
        simulation.tick();
        operations_completed += simulation.get_operations_completed();
    }
    
    let duration = start.elapsed();
    let throughput = operations_completed as f64 / duration.as_secs_f64();
    
    // Verify throughput is reasonable
    assert!(throughput > 100.0); // More than 100 operations per second
}
```

## Security Tests

### Testing Sandboxing

```rust
// tests/security_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_wasm_sandbox_isolation() {
    let mut wasm_host = create_test_wasm_host();
    let malicious_module = create_malicious_wasm_module();
    
    // Attempt to load malicious module
    let result = wasm_host.load_module(malicious_module);
    assert!(result.is_err());
    
    // Verify sandbox isolation
    assert!(!wasm_host.can_access_system_resources());
    assert!(!wasm_host.can_modify_system_state());
}

#[test]
fn test_lua_sandbox_isolation() {
    let mut lua_host = create_test_lua_host();
    let malicious_script = create_malicious_lua_script();
    
    // Attempt to load malicious script
    let result = lua_host.load_script(malicious_script);
    assert!(result.is_err());
    
    // Verify sandbox isolation
    assert!(!lua_host.can_access_file_system());
    assert!(!lua_host.can_execute_system_commands());
}

#[test]
fn test_capability_enforcement() {
    let mut mod_loader = create_test_mod_loader();
    let mod_path = ".";
    
    // Load mod
    mod_loader.load_mod(mod_path).unwrap();
    
    // Test capability enforcement
    let mod_info = mod_loader.get_mod_info("my-awesome-mod").unwrap();
    
    // Verify mod can only use declared capabilities
    assert!(mod_info.capabilities.contains(&"sim_time".to_string()));
    assert!(!mod_info.capabilities.contains(&"system_control".to_string()));
    
    // Test capability violations
    let result = mod_loader.attempt_unauthorized_access("system_control");
    assert!(result.is_err());
}

#[test]
fn test_resource_limits() {
    let mut simulation = create_test_simulation();
    let mod_path = ".";
    
    // Load mod
    simulation.load_mod(mod_path).unwrap();
    
    // Test resource limit enforcement
    let resource_limits = simulation.get_resource_limits();
    
    // Attempt to exceed limits
    let result = simulation.attempt_resource_exhaustion();
    assert!(result.is_err());
    
    // Verify limits are enforced
    let current_usage = simulation.get_resource_usage();
    assert!(current_usage.cpu_usage <= resource_limits.max_cpu_usage);
    assert!(current_usage.memory_usage <= resource_limits.max_memory_usage);
}
```

## Compatibility Tests

### Testing Different Environments

```rust
// tests/compatibility_tests.rs
use colony_modsdk_testing::*;

#[test]
fn test_mod_compatibility() {
    let environments = vec![
        "linux-x86_64",
        "windows-x86_64",
        "macos-x86_64",
        "macos-aarch64",
    ];
    
    for environment in environments {
        let mut simulation = create_test_simulation_for_environment(environment);
        let mod_path = ".";
        
        // Load mod
        let result = simulation.load_mod(mod_path);
        assert!(result.is_ok(), "Failed for environment: {}", environment);
        
        // Run simulation
        let result = simulation.run_for_ticks(100);
        assert!(result.is_ok(), "Failed for environment: {}", environment);
    }
}

#[test]
fn test_mod_with_different_versions() {
    let versions = vec!["0.8.0", "0.9.0", "1.0.0"];
    
    for version in versions {
        let mut simulation = create_test_simulation_with_version(version);
        let mod_path = ".";
        
        // Load mod
        let result = simulation.load_mod(mod_path);
        assert!(result.is_ok(), "Failed for version: {}", version);
        
        // Run simulation
        let result = simulation.run_for_ticks(100);
        assert!(result.is_ok(), "Failed for version: {}", version);
    }
}

#[test]
fn test_mod_with_different_configurations() {
    let configurations = vec![
        "minimal_config",
        "default_config",
        "full_config",
        "custom_config",
    ];
    
    for config_name in configurations {
        let mut simulation = create_test_simulation_with_config(config_name);
        let mod_path = ".";
        
        // Load mod
        let result = simulation.load_mod(mod_path);
        assert!(result.is_ok(), "Failed for configuration: {}", config_name);
        
        // Run simulation
        let result = simulation.run_for_ticks(100);
        assert!(result.is_ok(), "Failed for configuration: {}", config_name);
    }
}
```

## Running Tests

### Using Cargo

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_data_processing_operation

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

### Using Colony Mod CLI

```bash
# Run all tests
colony-mod test

# Run specific test suite
colony-mod test --test unit_tests

# Run tests with coverage
colony-mod test --coverage

# Run tests with verbose output
colony-mod test --verbose

# Run tests in parallel
colony-mod test --parallel
```

### Test Configuration

```toml
# In test configuration
[test]
timeout = 300
parallel = true
coverage = true
verbose = false

[test.unit_tests]
enabled = true
timeout = 60

[test.integration_tests]
enabled = true
timeout = 180

[test.performance_tests]
enabled = true
timeout = 600
iterations = 1000

[test.security_tests]
enabled = true
timeout = 120
strict_mode = true
```

## Test Fixtures

### Test Data

```json
// tests/fixtures/test_data.json
{
  "test_params": {
    "algorithm": "sort",
    "iterations": 100,
    "threshold": 0.5
  },
  "test_data": [
    {"id": 1, "value": 10},
    {"id": 2, "value": 5},
    {"id": 3, "value": 15}
  ],
  "expected_results": {
    "sorted": [
      {"id": 2, "value": 5},
      {"id": 1, "value": 10},
      {"id": 3, "value": 15}
    ]
  }
}
```

### Test Configuration

```toml
# tests/fixtures/test_config.toml
[simulation]
tick_rate = 60
max_ticks = 1000

[resources]
power_limit = 1000.0
bandwidth_limit = 100.0
heat_limit = 80.0

[mods]
enabled = ["my-awesome-mod"]
capabilities = ["sim_time", "event_register"]
```

## Best Practices

### Test Design

1. **Test Isolation**: Each test should be independent
2. **Clear Assertions**: Use clear, specific assertions
3. **Test Data**: Use realistic test data
4. **Error Cases**: Test error conditions
5. **Edge Cases**: Test boundary conditions

### Test Organization

1. **Logical Grouping**: Group related tests together
2. **Descriptive Names**: Use descriptive test names
3. **Setup/Teardown**: Properly set up and clean up
4. **Documentation**: Document test purposes
5. **Maintenance**: Keep tests up to date

### Performance Testing

1. **Realistic Scenarios**: Use realistic test scenarios
2. **Multiple Runs**: Run tests multiple times
3. **Statistical Analysis**: Analyze performance statistically
4. **Resource Monitoring**: Monitor resource usage
5. **Baseline Comparison**: Compare against baselines

## Troubleshooting

### Common Test Issues

1. **Test Failures**: Check test logic and assertions
2. **Timeout Issues**: Increase timeout or optimize tests
3. **Resource Issues**: Check resource limits and usage
4. **Environment Issues**: Verify test environment setup
5. **Dependency Issues**: Check test dependencies

### Debug Techniques

1. **Verbose Output**: Use verbose output for debugging
2. **Logging**: Add logging to tests
3. **Breakpoints**: Use debugger breakpoints
4. **Isolation**: Test components in isolation
5. **Incremental Testing**: Test incrementally

## Conclusion

Testing a mod thoroughly involves multiple levels of testing:

1. **Unit Tests**: Test individual components
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows
4. **Performance Tests**: Test performance and resource usage
5. **Security Tests**: Test security and sandboxing
6. **Compatibility Tests**: Test compatibility with different environments

By following these testing practices and using the Colony Mod SDK testing tools, you can ensure your mod is reliable, performant, and secure.

---

**Thorough testing is essential for creating reliable, high-quality mods.** 🏭🧪
