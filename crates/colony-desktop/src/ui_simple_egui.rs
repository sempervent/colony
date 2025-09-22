use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

pub struct SimpleEguiPlugin;

impl Plugin for SimpleEguiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, ui_system);
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn ui_system(
    mut egui_ctx: EguiContexts,
    app_state: Res<State<AppState>>,
    colony: Res<colony_core::Colony>,
    clock: Res<colony_core::SimClock>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };

    match app_state.get() {
        AppState::MainMenu => {
            egui::Window::new("Compute Colony - Setup Wizard")
                .default_size([500.0, 400.0])
                .show(ctx, |ui| {
                    ui.heading("Welcome to Compute Colony!");
                    ui.add_space(10.0);
                    ui.label("This is an interactive colony management interface.");
                    ui.add_space(10.0);
                    
                    ui.separator();
                    ui.add_space(10.0);
                    
                    ui.label("Colony Status:");
                    ui.label(format!("  Power: {:.0}/{:.0} kW", colony.meters.power_draw_kw, colony.power_cap_kw));
                    ui.label(format!("  Bandwidth: {:.1}%", colony.meters.bandwidth_util * 100.0));
                    ui.label(format!("  Corruption: {:.1}%", colony.corruption_field * 100.0));
                    ui.label(format!("  Time: {}", clock.now));
                    
                    ui.add_space(20.0);
                    if ui.button("Start Game").clicked() {
                        next_state.set(AppState::InGame);
                    }
                });
        }
        AppState::InGame | AppState::Paused => {
            // Simulator Controls
            egui::Window::new("Simulator Controls")
                .default_size([300.0, 300.0])
                .show(ctx, |ui| {
                    ui.heading("Simulator Controls");
                    ui.add_space(10.0);
                    
                    // UDP Simulator
                    ui.group(|ui| {
                        ui.label("UDP Simulator");
                        ui.horizontal(|ui| {
                            if ui.button("Start UDP").clicked() {
                                println!("Starting UDP simulator...");
                                // TODO: Send StartUdpSim event
                            }
                            if ui.button("Stop UDP").clicked() {
                                println!("Stopping UDP simulator...");
                                // TODO: Send StopUdpSim event
                            }
                        });
                        ui.label("Status: Not implemented yet");
                    });
                    
                    ui.add_space(10.0);
                    
                    // HTTP Simulator
                    ui.group(|ui| {
                        ui.label("HTTP Simulator");
                        ui.horizontal(|ui| {
                            if ui.button("Start HTTP").clicked() {
                                println!("Starting HTTP simulator...");
                                // TODO: Send StartHttpSim event
                            }
                            if ui.button("Stop HTTP").clicked() {
                                println!("Stopping HTTP simulator...");
                                // TODO: Send StopHttpSim event
                            }
                        });
                        ui.label("Status: Not implemented yet");
                    });
                    
                    ui.add_space(10.0);
                    
                    // Scheduler Controls
                    ui.group(|ui| {
                        ui.label("Scheduler");
                        ui.horizontal(|ui| {
                            if ui.button("FCFS").clicked() {
                                println!("Switching to FCFS scheduler...");
                                // TODO: Send SwitchScheduler event
                            }
                            if ui.button("SJF").clicked() {
                                println!("Switching to SJF scheduler...");
                                // TODO: Send SwitchScheduler event
                            }
                            if ui.button("EDF").clicked() {
                                println!("Switching to EDF scheduler...");
                                // TODO: Send SwitchScheduler event
                            }
                        });
                        ui.label("Current: Not implemented yet");
                    });
                    
                    ui.add_space(10.0);
                    
                    // Maintenance
                    ui.group(|ui| {
                        ui.label("Maintenance");
                        if ui.button("Run Maintenance").clicked() {
                            println!("Running maintenance...");
                            // TODO: Send Maintenance event
                        }
                        ui.label("Status: Not implemented yet");
                    });
                });

            // Colony Status
            egui::Window::new("Colony Status")
                .default_size([300.0, 300.0])
                .show(ctx, |ui| {
                    ui.heading("Colony Status");
                    ui.add_space(10.0);
                    
                    // Power usage
                    ui.group(|ui| {
                        ui.label("Power Usage");
                        ui.add(egui::ProgressBar::new(colony.meters.power_draw_kw / colony.power_cap_kw)
                            .text(format!("{:.0}/{:.0} kW", colony.meters.power_draw_kw, colony.power_cap_kw)));
                    });
                    
                    ui.add_space(10.0);
                    
                    // Bandwidth
                    ui.group(|ui| {
                        ui.label("Bandwidth");
                        ui.add(egui::ProgressBar::new(colony.meters.bandwidth_util)
                            .text(format!("{:.1}%", colony.meters.bandwidth_util * 100.0)));
                    });
                    
                    ui.add_space(10.0);
                    
                    // Corruption
                    ui.group(|ui| {
                        ui.label("Corruption");
                        ui.add(egui::ProgressBar::new(colony.corruption_field)
                            .text(format!("{:.1}%", colony.corruption_field * 100.0)));
                    });
                    
                    ui.add_space(10.0);
                    
                    // System info
                    ui.group(|ui| {
                        ui.label("System Overview");
                        ui.label(format!("Time: {}", clock.now));
                        ui.label(format!("Power Capacity: {:.0} kW", colony.power_cap_kw));
                        ui.label(format!("Bandwidth: {:.1} Gbps", colony.tunables.bandwidth_total_gbps));
                        ui.label(format!("Target Uptime: {} days", colony.target_uptime_days));
                        ui.label(format!("Seed: {}", colony.seed));
                    });
                });

            // Control buttons
            egui::Window::new("Controls")
                .default_size([200.0, 100.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Main Menu").clicked() {
                            next_state.set(AppState::MainMenu);
                        }
                        if ui.button("Pause").clicked() {
                            next_state.set(AppState::Paused);
                        }
                        if ui.button("Resume").clicked() {
                            next_state.set(AppState::InGame);
                        }
                    });
                });
        }
    }
}
