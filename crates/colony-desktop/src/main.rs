use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use colony_core::{ColonyPlugin, SimClock, TickScale, enqueue_maintenance, JobQueue};
use ron::ser::to_string_pretty;
use std::fs;

mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Compute Colony".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ColonyPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(DesktopUiPlugin)
        .run();
}

pub struct DesktopUiPlugin;

impl Plugin for DesktopUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_time_display,
                update_worker_display,
                update_scheduler_display,
                update_job_queue_display,
                handle_time_scale_input,
                handle_scheduler_input,
                handle_maintenance_input,
                handle_save_load_input,
            ));
    }
}

#[derive(Component)]
struct TimeDisplay;

#[derive(Component)]
struct WorkerDisplay;

#[derive(Component)]
struct TimeScaleSlider;

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    
    // Main UI container
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::srgb(0.1, 0.1, 0.1).into(),
        ..default()
    })
    .with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "Compute Colony Simulator",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ));

        // Time controls
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Time Scale: ",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(80.0),
                        height: Val::Px(30.0),
                        margin: UiRect::horizontal(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: Color::srgb(0.3, 0.3, 0.3).into(),
                    ..default()
                },
                TimeScaleSlider,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Real Time",
                    TextStyle {
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        });

        // Time display
        parent.spawn((
            TextBundle::from_section(
                "Time: 2024-01-01 00:00:00 UTC",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TimeDisplay,
        ));

        // Worker status
        parent.spawn((
            TextBundle::from_section(
                "Workers: 0/4 idle",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            WorkerDisplay,
        ));

        // Scheduler status
        parent.spawn((
            TextBundle::from_section(
                "Scheduler: FCFS",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            SchedulerDisplay,
        ));

        // Job queue status
        parent.spawn((
            TextBundle::from_section(
                "Jobs in queue: 0",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            JobQueueDisplay,
        ));
    });
}

fn update_time_display(
    clock: Res<SimClock>,
    mut query: Query<&mut Text, With<TimeDisplay>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Time: {}", clock.now.format("%Y-%m-%d %H:%M:%S UTC"));
    }
}

fn update_worker_display(
    workers: Query<&colony_core::Worker>,
    mut query: Query<&mut Text, With<WorkerDisplay>>,
) {
    let idle_count = workers.iter().filter(|w| w.state == colony_core::WorkerState::Idle).count();
    let total_count = workers.iter().count();
    
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Workers: {}/{} idle", idle_count, total_count);
    }
}

fn update_scheduler_display(
    scheduler: Res<colony_core::ActiveScheduler>,
    mut query: Query<&mut Text, With<SchedulerDisplay>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Scheduler: {}", scheduler.get_name());
    }
}

fn update_job_queue_display(
    jobq: Res<colony_core::JobQueue>,
    mut query: Query<&mut Text, With<JobQueueDisplay>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Jobs in queue: {}", jobq.jobs.len());
    }
}

fn handle_time_scale_input(
    mut clock: ResMut<SimClock>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<TimeScaleButton>)>,
) {
    for (interaction, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = Color::srgb(0.5, 0.5, 0.5).into();
                
                // Cycle through time scales
                clock.tick_scale = match clock.tick_scale {
                    TickScale::RealTime => TickScale::Seconds(1),
                    TickScale::Seconds(1) => TickScale::Seconds(10),
                    TickScale::Seconds(10) => TickScale::Days(1),
                    TickScale::Days(1) => TickScale::Days(7),
                    TickScale::Days(7) => TickScale::Years(1),
                    TickScale::Years(1) => TickScale::RealTime,
                    _ => TickScale::RealTime,
                };
            }
            Interaction::Hovered => {
                *background_color = Color::srgb(0.4, 0.4, 0.4).into();
            }
            Interaction::None => {
                *background_color = Color::srgb(0.3, 0.3, 0.3).into();
            }
        }
    }
}

#[derive(Component)]
struct TimeScaleButton;

#[derive(Component)]
struct SaveButton;

#[derive(Component)]
struct LoadButton;

#[derive(Component)]
struct SchedulerDisplay;

#[derive(Component)]
struct JobQueueDisplay;

fn handle_scheduler_input(
    keyboard: Res<Input<KeyCode>>,
    mut scheduler: ResMut<colony_core::ActiveScheduler>,
) {
    if keyboard.just_pressed(KeyCode::Key1) {
        *scheduler = colony_core::ActiveScheduler::new_fcfs();
    } else if keyboard.just_pressed(KeyCode::Key2) {
        *scheduler = colony_core::ActiveScheduler::new_sjf();
    } else if keyboard.just_pressed(KeyCode::Key3) {
        *scheduler = colony_core::ActiveScheduler::new_edf();
    } else if keyboard.just_pressed(KeyCode::Key4) {
        *scheduler = colony_core::ActiveScheduler::new_hetero_aware();
    }
}

fn handle_maintenance_input(
    keyboard: Res<Input<KeyCode>>,
    yards: Query<Entity, With<colony_core::Workyard>>,
    mut jobq: ResMut<JobQueue>,
) {
    if keyboard.just_pressed(KeyCode::M) {
        // Schedule maintenance for the first yard
        if let Some(yard_entity) = yards.iter().next() {
            enqueue_maintenance(yard_entity, &mut jobq);
        }
    }
}

fn handle_save_load_input(
    keyboard: Res<Input<KeyCode>>,
    clock: Res<SimClock>,
    colony: Res<colony_core::Colony>,
    workers: Query<&colony_core::Worker>,
    yards: Query<&colony_core::Workyard>,
) {
    if keyboard.just_pressed(KeyCode::S) {
        save_game(&clock, &colony, &workers, &yards);
    }
    
    if keyboard.just_pressed(KeyCode::L) {
        load_game();
    }
}

fn save_game(
    clock: &SimClock,
    colony: &colony_core::Colony,
    workers: &Query<&colony_core::Worker>,
    yards: &Query<&colony_core::Workyard>,
) {
    let save_data = SaveData {
        clock: clock.clone(),
        colony: colony.clone(),
        workers: workers.iter().cloned().collect(),
        yards: yards.iter().cloned().collect(),
    };

    match to_string_pretty(&save_data, ron::ser::PrettyConfig::default()) {
        Ok(serialized) => {
            if let Err(e) = fs::write("save.ron", serialized) {
                eprintln!("Failed to save game: {}", e);
            } else {
                println!("Game saved to save.ron");
            }
        }
        Err(e) => eprintln!("Failed to serialize save data: {}", e),
    }
}

fn load_game() {
    match fs::read_to_string("save.ron") {
        Ok(contents) => {
            match ron::from_str::<SaveData>(&contents) {
                Ok(_save_data) => {
                    println!("Game loaded from save.ron");
                    // In a real implementation, you'd apply the loaded data to the world
                }
                Err(e) => eprintln!("Failed to deserialize save data: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to read save file: {}", e),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveData {
    clock: SimClock,
    colony: colony_core::Colony,
    workers: Vec<colony_core::Worker>,
    yards: Vec<colony_core::Workyard>,
}
