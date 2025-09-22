use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

pub struct BevyTextUiPlugin;

impl Plugin for BevyTextUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, update_ui_text);
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    
    // Spawn a simple text entity for the UI
    commands.spawn(Text2d::new("Compute Colony - Setup Wizard\nWelcome to Compute Colony!\nThis is a working UI test.\n\nColony Status:\n  Power: 500/1000 kW\n  Bandwidth: 0.0%\n  Corruption: 0.0%\n  Time: Loading..."));
}

fn update_ui_text(
    mut text_query: Query<&mut Text2d>,
    app_state: Res<State<AppState>>,
    colony: Res<colony_core::Colony>,
    clock: Res<colony_core::SimClock>,
) {
    for mut text in text_query.iter_mut() {
        match app_state.get() {
            AppState::MainMenu => {
                text.0 = format!(
                    "Compute Colony - Setup Wizard\nWelcome to Compute Colony!\nThis is a working UI test.\n\nColony Status:\n  Power: {:.0}/{:.0} kW\n  Bandwidth: {:.1}%\n  Corruption: {:.1}%\n  Time: {}",
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    clock.now
                );
            }
            AppState::InGame | AppState::Paused => {
                text.0 = format!(
                    "Colony Status\n\nTime: {}\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\n\nSystem Overview:\n• Power Capacity: {:.0} kW\n• Bandwidth: {:.1} Gbps\n• Target Uptime: {} days\n• Seed: {}",
                    clock.now,
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    colony.power_cap_kw,
                    colony.tunables.bandwidth_total_gbps,
                    colony.target_uptime_days,
                    colony.seed
                );
            }
        }
    }
}