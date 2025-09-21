use bevy::prelude::*;
use colony_modsdk::{HotReloadTransaction, HotReloadStatus, ShadowWorldResult, KpiDeltas};
use std::collections::HashMap;
use anyhow::Result;

/// Hot reload manager for atomic mod updates
#[derive(Resource, Default)]
pub struct HotReloadManager {
    pub active_transactions: HashMap<String, HotReloadTransaction>,
    pub shadow_world_state: Option<ShadowWorldState>,
    pub validation_thresholds: ValidationThresholds,
    pub dry_run_ticks: u32,
}

/// State for shadow world validation
#[derive(Debug, Clone)]
pub struct ShadowWorldState {
    pub mod_id: String,
    pub baseline_kpis: KpiSnapshot,
    pub current_kpis: KpiSnapshot,
    pub ticks_simulated: u32,
    pub max_ticks: u32,
}

/// KPI snapshot for comparison
#[derive(Debug, Clone, Default)]
pub struct KpiSnapshot {
    pub deadline_hit_rate: f32,
    pub power_draw_kw: f32,
    pub bandwidth_util: f32,
    pub corruption_field: f32,
    pub heat_levels: Vec<f32>,
    pub gpu_util: f32,
    pub vram_used_mb: f32,
}

/// Validation thresholds for hot reload
#[derive(Debug, Clone)]
pub struct ValidationThresholds {
    pub max_deadline_hit_rate_change: f32,
    pub max_power_draw_change: f32,
    pub max_bandwidth_util_change: f32,
    pub max_corruption_field_change: f32,
    pub max_heat_level_change: f32,
    pub max_gpu_util_change: f32,
    pub max_vram_usage_change: f32,
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            max_deadline_hit_rate_change: 3.0, // ±3%
            max_power_draw_change: 10.0, // +10%
            max_bandwidth_util_change: 5.0, // ±5%
            max_corruption_field_change: 0.05, // ±0.05
            max_heat_level_change: 5.0, // ±5°C
            max_gpu_util_change: 10.0, // ±10%
            max_vram_usage_change: 20.0, // +20%
        }
    }
}

impl HotReloadManager {
    pub fn new() -> Self {
        Self {
            active_transactions: HashMap::new(),
            shadow_world_state: None,
            validation_thresholds: ValidationThresholds::default(),
            dry_run_ticks: 120, // Default 120 ticks for dry run
        }
    }

    pub fn start_hot_reload(&mut self, transaction: HotReloadTransaction) -> Result<()> {
        if self.active_transactions.contains_key(&transaction.mod_id) {
            return Err(anyhow::anyhow!("Hot reload already in progress for mod: {}", transaction.mod_id));
        }

        let mut transaction = transaction;
        transaction.status = HotReloadStatus::Validating;
        
        self.active_transactions.insert(transaction.mod_id.clone(), transaction);
        Ok(())
    }

    pub fn start_shadow_world(&mut self, mod_id: &str, baseline_kpis: KpiSnapshot) -> Result<()> {
        if self.shadow_world_state.is_some() {
            return Err(anyhow::anyhow!("Shadow world already running"));
        }

        let shadow_state = ShadowWorldState {
            mod_id: mod_id.to_string(),
            baseline_kpis: baseline_kpis.clone(),
            current_kpis: baseline_kpis,
            ticks_simulated: 0,
            max_ticks: self.dry_run_ticks,
        };

        self.shadow_world_state = Some(shadow_state);
        Ok(())
    }

