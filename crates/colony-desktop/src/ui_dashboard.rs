use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use colony_core::{Colony, Workyard, YardWorkload, thermal_throttle, enqueue_maintenance, JobQueue, IoRuntime, IoRolling, CorruptionField, FaultKpi, ActiveScheduler, SchedPolicy, Worker, WorkerState, GpuFarm, GpuBatchQueues, BlackSwanIndex, Debts, ResearchState, TechTree, WinLossState, SlaTracker, SessionCtl, ReplayLog, ReplayMode};

#[derive(Resource, Default)]
pub struct DashboardState {
    pub show_victory_modal: bool,
    pub show_loss_modal: bool,
    pub show_save_modal: bool,
    pub show_load_modal: bool,
    pub selected_save_slot: String,
    pub metrics_history: Vec<MetricsSnapshot>,
    pub max_history: usize,
}

#[derive(Clone, Debug)]
pub struct MetricsSnapshot {
    pub tick: u64,
    pub bandwidth_util: f32,
    pub corruption_field: f32,
    pub power_draw: f32,
    pub heat_levels: Vec<f32>,
    pub deadline_hit_rate: f32,
    pub gpu_util: f32,
    pub black_swan_events: Vec<String>,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            show_victory_modal: false,
            show_loss_modal: false,
            show_save_modal: false,
            show_load_modal: false,
            selected_save_slot: String::new(),
            metrics_history: Vec::new(),
            max_history: 1000,
        }
    }
}

pub fn dashboard_panel(
    mut contexts: EguiContexts,
    mut dashboard_state: ResMut<DashboardState>,
    colony: Res<Colony>,
    yards: Query<(&Workyard, &YardWorkload)>,
    workers: Query<&Worker>,
    fault_kpis: Res<FaultKpi>,
    black_swan_index: Res<BlackSwanIndex>,
    debts: Res<Debts>,
    research_state: Res<ResearchState>,
    win_loss_state: Res<WinLossState>,
    sla_tracker: Res<SlaTracker>,
    session_ctl: Res<SessionCtl>,
    replay_log: Res<ReplayLog>,
    clock: Res<colony_core::SimClock>,
    mut commands: Commands,
) {
    let ctx = contexts.ctx_mut();
    let current_tick = clock.now.timestamp_millis() as u64 / 16;

    // Update metrics history
    let mut heat_levels = Vec::new();
    for (yard, _) in &yards {
        heat_levels.push(yard.heat);
    }

    let snapshot = MetricsSnapshot {
        tick: current_tick,
        bandwidth_util: colony.meters.bandwidth_util,
        corruption_field: colony.corruption_field,
        power_draw: colony.meters.power_draw_kw,
        heat_levels,
        deadline_hit_rate: sla_tracker.get_recent_hit_rate(),
        gpu_util: 0.0, // TODO: Get from GPU meters
        black_swan_events: black_swan_index.meters.active.clone(),
    };

    dashboard_state.metrics_history.push(snapshot);
    if dashboard_state.metrics_history.len() > dashboard_state.max_history {
        dashboard_state.metrics_history.remove(0);
    }

    // Check for victory/loss
    if win_loss_state.victory && !dashboard_state.show_victory_modal {
        dashboard_state.show_victory_modal = true;
    }
    if win_loss_state.doom && !dashboard_state.show_loss_modal {
        dashboard_state.show_loss_modal = true;
    }

    // Main dashboard layout
    egui::TopBottomPanel::top("dashboard_header").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // SLA and day counter
            ui.label(format!("SLA: {:.1}%", sla_tracker.get_recent_hit_rate()));
            ui.label(format!("Days: {}", win_loss_state.achieved_days));
            ui.label(format!("Score: {}", win_loss_state.score));
            
            ui.separator();
            
            // Session controls
            if ui.button(if session_ctl.running { "Pause" } else { "Resume" }).clicked() {
                if session_ctl.running {
                    // TODO: Pause session
                } else {
                    // TODO: Resume session
                }
            }
            
            if ui.button(if session_ctl.fast_forward { "Normal Speed" } else { "Fast Forward" }).clicked() {
                // TODO: Toggle fast forward
            }
            
            ui.separator();
            
            // Save/Load buttons
            if ui.button("Save").clicked() {
                dashboard_state.show_save_modal = true;
            }
            
            if ui.button("Load").clicked() {
                dashboard_state.show_load_modal = true;
            }
            
            ui.separator();
            
            // Replay controls
            if ui.button(if replay_log.is_recording() { "Stop Recording" } else { "Start Recording" }).clicked() {
                // TODO: Toggle recording
            }
            
            if ui.button("Start Replay").clicked() {
                // TODO: Start replay
            }
        });
    });

    // Metrics graphs
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("System Metrics");
        
        // Bandwidth utilization
        ui.group(|ui| {
            ui.label("Bandwidth Utilization");
            let bandwidth_data: Vec<f32> = dashboard_state.metrics_history
                .iter()
                .map(|s| s.bandwidth_util)
                .collect();
            if !bandwidth_data.is_empty() {
                ui.add(egui::plot::Plot::new("bandwidth")
                    .view_aspect(2.0)
                    .height(100.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from_ys_f32(&bandwidth_data)));
                    }));
            }
        });
        
        ui.add_space(10.0);
        
        // Power draw vs cap
        ui.group(|ui| {
            ui.label("Power Draw vs Cap");
            let power_data: Vec<f32> = dashboard_state.metrics_history
                .iter()
                .map(|s| s.power_draw)
                .collect();
            if !power_data.is_empty() {
                ui.add(egui::plot::Plot::new("power")
                    .view_aspect(2.0)
                    .height(100.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from_ys_f32(&power_data)));
                        // Add power cap line
                        let cap_line = vec![colony.power_cap_kw; power_data.len()];
                        plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from_ys_f32(&cap_line)));
                    }));
            }
        });
        
        ui.add_space(10.0);
        
        // Heat levels
        ui.group(|ui| {
            ui.label("Heat Levels");
            for (i, (yard, _)) in yards.iter().enumerate() {
                let throttle = thermal_throttle(yard.heat, yard.heat_cap, colony.tunables.thermal_throttle_knee, colony.tunables.thermal_min_throttle);
                ui.label(format!("Yard {}: {:.1}°C / {:.1}°C (throttle: {:.2})", i, yard.heat, yard.heat_cap, throttle));
            }
        });
        
        ui.add_space(10.0);
        
        // Corruption and Black Swan events
        ui.group(|ui| {
            ui.label("Corruption & Events");
            ui.label(format!("Corruption Field: {:.3}", colony.corruption_field));
            ui.label(format!("Active Black Swans: {}", black_swan_index.meters.active.len()));
            for event_id in &black_swan_index.meters.active {
                ui.label(format!("• {}", event_id));
            }
        });
    });

    // Victory modal
    if dashboard_state.show_victory_modal {
        victory_modal(ctx, &mut dashboard_state, &win_loss_state);
    }

    // Loss modal
    if dashboard_state.show_loss_modal {
        loss_modal(ctx, &mut dashboard_state, &win_loss_state);
    }

    // Save modal
    if dashboard_state.show_save_modal {
        save_modal(ctx, &mut dashboard_state);
    }

    // Load modal
    if dashboard_state.show_load_modal {
        load_modal(ctx, &mut dashboard_state);
    }
}

