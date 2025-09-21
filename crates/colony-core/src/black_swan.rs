use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{Op, Pipeline, Debts, Debt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCond {
    pub metric: String,      // e.g., "bandwidth_util", "gpu_thermal_events", "corruption_field", "vram_frac"
    pub op: String,          // ">", ">=", "<", "<="
    pub value: f32,
    pub window_ms: u64,      // rolling window to evaluate condition
    pub count_at_least: Option<u32>, // e.g., 3 events in window
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    // Topology
    InsertOp { pipeline_id: String, where_: String, op: String },        // where_: "all_outbound" | "after:Decode" etc.
    ReplaceOp { pipeline_id: String, from: String, to: String },
    RemoveOp { pipeline_id: String, op: String },
    BranchDualRun { pipeline_id: String, adjudicator: String },           // duplicative path with compare
    QuarantinePipeline { pipeline_id: String, domain: Option<u32> },

    // System debuffs/debts
    DebtPowerMult { mult: f32, duration_ms: u64 },
    DebtHeatAdd { celsius: f32, duration_ms: u64 },
    UIIllusion { metric: String, delta: f32, duration_ms: u64 },       // display skew only
    VramLeak { mb_per_tick: f32, duration_ms: u64 },
    BandwidthTax { mult: f32, duration_ms: u64 },

    // Fault weighting tweaks
    FaultBias { kind: String, weight_mult: f32, duration_ms: u64 },       // e.g., "StickyConfig"

    // Cure hook request
    RequireRitual { ritual_id: String },                                  // Engine will expose as actionable cure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackSwanDef {
    pub id: String,
    pub name: String,
    pub triggers: Vec<TriggerCond>,
    pub effects: Vec<Effect>,
    pub cure: Option<String>,          // ritual_id
    pub weight: f32,                   // selection weight if multiple eligible
    pub cooldown_ms: u64,              // after firing
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct BlackSwanMeters {
    pub active: Vec<String>,           // ids currently affecting system
    pub recently_fired: Vec<(String, u64)>,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct BlackSwanIndex {
    pub defs: Vec<BlackSwanDef>,
    pub meters: BlackSwanMeters,
}

impl BlackSwanIndex {
    pub fn new() -> Self {
        Self {
            defs: Vec::new(),
            meters: BlackSwanMeters::default(),
        }
    }

    pub fn add_black_swan(&mut self, def: BlackSwanDef) {
        self.defs.push(def);
    }

    pub fn is_on_cooldown(&self, id: &str, current_tick: u64) -> bool {
        for (fired_id, fire_tick) in &self.meters.recently_fired {
            if fired_id == id {
                let cooldown_ticks = self.get_cooldown_ticks(id);
                return current_tick - fire_tick < cooldown_ticks;
            }
        }
        false
    }

    pub fn get_cooldown_ticks(&self, id: &str) -> u64 {
        for def in &self.defs {
            if def.id == id {
                return def.cooldown_ms / 16; // Convert ms to 16ms ticks
            }
        }
        0
    }

    pub fn mark_fired(&mut self, id: String, current_tick: u64) {
        // Remove any existing entry for this ID
        self.meters.recently_fired.retain(|(fired_id, _)| fired_id != &id);
        // Add new entry
        self.meters.recently_fired.push((id, current_tick));
    }

    pub fn clear_expired_cooldowns(&mut self, current_tick: u64) {
        self.meters.recently_fired.retain(|(id, fire_tick)| {
            let cooldown_ticks = self.get_cooldown_ticks(id);
            current_tick - fire_tick < cooldown_ticks
        });
    }
}

// KPI tracking for trigger evaluation
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct KpiRingBuffer {
    pub bandwidth_util: Vec<(f32, u64)>, // (value, tick)
    pub corruption_field: Vec<(f32, u64)>,
    pub gpu_thermal_events: Vec<(u32, u64)>, // (count, tick)
    pub vram_frac: Vec<(f32, u64)>,
    pub power_draw: Vec<(f32, u64)>,
    pub heat_levels: Vec<(f32, u64)>,
}

impl KpiRingBuffer {
    pub fn new() -> Self {
        Self {
            bandwidth_util: Vec::new(),
            corruption_field: Vec::new(),
            gpu_thermal_events: Vec::new(),
            vram_frac: Vec::new(),
            power_draw: Vec::new(),
            heat_levels: Vec::new(),
        }
    }

    pub fn add_bandwidth_util(&mut self, value: f32, tick: u64) {
        self.bandwidth_util.push((value, tick));
        // Keep only last 1000 entries
        if self.bandwidth_util.len() > 1000 {
            self.bandwidth_util.remove(0);
        }
    }

    pub fn add_corruption_field(&mut self, value: f32, tick: u64) {
        self.corruption_field.push((value, tick));
        if self.corruption_field.len() > 1000 {
            self.corruption_field.remove(0);
        }
    }

    pub fn add_gpu_thermal_event(&mut self, tick: u64) {
        self.gpu_thermal_events.push((1, tick));
        if self.gpu_thermal_events.len() > 1000 {
            self.gpu_thermal_events.remove(0);
        }
    }

    pub fn add_vram_frac(&mut self, value: f32, tick: u64) {
        self.vram_frac.push((value, tick));
        if self.vram_frac.len() > 1000 {
            self.vram_frac.remove(0);
        }
    }

    pub fn add_power_draw(&mut self, value: f32, tick: u64) {
        self.power_draw.push((value, tick));
        if self.power_draw.len() > 1000 {
            self.power_draw.remove(0);
        }
    }

    pub fn add_heat_level(&mut self, value: f32, tick: u64) {
        self.heat_levels.push((value, tick));
        if self.heat_levels.len() > 1000 {
            self.heat_levels.remove(0);
        }
    }

    pub fn get_metric_in_window(&self, metric: &str, window_ms: u64, current_tick: u64) -> Vec<f32> {
        let window_ticks = window_ms / 16;
        let cutoff_tick = current_tick.saturating_sub(window_ticks);

        match metric {
            "bandwidth_util" => self.bandwidth_util
                .iter()
                .filter(|(_, tick)| *tick >= cutoff_tick)
                .map(|(value, _)| *value)
                .collect(),
            "corruption_field" => self.corruption_field
                .iter()
                .filter(|(_, tick)| *tick >= cutoff_tick)
                .map(|(value, _)| *value)
                .collect(),
            "gpu_thermal_events" => self.gpu_thermal_events
                .iter()
                .filter(|(_, tick)| *tick >= cutoff_tick)
                .map(|(count, _)| *count as f32)
                .collect(),
            "vram_frac" => self.vram_frac
                .iter()
                .filter(|(_, tick)| *tick >= cutoff_tick)
                .map(|(value, _)| *value)
                .collect(),
            "power_draw" => self.power_draw
                .iter()
                .filter(|(_, tick)| *tick >= cutoff_tick)
                .map(|(value, _)| *value)
                .collect(),
            "heat_levels" => self.heat_levels
                .iter()
                .filter(|(_, tick)| *tick >= cutoff_tick)
                .map(|(value, _)| *value)
                .collect(),
            _ => Vec::new(),
        }
    }
}

pub fn evaluate_triggers(
    black_swan_index: &BlackSwanIndex,
    kpi_buffer: &KpiRingBuffer,
    current_tick: u64,
) -> Vec<String> {
    let mut eligible = Vec::new();

    for def in &black_swan_index.defs {
        if black_swan_index.is_on_cooldown(&def.id, current_tick) {
            continue;
        }

        let mut all_conditions_met = true;

        for trigger in &def.triggers {
            let values = kpi_buffer.get_metric_in_window(&trigger.metric, trigger.window_ms, current_tick);
            
            if values.is_empty() {
                all_conditions_met = false;
                break;
            }

            let condition_met = match trigger.op.as_str() {
                ">" => values.iter().any(|v| *v > trigger.value),
                ">=" => values.iter().any(|v| *v >= trigger.value),
                "<" => values.iter().any(|v| *v < trigger.value),
                "<=" => values.iter().any(|v| *v <= trigger.value),
                _ => false,
            };

            if let Some(count_threshold) = trigger.count_at_least {
                let count = values.len() as u32;
                if count < count_threshold {
                    all_conditions_met = false;
                    break;
                }
            } else if !condition_met {
                all_conditions_met = false;
                break;
            }
        }

        if all_conditions_met {
            eligible.push(def.id.clone());
        }
    }

    eligible
}

pub fn apply_effects(
    effects: &[Effect],
    mut debts: ResMut<Debts>,
    current_tick: u64,
    mut commands: Commands,
) {
    for effect in effects {
        match effect {
            Effect::DebtPowerMult { mult, duration_ms } => {
                let until_tick = current_tick + (duration_ms / 16);
                debts.add_debt(Debt::PowerMult { mult: *mult, until_tick });
            }
            Effect::DebtHeatAdd { celsius, duration_ms } => {
                let until_tick = current_tick + (duration_ms / 16);
                debts.add_debt(Debt::HeatAdd { celsius: *celsius, until_tick });
            }
            Effect::BandwidthTax { mult, duration_ms } => {
                let until_tick = current_tick + (duration_ms / 16);
                debts.add_debt(Debt::BandwidthTax { mult: *mult, until_tick });
            }
            Effect::VramLeak { mb_per_tick, duration_ms } => {
                let until_tick = current_tick + (duration_ms / 16);
                debts.add_debt(Debt::VramLeak { mb_per_tick: *mb_per_tick, until_tick });
            }
            Effect::FaultBias { kind, weight_mult, duration_ms } => {
                let until_tick = current_tick + (duration_ms / 16);
                debts.add_debt(Debt::FaultBias { 
                    kind: kind.clone(), 
                    weight_mult: *weight_mult, 
                    until_tick 
                });
            }
            Effect::UIIllusion { metric, delta, duration_ms } => {
                let until_tick = current_tick + (duration_ms / 16);
                debts.add_debt(Debt::Illusion { 
                    metric: metric.clone(), 
                    delta: *delta, 
                    until_tick 
                });
            }
            Effect::InsertOp { pipeline_id, where_, op } => {
                // TODO: Implement pipeline mutation
                println!("Black Swan: InsertOp {} in pipeline {} at {}", op, pipeline_id, where_);
            }
            Effect::ReplaceOp { pipeline_id, from, to } => {
                // TODO: Implement pipeline mutation
                println!("Black Swan: ReplaceOp {} with {} in pipeline {}", from, to, pipeline_id);
            }
            Effect::RemoveOp { pipeline_id, op } => {
                // TODO: Implement pipeline mutation
                println!("Black Swan: RemoveOp {} from pipeline {}", op, pipeline_id);
            }
            Effect::BranchDualRun { pipeline_id, adjudicator } => {
                // TODO: Implement pipeline mutation
                println!("Black Swan: BranchDualRun {} in pipeline {}", adjudicator, pipeline_id);
            }
            Effect::QuarantinePipeline { pipeline_id, domain } => {
                // TODO: Implement pipeline quarantine
                println!("Black Swan: QuarantinePipeline {} in domain {:?}", pipeline_id, domain);
            }
            Effect::RequireRitual { ritual_id } => {
                // TODO: Implement ritual requirement
                println!("Black Swan: RequireRitual {}", ritual_id);
            }
        }
    }
}

pub fn black_swan_scan_system(
    mut black_swan_index: ResMut<BlackSwanIndex>,
    kpi_buffer: Res<KpiRingBuffer>,
    clock: Res<super::SimClock>,
    mut debts: ResMut<Debts>,
    commands: Commands,
) {
    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    
    // Clear expired cooldowns
    black_swan_index.clear_expired_cooldowns(current_tick);
    
    // Evaluate triggers
    let eligible = evaluate_triggers(&black_swan_index, &kpi_buffer, current_tick);
    
    // Fire eligible Black Swans (for now, fire the first one)
    if let Some(swan_id) = eligible.first() {
        if let Some(swan_def) = black_swan_index.defs.iter().find(|def| def.id == *swan_id) {
            println!("Black Swan fired: {} - {}", swan_def.id, swan_def.name);
            
            // Apply effects
            apply_effects(&swan_def.effects, debts, current_tick, commands);
            
            // Mark as fired
            black_swan_index.mark_fired(swan_id.clone(), current_tick);
            black_swan_index.meters.active.push(swan_id.clone());
        }
    }
}

pub fn update_kpi_buffer_system(
    mut kpi_buffer: ResMut<KpiRingBuffer>,
    colony: Res<super::Colony>,
    clock: Res<super::SimClock>,
) {
    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    
    kpi_buffer.add_bandwidth_util(colony.meters.bandwidth_util, current_tick);
    kpi_buffer.add_corruption_field(colony.corruption_field, current_tick);
    kpi_buffer.add_power_draw(colony.meters.power_draw_kw, current_tick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_evaluation() {
        let mut black_swan_index = BlackSwanIndex::new();
        let mut kpi_buffer = KpiRingBuffer::new();
        let current_tick = 1000;

        // Add a Black Swan definition
        let swan_def = BlackSwanDef {
            id: "test_swan".to_string(),
            name: "Test Swan".to_string(),
            triggers: vec![
                TriggerCond {
                    metric: "bandwidth_util".to_string(),
                    op: ">".to_string(),
                    value: 0.9,
                    window_ms: 5000,
                    count_at_least: None,
                }
            ],
            effects: vec![],
            cure: None,
            weight: 1.0,
            cooldown_ms: 10000,
        };
        black_swan_index.add_black_swan(swan_def);

        // Add some KPI data
        kpi_buffer.add_bandwidth_util(0.95, current_tick - 100);
        kpi_buffer.add_bandwidth_util(0.85, current_tick - 200);

        let eligible = evaluate_triggers(&black_swan_index, &kpi_buffer, current_tick);
        assert!(eligible.contains(&"test_swan".to_string()));
    }

    #[test]
    fn test_cooldown_mechanism() {
        let mut black_swan_index = BlackSwanIndex::new();
        let current_tick = 1000;

        let swan_def = BlackSwanDef {
            id: "test_swan".to_string(),
            name: "Test Swan".to_string(),
            triggers: vec![],
            effects: vec![],
            cure: None,
            weight: 1.0,
            cooldown_ms: 10000, // 10 seconds
        };
        black_swan_index.add_black_swan(swan_def);

        // Mark as fired
        black_swan_index.mark_fired("test_swan".to_string(), current_tick);

        // Should be on cooldown
        assert!(black_swan_index.is_on_cooldown("test_swan", current_tick + 100));
        
        // Should be off cooldown after enough time
        assert!(!black_swan_index.is_on_cooldown("test_swan", current_tick + 1000));
    }
}
