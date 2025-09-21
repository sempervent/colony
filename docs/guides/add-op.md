# Guide: How to Add a New Operation

This guide will walk you through the process of adding a new operation to the Colony Simulator. Operations are the fundamental building blocks of work in the simulation.

## Overview

Operations (Ops) are single units of work that can be performed by workers. Each operation has:

- **Resource Requirements**: CPU, GPU, I/O, and time costs
- **Input/Output**: Data that flows through the operation
- **Behavior**: What the operation actually does
- **Metadata**: Name, description, and other properties

## Types of Operations

### Built-in Operations

These are operations that are part of the core simulation:

- **CPU Operations**: Decode, Encrypt, Analyze, Compress, Decompress
- **GPU Operations**: Render, Process, Transform
- **I/O Operations**: Read, Write, Network, Route

### Custom Operations

These are operations you create for specific needs:

- **WASM Operations**: High-performance operations in WebAssembly
- **Lua Operations**: Flexible operations in Lua scripts
- **Hybrid Operations**: Combinations of different operation types

## Step 1: Plan Your Operation

### Define Requirements

Before coding, define what your operation needs to do:

1. **Purpose**: What is the operation's main purpose?
2. **Inputs**: What data does it need?
3. **Outputs**: What data does it produce?
4. **Resources**: What resources does it consume?
5. **Performance**: What are the performance requirements?

### Example: Data Encryption Operation

Let's create a data encryption operation:

- **Purpose**: Encrypt sensitive data
- **Inputs**: Plain text data, encryption key
- **Outputs**: Encrypted data
- **Resources**: CPU-intensive, moderate memory usage
- **Performance**: Should handle large data sets efficiently

## Step 2: Choose Implementation Method

### Option 1: Built-in Operation (Rust)

For core functionality that needs maximum performance:

```rust
// In colony-core/src/ops/mod.rs
pub struct DataEncryptionOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
    pub encryption_algorithm: EncryptionAlgorithm,
}

impl DataEncryptionOp {
    pub fn new(algorithm: EncryptionAlgorithm) -> Self {
        Self {
            name: "Data Encryption".to_string(),
            description: "Encrypts data using specified algorithm".to_string(),
            resource_cost: OpResourceCost {
                cpu: 20.0,        // High CPU usage
                gpu: 0.0,         // No GPU usage
                io: 5.0,          // Moderate I/O
                time: 15.0,       // 15 ticks
                memory: 2048,     // 2KB memory
                bandwidth: 1024,  // 1KB bandwidth
            },
            encryption_algorithm: algorithm,
        }
    }
}

impl Op for DataEncryptionOp {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    
    fn get_description(&self) -> String {
        self.description.clone()
    }
    
    fn get_resource_cost(&self) -> &OpResourceCost {
        &self.resource_cost
    }
    
    fn execute(&self, context: &mut OpContext) -> OpResult {
        // Get input data
        let input_data = context.get_input_data()?;
        
        // Perform encryption
        let encrypted_data = self.encrypt_data(&input_data)?;
        
        // Set output data
        context.set_output_data(&encrypted_data)?;
        
        OpResult::Success
    }
}

impl DataEncryptionOp {
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        match self.encryption_algorithm {
            EncryptionAlgorithm::AES256 => self.encrypt_aes256(data),
            EncryptionAlgorithm::RSA => self.encrypt_rsa(data),
            EncryptionAlgorithm::XOR => self.encrypt_xor(data),
        }
    }
    
    fn encrypt_aes256(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // AES-256 encryption implementation
        // This would use a crypto library like aes-gcm
        Ok(vec![]) // Placeholder
    }
    
    fn encrypt_rsa(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // RSA encryption implementation
        // This would use a crypto library like rsa
        Ok(vec![]) // Placeholder
    }
    
    fn encrypt_xor(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // Simple XOR encryption
        let key = 0x42; // Simple key
        let encrypted: Vec<u8> = data.iter().map(|&b| b ^ key).collect();
        Ok(encrypted)
    }
}
```