fn victory_modal(ctx: &egui::Context, dashboard_state: &mut DashboardState, win_loss_state: &WinLossState) {
    egui::Window::new("Victory!")
        .open(&mut dashboard_state.show_victory_modal)
        .show(ctx, |ui| {
            ui.heading("🎉 Victory Achieved! 🎉");
            ui.label(format!("Score: {}", win_loss_state.score));
            ui.label(format!("Days Achieved: {}", win_loss_state.achieved_days));
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Replay from Start").clicked() {
                    // TODO: Start replay from beginning
                    dashboard_state.show_victory_modal = false;
                }
                
                if ui.button("Export Replay").clicked() {
                    // TODO: Export replay
                    dashboard_state.show_victory_modal = false;
                }
                
                if ui.button("Return to Menu").clicked() {
                    // TODO: Return to main menu
                    dashboard_state.show_victory_modal = false;
                }
            });
        });
}

fn loss_modal(ctx: &egui::Context, dashboard_state: &mut DashboardState, win_loss_state: &WinLossState) {
    egui::Window::new("Defeat")
        .open(&mut dashboard_state.show_loss_modal)
        .show(ctx, |ui| {
            ui.heading("💀 Defeat 💀");
            ui.label(format!("Reason: {:?}", win_loss_state.doom_reason));
            ui.label(format!("Days Survived: {}", win_loss_state.achieved_days));
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Replay from Start").clicked() {
                    // TODO: Start replay from beginning
                    dashboard_state.show_loss_modal = false;
                }
                
                if ui.button("Try Again").clicked() {
                    // TODO: Restart with same scenario
                    dashboard_state.show_loss_modal = false;
                }
                
                if ui.button("Return to Menu").clicked() {
                    // TODO: Return to main menu
                    dashboard_state.show_loss_modal = false;
                }
            });
        });
}

fn save_modal(ctx: &egui::Context, dashboard_state: &mut DashboardState) {
    egui::Window::new("Save Game")
        .open(&mut dashboard_state.show_save_modal)
        .show(ctx, |ui| {
            ui.label("Save Slot Name:");
            ui.text_edit_singleline(&mut dashboard_state.selected_save_slot);
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    // TODO: Save to slot
                    dashboard_state.show_save_modal = false;
                }
                
                if ui.button("Cancel").clicked() {
                    dashboard_state.show_save_modal = false;
                }
            });
        });
}

fn load_modal(ctx: &egui::Context, dashboard_state: &mut DashboardState) {
    egui::Window::new("Load Game")
        .open(&mut dashboard_state.show_load_modal)
        .show(ctx, |ui| {
            ui.label("Available Save Slots:");
            
            // TODO: Load actual save slots
            let mock_slots = vec![
                "autosave_1",
                "manual_save_1",
                "test_save",
            ];
            
            for slot in mock_slots {
                if ui.button(slot).clicked() {
                    // TODO: Load from slot
                    dashboard_state.show_load_modal = false;
                }
            }
            
            ui.add_space(10.0);
            
            if ui.button("Cancel").clicked() {
                dashboard_state.show_load_modal = false;
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_state_creation() {
        let state = DashboardState::default();
        assert!(!state.show_victory_modal);
        assert!(!state.show_loss_modal);
        assert_eq!(state.max_history, 1000);
    }

    #[test]
    fn test_metrics_snapshot() {
        let snapshot = MetricsSnapshot {
            tick: 1000,
            bandwidth_util: 0.5,
            corruption_field: 0.1,
            power_draw: 800.0,
            heat_levels: vec![50.0, 60.0],
            deadline_hit_rate: 99.0,
            gpu_util: 0.8,
            black_swan_events: vec!["test_event".to_string()],
        };
        
        assert_eq!(snapshot.tick, 1000);
        assert_eq!(snapshot.bandwidth_util, 0.5);
        assert_eq!(snapshot.heat_levels.len(), 2);
    }
}
