use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Debt {
    PowerMult { mult: f32, until_tick: u64 },
    HeatAdd { celsius: f32, until_tick: u64 },
    BandwidthTax { mult: f32, until_tick: u64 },
    VramLeak { mb_per_tick: f32, until_tick: u64 },
    FaultBias { kind: String, weight_mult: f32, until_tick: u64 },
    Illusion { metric: String, delta: f32, until_tick: u64 }, // UI only
}

impl Debt {
    pub fn is_expired(&self, current_tick: u64) -> bool {
        match self {
            Debt::PowerMult { until_tick, .. } => current_tick >= *until_tick,
            Debt::HeatAdd { until_tick, .. } => current_tick >= *until_tick,
            Debt::BandwidthTax { until_tick, .. } => current_tick >= *until_tick,
            Debt::VramLeak { until_tick, .. } => current_tick >= *until_tick,
            Debt::FaultBias { until_tick, .. } => current_tick >= *until_tick,
            Debt::Illusion { until_tick, .. } => current_tick >= *until_tick,
        }
    }

    pub fn get_until_tick(&self) -> u64 {
        match self {
            Debt::PowerMult { until_tick, .. } => *until_tick,
            Debt::HeatAdd { until_tick, .. } => *until_tick,
            Debt::BandwidthTax { until_tick, .. } => *until_tick,
            Debt::VramLeak { until_tick, .. } => *until_tick,
            Debt::FaultBias { until_tick, .. } => *until_tick,
            Debt::Illusion { until_tick, .. } => *until_tick,
        }
    }
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct Debts {
    pub active: Vec<Debt>,
}

impl Debts {
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
        }
    }

    pub fn add_debt(&mut self, debt: Debt) {
        self.active.push(debt);
    }

    pub fn remove_debt(&mut self, index: usize) {
        if index < self.active.len() {
            self.active.remove(index);
        }
    }

    pub fn clear_expired(&mut self, current_tick: u64) {
        self.active.retain(|debt| !debt.is_expired(current_tick));
    }

    pub fn get_power_multiplier(&self, current_tick: u64) -> f32 {
        self.active
            .iter()
            .filter(|debt| !debt.is_expired(current_tick))
            .filter_map(|debt| {
                if let Debt::PowerMult { mult, .. } = debt {
                    Some(*mult)
                } else {
                    None
                }
            })
            .fold(1.0, |acc, mult| acc * mult)
    }

    pub fn get_heat_addition(&self, current_tick: u64) -> f32 {
        self.active
            .iter()
            .filter(|debt| !debt.is_expired(current_tick))
            .filter_map(|debt| {
                if let Debt::HeatAdd { celsius, .. } = debt {
                    Some(*celsius)
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn get_bandwidth_tax(&self, current_tick: u64) -> f32 {
        self.active
            .iter()
            .filter(|debt| !debt.is_expired(current_tick))
            .filter_map(|debt| {
                if let Debt::BandwidthTax { mult, .. } = debt {
                    Some(*mult)
                } else {
                    None
                }
            })
            .fold(1.0, |acc, mult| acc * mult)
    }

    pub fn get_vram_leak(&self, current_tick: u64) -> f32 {
        self.active
            .iter()
            .filter(|debt| !debt.is_expired(current_tick))
            .filter_map(|debt| {
                if let Debt::VramLeak { mb_per_tick, .. } = debt {
                    Some(*mb_per_tick)
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn get_fault_bias(&self, fault_kind: &str, current_tick: u64) -> f32 {
        self.active
            .iter()
            .filter(|debt| !debt.is_expired(current_tick))
            .filter_map(|debt| {
                if let Debt::FaultBias { kind, weight_mult, .. } = debt {
                    if kind == fault_kind {
                        Some(*weight_mult)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .fold(1.0, |acc, mult| acc * mult)
    }

    pub fn get_illusions(&self, current_tick: u64) -> HashMap<String, f32> {
        let mut illusions = HashMap::new();
        for debt in &self.active {
            if !debt.is_expired(current_tick) {
                if let Debt::Illusion { metric, delta, .. } = debt {
                    illusions.insert(metric.clone(), *delta);
                }
            }
        }
        illusions
    }

    pub fn clear_debts_by_type(&mut self, debt_type: &str) {
        match debt_type {
            "PowerMult" => {
                self.active.retain(|debt| !matches!(debt, Debt::PowerMult { .. }));
            }
            "HeatAdd" => {
                self.active.retain(|debt| !matches!(debt, Debt::HeatAdd { .. }));
            }
            "BandwidthTax" => {
                self.active.retain(|debt| !matches!(debt, Debt::BandwidthTax { .. }));
            }
            "VramLeak" => {
                self.active.retain(|debt| !matches!(debt, Debt::VramLeak { .. }));
            }
            "FaultBias" => {
                self.active.retain(|debt| !matches!(debt, Debt::FaultBias { .. }));
            }
            "Illusion" => {
                self.active.retain(|debt| !matches!(debt, Debt::Illusion { .. }));
            }
            _ => {}
        }
    }
}

pub fn apply_debts_system(
    mut debts: ResMut<Debts>,
    clock: Res<super::SimClock>,
) {
    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    debts.clear_expired(current_tick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debt_expiration() {
        let debt = Debt::PowerMult { mult: 1.2, until_tick: 100 };
        assert!(!debt.is_expired(50));
        assert!(debt.is_expired(100));
        assert!(debt.is_expired(150));
    }

    #[test]
    fn test_debt_accumulation() {
        let mut debts = Debts::new();
        let current_tick = 100;

        debts.add_debt(Debt::PowerMult { mult: 1.2, until_tick: 200 });
        debts.add_debt(Debt::PowerMult { mult: 1.1, until_tick: 200 });
        debts.add_debt(Debt::HeatAdd { celsius: 5.0, until_tick: 200 });

        assert_eq!(debts.get_power_multiplier(current_tick), 1.2 * 1.1);
        assert_eq!(debts.get_heat_addition(current_tick), 5.0);
    }

    #[test]
    fn test_debt_clearing() {
        let mut debts = Debts::new();
        let current_tick = 100;

        debts.add_debt(Debt::PowerMult { mult: 1.2, until_tick: 50 }); // expired
        debts.add_debt(Debt::PowerMult { mult: 1.1, until_tick: 200 }); // active

        debts.clear_expired(current_tick);
        assert_eq!(debts.active.len(), 1);
        assert_eq!(debts.get_power_multiplier(current_tick), 1.1);
    }

    #[test]
    fn test_fault_bias() {
        let mut debts = Debts::new();
        let current_tick = 100;

        debts.add_debt(Debt::FaultBias { 
            kind: "StickyConfig".to_string(), 
            weight_mult: 1.5, 
            until_tick: 200 
        });

        assert_eq!(debts.get_fault_bias("StickyConfig", current_tick), 1.5);
        assert_eq!(debts.get_fault_bias("Transient", current_tick), 1.0);
    }
}
