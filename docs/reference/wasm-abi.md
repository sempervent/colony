# Reference: WebAssembly (WASM) ABI

This document defines the Application Binary Interface (ABI) for WebAssembly modules in the Asynchronous Colony Simulator modding system.

## Overview

WASM modules in the colony simulator communicate with the host through a well-defined ABI that ensures:
- **Safety**: Sandboxed execution with controlled memory access
- **Performance**: Efficient data transfer between host and WASM
- **Determinism**: Reproducible execution for replay compatibility
- **Capability Gating**: Restricted access based on declared permissions

## Memory Layout

### Linear Memory

Each WASM module has access to a linear memory space that is managed by the host:

```rust
// Memory layout (conceptual)
struct WasmMemory {
    // 0x0000 - 0x1000: Reserved for ABI structures
    abi_region: [u8; 4096],
    
    // 0x1000 - 0x10000: Host-provided data
    input_data: [u8; 61440],
    
    // 0x10000 - 0x20000: WASM module workspace
    workspace: [u8; 65536],
    
    // 0x20000 - 0x40000: Output buffer
    output_buffer: [u8; 131072],
}
```

### ABI Structures

The first 4KB of memory contains ABI control structures:

```rust
#[repr(C)]
struct AbiHeader {
    magic: u32,           // 0x434F4C59 ("COLY")
    version: u32,         // ABI version
    op_context_ptr: u32,  // Pointer to OpContext
    input_data_ptr: u32,  // Pointer to input data
    input_data_len: u32,  // Length of input data
    output_ptr: u32,      // Pointer to output buffer
    output_capacity: u32, // Output buffer capacity
    return_code: u32,     // Return code from WASM function
    bytes_written: u32,   // Bytes written to output
}

#[repr(C)]
struct OpContext {
    current_tick: u64,
    worker_id: u32,
    job_id: u32,
    pipeline_id: u32,
    op_index: u32,
    fault_probability: f32,
    random_seed: u64,
    capabilities: u64,    // Bitfield of granted capabilities
}
```

## Function Signatures

### Entry Point Function

Every WASM Op must export a function with this exact signature:

```rust
#[no_mangle]
pub extern "C" fn op_entrypoint(
    abi_header_ptr: *mut AbiHeader,
    workspace_ptr: *mut u8,
    workspace_size: u32,
) -> u32;
```

**Parameters:**
- `abi_header_ptr`: Pointer to the ABI header structure
- `workspace_ptr`: Pointer to the workspace memory region
- `workspace_size`: Size of the workspace in bytes

**Return Value:**
- `0`: Success
- `1`: General error
- `2`: Insufficient output buffer space
- `3`: Invalid input data
- `4`: Capability denied

### Host Import Functions

WASM modules can import functions from the host (subject to capability gating):

```rust
// Logging functions
extern "C" {
    fn colony_log_debug(message_ptr: *const u8, message_len: u32);
    fn colony_log_info(message_ptr: *const u8, message_len: u32);
    fn colony_log_warn(message_ptr: *const u8, message_len: u32);
    fn colony_log_error(message_ptr: *const u8, message_len: u32);
}

// Simulation state access
extern "C" {
    fn colony_get_current_tick() -> u64;
    fn colony_get_random_u32(seed: u64) -> u32;
    fn colony_get_random_f32(seed: u64) -> f32;
}

// Job management (requires "enqueue_job" capability)
extern "C" {
    fn colony_enqueue_job(
        pipeline_id_ptr: *const u8,
        pipeline_id_len: u32,
        priority: u32,
    ) -> u32; // Returns job_id or 0 on error
}

// KPI access (requires "read_kpis" capability)
extern "C" {
    fn colony_get_kpi(kpi_name_ptr: *const u8, kpi_name_len: u32) -> f64;
}
```

## Data Serialization

### Input Data Format

Input data is provided as a contiguous byte array. The format depends on the Op type:

```rust
// Example: Decode operation input
struct DecodeInput {
    data_type: u8,        // 0=raw, 1=compressed, 2=encrypted
    data_len: u32,        // Length of data to decode
    data: [u8; N],        // Actual data (variable length)
}

// Example: Render operation input
struct RenderInput {
    width: u32,
    height: u32,
    format: u8,           // 0=RGB, 1=RGBA, 2=YUV
    frame_data: [u8; N],  // Frame data (variable length)
}
```

### Output Data Format

Output data follows a similar structured format:

