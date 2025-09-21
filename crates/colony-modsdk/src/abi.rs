use serde::{Serialize, Deserialize};

/// WASM ABI version for compatibility checking
pub const WASM_ABI_VERSION: u32 = 1;

/// Maximum input payload size (1MB)
pub const MAX_INPUT_SIZE: usize = 1024 * 1024;

/// Maximum output payload size (1MB)
pub const MAX_OUTPUT_SIZE: usize = 1024 * 1024;

/// Maximum metadata size (64KB)
pub const MAX_METADATA_SIZE: usize = 64 * 1024;

/// Default fuel limit per WASM op call
pub const DEFAULT_FUEL_LIMIT: u64 = 5_000_000;

/// Default memory limit for WASM modules (64MB)
pub const DEFAULT_MEMORY_LIMIT: usize = 64 * 1024 * 1024;

/// Return codes for WASM operations
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmReturnCode {
    /// Operation completed successfully
    Success = 0,
    /// Transient fault - operation failed but can be retried
    TransientFault = 1,
    /// Sticky fault - operation failed and should be quarantined
    StickyFault = 2,
    /// Data corruption detected
    DataCorruption = 3,
    /// Resource exhaustion (memory, fuel, etc.)
    ResourceExhaustion = 4,
    /// Invalid input data
    InvalidInput = 5,
    /// Operation not implemented
    NotImplemented = 6,
    /// Generic error
    Error = -1,
    /// Invalid context
    InvalidContext = -2,
    /// Memory access violation
    MemoryViolation = -3,
    /// Fuel exhausted
    FuelExhausted = -4,
}

impl WasmReturnCode {
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Success),
            1 => Some(Self::TransientFault),
            2 => Some(Self::StickyFault),
            3 => Some(Self::DataCorruption),
            4 => Some(Self::ResourceExhaustion),
            5 => Some(Self::InvalidInput),
            6 => Some(Self::NotImplemented),
            -1 => Some(Self::Error),
            -2 => Some(Self::InvalidContext),
            -3 => Some(Self::MemoryViolation),
            -4 => Some(Self::FuelExhausted),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    pub fn is_fault(&self) -> bool {
        matches!(self, Self::TransientFault | Self::StickyFault | Self::DataCorruption)
    }

    pub fn is_error(&self) -> bool {
        (*self as i32) < 0
    }
}

/// Context for WASM operations - opaque handle
#[repr(C)]
pub struct OpCtx {
    /// Opaque handle ID
    pub handle_id: u64,
    /// Operation ID
    pub op_id: u64,
    /// Current simulation tick
    pub tick: u64,
    /// Random seed for this operation
    pub seed: u64,
    /// Available fuel
    pub fuel: u64,
    /// Memory limit
    pub memory_limit: usize,
}

/// WASM operation result
#[derive(Debug, Clone)]
pub struct WasmOpResult {
    pub return_code: WasmReturnCode,
    pub output_size: usize,
    pub fuel_consumed: u64,
    pub memory_used: usize,
    pub execution_time_ms: u64,
}

/// WASM host functions available to modules
#[repr(C)]
pub struct WasmHostFunctions {
    /// Get current simulation time
    pub get_sim_time: extern "C" fn() -> u64,
    /// Get deterministic random number
    pub get_random: extern "C" fn() -> u64,
    /// Log a message
    pub log: extern "C" fn(level: i32, msg_ptr: *const u8, msg_len: usize) -> i32,
    /// Get a metric value
    pub get_metric: extern "C" fn(name_ptr: *const u8, name_len: usize) -> f64,
    /// Enqueue a job
    pub enqueue_job: extern "C" fn(pipeline_ptr: *const u8, pipeline_len: usize, payload_ptr: *const u8, payload_len: usize) -> i32,
}

/// WASM module exports that must be implemented
#[repr(C)]
pub struct WasmModuleExports {
    /// Initialize the operation
    pub op_init: extern "C" fn(ctx: *mut OpCtx) -> i32,
    /// Process input data
    pub op_process: extern "C" fn(
        ctx: *mut OpCtx,
        in_ptr: *const u8,
        in_len: usize,
        out_ptr: *mut u8,
        out_cap: usize,
        meta_ptr: *const u8,
        meta_len: usize,
    ) -> i32,
    /// Clean up the operation
    pub op_end: extern "C" fn(ctx: *mut OpCtx) -> i32,
}

/// WASM module metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub abi_version: u32,
    pub fuel_hint: u64,
    pub memory_hint: usize,
    pub capabilities_required: Vec<String>,
}

/// WASM module validation result
#[derive(Debug, Clone)]
pub struct WasmModuleValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: Option<WasmModuleMetadata>,
    pub estimated_fuel: u64,
    pub estimated_memory: usize,
}

