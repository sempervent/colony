use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

pub struct BasicUiPlugin;

impl Plugin for BasicUiPlugin {
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
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };

    match app_state.get() {
        AppState::MainMenu => {
            egui::Window::new("Compute Colony")
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.heading("Setup Wizard");
                    ui.add_space(20.0);
                    ui.label("Welcome to Compute Colony!");
                    ui.add_space(10.0);
                    ui.label("This is a basic UI test.");
                    ui.add_space(10.0);
                    if ui.button("Start Game").clicked() {
                        // TODO: Start game
                    }
                });
        }
        AppState::InGame | AppState::Paused => {
            egui::Window::new("Colony Status")
                .default_size([600.0, 400.0])
                .show(ctx, |ui| {
                    ui.heading("Colony Status");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Power Usage");
                            ui.add(egui::ProgressBar::new(colony.meters.power_draw_kw / colony.power_cap_kw)
                                .text(format!("{:.0}/{:.0} kW", colony.meters.power_draw_kw, colony.power_cap_kw)));
                        });
                        
                        ui.vertical(|ui| {
                            ui.label("Bandwidth");
                            ui.add(egui::ProgressBar::new(colony.meters.bandwidth_util)
                                .text(format!("{:.1}%", colony.meters.bandwidth_util * 100.0)));
                        });
                        
                        ui.vertical(|ui| {
                            ui.label("Corruption");
                            ui.add(egui::ProgressBar::new(colony.corruption_field)
                                .text(format!("{:.1}%", colony.corruption_field * 100.0)));
                        });
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.label("System Overview:");
                    ui.label(format!("• Power Capacity: {:.0} kW", colony.power_cap_kw));
                    ui.label(format!("• Bandwidth: {:.1} Gbps", colony.tunables.bandwidth_total_gbps));
                    ui.label(format!("• Target Uptime: {} days", colony.target_uptime_days));
                    ui.label(format!("• Seed: {}", colony.seed));
                    ui.label(format!("• Time: {}", clock.now));
                });
        }
    }
}