```rust
// Example: Decode operation output
struct DecodeOutput {
    success: u8,          // 0=failure, 1=success
    decoded_len: u32,     // Length of decoded data
    decoded_data: [u8; N], // Decoded data (variable length)
}

// Example: Render operation output
struct RenderOutput {
    success: u8,
    render_time_ms: u32,  // Time taken to render
    output_data: [u8; N], // Rendered output (variable length)
}
```

## Error Handling

### Error Codes

Standard error codes returned by WASM functions:

```rust
pub const WASM_SUCCESS: u32 = 0;
pub const WASM_ERROR_GENERAL: u32 = 1;
pub const WASM_ERROR_OUTPUT_BUFFER: u32 = 2;
pub const WASM_ERROR_INVALID_INPUT: u32 = 3;
pub const WASM_ERROR_CAPABILITY_DENIED: u32 = 4;
pub const WASM_ERROR_MEMORY_ALLOCATION: u32 = 5;
pub const WASM_ERROR_INVALID_STATE: u32 = 6;
```

### Fault Simulation

WASM modules can trigger faults by setting specific return codes:

```rust
pub const WASM_FAULT_SOFT: u32 = 0x1000;  // Soft fault occurred
pub const WASM_FAULT_STICKY: u32 = 0x2000; // Sticky fault occurred
```

## Capability System

### Capability Bitfield

Capabilities are represented as a 64-bit bitfield:

```rust
pub const CAP_SIM_TIME: u64 = 1 << 0;
pub const CAP_LOG_MESSAGE: u64 = 1 << 1;
pub const CAP_ENQUEUE_JOB: u64 = 1 << 2;
pub const CAP_READ_KPIS: u64 = 1 << 3;
pub const CAP_MODIFY_STATE: u64 = 1 << 4;
pub const CAP_DETERMINISTIC_RNG: u64 = 1 << 5;
// ... more capabilities
```

### Capability Checking

WASM modules should check capabilities before calling host functions:

```rust
fn has_capability(context: &OpContext, capability: u64) -> bool {
    (context.capabilities & capability) != 0
}

// Example usage
if has_capability(&context, CAP_LOG_MESSAGE) {
    colony_log_info(b"Operation completed successfully", 30);
}
```

## Development Guidelines

### Memory Safety

- Always validate pointer bounds before dereferencing
- Use the provided workspace for temporary allocations
- Never write beyond the output buffer capacity
- Initialize all output data to prevent information leaks

### Performance Considerations

- Minimize data copying between host and WASM
- Use efficient serialization formats
- Cache frequently accessed data in workspace
- Avoid complex memory allocations

### Determinism

- Use only the provided deterministic RNG
- Avoid system calls or external dependencies
- Ensure all operations are reproducible
- Handle floating-point precision consistently

## Example Implementation

Here's a complete example of a WASM Op implementation:

```rust
// Cargo.toml
[package]
name = "example_op"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Minimal dependencies for WASM

// src/lib.rs
use core::slice;

const CAP_LOG_MESSAGE: u64 = 1 << 1;

#[no_mangle]
pub extern "C" fn op_entrypoint(
    abi_header_ptr: *mut AbiHeader,
    workspace_ptr: *mut u8,
    workspace_size: u32,
) -> u32 {
    // Safety: Host guarantees valid pointers
    let header = unsafe { &*abi_header_ptr };
    
    // Check input data
    if header.input_data_len == 0 {
        return 3; // Invalid input
    }
    
    // Check output buffer capacity
    if header.output_capacity < 4 {
        return 2; // Insufficient output buffer
    }
    
    // Read input data
    let input_data = unsafe {
        slice::from_raw_parts(
            header.input_data_ptr as *const u8,
            header.input_data_len as usize
        )
    };
    
    // Process data (example: simple byte reversal)
    let mut output = [0u8; 4];
    let bytes_to_process = input_data.len().min(4);
    
    for i in 0..bytes_to_process {
        output[i] = input_data[bytes_to_process - 1 - i];
    }
    
    // Write output
    let output_ptr = header.output_ptr as *mut u8;
    unsafe {
        slice::from_raw_parts_mut(output_ptr, header.output_capacity as usize)
            [..4]
            .copy_from_slice(&output);
    }
    
    // Update header
    unsafe {
        (*abi_header_ptr).bytes_written = 4;
        (*abi_header_ptr).return_code = 0;
    }
    
    0 // Success
}
```

This ABI provides a robust foundation for creating high-performance, secure WASM operations that integrate seamlessly with the colony simulator's modding system.
