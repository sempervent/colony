use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

pub struct MinimalUiPlugin;

impl Plugin for MinimalUiPlugin {
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
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };

    match app_state.get() {
        AppState::MainMenu => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Compute Colony - Setup Wizard");
                ui.add_space(20.0);
                ui.label("Welcome to Compute Colony!");
                ui.add_space(10.0);
                ui.label("This is a minimal UI test.");
            });
        }
        AppState::InGame | AppState::Paused => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Compute Colony - Running");
                ui.add_space(20.0);
                ui.label("Simulation is running!");
                ui.add_space(10.0);
                ui.label("This is the main game UI.");
            });
        }
    }
}
