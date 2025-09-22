use bevy::prelude::*;
use bevy::ui::{Interaction, Val, UiRect, JustifyContent, AlignItems, FlexDirection, NodeBundle, ButtonBundle, TextBundle, TextStyle, Style};

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

#[derive(Component)]
pub struct StartGameButton;

#[derive(Component)]
pub struct StartUdpButton;

#[derive(Component)]
pub struct StopUdpButton;

#[derive(Component)]
pub struct StartHttpButton;

#[derive(Component)]
pub struct StopHttpButton;

#[derive(Component)]
pub struct FCFSButton;

#[derive(Component)]
pub struct SJFButton;

#[derive(Component)]
pub struct EDFButton;

#[derive(Component)]
pub struct MaintenanceButton;

#[derive(Component)]
pub struct PauseButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct MainMenuButton;

#[derive(Component)]
pub struct StatusText;

pub struct BevyNativeUiPlugin;

impl Plugin for BevyNativeUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, (
               handle_button_clicks,
               update_status_text,
           ));
    }
}

fn setup_ui(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Main menu UI
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::srgb(0.1, 0.1, 0.1).into(),
        ..default()
    }).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "Compute Colony",
            TextStyle {
                font_size: 48.0,
                color: Color::WHITE,
                ..default()
            },
        ));

        // Start Game Button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.2, 0.6, 0.2).into(),
                ..default()
            },
            StartGameButton,
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Start Game",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });

        // Status text
        parent.spawn((
            TextBundle::from_section(
                "Colony Status: Loading...",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            StatusText,
        ));
    });
}

fn handle_button_clicks(
    interaction_query: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<Button>)
    >,
    start_game_query: Query<&StartGameButton>,
    start_udp_query: Query<&StartUdpButton>,
    stop_udp_query: Query<&StopUdpButton>,
    start_http_query: Query<&StartHttpButton>,
    stop_http_query: Query<&StopHttpButton>,
    fcfs_query: Query<&FCFSButton>,
    sjf_query: Query<&SJFButton>,
    edf_query: Query<&EDFButton>,
    maintenance_query: Query<&MaintenanceButton>,
    pause_query: Query<&PauseButton>,
    resume_query: Query<&ResumeButton>,
    main_menu_query: Query<&MainMenuButton>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, entity) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Check which button was pressed
            if start_game_query.get(entity).is_ok() {
                println!("Start Game clicked!");
                next_state.set(AppState::InGame);
            } else if start_udp_query.get(entity).is_ok() {
                println!("Start UDP Simulator clicked!");
                // TODO: Send StartUdpSim event
            } else if stop_udp_query.get(entity).is_ok() {
                println!("Stop UDP Simulator clicked!");
                // TODO: Send StopUdpSim event
            } else if start_http_query.get(entity).is_ok() {
                println!("Start HTTP Simulator clicked!");
                // TODO: Send StartHttpSim event
            } else if stop_http_query.get(entity).is_ok() {
                println!("Stop HTTP Simulator clicked!");
                // TODO: Send StopHttpSim event
            } else if fcfs_query.get(entity).is_ok() {
                println!("Switch to FCFS Scheduler clicked!");
                // TODO: Send SwitchScheduler event
            } else if sjf_query.get(entity).is_ok() {
                println!("Switch to SJF Scheduler clicked!");
                // TODO: Send SwitchScheduler event
            } else if edf_query.get(entity).is_ok() {
                println!("Switch to EDF Scheduler clicked!");
                // TODO: Send SwitchScheduler event
            } else if maintenance_query.get(entity).is_ok() {
                println!("Run Maintenance clicked!");
                // TODO: Send Maintenance event
            } else if pause_query.get(entity).is_ok() {
                println!("Pause clicked!");
                next_state.set(AppState::Paused);
            } else if resume_query.get(entity).is_ok() {
                println!("Resume clicked!");
                next_state.set(AppState::InGame);
            } else if main_menu_query.get(entity).is_ok() {
                println!("Main Menu clicked!");
                next_state.set(AppState::MainMenu);
            }
        }
    }
}

fn update_status_text(
    mut text_query: Query<&mut Text, With<StatusText>>,
    app_state: Res<State<AppState>>,
    colony: Res<colony_core::Colony>,
    clock: Res<colony_core::SimClock>,
) {
    for mut text in text_query.iter_mut() {
        match app_state.get() {
            AppState::MainMenu => {
                text.0 = format!(
                    "Colony Status:\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\nTime: {}",
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    clock.now
                );
            }
            AppState::InGame => {
                text.0 = format!(
                    "Game Running:\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\nTime: {}",
                    colony.meters.power_draw_kw,
                    colony.power_cap_kw,
                    colony.meters.bandwidth_util * 100.0,
                    colony.corruption_field * 100.0,
                    clock.now
                );
            }
            AppState::Paused => {
                text.0 = format!(
                    "Game Paused:\nPower: {:.0}/{:.0} kW\nBandwidth: {:.1}%\nCorruption: {:.1}%\nTime: {}",
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
