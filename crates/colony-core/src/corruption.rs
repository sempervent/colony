use serde::{Serialize, Deserialize};
use rand::SeedableRng;
use rand_pcg::Pcg64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorruptionTunables {
    pub base_fault_rate: f32,       // e.g., 0.002 faults per op at zero stress
    pub heat_weight: f32,           // contribution of thermal stress
    pub bw_weight: f32,             // contribution of bandwidth saturation
    pub starvation_weight: f32,     // long queue waiting
    pub decay_per_tick: f32,        // global field decay
    pub worker_decay_per_tick: f32, // individual decay
    pub recover_boost: f32,         // bonus decay when idle/maintenance
    pub retry_backoff_ms: u64,      // base backoff for transient retries
    pub max_retries: u8,            // default retry cap
}

impl Default for CorruptionTunables {
    fn default() -> Self {
        Self {
            base_fault_rate: 0.002,
            heat_weight: 0.8,
            bw_weight: 0.6,
            starvation_weight: 0.4,
            decay_per_tick: 0.0015,
            worker_decay_per_tick: 0.004,
            recover_boost: 0.01,
            retry_backoff_ms: 8,
            max_retries: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, bevy::prelude::Resource)]
pub struct CorruptionField {
    pub global: f32,  // 0..1
}

impl CorruptionField {
    pub fn new() -> Self {
        Self { global: 0.0 }
    }
}

pub fn fault_probability(
    base: f32, 
    global: f32, 
    worker: f32, 
    heat_frac: f32, 
    bw_util: f32, 
    queue_starvation: f32,
    t: &CorruptionTunables,
) -> f32 {
    let stress = t.heat_weight * heat_frac + t.bw_weight * bw_util + t.starvation_weight * queue_starvation;
    (base + global * 0.5 + worker * 0.5 + stress).clamp(0.0, 0.35) // cap soft faults
}

// Simple seeded RNG per tick for deterministic fault injection
pub fn tick_rng(seed: u64, tick: u64) -> Pcg64 {
    Pcg64::seed_from_u64(seed ^ (tick.wrapping_mul(0x9E3779B97F4A7C15)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_probability_increases_with_stress() {
        let tunables = CorruptionTunables::default();
        
        // Low stress scenario
        let low_prob = fault_probability(
            0.002, 0.0, 0.0, 0.0, 0.0, 0.0, &tunables
        );
        
        // High stress scenario
        let high_prob = fault_probability(
            0.002, 0.5, 0.3, 0.8, 0.9, 0.7, &tunables
        );
        
        assert!(high_prob > low_prob);
        assert!(high_prob <= 0.35); // Should be capped
    }

    #[test]
    fn test_tick_rng_deterministic() {
        let rng1 = tick_rng(42, 100);
        let rng2 = tick_rng(42, 100);
        
        // Should produce same sequence
        assert_eq!(rng1.gen::<u32>(), rng2.gen::<u32>());
    }
}