### Option 2: WASM Operation

For modding with high performance:

```rust
// In your WASM mod
use colony_modsdk::ops::WasmOpSpec;
use colony_modsdk::types::*;

pub struct WasmDataEncryptionOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
    pub algorithm: String,
}

impl WasmDataEncryptionOp {
    pub fn new() -> Self {
        Self {
            name: "WASM Data Encryption".to_string(),
            description: "High-performance data encryption in WASM".to_string(),
            resource_cost: OpResourceCost {
                cpu: 25.0,        // Higher CPU usage for WASM
                gpu: 0.0,         // No GPU usage
                io: 8.0,          // Higher I/O for WASM
                time: 12.0,       // 12 ticks
                memory: 4096,     // 4KB memory
                bandwidth: 2048,  // 2KB bandwidth
            },
            algorithm: "AES256".to_string(),
        }
    }
}

impl WasmOpSpec for WasmDataEncryptionOp {
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
        
        // Parse operation parameters
        let params: EncryptionParams = serde_json::from_slice(&input_data)
            .map_err(|e| OpError::DeserializationError(e.to_string()))?;
        
        // Perform encryption
        let encrypted_data = self.encrypt_data(&params.data, &params.key)?;
        
        // Set output data
        context.set_output_data(&encrypted_data)?;
        
        OpResult::Success
    }
}

impl WasmDataEncryptionOp {
    fn encrypt_data(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, OpError> {
        // WASM-optimized encryption
        // This would use WASM-compatible crypto libraries
        Ok(vec![]) // Placeholder
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EncryptionParams {
    data: Vec<u8>,
    key: Vec<u8>,
    algorithm: String,
}
```

### Option 3: Lua Operation

For flexible scripting:

```lua
-- In your Lua mod
local function create_encryption_operation()
    local op = {
        name = "Lua Data Encryption",
        description = "Flexible data encryption in Lua",
        resource_cost = {
            cpu = 15.0,
            gpu = 0.0,
            io = 6.0,
            time = 18.0,
            memory = 1024,
            bandwidth = 512
        },
        algorithm = "XOR"
    }
    
    function op.execute(context)
        -- Get input data
        local input_data = context.get_input_data()
        if not input_data then
            return { success = false, error = "No input data" }
        end
        
        -- Parse parameters
        local params = json.decode(input_data)
        if not params then
            return { success = false, error = "Invalid parameters" }
        end
        
        -- Perform encryption
        local encrypted_data = op.encrypt_data(params.data, params.key)
        if not encrypted_data then
            return { success = false, error = "Encryption failed" }
        end
        
        -- Set output data
        context.set_output_data(encrypted_data)
        
        return { success = true }
    end
    
    function op.encrypt_data(data, key)
        -- Simple XOR encryption in Lua
        local encrypted = {}
        local key_len = #key
        
        for i = 1, #data do
            local key_byte = string.byte(key, ((i - 1) % key_len) + 1)
            encrypted[i] = string.char(string.byte(data, i) ~ key_byte)
        end
        
        return table.concat(encrypted)
    end
    
    return op
end

-- Register the operation
local encryption_op = create_encryption_operation()
colony.ops.register("data_encryption", encryption_op)
```

## Step 3: Register the Operation

### Built-in Operations

Register in the operation registry:

```rust
// In colony-core/src/ops/registry.rs
pub fn register_ops(registry: &mut OpRegistry) {
    // Register existing operations
    registry.register("Decode", Box::new(DecodeOp::new()));
    registry.register("Encrypt", Box::new(EncryptOp::new()));
    
    // Register new operation
    registry.register("DataEncryption", Box::new(DataEncryptionOp::new(EncryptionAlgorithm::AES256)));
}
```

### WASM Operations

Register in the mod manifest:

```toml
# In mod.toml
[mod.operations]
operations = [
    { name = "data_encryption", type = "wasm", file = "src/ops/encryption.wasm" }
]
```

