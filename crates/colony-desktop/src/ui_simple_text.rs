use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

#[derive(Component)]
pub struct UiText;

pub struct SimpleTextUiPlugin;

impl Plugin for SimpleTextUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, (
               update_ui_text,
               handle_keyboard_input,
           ));
    }
}

fn setup_ui(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d::default());
    
    // Spawn UI text
    commands.spawn((
        Text2d::new("Compute Colony - Press SPACE to start, S to stop simulators, M for maintenance"),
        UiText,
    ));
}

fn update_ui_text(
    mut text_query: Query<&mut Text2d, With<UiText>>,
    app_state: Res<State<AppState>>,
    colony: Res<colony_core::Colony>,
    clock: Res<colony_core::SimClock>,
) {
    for mut text in text_query.iter_mut() {
        match app_state.get() {
            AppState::MainMenu => {
                text.0 = format!(
                    "Compute Colony - Main Menu\n\nPress SPACE to start game\n\nColony Status:\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\nTime: {}\n\nControls:\nSPACE - Start Game\nS - Stop Simulators\nM - Maintenance\n1 - FCFS Scheduler\n2 - SJF Scheduler\n3 - EDF Scheduler",
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    clock.now
                );
            }
            AppState::InGame => {
                text.0 = format!(
                    "Compute Colony - Game Running\n\nColony Status:\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\nTime: {}\n\nControls:\nP - Pause Game\nS - Stop Simulators\nM - Maintenance\n1 - FCFS Scheduler\n2 - SJF Scheduler\n3 - EDF Scheduler\n\nSimulator Controls:\nU - Start/Stop UDP\nH - Start/Stop HTTP",
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    clock.now
                );
            }
            AppState::Paused => {
                text.0 = format!(
                    "Compute Colony - Game Paused\n\nColony Status:\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\nTime: {}\n\nControls:\nR - Resume Game\nS - Stop Simulators\nM - Maintenance\n1 - FCFS Scheduler\n2 - SJF Scheduler\n3 - EDF Scheduler",
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    clock.now
                );
            }
        }
    }
}

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    app_state: Res<State<AppState>>,
) {
    match app_state.get() {
        AppState::MainMenu => {
            if keyboard.just_pressed(KeyCode::Space) {
                println!("Starting game...");
                next_state.set(AppState::InGame);
            }
        }
        AppState::InGame => {
            if keyboard.just_pressed(KeyCode::KeyP) {
                println!("Pausing game...");
                next_state.set(AppState::Paused);
            } else if keyboard.just_pressed(KeyCode::KeyS) {
                println!("Stopping simulators...");
                // TODO: Send stop simulator events
            } else if keyboard.just_pressed(KeyCode::KeyM) {
                println!("Running maintenance...");
                // TODO: Send maintenance event
            } else if keyboard.just_pressed(KeyCode::Digit1) {
                println!("Switching to FCFS scheduler...");
                // TODO: Send scheduler switch event
            } else if keyboard.just_pressed(KeyCode::Digit2) {
                println!("Switching to SJF scheduler...");
                // TODO: Send scheduler switch event
            } else if keyboard.just_pressed(KeyCode::Digit3) {
                println!("Switching to EDF scheduler...");
                // TODO: Send scheduler switch event
            } else if keyboard.just_pressed(KeyCode::KeyU) {
                println!("Toggling UDP simulator...");
                // TODO: Send UDP simulator toggle event
            } else if keyboard.just_pressed(KeyCode::KeyH) {
                println!("Toggling HTTP simulator...");
                // TODO: Send HTTP simulator toggle event
            }
        }
        AppState::Paused => {
            if keyboard.just_pressed(KeyCode::KeyR) {
                println!("Resuming game...");
                next_state.set(AppState::InGame);
            } else if keyboard.just_pressed(KeyCode::KeyS) {
                println!("Stopping simulators...");
                // TODO: Send stop simulator events
            } else if keyboard.just_pressed(KeyCode::KeyM) {
                println!("Running maintenance...");
                // TODO: Send maintenance event
            } else if keyboard.just_pressed(KeyCode::Digit1) {
                println!("Switching to FCFS scheduler...");
                // TODO: Send scheduler switch event
            } else if keyboard.just_pressed(KeyCode::Digit2) {
                println!("Switching to SJF scheduler...");
                // TODO: Send scheduler switch event
            } else if keyboard.just_pressed(KeyCode::Digit3) {
                println!("Switching to EDF scheduler...");
                // TODO: Send scheduler switch event
            }
        }
    }
}
