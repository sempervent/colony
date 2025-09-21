use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WorkClass {
    Cpu,
    Gpu,
    Io(IoKind),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum IoKind {
    Udp,
    Can,
    Socket,
    Modbus,
    Http,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Worker {
    pub id: u64,
    pub class: WorkClass,
    pub skill_cpu: f32,
    pub skill_gpu: f32,
    pub skill_io: f32,
    pub discipline: f32,
    pub focus: f32,
    pub corruption: f32,
    pub state: WorkerState,
    pub retry: RetryPolicy,
    pub sticky_faults: u32,          // count
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WorkerState {
    Idle,
    Queued,
    Running,
    Blocked,
    Recovering,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FaultKind {
    Transient,       // retry/backoff helps
    DataSkew,        // output drift; requires re-run
    StickyConfig,    // worker enters Recovering; needs reimage/maintenance
    QueueDrop,       // packet/job dropped; deadline likely missed
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u8,
    pub backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            backoff_ms: 8,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Workyard {
    pub kind: WorkyardKind,
    pub slots: u32,
    pub heat: f32,
    pub heat_cap: f32,
    pub power_draw_kw: f32,
    pub bandwidth_share: f32,   // proportion of global bandwidth when saturated
    pub isolation_domain: u32,
}

#[derive(Component, Default)]
pub struct YardWorkload { 
    pub units_this_tick: f32 
} // set by dispatcher/segments

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkyardKind {
    CpuArray,
    GpuFarm,
    SignalHub,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub pipeline: Pipeline,
    pub qos: QoS,
    pub deadline_ms: u64,
    pub payload_sz: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pipeline {
    pub ops: Vec<Op>,
    pub mutation_tag: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    UdpDemux,
    Decode,
    Kalman,
    Export,
    HttpParse,
    HttpExport,
    Fft,
    Yolo,
    Crc,
    CanParse,
    TcpSessionize,
    ModbusMap,
    MaintenanceCool,
    DynamicWasm { op_id: String },
    DynamicLua { func: String },
}

impl Op {
    pub fn cost_ms(&self) -> u32 {
        match self {
            Op::UdpDemux => 2,
            Op::Decode => 4,
            Op::Kalman => 3,
            Op::Export => 2,
            Op::HttpParse => 3,
            Op::HttpExport => 2,
            Op::Fft => 6,
            Op::Yolo => 18,
            Op::Crc => 1,
            Op::CanParse => 2,
            Op::TcpSessionize => 5,
            Op::ModbusMap => 2,
            Op::MaintenanceCool => 8,
            Op::DynamicWasm { .. } => 5, // Default cost for WASM ops
            Op::DynamicLua { .. } => 2,  // Default cost for Lua ops
        }
    }

    pub fn work_units(&self) -> f32 {
        match self {
            Op::UdpDemux => 0.5,
            Op::Decode => 1.2,
            Op::Kalman => 0.8,
            Op::Export => 0.3,
            Op::HttpParse => 0.6,
            Op::HttpExport => 0.3,
            Op::Fft => 1.5,
            Op::Yolo => 4.5,
            Op::Crc => 0.3,
            Op::CanParse => 0.5,
            Op::TcpSessionize => 1.2,
            Op::ModbusMap => 0.5,
            Op::MaintenanceCool => 0.0, // No heat generation
            Op::DynamicWasm { .. } => 1.0, // Default work units for WASM ops
            Op::DynamicLua { .. } => 0.5,  // Default work units for Lua ops
        }
    }

    pub fn bandwidth_gbps(&self, payload_sz: usize) -> f32 {
        // Convert bytes to gigabits per tick (assuming 16ms tick)
        let bytes_per_tick = payload_sz as f32;
        let gbits_per_tick = (bytes_per_tick * 8.0) / 1_000_000_000.0;
        gbits_per_tick
    }

    pub fn vram_needed_mb(&self, payload_sz: usize) -> f32 {
        match self {
            Op::Yolo => (payload_sz as f32 / 1_000_000.0) * 3.0 + 150.0, // rough: activations+weights
            Op::Fft => (payload_sz as f32 / 1_000_000.0) * 1.0 + 30.0, // FFT uses GPU memory
            Op::DynamicWasm { .. } => (payload_sz as f32 / 1_000_000.0) * 0.5 + 10.0, // Default VRAM for WASM ops
            Op::DynamicLua { .. } => 0.0, // Lua ops don't use VRAM
            _ => 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QoS {
    Throughput,
    Latency,
    Balanced,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FaultKind {
    Thermal,
    Power,
    Corruption,
    Network,
    Hardware,
}
