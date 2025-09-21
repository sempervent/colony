use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct SaveFileV1 {
    pub version: u32,                // =1
    pub game_setup: super::game_config::GameSetup,
    pub colony_state: ColonyState,
    pub research_state: super::ResearchState,
    pub black_swan_state: super::BlackSwanIndex,
    pub debts: super::Debts,
    pub winloss: super::victory::WinLossState,
    pub session_ctl: super::session::SessionCtl,
    pub replay_log: super::session::ReplayLog,
    pub kpis: KpiSummary,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ColonyState {
    pub power_cap_kw: f32,
    pub bandwidth_total_gbps: f32,
    pub corruption_field: f32,
    pub target_uptime_days: u32,
    pub meters: super::GlobalMeters,
    pub tunables: super::ResourceTunables,
    pub corruption_tun: super::CorruptionTunables,
    pub seed: u64,
}

#[derive(Serialize, Deserialize)]
pub struct KpiSummary {
    pub bandwidth_util_history: Vec<f32>,
    pub corruption_field_history: Vec<f32>,
    pub power_draw_history: Vec<f32>,
    pub heat_levels_history: Vec<f32>,
    pub deadline_hit_rates: Vec<f32>,
    pub black_swan_events: Vec<(String, u64)>, // (event_id, tick)
}

impl SaveFileV1 {
    pub fn new(
        game_setup: super::game_config::GameSetup,
        colony: &super::Colony,
        research_state: &super::ResearchState,
        black_swan_state: &super::BlackSwanIndex,
        debts: &super::Debts,
        winloss: &super::victory::WinLossState,
        session_ctl: &super::session::SessionCtl,
        replay_log: &super::session::ReplayLog,
        kpi_summary: KpiSummary,
    ) -> Self {
        Self {
            version: 1,
            game_setup,
            colony_state: ColonyState {
                power_cap_kw: colony.power_cap_kw,
                bandwidth_total_gbps: colony.bandwidth_total_gbps,
                corruption_field: colony.corruption_field,
                target_uptime_days: colony.target_uptime_days,
                meters: colony.meters.clone(),
                tunables: colony.tunables.clone(),
                corruption_tun: colony.corruption_tun.clone(),
                seed: colony.seed,
            },
            research_state: research_state.clone(),
            black_swan_state: black_swan_state.clone(),
            debts: debts.clone(),
            winloss: winloss.clone(),
            session_ctl: session_ctl.clone(),
            replay_log: replay_log.clone(),
            kpis: kpi_summary,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

pub fn migrate_any_to_latest(bytes: &[u8]) -> anyhow::Result<SaveFileV1> {
    // Try to deserialize as V1 first
    if let Ok(save) = serde_json::from_slice::<SaveFileV1>(bytes) {
        return Ok(save);
    }

    // Try to deserialize as raw JSON and check version
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(version) = json.get("version").and_then(|v| v.as_u64()) {
            match version {
                1 => {
                    // Already V1, try to deserialize again
                    if let Ok(save) = serde_json::from_value::<SaveFileV1>(json) {
                        return Ok(save);
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!("Unsupported save version: {}", version));
                }
            }
        }
    }

    // If we get here, it's not a recognized save format
    Err(anyhow::anyhow!("Invalid save file format"))
}

pub fn save_to_file(
    save_data: &SaveFileV1,
    file_path: &str,
) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(save_data)?;
    std::fs::write(file_path, json)?;
    Ok(())
}

pub fn load_from_file(
    file_path: &str,
) -> anyhow::Result<SaveFileV1> {
    let bytes = std::fs::read(file_path)?;
    migrate_any_to_latest(&bytes)
}

pub fn get_save_slots() -> anyhow::Result<Vec<String>> {
    let save_dir = "saves";
    if !std::path::Path::new(save_dir).exists() {
        std::fs::create_dir_all(save_dir)?;
    }

    let mut slots = Vec::new();
    for entry in std::fs::read_dir(save_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                slots.push(stem.to_string());
            }
        }
    }
    slots.sort();
    Ok(slots)
}

pub fn save_to_slot(
    save_data: &SaveFileV1,
    slot_name: &str,
) -> anyhow::Result<()> {
    let save_dir = "saves";
    if !std::path::Path::new(save_dir).exists() {
        std::fs::create_dir_all(save_dir)?;
    }

    let file_path = format!("{}/{}.json", save_dir, slot_name);
    save_to_file(save_data, &file_path)
}

pub fn load_from_slot(
    slot_name: &str,
) -> anyhow::Result<SaveFileV1> {
    let file_path = format!("saves/{}.json", slot_name);
    load_from_file(&file_path)
}

pub fn delete_slot(
    slot_name: &str,
) -> anyhow::Result<()> {
    let file_path = format!("saves/{}.json", slot_name);
    if std::path::Path::new(&file_path).exists() {
        std::fs::remove_file(file_path)?;
    }
    Ok(())
}