/// WASM execution environment
#[derive(Debug, Clone)]
pub struct WasmExecutionEnv {
    pub fuel_limit: u64,
    pub memory_limit: usize,
    pub time_limit_ms: u64,
    pub capabilities: Vec<String>,
    pub rng_seed: u64,
    pub sim_time: u64,
}

impl Default for WasmExecutionEnv {
    fn default() -> Self {
        Self {
            fuel_limit: DEFAULT_FUEL_LIMIT,
            memory_limit: DEFAULT_MEMORY_LIMIT,
            time_limit_ms: 1000, // 1 second
            capabilities: Vec::new(),
            rng_seed: 0,
            sim_time: 0,
        }
    }
}

impl WasmOpResult {
    pub fn new(return_code: WasmReturnCode) -> Self {
        Self {
            return_code,
            output_size: 0,
            fuel_consumed: 0,
            memory_used: 0,
            execution_time_ms: 0,
        }
    }

    pub fn with_output_size(mut self, size: usize) -> Self {
        self.output_size = size;
        self
    }

    pub fn with_fuel_consumed(mut self, fuel: u64) -> Self {
        self.fuel_consumed = fuel;
        self
    }

    pub fn with_memory_used(mut self, memory: usize) -> Self {
        self.memory_used = memory;
        self
    }

    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }
}

impl OpCtx {
    pub fn new(handle_id: u64, op_id: u64, tick: u64, seed: u64) -> Self {
        Self {
            handle_id,
            op_id,
            tick,
            seed,
            fuel: DEFAULT_FUEL_LIMIT,
            memory_limit: DEFAULT_MEMORY_LIMIT,
        }
    }

    pub fn consume_fuel(&mut self, amount: u64) -> bool {
        if self.fuel >= amount {
            self.fuel -= amount;
            true
        } else {
            false
        }
    }

    pub fn has_fuel(&self) -> bool {
        self.fuel > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_return_code() {
        assert_eq!(WasmReturnCode::Success as i32, 0);
        assert_eq!(WasmReturnCode::TransientFault as i32, 1);
        assert_eq!(WasmReturnCode::Error as i32, -1);

        assert!(WasmReturnCode::Success.is_success());
        assert!(!WasmReturnCode::TransientFault.is_success());

        assert!(WasmReturnCode::TransientFault.is_fault());
        assert!(!WasmReturnCode::Success.is_fault());

        assert!(WasmReturnCode::Error.is_error());
        assert!(!WasmReturnCode::Success.is_error());
    }

    #[test]
    fn test_wasm_return_code_from_i32() {
        assert_eq!(WasmReturnCode::from_i32(0), Some(WasmReturnCode::Success));
        assert_eq!(WasmReturnCode::from_i32(1), Some(WasmReturnCode::TransientFault));
        assert_eq!(WasmReturnCode::from_i32(-1), Some(WasmReturnCode::Error));
        assert_eq!(WasmReturnCode::from_i32(999), None);
    }

    #[test]
    fn test_op_ctx() {
        let mut ctx = OpCtx::new(1, 2, 1000, 42);
        assert_eq!(ctx.handle_id, 1);
        assert_eq!(ctx.op_id, 2);
        assert_eq!(ctx.tick, 1000);
        assert_eq!(ctx.seed, 42);
        assert_eq!(ctx.fuel, DEFAULT_FUEL_LIMIT);

        assert!(ctx.consume_fuel(1000));
        assert_eq!(ctx.fuel, DEFAULT_FUEL_LIMIT - 1000);

        assert!(!ctx.consume_fuel(DEFAULT_FUEL_LIMIT + 1));
        assert_eq!(ctx.fuel, DEFAULT_FUEL_LIMIT - 1000);
    }

    #[test]
    fn test_wasm_op_result() {
        let result = WasmOpResult::new(WasmReturnCode::Success)
            .with_output_size(1024)
            .with_fuel_consumed(5000)
            .with_memory_used(1024 * 1024)
            .with_execution_time(10);

        assert_eq!(result.return_code, WasmReturnCode::Success);
        assert_eq!(result.output_size, 1024);
        assert_eq!(result.fuel_consumed, 5000);
        assert_eq!(result.memory_used, 1024 * 1024);
        assert_eq!(result.execution_time_ms, 10);
    }

    #[test]
    fn test_wasm_execution_env() {
        let env = WasmExecutionEnv::default();
        assert_eq!(env.fuel_limit, DEFAULT_FUEL_LIMIT);
        assert_eq!(env.memory_limit, DEFAULT_MEMORY_LIMIT);
        assert_eq!(env.time_limit_ms, 1000);
    }
}
