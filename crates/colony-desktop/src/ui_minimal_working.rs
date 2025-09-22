use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

pub struct MinimalWorkingUiPlugin;

impl Plugin for MinimalWorkingUiPlugin {
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

    // Just show a simple label without any complex widgets
    match app_state.get() {
        AppState::MainMenu => {
            egui::SidePanel::left("main_menu")
                .resizable(false)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Compute Colony");
                    ui.add_space(10.0);
                    ui.label("Welcome to Compute Colony!");
                    ui.add_space(10.0);
                    ui.label("This is a minimal working UI.");
                    ui.add_space(10.0);
                    if ui.button("Start Game").clicked() {
                        println!("Start Game clicked!");
                    }
                });
        }
        AppState::InGame | AppState::Paused => {
            egui::SidePanel::left("colony_status")
                .resizable(false)
                .default_width(300.0)
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
