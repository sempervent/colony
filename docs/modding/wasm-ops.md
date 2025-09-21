# Modding: WASM Operations

WebAssembly (WASM) operations provide high-performance, sandboxed execution for custom pipeline operations. This guide explains how to create, implement, and integrate WASM operations into the Colony Simulator.

## Overview

WASM operations offer:

- **High Performance**: Near-native performance for compute-intensive tasks
- **Sandboxed Execution**: Secure execution environment with strict resource limits
- **Cross-Platform**: Runs on any platform that supports WebAssembly
- **Type Safety**: Strong typing and memory safety
- **Deterministic**: Deterministic execution for reproducible simulations

## WASM Operation Structure

### Basic Structure

A WASM operation is a WebAssembly module that implements the `WasmOpSpec` trait:

```rust
// In your WASM module
use colony_modsdk::ops::WasmOpSpec;
use colony_modsdk::types::*;

pub struct MyCustomOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
}

impl WasmOpSpec for MyCustomOp {
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
        // Your operation logic here
        OpResult::Success
    }
}
```

### Resource Cost Definition

```rust
use colony_modsdk::types::OpResourceCost;

pub fn create_resource_cost() -> OpResourceCost {
    OpResourceCost {
        cpu: 10.0,        // CPU cycles required
        gpu: 0.0,         // GPU compute units (0 for CPU-only ops)
        io: 5.0,          // I/O operations required
        time: 8.0,        // Time in ticks
        memory: 1024,     // Memory in bytes
        bandwidth: 512,   // Network bandwidth in bytes
    }
}
```

## Creating a WASM Operation

### Step 1: Project Setup

Create a new Rust project for your WASM operation:

```bash
cargo new my-wasm-op --lib
cd my-wasm-op
```

### Step 2: Dependencies

Add the required dependencies to `Cargo.toml`:

```toml
[package]
name = "my-wasm-op"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
colony-modsdk = "0.9.0"
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"

[dependencies.web-sys]
version = "0.3"
features = [
    "console",
    "js-sys",
]
```

### Step 3: Implementation

Implement your WASM operation in `src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;
use colony_modsdk::ops::WasmOpSpec;
use colony_modsdk::types::*;
use serde::{Deserialize, Serialize};

// Define your operation data structure
#[derive(Serialize, Deserialize)]
pub struct MyOpData {
    pub input_data: Vec<u8>,
    pub processing_params: ProcessingParams,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessingParams {
    pub algorithm: String,
    pub iterations: u32,
    pub threshold: f32,
}

// Implement the WASM operation
pub struct MyWasmOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
}

impl MyWasmOp {
    pub fn new() -> Self {
        Self {
            name: "My Custom Operation".to_string(),
            description: "A custom WASM operation for data processing".to_string(),
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

impl WasmOpSpec for MyWasmOp {
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
        // Get input data from context
        let input_data = context.get_input_data()?;
        
        // Deserialize the operation data
        let op_data: MyOpData = serde_json::from_slice(&input_data)
            .map_err(|e| OpError::DeserializationError(e.to_string()))?;
        
        // Process the data
        let result = self.process_data(&op_data)?;
        
        // Set output data in context
        context.set_output_data(&result)?;
        
        OpResult::Success
    }
}

impl MyWasmOp {
    fn process_data(&self, data: &MyOpData) -> Result<Vec<u8>, OpError> {
        let mut result = data.input_data.clone();
        
        // Apply processing based on parameters
        match data.processing_params.algorithm.as_str() {
            "encrypt" => {
                result = self.encrypt_data(&result, &data.processing_params)?;
            },
            "compress" => {
                result = self.compress_data(&result)?;
            },
            "analyze" => {
                result = self.analyze_data(&result, &data.processing_params)?;
            },
            _ => {
                return Err(OpError::InvalidParameter(
                    format!("Unknown algorithm: {}", data.processing_params.algorithm)
                ));
            }
        }
        
        Ok(result)
    }
    
    fn encrypt_data(&self, data: &[u8], params: &ProcessingParams) -> Result<Vec<u8>, OpError> {
        // Simple XOR encryption for demonstration
        let key = params.threshold as u8;
        let encrypted: Vec<u8> = data.iter()
            .map(|&b| b ^ key)
            .collect();
        
        Ok(encrypted)
    }
    
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // Simple run-length encoding for demonstration
        let mut compressed = Vec::new();
        let mut current_byte = data[0];
        let mut count = 1;
        
        for &byte in &data[1..] {
            if byte == current_byte && count < 255 {
                count += 1;
            } else {
                compressed.push(count);
                compressed.push(current_byte);
                current_byte = byte;
                count = 1;
            }
        }
        
        compressed.push(count);
        compressed.push(current_byte);
        
        Ok(compressed)
    }
    
    fn analyze_data(&self, data: &[u8], params: &ProcessingParams) -> Result<Vec<u8>, OpError> {
        // Simple statistical analysis
        let mut histogram = [0u32; 256];
        
        for &byte in data {
            histogram[byte as usize] += 1;
        }
        
        let mut result = Vec::new();
        for (i, &count) in histogram.iter().enumerate() {
            if count > params.threshold as u32 {
                result.push(i as u8);
                result.extend_from_slice(&count.to_le_bytes());
            }
        }
        
        Ok(result)
    }
}

// Export the operation for WASM
#[wasm_bindgen]
pub fn create_operation() -> JsValue {
    let op = MyWasmOp::new();
    serde_wasm_bindgen::to_value(&op).unwrap()
}

// Export operation metadata
#[wasm_bindgen]
pub fn get_operation_metadata() -> JsValue {
    let metadata = OperationMetadata {
        name: "My Custom Operation".to_string(),
        description: "A custom WASM operation for data processing".to_string(),
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

### Step 4: Build Configuration

Create a `build.rs` file for build configuration:

```rust
fn main() {
    println!("cargo:rustc-link-arg=-s");
    println!("cargo:rustc-link-arg=WASM_BIGINT");
}
```

### Step 5: Build the WASM Module

Build your WASM operation:

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build the WASM module
wasm-pack build --target web --out-dir pkg
```

