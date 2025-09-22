use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

pub struct NoEguiUiPlugin;

impl Plugin for NoEguiUiPlugin {
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
            egui::Window::new("Compute Colony - Setup Wizard")
                .default_size([500.0, 400.0])
                .show(ctx, |ui| {
                    ui.heading("Welcome to Compute Colony!");
                    ui.add_space(10.0);
                    ui.label("This is a working UI test.");
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
                        println!("Start Game clicked!");
                    }
                });
        }
        AppState::InGame | AppState::Paused => {
            egui::Window::new("Colony Status")
                .default_size([600.0, 500.0])
                .show(ctx, |ui| {
                    ui.heading("Colony Status");
                    ui.add_space(10.0);
                    
                    ui.label(format!("Time: {}", clock.now));
                    ui.label(format!("Power: {:.0}/{:.0} kW", 
                        colony.meters.power_draw_kw, colony.power_cap_kw));
                    ui.label(format!("Bandwidth: {:.1}%", 
                        colony.meters.bandwidth_util * 100.0));
                    ui.label(format!("Corruption: {:.1}%", 
                        colony.corruption_field * 100.0));
                    
                    ui.add_space(10.0);
                    ui.label("System Overview:");
                    ui.label(format!("• Power Capacity: {:.0} kW", colony.power_cap_kw));
                    ui.label(format!("• Bandwidth: {:.1} Gbps", colony.tunables.bandwidth_total_gbps));
                    ui.label(format!("• Target Uptime: {} days", colony.target_uptime_days));
                    ui.label(format!("• Seed: {}", colony.seed));
                });
        }
    }
}
