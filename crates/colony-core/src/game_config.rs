use serde::{Serialize, Deserialize};
use serde_json;
// HashMap import removed - not used in this file

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difficulty {
    pub name: String,                    // "Chill", "Nominal", "Abyssal"
    pub power_cap_mult: f32,
    pub heat_cap_mult: f32,
    pub bw_total_mult: f32,
    pub fault_rate_mult: f32,
    pub black_swan_weight_mult: f32,
    pub research_rate_mult: f32,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self {
            name: "Nominal".to_string(),
            power_cap_mult: 1.0,
            heat_cap_mult: 1.0,
            bw_total_mult: 1.0,
            fault_rate_mult: 1.0,
            black_swan_weight_mult: 1.0,
            research_rate_mult: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryRules {
    pub target_uptime_days: u32,         // win after maintaining SLA for N sim days
    pub min_deadline_hit_pct: f32,       // e.g., 99.5
    pub max_corruption_field: f32,       // e.g., 0.35
    pub observation_window_days: u32,    // rolling window for SLA verification
}

impl Default for VictoryRules {
    fn default() -> Self {
        Self {
            target_uptime_days: 365,
            min_deadline_hit_pct: 99.5,
            max_corruption_field: 0.35,
            observation_window_days: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossRules {
    pub hard_power_deficit_ticks: u32,   // exceed cap for X ticks
    pub sustained_deadline_miss_pct: f32,// e.g., >5% over window
    pub max_sticky_workers: u32,         // doom if too many quarantined
    pub black_swan_chain_len: u32,       // doom if Y swans stack without cure
    pub time_limit_days: Option<u32>,    // optional sudden death
}

impl Default for LossRules {
    fn default() -> Self {
        Self {
            hard_power_deficit_ticks: 1000, // ~16 seconds at 16ms ticks
            sustained_deadline_miss_pct: 5.0,
            max_sticky_workers: 3,
            black_swan_chain_len: 3,
            time_limit_days: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub seed: u64,
    pub difficulty: Difficulty,
    pub victory: VictoryRules,
    pub loss: LossRules,
    pub start_tunables: Option<serde_json::Value>, // override knobs (power, heat, gpu, corruption, etc.)
    pub enabled_pipelines: Option<Vec<String>>,    // subset for small starts
    pub enabled_events: Option<Vec<String>>,       // restrict Black Swans
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSetup {
    pub scenario: Scenario,
    pub mods: Vec<String>,               // loaded mod IDs
    pub tick_scale: String,              // "RealTime" | "Seconds:1" | "Days:1" | "Years:1..10"
}

impl GameSetup {
    pub fn new(scenario: Scenario) -> Self {
        Self {
            scenario,
            mods: vec!["vanilla".to_string()],
            tick_scale: "RealTime".to_string(),
        }
    }
}

pub fn load_scenarios() -> anyhow::Result<Vec<Scenario>> {
    // For now, return hardcoded scenarios
    // In a real implementation, this would read from colony-content/scenarios.toml + mods/*/scenarios.toml
    Ok(vec![
        Scenario {
            id: "first_light_chill".to_string(),
            name: "First Light (Chill)".to_string(),
            description: "A gentle introduction to colony management. Small CPU yard, low I/O load, lenient rules.".to_string(),
            seed: 42,
            difficulty: Difficulty {
                name: "Chill".to_string(),
                power_cap_mult: 1.2,
                heat_cap_mult: 1.1,
                bw_total_mult: 1.1,
                fault_rate_mult: 0.5,
                black_swan_weight_mult: 0.3,
                research_rate_mult: 1.5,
            },
            victory: VictoryRules {
                target_uptime_days: 30,
                min_deadline_hit_pct: 95.0,
                max_corruption_field: 0.5,
                observation_window_days: 3,
            },
            loss: LossRules {
                hard_power_deficit_ticks: 2000,
                sustained_deadline_miss_pct: 10.0,
                max_sticky_workers: 5,
                black_swan_chain_len: 5,
                time_limit_days: None,
            },
            start_tunables: None,
            enabled_pipelines: Some(vec![
                "udp_telemetry_ingest".to_string(),
                "http_ingest".to_string(),
            ]),
            enabled_events: Some(vec![
                "pcie_link_flap".to_string(),
            ]),
        },
        Scenario {
            id: "factory_horizon_nominal".to_string(),
            name: "Factory Horizon (Nominal)".to_string(),
            description: "Standard industrial operation. GPU enabled, moderate I/O load, balanced rules.".to_string(),
            seed: 123,
            difficulty: Difficulty {
                name: "Nominal".to_string(),
                power_cap_mult: 1.0,
                heat_cap_mult: 1.0,
                bw_total_mult: 1.0,
                fault_rate_mult: 1.0,
                black_swan_weight_mult: 1.0,
                research_rate_mult: 1.0,
            },
            victory: VictoryRules {
                target_uptime_days: 365,
                min_deadline_hit_pct: 99.5,
                max_corruption_field: 0.35,
                observation_window_days: 7,
            },
            loss: LossRules {
                hard_power_deficit_ticks: 1000,
                sustained_deadline_miss_pct: 5.0,
                max_sticky_workers: 3,
                black_swan_chain_len: 3,
                time_limit_days: None,
            },
            start_tunables: None,
            enabled_pipelines: None, // All pipelines enabled
            enabled_events: None,    // All events enabled
        },
        Scenario {
            id: "signal_tempest_abyssal".to_string(),
            name: "Signal Tempest (Abyssal)".to_string(),
            description: "Extreme conditions. High I/O bursts, strict corruption limits, aggressive Black Swans.".to_string(),
            seed: 666,
            difficulty: Difficulty {
                name: "Abyssal".to_string(),
                power_cap_mult: 0.8,
                heat_cap_mult: 0.9,
                bw_total_mult: 0.8,
                fault_rate_mult: 2.0,
                black_swan_weight_mult: 2.5,
                research_rate_mult: 0.7,
            },
            victory: VictoryRules {
                target_uptime_days: 180,
                min_deadline_hit_pct: 99.8,
                max_corruption_field: 0.25,
                observation_window_days: 14,
            },
            loss: LossRules {
                hard_power_deficit_ticks: 500,
                sustained_deadline_miss_pct: 2.0,
                max_sticky_workers: 2,
                black_swan_chain_len: 2,
                time_limit_days: Some(200),
            },
            start_tunables: None,
            enabled_pipelines: None, // All pipelines enabled
            enabled_events: None,    // All events enabled
        },
    ])
}

pub fn apply_difficulty_scaling(
    difficulty: &Difficulty,
    colony: &mut super::Colony,
    corruption_tun: &mut super::CorruptionTunables,
) {
    // Scale power cap
    colony.power_cap_kw *= difficulty.power_cap_mult;
    
    // Scale bandwidth
    colony.bandwidth_total_gbps *= difficulty.bw_total_mult;
    
    // Scale fault rates
    corruption_tun.base_fault_rate *= difficulty.fault_rate_mult;
    
    // Note: Heat caps and other tunables would be scaled here in a full implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_defaults() {
        let diff = Difficulty::default();
        assert_eq!(diff.name, "Nominal");
        assert_eq!(diff.power_cap_mult, 1.0);
    }

    #[test]
    fn test_victory_rules_defaults() {
        let rules = VictoryRules::default();
        assert_eq!(rules.target_uptime_days, 365);
        assert_eq!(rules.min_deadline_hit_pct, 99.5);
    }

    #[test]
    fn test_loss_rules_defaults() {
        let rules = LossRules::default();
        assert_eq!(rules.hard_power_deficit_ticks, 1000);
        assert_eq!(rules.sustained_deadline_miss_pct, 5.0);
    }

    #[test]
    fn test_scenario_loading() {
        let scenarios = load_scenarios().unwrap();
        assert!(!scenarios.is_empty());
        
        let first_light = scenarios.iter().find(|s| s.id == "first_light_chill").unwrap();
        assert_eq!(first_light.name, "First Light (Chill)");
        assert_eq!(first_light.difficulty.name, "Chill");
    }

    #[test]
    fn test_game_setup_creation() {
        let scenarios = load_scenarios().unwrap();
        let scenario = scenarios[0].clone();
        let setup = GameSetup::new(scenario);
        
        assert_eq!(setup.mods, vec!["vanilla"]);
        assert_eq!(setup.tick_scale, "RealTime");
    }
}