### Lua Operations

Register in the mod manifest:

```toml
# In mod.toml
[mod.operations]
operations = [
    { name = "data_encryption", type = "lua", file = "lua/encryption.lua" }
]
```

## Step 4: Configure the Operation

### Operation Configuration

Add configuration to the game config:

```toml
# In game config
[ops.data_encryption]
name = "Data Encryption"
description = "Encrypts data using specified algorithm"
cpu = 20.0
gpu = 0.0
io = 5.0
time = 15.0
memory = 2048
bandwidth = 1024
algorithm = "AES256"
key_size = 256
```

### Pipeline Integration

Add to pipelines:

```toml
# In pipeline config
[pipelines.secure_data_processing]
name = "Secure Data Processing"
description = "Processes data with encryption"

[pipelines.secure_data_processing.ops]
ops = [
    { type = "Read", io = 10.0, time = 5.0 },
    { type = "DataEncryption", cpu = 20.0, time = 15.0 },
    { type = "Write", io = 10.0, time = 5.0 }
]
```

## Step 5: Test the Operation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_encryption_operation() {
        let op = DataEncryptionOp::new(EncryptionAlgorithm::XOR);
        let mut context = create_test_op_context();
        
        // Set test input data
        let test_data = b"Hello, World!";
        context.set_input_data(test_data);
        
        // Execute operation
        let result = op.execute(&mut context);
        assert!(result.is_ok());
        
        // Verify output
        let output_data = context.get_output_data().unwrap();
        assert!(!output_data.is_empty());
        assert_ne!(output_data, test_data); // Should be encrypted
    }
    
    #[test]
    fn test_encryption_decryption() {
        let op = DataEncryptionOp::new(EncryptionAlgorithm::XOR);
        let test_data = b"Test data";
        
        // Encrypt
        let encrypted = op.encrypt_data(test_data).unwrap();
        
        // Decrypt (XOR is symmetric)
        let decrypted = op.encrypt_data(&encrypted).unwrap();
        
        assert_eq!(decrypted, test_data);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_operation_in_pipeline() {
        let mut pipeline = create_test_pipeline();
        let op = DataEncryptionOp::new(EncryptionAlgorithm::XOR);
        
        // Add operation to pipeline
        pipeline.add_operation(Box::new(op));
        
        // Run pipeline
        let result = pipeline.execute();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_operation_with_workers() {
        let mut simulation = create_test_simulation();
        let op = DataEncryptionOp::new(EncryptionAlgorithm::XOR);
        
        // Create job with operation
        let job = Job::new_with_operation(op);
        simulation.enqueue_job(job);
        
        // Run simulation
        for _ in 0..100 {
            simulation.tick();
        }
        
        // Verify job completion
        assert!(simulation.is_job_complete(job.id));
    }
}
```

## Step 6: Optimize Performance

### CPU Optimization

```rust
impl DataEncryptionOp {
    fn encrypt_data_optimized(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // Use SIMD instructions for large data
        if data.len() >= 1024 {
            return self.encrypt_data_simd(data);
        }
        
        // Use optimized algorithms for small data
        self.encrypt_data_scalar(data)
    }
    
    fn encrypt_data_simd(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // SIMD-optimized encryption
        // This would use SIMD intrinsics
        Ok(vec![]) // Placeholder
    }
    
    fn encrypt_data_scalar(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // Scalar encryption for small data
        Ok(vec![]) // Placeholder
    }
}
```

### Memory Optimization

```rust
impl DataEncryptionOp {
    fn encrypt_data_memory_optimized(&self, data: &[u8]) -> Result<Vec<u8>, OpError> {
        // Pre-allocate result vector
        let mut result = Vec::with_capacity(data.len());
        
        // Process data in chunks to reduce memory pressure
        const CHUNK_SIZE: usize = 1024;
        for chunk in data.chunks(CHUNK_SIZE) {
            let encrypted_chunk = self.encrypt_chunk(chunk)?;
            result.extend_from_slice(&encrypted_chunk);
        }
        
        Ok(result)
    }
    
    fn encrypt_chunk(&self, chunk: &[u8]) -> Result<Vec<u8>, OpError> {
        // Encrypt chunk with minimal memory allocation
        Ok(vec![]) // Placeholder
    }
}
```

## Step 7: Document the Operation

### Operation Documentation

```rust
impl DataEncryptionOp {
    /// Encrypts data using the specified algorithm
    /// 
    /// # Arguments
    /// * `data` - The data to encrypt
    /// * `key` - The encryption key
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - The encrypted data
    /// * `Err(OpError)` - If encryption fails
    /// 
    /// # Examples
    /// ```
    /// let op = DataEncryptionOp::new(EncryptionAlgorithm::AES256);
    /// let encrypted = op.encrypt_data(b"Hello, World!", b"secret_key")?;
    /// ```
    pub fn encrypt_data(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, OpError> {
        // Implementation
    }
}
```

### Usage Examples

```rust
// Example usage in a pipeline
let mut pipeline = Pipeline::new("Secure Data Processing");
pipeline.add_operation(Box::new(DataEncryptionOp::new(EncryptionAlgorithm::AES256)));
pipeline.add_operation(Box::new(DataCompressionOp::new()));

// Execute pipeline
let result = pipeline.execute();
```

## Step 8: Deploy and Monitor

### Deployment

1. **Build the Operation**: Compile and test the operation
2. **Update Configuration**: Add operation to game config
3. **Deploy**: Deploy to the simulation environment
4. **Verify**: Verify the operation works correctly

### Monitoring

1. **Performance Metrics**: Monitor operation performance
2. **Resource Usage**: Track resource consumption
3. **Error Rates**: Monitor error rates and types
4. **Usage Patterns**: Track how the operation is used

## Best Practices

### Design Guidelines

1. **Single Responsibility**: Each operation should do one thing well
2. **Resource Awareness**: Be mindful of resource consumption
3. **Error Handling**: Implement robust error handling
4. **Performance**: Optimize for performance where possible
5. **Documentation**: Document your operations clearly

### Testing Guidelines

1. **Unit Tests**: Test individual operation functionality
2. **Integration Tests**: Test operation integration
3. **Performance Tests**: Test operation performance
4. **Error Tests**: Test error handling
5. **Edge Cases**: Test edge cases and boundary conditions

### Performance Guidelines

1. **Efficient Algorithms**: Use efficient algorithms
2. **Memory Management**: Manage memory efficiently
3. **Resource Optimization**: Optimize resource usage
4. **Caching**: Cache frequently used data
5. **Profiling**: Profile operations for bottlenecks

## Common Pitfalls

### What to Avoid

1. **Resource Overuse**: Don't consume too many resources
2. **Poor Error Handling**: Don't ignore error conditions
3. **Inefficient Algorithms**: Don't use inefficient algorithms
4. **Memory Leaks**: Don't create memory leaks
5. **Poor Documentation**: Don't skip documentation

### Common Issues

1. **Resource Limits**: Operations exceeding resource limits
2. **Error Propagation**: Errors not being handled properly
3. **Performance Issues**: Operations running too slowly
4. **Memory Issues**: Operations using too much memory
5. **Integration Problems**: Operations not integrating properly

## Conclusion

Adding a new operation to the Colony Simulator involves several steps:

1. **Plan**: Define requirements and choose implementation method
2. **Implement**: Write the operation code
3. **Register**: Register the operation in the system
4. **Configure**: Add configuration and pipeline integration
5. **Test**: Write comprehensive tests
6. **Optimize**: Optimize for performance
7. **Document**: Document the operation
8. **Deploy**: Deploy and monitor the operation

By following these steps and best practices, you can create effective, efficient operations that enhance the Colony Simulator's capabilities.

---

**Operations are the building blocks of the Colony Simulator. Creating effective operations is key to building powerful simulations.** 🏭⚙️