## Advanced WASM Operations

### GPU Operations

For GPU-accelerated operations:

```rust
use colony_modsdk::gpu::GpuContext;

pub struct GpuWasmOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
}

impl GpuWasmOp {
    pub fn new() -> Self {
        Self {
            name: "GPU Processing Operation".to_string(),
            description: "A GPU-accelerated WASM operation".to_string(),
            resource_cost: OpResourceCost {
                cpu: 5.0,
                gpu: 20.0,        // High GPU usage
                io: 10.0,
                time: 15.0,
                memory: 4096,     // More memory for GPU operations
                bandwidth: 2048,
            },
        }
    }
}

impl WasmOpSpec for GpuWasmOp {
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
        // Get GPU context
        let gpu_context = context.get_gpu_context()?;
        
        // Perform GPU operations
        let result = self.gpu_process(gpu_context)?;
        
        // Set output data
        context.set_output_data(&result)?;
        
        OpResult::Success
    }
}

impl GpuWasmOp {
    fn gpu_process(&self, gpu_context: &GpuContext) -> Result<Vec<u8>, OpError> {
        // GPU processing logic here
        // This would typically involve:
        // 1. Uploading data to GPU memory
        // 2. Running compute shaders
        // 3. Downloading results from GPU memory
        
        Ok(vec![]) // Placeholder
    }
}
```

### Network Operations

For network-based operations:

```rust
use colony_modsdk::network::NetworkContext;

pub struct NetworkWasmOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
}

impl NetworkWasmOp {
    pub fn new() -> Self {
        Self {
            name: "Network Operation".to_string(),
            description: "A network-based WASM operation".to_string(),
            resource_cost: OpResourceCost {
                cpu: 8.0,
                gpu: 0.0,
                io: 15.0,         // High I/O for network operations
                time: 20.0,       // Longer time for network latency
                memory: 1024,
                bandwidth: 4096,  // High bandwidth usage
            },
        }
    }
}

impl WasmOpSpec for NetworkWasmOp {
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
        // Get network context
        let network_context = context.get_network_context()?;
        
        // Perform network operations
        let result = self.network_process(network_context)?;
        
        // Set output data
        context.set_output_data(&result)?;
        
        OpResult::Success
    }
}

impl NetworkWasmOp {
    fn network_process(&self, network_context: &NetworkContext) -> Result<Vec<u8>, OpError> {
        // Network processing logic here
        // This would typically involve:
        // 1. Making HTTP requests
        // 2. Processing network responses
        // 3. Handling network errors
        
        Ok(vec![]) // Placeholder
    }
}
```

## Error Handling

### Custom Error Types

Define custom error types for your operations:

```rust
use colony_modsdk::types::OpError;

#[derive(Debug, thiserror::Error)]
pub enum MyOpError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<MyOpError> for OpError {
    fn from(err: MyOpError) -> Self {
        OpError::CustomError(err.to_string())
    }
}
```

### Error Handling in Operations

```rust
impl WasmOpSpec for MyWasmOp {
    fn execute(&self, context: &mut WasmOpContext) -> OpResult {
        // Validate input
        let input_data = context.get_input_data()
            .map_err(|e| MyOpError::InvalidInput(e.to_string()))?;
        
        if input_data.is_empty() {
            return Err(MyOpError::InvalidInput("Empty input data".to_string()).into());
        }
        
        // Check resource limits
        if input_data.len() > 1024 * 1024 { // 1MB limit
            return Err(MyOpError::ResourceLimitExceeded(
                "Input data too large".to_string()
            ).into());
        }
        
        // Process data with error handling
        let result = self.process_data(&input_data)
            .map_err(|e| MyOpError::ProcessingFailed(e.to_string()))?;
        
        // Set output data
        context.set_output_data(&result)
            .map_err(|e| MyOpError::ProcessingFailed(e.to_string()))?;
        
        OpResult::Success
    }
}
```