pub fn get_slot_info(
    slot_name: &str,
) -> anyhow::Result<SlotInfo> {
    let save_data = load_from_slot(slot_name)?;
    Ok(SlotInfo {
        name: slot_name.to_string(),
        scenario: save_data.game_setup.scenario.name,
        difficulty: save_data.game_setup.scenario.difficulty.name,
        timestamp: save_data.timestamp,
        victory: save_data.winloss.victory,
        doom: save_data.winloss.doom,
        score: save_data.winloss.score,
        achieved_days: save_data.winloss.achieved_days,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub name: String,
    pub scenario: String,
    pub difficulty: String,
    pub timestamp: u64,
    pub victory: bool,
    pub doom: bool,
    pub score: i64,
    pub achieved_days: u32,
}

impl SlotInfo {
    pub fn format_timestamp(&self) -> String {
        let dt = chrono::DateTime::from_timestamp(self.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn status(&self) -> String {
        if self.victory {
            "Victory".to_string()
        } else if self.doom {
            "Defeat".to_string()
        } else {
            "In Progress".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_file_creation() {
        let game_setup = super::super::game_config::GameSetup::new(
            super::super::game_config::Scenario {
                id: "test".to_string(),
                name: "Test Scenario".to_string(),
                description: "Test".to_string(),
                seed: 42,
                difficulty: super::super::game_config::Difficulty::default(),
                victory: super::super::game_config::VictoryRules::default(),
                loss: super::super::game_config::LossRules::default(),
                start_tunables: None,
                enabled_pipelines: None,
                enabled_events: None,
            }
        );

        let colony = super::super::Colony {
            power_cap_kw: 1000.0,
            bandwidth_total_gbps: 32.0,
            corruption_field: 0.1,
            target_uptime_days: 365,
            meters: super::super::GlobalMeters::new(),
            tunables: super::super::ResourceTunables::default(),
            corruption_tun: super::super::CorruptionTunables::default(),
            seed: 42,
        };

        let research_state = super::super::ResearchState::new();
        let black_swan_state = super::super::BlackSwanIndex::new();
        let debts = super::super::Debts::new();
        let winloss = super::super::victory::WinLossState::new();
        let session_ctl = super::super::session::SessionCtl::new();
        let replay_log = super::super::session::ReplayLog::new();
        let kpi_summary = KpiSummary {
            bandwidth_util_history: vec![0.5, 0.6, 0.7],
            corruption_field_history: vec![0.1, 0.2, 0.3],
            power_draw_history: vec![800.0, 900.0, 1000.0],
            heat_levels_history: vec![50.0, 60.0, 70.0],
            deadline_hit_rates: vec![99.0, 98.5, 99.2],
            black_swan_events: vec![("test_event".to_string(), 1000)],
        };

        let save_data = SaveFileV1::new(
            game_setup,
            &colony,
            &research_state,
            &black_swan_state,
            &debts,
            &winloss,
            &session_ctl,
            &replay_log,
            kpi_summary,
        );

        assert_eq!(save_data.version, 1);
        assert_eq!(save_data.colony_state.power_cap_kw, 1000.0);
    }

    #[test]
    fn test_save_slot_operations() {
        // Create a temporary save file
        let game_setup = super::super::game_config::GameSetup::new(
            super::super::game_config::Scenario {
                id: "test".to_string(),
                name: "Test Scenario".to_string(),
                description: "Test".to_string(),
                seed: 42,
                difficulty: super::super::game_config::Difficulty::default(),
                victory: super::super::game_config::VictoryRules::default(),
                loss: super::super::game_config::LossRules::default(),
                start_tunables: None,
                enabled_pipelines: None,
                enabled_events: None,
            }
        );

        let colony = super::super::Colony {
            power_cap_kw: 1000.0,
            bandwidth_total_gbps: 32.0,
            corruption_field: 0.1,
            target_uptime_days: 365,
            meters: super::super::GlobalMeters::new(),
            tunables: super::super::ResourceTunables::default(),
            corruption_tun: super::super::CorruptionTunables::default(),
            seed: 42,
        };

        let research_state = super::super::ResearchState::new();
        let black_swan_state = super::super::BlackSwanIndex::new();
        let debts = super::super::Debts::new();
        let winloss = super::super::victory::WinLossState::new();
        let session_ctl = super::super::session::SessionCtl::new();
        let replay_log = super::super::session::ReplayLog::new();
        let kpi_summary = KpiSummary {
            bandwidth_util_history: vec![],
            corruption_field_history: vec![],
            power_draw_history: vec![],
            heat_levels_history: vec![],
            deadline_hit_rates: vec![],
            black_swan_events: vec![],
        };

        let save_data = SaveFileV1::new(
            game_setup,
            &colony,
            &research_state,
            &black_swan_state,
            &debts,
            &winloss,
            &session_ctl,
            &replay_log,
            kpi_summary,
        );

        // Test save/load cycle
        let slot_name = "test_slot";
        save_to_slot(&save_data, slot_name).unwrap();
        let loaded_data = load_from_slot(slot_name).unwrap();
        assert_eq!(loaded_data.version, save_data.version);

        // Test slot info
        let slot_info = get_slot_info(slot_name).unwrap();
        assert_eq!(slot_info.name, slot_name);
        assert_eq!(slot_info.scenario, "Test Scenario");

        // Clean up
        delete_slot(slot_name).unwrap();
    }
}
