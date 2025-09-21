# Developer Docs: GPU Yard

The GPU Yard is a specialized workyard designed for GPU-intensive operations. This document explains how it works, its unique characteristics, and how to optimize GPU workloads in the Colony Simulator.

## Overview

The GPU Yard represents a dedicated processing unit optimized for parallel computation tasks. Unlike CPU workyards that excel at sequential operations, the GPU Yard is designed for:

- **Parallel Processing**: Execute many operations simultaneously
- **High Throughput**: Process large volumes of data
- **Specialized Operations**: Render, transform, and compute tasks
- **Memory Bandwidth**: High-speed data transfer capabilities

## Architecture

### GPU Yard Components

```rust
pub struct GpuYard {
    pub id: GpuYardId,
    pub name: String,
    pub capacity: usize,           // Number of GPU slots
    pub vram_capacity: u64,        // VRAM in bytes
    pub pcie_bandwidth: f32,       // PCIe bandwidth in Gbps
    pub thermal_throttle_temp: f32, // Temperature threshold
    pub current_temp: f32,         // Current temperature
    pub power_draw: f32,           // Current power consumption
    pub utilization: f32,          // Current utilization (0.0-1.0)
    pub workers: Vec<GpuWorker>,   // Active GPU workers
    pub queue: Vec<GpuJob>,        // Queued GPU jobs
}
```

### GPU Workers

```rust
pub struct GpuWorker {
    pub id: GpuWorkerId,
    pub gpu_yard_id: GpuYardId,
    pub slot: usize,               // GPU slot number
    pub vram_usage: u64,          // VRAM usage in bytes
    pub compute_units: f32,       // Available compute units
    pub memory_bandwidth: f32,    // Memory bandwidth utilization
    pub current_job: Option<GpuJob>,
    pub status: GpuWorkerStatus,
    pub thermal_state: ThermalState,
}

pub enum GpuWorkerStatus {
    Idle,
    Running,
    ThermalThrottled,
    Faulty,
    Offline,
}
```

## GPU-Specific Features

### VRAM Management

GPU workers have limited VRAM that must be managed carefully:

```rust
pub struct VramManager {
    pub total_vram: u64,
    pub allocated_vram: u64,
    pub free_vram: u64,
    pub fragmentation: f32,
    pub allocation_strategy: VramAllocationStrategy,
}

pub enum VramAllocationStrategy {
    FirstFit,      // Allocate first available block
    BestFit,       // Allocate smallest suitable block
    WorstFit,      // Allocate largest available block
    Buddy,         // Use buddy allocation system
}
```

### PCIe Bandwidth

Data transfer between CPU and GPU is limited by PCIe bandwidth:

```rust
pub struct PcieBandwidth {
    pub total_bandwidth: f32,      // Total bandwidth in Gbps
    pub used_bandwidth: f32,       // Currently used bandwidth
    pub available_bandwidth: f32,  // Available bandwidth
    pub utilization: f32,          // Utilization percentage
    pub latency: f32,              // Transfer latency in ms
}
```

### Thermal Management

GPUs generate significant heat and can throttle performance:

```rust
pub struct ThermalManagement {
    pub current_temp: f32,
    pub max_temp: f32,
    pub throttle_temp: f32,
    pub cooling_efficiency: f32,
    pub thermal_throttling: bool,
    pub performance_scale: f32,    // 0.0-1.0 performance scaling
}
```

## GPU Operations

### Supported Operations

The GPU Yard supports specialized operations:

```rust
pub enum GpuOpType {
    Render,         // Graphics rendering
    Compute,        // General compute shaders
    Transform,      // Data transformation
    Convolution,    // Convolution operations
    Matrix,         // Matrix operations
    FFT,            // Fast Fourier Transform
    Hash,           // Cryptographic hashing
    Custom,         // Custom GPU operations
}
```

### Operation Characteristics

```rust
pub struct GpuOpSpec {
    pub op_type: GpuOpType,
    pub compute_units: f32,        // Required compute units
    pub vram_requirement: u64,     // VRAM needed in bytes
    pub memory_bandwidth: f32,     // Memory bandwidth requirement
    pub pcie_transfer: u64,        // PCIe data transfer in bytes
    pub execution_time: f32,       // Execution time in ticks
    pub parallel_factor: f32,      // Parallelization factor
    pub thermal_impact: f32,       // Heat generation
}
```