## Testing WASM Operations

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use colony_modsdk::testing::*;
    
    #[test]
    fn test_operation_creation() {
        let op = MyWasmOp::new();
        assert_eq!(op.get_name(), "My Custom Operation");
        assert_eq!(op.get_description(), "A custom WASM operation for data processing");
    }
    
    #[test]
    fn test_operation_execution() {
        let op = MyWasmOp::new();
        let mut context = create_test_wasm_context();
        
        // Set test input data
        let test_data = MyOpData {
            input_data: vec![1, 2, 3, 4, 5],
            processing_params: ProcessingParams {
                algorithm: "encrypt".to_string(),
                iterations: 1,
                threshold: 0.5,
            },
        };
        
        let input_bytes = serde_json::to_vec(&test_data).unwrap();
        context.set_input_data(&input_bytes);
        
        // Execute operation
        let result = op.execute(&mut context);
        assert!(result.is_ok());
        
        // Verify output
        let output_data = context.get_output_data().unwrap();
        assert!(!output_data.is_empty());
    }
    
    #[test]
    fn test_resource_cost() {
        let op = MyWasmOp::new();
        let cost = op.get_resource_cost();
        
        assert_eq!(cost.cpu, 15.0);
        assert_eq!(cost.gpu, 0.0);
        assert_eq!(cost.io, 8.0);
        assert_eq!(cost.time, 12.0);
        assert_eq!(cost.memory, 2048);
        assert_eq!(cost.bandwidth, 1024);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use colony_modsdk::testing::*;
    
    #[test]
    fn test_operation_in_pipeline() {
        let op = MyWasmOp::new();
        let mut pipeline = create_test_pipeline();
        
        // Add operation to pipeline
        pipeline.add_operation(Box::new(op));
        
        // Run pipeline
        let result = pipeline.execute();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_operation_with_mod() {
        let mut mod_loader = create_test_mod_loader();
        let wasm_module = load_test_wasm_module();
        
        mod_loader.load_wasm_module(wasm_module);
        
        let op = mod_loader.get_operation("My Custom Operation").unwrap();
        let mut context = create_test_wasm_context();
        
        let result = op.execute(&mut context);
        assert!(result.is_ok());
    }
}
```

## Performance Optimization

### Memory Management

```rust
impl MyWasmOp {
    fn process_data_optimized(&self, data: &MyOpData) -> Result<Vec<u8>, OpError> {
        // Pre-allocate result vector with estimated size
        let estimated_size = data.input_data.len();
        let mut result = Vec::with_capacity(estimated_size);
        
        // Process data in chunks to reduce memory pressure
        const CHUNK_SIZE: usize = 1024;
        for chunk in data.input_data.chunks(CHUNK_SIZE) {
            let processed_chunk = self.process_chunk(chunk, &data.processing_params)?;
            result.extend_from_slice(&processed_chunk);
        }
        
        Ok(result)
    }
    
    fn process_chunk(&self, chunk: &[u8], params: &ProcessingParams) -> Result<Vec<u8>, OpError> {
        // Process chunk with minimal memory allocation
        match params.algorithm.as_str() {
            "encrypt" => Ok(self.encrypt_chunk(chunk, params.threshold as u8)),
            "compress" => Ok(self.compress_chunk(chunk)),
            "analyze" => Ok(self.analyze_chunk(chunk, params.threshold)),
            _ => Err(OpError::InvalidParameter("Unknown algorithm".to_string())),
        }
    }
}
```

### CPU Optimization

```rust
impl MyWasmOp {
    fn process_data_cpu_optimized(&self, data: &MyOpData) -> Result<Vec<u8>, OpError> {
        // Use SIMD instructions where possible
        if data.input_data.len() >= 16 {
            return self.process_data_simd(data);
        }
        
        // Fall back to scalar processing
        self.process_data_scalar(data)
    }
    
    fn process_data_simd(&self, data: &MyOpData) -> Result<Vec<u8>, OpError> {
        // SIMD-optimized processing
        // This would use SIMD intrinsics for vectorized operations
        Ok(vec![]) // Placeholder
    }
    
    fn process_data_scalar(&self, data: &MyOpData) -> Result<Vec<u8>, OpError> {
        // Scalar processing for small data
        Ok(vec![]) // Placeholder
    }
}
```

## Best Practices

### Design Guidelines

1. **Single Responsibility**: Each operation should do one thing well
2. **Resource Awareness**: Be mindful of resource consumption
3. **Error Handling**: Implement robust error handling
4. **Performance**: Optimize for performance where possible
5. **Documentation**: Document your operations clearly

### Security Considerations

1. **Input Validation**: Always validate input data
2. **Resource Limits**: Respect resource limits
3. **Memory Safety**: Avoid memory leaks and buffer overflows
4. **Sandboxing**: Work within the sandbox constraints
5. **Error Information**: Don't leak sensitive information in errors

### Performance Tips

1. **Memory Management**: Use efficient memory management
2. **Algorithm Choice**: Choose appropriate algorithms
3. **Batch Processing**: Process data in batches when possible
4. **Caching**: Cache frequently used data
5. **Profiling**: Profile your operations for bottlenecks

---

**WASM operations provide powerful, high-performance capabilities for the Colony Simulator. Understanding these concepts is key to creating effective mods.** 🏭⚡
