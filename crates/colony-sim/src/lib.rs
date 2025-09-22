pub mod black_swan;
pub mod thermal;
pub mod corruption;

pub use black_swan::*;
pub use thermal::*;
pub use corruption::*;

use colony_core::{Worker, Workyard, Colony};
use rand::Rng;

pub fn thermal_throttle(heat: f32, cap: f32) -> f32 {
    if heat < cap * 0.85 {
        1.0
    } else {
        (cap / heat).clamp(0.4, 1.0)
    }
}

pub fn corruption_noise(corruption_field: f32) -> f32 {
    let mut rng = rand::thread_rng();
    let noise = rng.gen_range(-0.1..0.1) * corruption_field;
    1.0 + noise
}

pub fn bandwidth_factor(bandwidth_util: f32) -> f32 {
    if bandwidth_util < 0.9 {
        1.0
    } else {
        // Exponential degradation above 90% utilization
        (1.0 - bandwidth_util).powf(2.0)
    }
}

pub fn skill_mult(worker: &Worker, op: &colony_core::Op) -> f32 {
    match op {
        colony_core::Op::Decode | colony_core::Op::Fft | colony_core::Op::Kalman => worker.skill_cpu,
        colony_core::Op::Yolo => worker.skill_gpu,
        colony_core::Op::UdpDemux | colony_core::Op::TcpSessionize | colony_core::Op::HttpParse | colony_core::Op::CanParse | colony_core::Op::ModbusMap => worker.skill_io,
        colony_core::Op::Crc => (worker.skill_cpu + worker.skill_io) / 2.0,
        colony_core::Op::Export | colony_core::Op::HttpExport => worker.skill_io,
        colony_core::Op::MaintenanceCool => worker.skill_cpu,
        colony_core::Op::GpuPreprocess | colony_core::Op::GpuExport => worker.skill_gpu,
        colony_core::Op::DynamicWasm { .. } => worker.skill_cpu, // Default to CPU for dynamic WASM ops
        colony_core::Op::DynamicLua { .. } => worker.skill_cpu, // Default to CPU for dynamic Lua ops
    }
}

pub fn base_speed(op: &colony_core::Op) -> f32 {
    match op {
        colony_core::Op::Decode => 1.0,
        colony_core::Op::Fft => 0.8,
        colony_core::Op::Kalman => 0.6,
        colony_core::Op::Yolo => 0.3,
        colony_core::Op::Crc => 2.0,
        colony_core::Op::CanParse => 3.0,
        colony_core::Op::UdpDemux => 2.5,
        colony_core::Op::TcpSessionize => 1.5,
        colony_core::Op::ModbusMap => 2.0,
        colony_core::Op::HttpParse => 1.2,
        colony_core::Op::Export => 1.5,
        colony_core::Op::HttpExport => 1.3,
        colony_core::Op::MaintenanceCool => 0.5,
        colony_core::Op::GpuPreprocess => 0.4,
        colony_core::Op::GpuExport => 0.6,
        colony_core::Op::DynamicWasm { .. } => 1.0, // Default speed for dynamic WASM ops
        colony_core::Op::DynamicLua { .. } => 1.2, // Default speed for dynamic Lua ops
    }
}

pub fn roll_fault(env: &SimulationEnv, worker: &Worker, op: &colony_core::Op) -> bool {
    let mut rng = rand::thread_rng();
    let base_fault_rate = 0.001; // 0.1% base fault rate
    let corruption_mult = 1.0 + env.corruption_field * 2.0;
    let thermal_mult = if env.heat > env.heat_cap * 0.8 { 2.0 } else { 1.0 };
    let worker_mult = 1.0 - worker.discipline * 0.5;
    
    let fault_rate = base_fault_rate * corruption_mult * thermal_mult * worker_mult;
    rng.gen::<f32>() < fault_rate
}

pub struct SimulationEnv {
    pub heat: f32,
    pub heat_cap: f32,
    pub bandwidth: f32,
    pub corruption_field: f32,
    pub power_draw: f32,
    pub power_cap: f32,
}

impl SimulationEnv {
    pub fn from_colony_and_yard(colony: &Colony, yard: &Workyard) -> Self {
        Self {
            heat: yard.heat,
            heat_cap: yard.heat_cap,
            bandwidth: colony.bandwidth_total_gbps,
            corruption_field: colony.corruption_field,
            power_draw: yard.power_draw_kw,
            power_cap: colony.power_cap_kw,
        }
    }
}
