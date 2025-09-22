use serde::{Serialize, Deserialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTunables {
    pub vram_gb: f32,                // total VRAM per GPU yard
    pub pcie_gbps: f32,              // host<->gpu bandwidth
    pub kernel_launch_ms: u32,       // per-op fixed cost
    pub batch_max: usize,            // max items per micro-batch
    pub batch_timeout_ms: u32,       // flush if not filled in time
    pub mixed_precision_speedup: f32,// multiplier if enabled
    pub warmup_ms: u32,              // first-op warmup
}

impl Default for GpuTunables {
    fn default() -> Self {
        Self {
            vram_gb: 16.0,
            pcie_gbps: 12.0,
            kernel_launch_ms: 0,     // model as part of op; leave 0 if duplicative
            batch_max: 32,
            batch_timeout_ms: 8,
            mixed_precision_speedup: 1.4,
            warmup_ms: 50,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GpuMeters {
    pub util: f32,           // 0..1 estimated SM utilization
    pub vram_used_gb: f32,
    pub batches_inflight: u32,
    pub batch_latency_ms: f32, // EWMA
}

impl GpuMeters {
    pub fn new() -> Self {
        Self {
            util: 0.0,
            vram_used_gb: 0.0,
            batches_inflight: 0,
            batch_latency_ms: 0.0,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GpuFlags {
    pub mixed_precision: bool,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct GpuFarm {
    pub gpus: u32,                 // number of logical GPUs
    pub per_gpu: GpuTunables,
    pub meters: GpuMeters,
    pub flags: GpuFlags,
}

impl GpuFarm {
    pub fn new() -> Self {
        Self {
            gpus: 1,
            per_gpu: GpuTunables::default(),
            meters: GpuMeters::new(),
            flags: GpuFlags::default(),
        }
    }
}

// Batching buffer for GPU operations
#[derive(Default, Clone, Debug)]
pub struct GpuBatchBuffer {
    pub items: Vec<GpuBatchItem>,
    pub first_enqueue_tick: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct GpuBatchItem {
    pub job_id: u64,
    pub op: super::Op,
    pub payload_sz: usize,
    pub enqueue_tick: u64,
}

impl GpuBatchBuffer {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            first_enqueue_tick: None,
        }
    }

    pub fn add_item(&mut self, item: GpuBatchItem) {
        if self.first_enqueue_tick.is_none() {
            self.first_enqueue_tick = Some(item.enqueue_tick);
        }
        self.items.push(item);
    }

    pub fn should_flush(&self, tunables: &GpuTunables, now_tick: u64) -> bool {
        if self.items.is_empty() {
            return false;
        }

        // Full batch
        if self.items.len() >= tunables.batch_max {
            return true;
        }

        // Timeout
        if let Some(first_tick) = self.first_enqueue_tick {
            if now_tick >= first_tick {
                let wait_ticks = now_tick - first_tick;
                let timeout_ticks = (tunables.batch_timeout_ms as u64 * 1000) / 16; // Convert ms to 16ms ticks
                if wait_ticks >= timeout_ticks {
                    return true;
                }
            }
        }

        false
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.first_enqueue_tick = None;
    }

    pub fn total_vram_mb(&self) -> f32 {
        self.items.iter()
            .map(|item| item.op.vram_needed_mb(item.payload_sz))
            .sum()
    }

    pub fn total_payload_bytes(&self) -> usize {
        self.items.iter().map(|item| item.payload_sz).sum()
    }
}

// GPU timing calculations
pub fn calculate_batch_timing(
    batch: &GpuBatchBuffer,
    tunables: &GpuTunables,
    flags: &GpuFlags,
    is_first_batch: bool,
) -> f32 {
    if batch.items.is_empty() {
        return 0.0;
    }

    let batch_n = batch.items.len() as f32;
    let total_bytes = batch.total_payload_bytes() as f32;
    
    // PCIe transfer time (host->device + device->host)
    let pcie_ms = (total_bytes * 8.0) / (tunables.pcie_gbps * 1e9) * 1000.0;
    
    // Kernel launch overhead
    let kernel_ms = tunables.kernel_launch_ms as f32;
    
    // Warmup for first batch
    let warmup_ms = if is_first_batch { tunables.warmup_ms as f32 } else { 0.0 };
    
    // Per-item operation cost (amortized by batch efficiency)
    let batch_efficiency = (batch_n / tunables.batch_max as f32).min(1.0);
    let base_op_cost = batch.items[0].op.cost_ms() as f32;
    let per_item_ms = base_op_cost / batch_efficiency;
    
    // Apply mixed precision speedup
    let speedup = if flags.mixed_precision { tunables.mixed_precision_speedup } else { 1.0 };
    let per_item_ms = per_item_ms / speedup;
    
    // Total execution time
    let exec_ms = kernel_ms + warmup_ms + pcie_ms + (per_item_ms * batch_n);
    
    exec_ms.max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Op;

    #[test]
    fn test_batch_timing_calculation() {
        let tunables = GpuTunables::default();
        let flags = GpuFlags::default();
        let mut batch = GpuBatchBuffer::new();
        
        // Add a single item
        batch.add_item(GpuBatchItem {
            job_id: 1,
            op: Op::Yolo,
            payload_sz: 1024,
            enqueue_tick: 100,
        });
        
        let timing = calculate_batch_timing(&batch, &tunables, &flags, true);
        assert!(timing > 0.0);
    }

    #[test]
    fn test_batch_flush_conditions() {
        let tunables = GpuTunables {
            batch_max: 2,
            batch_timeout_ms: 100,
            ..Default::default()
        };
        let mut batch = GpuBatchBuffer::new();
        
        // Empty batch shouldn't flush
        assert!(!batch.should_flush(&tunables, 100));
        
        // Add one item
        batch.add_item(GpuBatchItem {
            job_id: 1,
            op: Op::Yolo,
            payload_sz: 1024,
            enqueue_tick: 100,
        });
        
        // Shouldn't flush yet (not full, not timed out)
        assert!(!batch.should_flush(&tunables, 150));
        
        // Add second item (now full)
        batch.add_item(GpuBatchItem {
            job_id: 2,
            op: Op::Yolo,
            payload_sz: 1024,
            enqueue_tick: 150,
        });
        
        // Should flush due to being full
        assert!(batch.should_flush(&tunables, 200));
    }

    #[test]
    fn test_vram_calculation() {
        let mut batch = GpuBatchBuffer::new();
        batch.add_item(GpuBatchItem {
            job_id: 1,
            op: Op::Yolo,
            payload_sz: 1024,
            enqueue_tick: 100,
        });
        
        let vram_mb = batch.total_vram_mb();
        assert!(vram_mb > 0.0);
    }
}