## Batching and Optimization

### Job Batching

GPU operations benefit from batching:

```rust
pub struct GpuBatch {
    pub batch_id: GpuBatchId,
    pub operations: Vec<GpuOpSpec>,
    pub total_vram: u64,
    pub total_compute: f32,
    pub batch_size: usize,
    pub efficiency: f32,           // Batching efficiency
    pub priority: BatchPriority,
}

pub enum BatchPriority {
    Low,
    Normal,
    High,
    Critical,
}
```

### Batching Strategies

```rust
pub enum BatchingStrategy {
    TimeBased,      // Batch by time window
    SizeBased,      // Batch by operation count
    ResourceBased,  // Batch by resource usage
    PriorityBased,  // Batch by priority
    Hybrid,         // Combine multiple strategies
}
```

## Performance Metrics

### GPU Yard KPIs

```rust
pub struct GpuYardKpis {
    pub utilization: f32,          // Overall utilization
    pub throughput: f32,           // Operations per second
    pub vram_utilization: f32,     // VRAM usage percentage
    pub pcie_utilization: f32,     // PCIe bandwidth usage
    pub thermal_efficiency: f32,   // Thermal performance
    pub batch_efficiency: f32,     // Batching effectiveness
    pub fault_rate: f32,           // Fault rate
    pub power_efficiency: f32,     // Power per operation
}
```

### Monitoring

```rust
pub struct GpuMonitoring {
    pub real_time_metrics: GpuYardKpis,
    pub historical_data: Vec<GpuYardKpis>,
    pub alerts: Vec<GpuAlert>,
    pub performance_trends: PerformanceTrends,
}

pub enum GpuAlert {
    HighTemperature,
    LowVram,
    HighPcieUtilization,
    ThermalThrottling,
    FaultDetected,
    PerformanceDegradation,
}
```

## Configuration

### GPU Yard Setup

```toml
# In game configuration
[gpu_yards.main_gpu]
name = "Main GPU Yard"
capacity = 4                    # 4 GPU slots
vram_capacity = 16777216        # 16GB VRAM
pcie_bandwidth = 32.0           # 32 Gbps PCIe
thermal_throttle_temp = 83.0    # 83°C throttle point
cooling_efficiency = 0.8        # 80% cooling efficiency
power_limit = 300.0             # 300W power limit

[gpu_yards.main_gpu.workers]
count = 4
compute_units_per_worker = 2048
memory_bandwidth_per_worker = 8.0  # 8 Gbps per worker
```

### Operation Configuration

```toml
[gpu_ops.render]
name = "Render Operation"
compute_units = 512.0
vram_requirement = 1048576      # 1MB VRAM
memory_bandwidth = 2.0          # 2 Gbps
pcie_transfer = 524288          # 512KB transfer
execution_time = 10.0           # 10 ticks
parallel_factor = 4.0           # 4x parallelization
thermal_impact = 0.1            # 10% heat generation

[gpu_ops.compute]
name = "Compute Operation"
compute_units = 1024.0
vram_requirement = 2097152      # 2MB VRAM
memory_bandwidth = 4.0          # 4 Gbps
pcie_transfer = 1048576         # 1MB transfer
execution_time = 15.0           # 15 ticks
parallel_factor = 8.0           # 8x parallelization
thermal_impact = 0.15           # 15% heat generation
```

## Integration with Simulation

### Job Assignment

GPU jobs are assigned based on:

```rust
pub struct GpuJobAssignment {
    pub job_id: JobId,
    pub gpu_yard_id: GpuYardId,
    pub worker_id: GpuWorkerId,
    pub slot: usize,
    pub vram_allocation: VramAllocation,
    pub priority: JobPriority,
    pub estimated_completion: u64,
}
```

### Resource Scheduling

```rust
pub struct GpuResourceScheduler {
    pub vram_scheduler: VramScheduler,
    pub compute_scheduler: ComputeScheduler,
    pub pcie_scheduler: PcieScheduler,
    pub thermal_scheduler: ThermalScheduler,
}
```

