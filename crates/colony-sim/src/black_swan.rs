use colony_core::{Colony, Worker, Workyard};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct BlackSwanEngine {
    events: Vec<BlackSwanEvent>,
}

#[derive(Debug, Clone)]
pub struct BlackSwanEvent {
    pub id: String,
    pub name: String,
    pub triggers: Vec<Trigger>,
    pub effects: Vec<Effect>,
    pub cure: Option<String>,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub enum Trigger {
    BandwidthUtil { threshold: f32, window: u32 },
    ThermalEvents { count: u32, window_seconds: u32 },
    CorruptionField { threshold: f32 },
    PowerDraw { threshold: f32 },
}

#[derive(Debug, Clone)]
pub enum Effect {
    PipelineInsert { op: String, target: String },
    Debt { multiplier: f32, duration_days: u32 },
    UiIllusion { metric: String, offset: f32, duration_hours: u32 },
    ThrottleAll { factor: f32 },
    CorruptionField { delta: f32 },
}

impl BlackSwanEngine {
    pub fn new() -> Self {
        Self {
            events: vec![
                BlackSwanEvent {
                    id: "vram_ecc_propagation".to_string(),
                    name: "VRAM: Snow of Ash".to_string(),
                    triggers: vec![
                        Trigger::BandwidthUtil { threshold: 0.95, window: 5 },
                        Trigger::ThermalEvents { count: 3, window_seconds: 3600 },
                        Trigger::CorruptionField { threshold: 0.6 },
                    ],
                    effects: vec![
                        Effect::PipelineInsert { op: "CRC".to_string(), target: "all_outbound".to_string() },
                        Effect::Debt { multiplier: 1.08, duration_days: 7 },
                        Effect::UiIllusion { metric: "temperature".to_string(), offset: -5.0, duration_hours: 12 },
                    ],
                    cure: Some("maintenance.run=memtest_vram,parts=3,time=8h".to_string()),
                    weight: 1.0,
                },
                BlackSwanEvent {
                    id: "thermal_cascade".to_string(),
                    name: "Thermal Cascade".to_string(),
                    triggers: vec![
                        Trigger::BandwidthUtil { threshold: 0.9, window: 10 },
                        Trigger::PowerDraw { threshold: 0.95 },
                    ],
                    effects: vec![
                        Effect::ThrottleAll { factor: 0.5 },
                        Effect::CorruptionField { delta: 0.2 },
                    ],
                    cure: Some("maintenance.run=cooling_cycle,time=2h".to_string()),
                    weight: 0.8,
                },
            ],
        }
    }

    pub fn check_triggers(
        &self,
        colony: &Colony,
        yards: &[Workyard],
        workers: &[Worker],
    ) -> Vec<&BlackSwanEvent> {
        let mut triggered = Vec::new();
        let mut rng = rand::thread_rng();

        for event in &self.events {
            if self.evaluate_triggers(&event.triggers, colony, yards, workers) {
                // Weighted random selection
                if rng.gen::<f32>() < event.weight {
                    triggered.push(event);
                }
            }
        }

        triggered
    }

    fn evaluate_triggers(
        &self,
        triggers: &[Trigger],
        colony: &Colony,
        yards: &[Workyard],
        _workers: &[Worker],
    ) -> bool {
        for trigger in triggers {
            if !self.evaluate_trigger(trigger, colony, yards) {
                return false;
            }
        }
        true
    }

    fn evaluate_trigger(&self, trigger: &Trigger, colony: &Colony, yards: &[Workyard]) -> bool {
        match trigger {
            Trigger::BandwidthUtil { threshold, .. } => {
                // Simplified: check if any yard is over threshold
                yards.iter().any(|yard| yard.bandwidth_share > *threshold)
            }
            Trigger::ThermalEvents { count, .. } => {
                // Simplified: check if any yard is hot
                yards.iter().filter(|yard| yard.heat > yard.heat_cap * 0.8).count() >= *count as usize
            }
            Trigger::CorruptionField { threshold } => {
                colony.corruption_field > *threshold
            }
            Trigger::PowerDraw { threshold } => {
                let total_power: f32 = yards.iter().map(|yard| yard.power_draw_kw).sum();
                total_power / colony.power_cap_kw > *threshold
            }
        }
    }

    pub fn apply_effects(&self, effects: &[Effect], colony: &mut Colony, yards: &mut [Workyard]) {
        for effect in effects {
            match effect {
                Effect::Debt { multiplier, .. } => {
                    // Apply power multiplier
                    for yard in yards.iter_mut() {
                        yard.power_draw_kw *= multiplier;
                    }
                }
                Effect::CorruptionField { delta } => {
                    colony.corruption_field += delta;
                }
                Effect::ThrottleAll { factor } => {
                    // This would be handled by the thermal system
                    for yard in yards.iter_mut() {
                        yard.heat_cap *= factor;
                    }
                }
                _ => {
                    // Other effects would be handled by their respective systems
                }
            }
        }
    }
}
