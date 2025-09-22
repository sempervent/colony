use bevy::prelude::*;
use serde::{Deserialize, Serialize};
// VecDeque import removed - no longer needed after removing duplicate JobQueue

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTunables {
    pub power_cap_kw: f32,          // global cap
    pub heat_decay_per_tick: f32,   // °C per tick in each yard (ambient pull)
    pub heat_generated_per_unit: f32, // °C per unit of work (scaled by yard/op)
    pub bandwidth_total_gbps: f32,  // global shared bus
    pub bandwidth_tail_exp: f32,    // >1.0, tail latency growth exponent at high util
    pub thermal_throttle_knee: f32, // fraction of heat_cap where throttle starts (e.g., 0.85)
    pub thermal_min_throttle: f32,  // floor for throttle multiplier (e.g., 0.4)
}

impl Default for ResourceTunables {
    fn default() -> Self {
        Self {
            power_cap_kw: 1_000.0,
            heat_decay_per_tick: 1.5,
            heat_generated_per_unit: 0.02,
            bandwidth_total_gbps: 32.0,
            bandwidth_tail_exp: 2.2,
            thermal_throttle_knee: 0.85,
            thermal_min_throttle: 0.4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMeters {
    pub power_draw_kw: f32,
    pub bandwidth_util: f32, // 0..1
}

impl GlobalMeters {
    pub fn new() -> Self { 
        Self { 
            power_draw_kw: 0.0, 
            bandwidth_util: 0.0 
        } 
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Colony {
    pub power_cap_kw: f32,
    pub bandwidth_total_gbps: f32,
    pub corruption_field: f32,
    pub target_uptime_days: u32,
    pub meters: GlobalMeters,
    pub tunables: ResourceTunables,
    pub corruption_tun: super::corruption::CorruptionTunables,
    pub seed: u64,
}

// JobQueue is defined in queue.rs to avoid conflicts

#[derive(Resource, Default, Clone, Copy)]
pub struct DispatchScale(pub f32); // 0..1

#[derive(Resource, Default)]
pub struct IoRolling {
    pub gbits_this_tick: f32,
}

impl IoRolling {
    pub fn add_bytes(&mut self, n: usize) {
        self.gbits_this_tick += (n as f32 * 8.0) / 1_000_000_000.0;
    }
    
    pub fn take_and_reset(&mut self) -> f32 {
        let v = self.gbits_this_tick;
        self.gbits_this_tick = 0.0;
        v
    }
}

pub fn thermal_throttle(heat: f32, cap: f32, knee: f32, floor: f32) -> f32 {
    if heat < cap * knee { 
        1.0 
    } else { 
        (cap / heat).clamp(floor, 1.0) 
    }
}

// Bandwidth-induced latency multiplier: blows up near saturation
pub fn bandwidth_latency_multiplier(util: f32, tail_exp: f32) -> f32 {
    if util <= 0.7 { 
        1.0 
    } else { 
        1.0 + ((util - 0.7) / 0.3).max(0.0).powf(tail_exp) 
    }
}