## Fault Handling

### GPU-Specific Faults

```rust
pub enum GpuFault {
    VramExhaustion,      // Out of VRAM
    ThermalThrottling,   // Temperature too high
    PcieBandwidthLimit,  // PCIe bandwidth exceeded
    ComputeUnitFailure,  // Compute unit failure
    MemoryError,         // Memory access error
    DriverFault,         // GPU driver fault
    HardwareFault,       // Hardware failure
}
```

### Recovery Strategies

```rust
pub enum GpuRecoveryStrategy {
    Retry,              // Simple retry
    ReduceBatchSize,    // Reduce batch size
    LowerQuality,       // Lower quality settings
    ThermalCooling,     // Increase cooling
    ResourceReallocation, // Reallocate resources
    WorkerReplacement,  // Replace worker
}
```

## Performance Tuning

### Optimization Strategies

1. **Batching Optimization**:
   - Group similar operations
   - Optimize batch sizes
   - Balance resource usage

2. **Memory Management**:
   - Minimize VRAM fragmentation
   - Use memory pools
   - Implement efficient allocation

3. **Thermal Management**:
   - Monitor temperature trends
   - Implement dynamic throttling
   - Optimize cooling systems

4. **Bandwidth Optimization**:
   - Minimize PCIe transfers
   - Use local memory efficiently
   - Implement data prefetching

### Benchmarking

```rust
pub struct GpuBenchmark {
    pub operation_type: GpuOpType,
    pub batch_sizes: Vec<usize>,
    pub performance_metrics: Vec<GpuPerformanceMetric>,
    pub thermal_impact: Vec<ThermalImpact>,
    pub power_consumption: Vec<PowerConsumption>,
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_yard_creation() {
        let gpu_yard = GpuYard::new(
            "Test GPU Yard",
            4,      // 4 slots
            16777216, // 16GB VRAM
            32.0,   // 32 Gbps PCIe
        );
        
        assert_eq!(gpu_yard.capacity, 4);
        assert_eq!(gpu_yard.vram_capacity, 16777216);
    }
    
    #[test]
    fn test_vram_allocation() {
        let mut vram_manager = VramManager::new(16777216);
        let allocation = vram_manager.allocate(1048576);
        
        assert!(allocation.is_some());
        assert_eq!(vram_manager.allocated_vram, 1048576);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_gpu_job_execution() {
        let mut gpu_yard = create_test_gpu_yard();
        let gpu_job = create_test_gpu_job();
        
        gpu_yard.enqueue_job(gpu_job.clone());
        
        // Simulate job execution
        for _ in 0..100 {
            gpu_yard.tick();
        }
        
        assert!(gpu_yard.is_job_complete(gpu_job.id));
    }
}
```

## Best Practices

### Design Guidelines

1. **Resource Planning**: Plan VRAM and compute requirements carefully
2. **Batching Strategy**: Implement effective batching for throughput
3. **Thermal Awareness**: Monitor and manage thermal conditions
4. **Fault Tolerance**: Implement robust fault handling
5. **Performance Monitoring**: Track key performance metrics

### Optimization Tips

1. **Minimize PCIe Transfers**: Keep data on GPU when possible
2. **Optimize Batch Sizes**: Find the sweet spot for batching
3. **Manage VRAM**: Avoid fragmentation and over-allocation
4. **Thermal Management**: Implement dynamic throttling
5. **Load Balancing**: Distribute work evenly across workers

## Troubleshooting

### Common Issues

1. **VRAM Exhaustion**: Out of GPU memory
2. **Thermal Throttling**: Performance degradation due to heat
3. **PCIe Bottlenecks**: Limited data transfer bandwidth
4. **Fault Cascades**: Failures spreading through GPU workers
5. **Performance Degradation**: Slowing over time

### Debug Tools

- **VRAM Monitor**: Track VRAM usage and fragmentation
- **Thermal Monitor**: Monitor temperature and throttling
- **Performance Profiler**: Profile GPU operation performance
- **Fault Analyzer**: Analyze fault patterns and causes

---

**The GPU Yard is a powerful component for high-performance computing tasks. Understanding its capabilities and limitations is key to building efficient simulations.** 🏭🎮