    pub fn update_shadow_world(&mut self, current_kpis: KpiSnapshot) -> Result<Option<ShadowWorldResult>> {
        let shadow_state = self.shadow_world_state.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No shadow world running"))?;

        shadow_state.current_kpis = current_kpis;
        shadow_state.ticks_simulated += 1;

        if shadow_state.ticks_simulated >= shadow_state.max_ticks {
            // Shadow world simulation complete
            let result = self.evaluate_shadow_world_result(shadow_state)?;
            self.shadow_world_state = None;
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn evaluate_shadow_world_result(&self, shadow_state: &ShadowWorldState) -> Result<ShadowWorldResult> {
        let kpi_deltas = self.calculate_kpi_deltas(&shadow_state.baseline_kpis, &shadow_state.current_kpis);
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check validation thresholds
        if kpi_deltas.deadline_hit_rate_change.abs() > self.validation_thresholds.max_deadline_hit_rate_change {
            errors.push(format!(
                "Deadline hit rate change too large: {:.2}% (max: {:.2}%)",
                kpi_deltas.deadline_hit_rate_change,
                self.validation_thresholds.max_deadline_hit_rate_change
            ));
        }

        if kpi_deltas.power_draw_change > self.validation_thresholds.max_power_draw_change {
            errors.push(format!(
                "Power draw increase too large: {:.2}% (max: {:.2}%)",
                kpi_deltas.power_draw_change,
                self.validation_thresholds.max_power_draw_change
            ));
        }

        if kpi_deltas.bandwidth_util_change.abs() > self.validation_thresholds.max_bandwidth_util_change {
            warnings.push(format!(
                "Bandwidth utilization change: {:.2}% (max: {:.2}%)",
                kpi_deltas.bandwidth_util_change,
                self.validation_thresholds.max_bandwidth_util_change
            ));
        }

        if kpi_deltas.corruption_field_change.abs() > self.validation_thresholds.max_corruption_field_change {
            errors.push(format!(
                "Corruption field change too large: {:.3} (max: {:.3})",
                kpi_deltas.corruption_field_change,
                self.validation_thresholds.max_corruption_field_change
            ));
        }

        // Check heat level changes
        for (i, heat_change) in kpi_deltas.heat_levels_change.iter().enumerate() {
            if heat_change.abs() > self.validation_thresholds.max_heat_level_change {
                warnings.push(format!(
                    "Heat level change in yard {}: {:.1}°C (max: {:.1}°C)",
                    i, heat_change, self.validation_thresholds.max_heat_level_change
                ));
            }
        }

        let success = errors.is_empty();

        Ok(ShadowWorldResult {
            success,
            kpi_deltas,
            errors,
            warnings,
            ticks_simulated: shadow_state.ticks_simulated,
        })
    }

    fn calculate_kpi_deltas(&self, baseline: &KpiSnapshot, current: &KpiSnapshot) -> KpiDeltas {
        KpiDeltas {
            deadline_hit_rate_change: current.deadline_hit_rate - baseline.deadline_hit_rate,
            power_draw_change: if baseline.power_draw_kw > 0.0 {
                ((current.power_draw_kw - baseline.power_draw_kw) / baseline.power_draw_kw) * 100.0
            } else {
                0.0
            },
            bandwidth_util_change: current.bandwidth_util - baseline.bandwidth_util,
            corruption_field_change: current.corruption_field - baseline.corruption_field,
            heat_levels_change: current.heat_levels.iter()
                .zip(baseline.heat_levels.iter())
                .map(|(curr, base)| curr - base)
                .collect(),
        }
    }

    pub fn complete_hot_reload(&mut self, mod_id: &str, success: bool) -> Result<()> {
        if let Some(transaction) = self.active_transactions.get_mut(mod_id) {
            transaction.status = if success {
                HotReloadStatus::Applied
            } else {
                HotReloadStatus::Failed
            };
        }

        Ok(())
    }

    pub fn cancel_hot_reload(&mut self, mod_id: &str) -> Result<()> {
        if let Some(transaction) = self.active_transactions.get_mut(mod_id) {
            transaction.status = HotReloadStatus::Reverted;
        }

        // Cancel shadow world if running for this mod
        if let Some(ref shadow_state) = self.shadow_world_state {
            if shadow_state.mod_id == mod_id {
                self.shadow_world_state = None;
            }
        }

        Ok(())
    }

    pub fn get_active_transactions(&self) -> Vec<&HotReloadTransaction> {
        self.active_transactions.values().collect()
    }

    pub fn is_shadow_world_running(&self) -> bool {
        self.shadow_world_state.is_some()
    }

    pub fn get_shadow_world_progress(&self) -> Option<(u32, u32)> {
        self.shadow_world_state.as_ref().map(|s| (s.ticks_simulated, s.max_ticks))
    }

    pub fn set_validation_thresholds(&mut self, thresholds: ValidationThresholds) {
        self.validation_thresholds = thresholds;
    }

    pub fn set_dry_run_ticks(&mut self, ticks: u32) {
        self.dry_run_ticks = ticks;
    }
}

/// System to process hot reload transactions
pub fn process_hot_reload_system(
    mut hot_reload_manager: ResMut<HotReloadManager>,
    // TODO: Add resources needed for KPI collection
) {
    // Process active transactions
    let mut completed_transactions = Vec::new();
    
    for (mod_id, transaction) in &mut hot_reload_manager.active_transactions {
        match transaction.status {
            HotReloadStatus::Validating => {
                // Start shadow world validation
                if let Err(e) = hot_reload_manager.start_shadow_world(mod_id, KpiSnapshot::default()) {
                    eprintln!("Failed to start shadow world for {}: {}", mod_id, e);
                    transaction.status = HotReloadStatus::Failed;
                }
            }
            HotReloadStatus::Applied | HotReloadStatus::Failed | HotReloadStatus::Reverted => {
                completed_transactions.push(mod_id.clone());
            }
            _ => {}
        }
    }

    // Remove completed transactions
    for mod_id in completed_transactions {
        hot_reload_manager.active_transactions.remove(&mod_id);
    }
}

/// System to update shadow world simulation
pub fn update_shadow_world_system(
    mut hot_reload_manager: ResMut<HotReloadManager>,
    // TODO: Add resources needed for KPI collection
) {
    if let Some(shadow_state) = &hot_reload_manager.shadow_world_state {
        // Collect current KPIs
        let current_kpis = KpiSnapshot::default(); // TODO: Collect actual KPIs
        
        if let Ok(Some(result)) = hot_reload_manager.update_shadow_world(current_kpis) {
            // Shadow world simulation complete
            if let Some(transaction) = hot_reload_manager.active_transactions.get_mut(&shadow_state.mod_id) {
                transaction.shadow_world_result = Some(result.clone());
                transaction.status = if result.success {
                    HotReloadStatus::Ready
                } else {
                    HotReloadStatus::Failed
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_manager_creation() {
        let manager = HotReloadManager::new();
        assert!(manager.active_transactions.is_empty());
        assert!(manager.shadow_world_state.is_none());
        assert_eq!(manager.dry_run_ticks, 120);
    }

    #[test]
    fn test_validation_thresholds_default() {
        let thresholds = ValidationThresholds::default();
        assert_eq!(thresholds.max_deadline_hit_rate_change, 3.0);
        assert_eq!(thresholds.max_power_draw_change, 10.0);
        assert_eq!(thresholds.max_bandwidth_util_change, 5.0);
        assert_eq!(thresholds.max_corruption_field_change, 0.05);
        assert_eq!(thresholds.max_heat_level_change, 5.0);
    }

    #[test]
    fn test_kpi_snapshot_default() {
        let snapshot = KpiSnapshot::default();
        assert_eq!(snapshot.deadline_hit_rate, 0.0);
        assert_eq!(snapshot.power_draw_kw, 0.0);
        assert_eq!(snapshot.bandwidth_util, 0.0);
        assert_eq!(snapshot.corruption_field, 0.0);
        assert!(snapshot.heat_levels.is_empty());
        assert_eq!(snapshot.gpu_util, 0.0);
        assert_eq!(snapshot.vram_used_mb, 0.0);
    }

    #[test]
    fn test_shadow_world_state() {
        let baseline = KpiSnapshot {
            deadline_hit_rate: 99.0,
            power_draw_kw: 800.0,
            bandwidth_util: 0.6,
            corruption_field: 0.1,
            heat_levels: vec![60.0, 65.0],
            gpu_util: 0.7,
            vram_used_mb: 1000.0,
        };

        let shadow_state = ShadowWorldState {
            mod_id: "test.mod".to_string(),
            baseline_kpis: baseline.clone(),
            current_kpis: baseline,
            ticks_simulated: 0,
            max_ticks: 120,
        };

        assert_eq!(shadow_state.mod_id, "test.mod");
        assert_eq!(shadow_state.ticks_simulated, 0);
        assert_eq!(shadow_state.max_ticks, 120);
    }

    #[test]
    fn test_hot_reload_manager_operations() {
        let mut manager = HotReloadManager::new();
        
        // Test shadow world state
        assert!(!manager.is_shadow_world_running());
        assert!(manager.get_shadow_world_progress().is_none());
        
        // Test active transactions
        assert!(manager.get_active_transactions().is_empty());
        
        // Test configuration
        manager.set_dry_run_ticks(200);
        assert_eq!(manager.dry_run_ticks, 200);
    }

    #[test]
    fn test_kpi_delta_calculation() {
        let manager = HotReloadManager::new();
        
        let baseline = KpiSnapshot {
            deadline_hit_rate: 99.0,
            power_draw_kw: 800.0,
            bandwidth_util: 0.6,
            corruption_field: 0.1,
            heat_levels: vec![60.0, 65.0],
            gpu_util: 0.7,
            vram_used_mb: 1000.0,
        };

        let current = KpiSnapshot {
            deadline_hit_rate: 98.5,
            power_draw_kw: 850.0,
            bandwidth_util: 0.65,
            corruption_field: 0.12,
            heat_levels: vec![62.0, 67.0],
            gpu_util: 0.75,
            vram_used_mb: 1100.0,
        };

        let deltas = manager.calculate_kpi_deltas(&baseline, &current);
        
        assert_eq!(deltas.deadline_hit_rate_change, -0.5);
        assert_eq!(deltas.power_draw_change, 6.25); // (850-800)/800 * 100
        assert_eq!(deltas.bandwidth_util_change, 0.05);
        assert_eq!(deltas.corruption_field_change, 0.02);
        assert_eq!(deltas.heat_levels_change, vec![2.0, 2.0]);
    }
}
