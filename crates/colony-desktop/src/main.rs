use bevy::prelude::*;
use colony_core::{ColonyPlugin, SimClock, TickScale, enqueue_maintenance, JobQueue};
use ron::ser::to_string_pretty;
use std::fs;

mod ui_simple_text;

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
        .add_plugins(ui_simple_text::SimpleTextUiPlugin)
        .run();
}

// Legacy keyboard input handlers (now handled by UI)
fn handle_legacy_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scheduler: ResMut<colony_core::ActiveScheduler>,
    yards: Query<&colony_core::Workyard>,
    _jobq: ResMut<JobQueue>,
    clock: Res<SimClock>,
    colony: Res<colony_core::Colony>,
    workers: Query<&colony_core::Worker>,
) {
    // Legacy hotkeys for quick access
    if keyboard.just_pressed(KeyCode::Digit1) {
        *scheduler = colony_core::ActiveScheduler::new_fcfs();
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        *scheduler = colony_core::ActiveScheduler::new_sjf();
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        *scheduler = colony_core::ActiveScheduler::new_edf();
    }
    
    if keyboard.just_pressed(KeyCode::KeyM) {
        // Schedule maintenance for the first yard
        // Note: This is a simplified version - in practice you'd need the entity
        // For now, we'll just log that maintenance was requested
        println!("Maintenance requested (legacy hotkey)");
    }
    
    if keyboard.just_pressed(KeyCode::KeyS) {
        save_game(&clock, &colony, &workers, &yards);
    }
    
    if keyboard.just_pressed(KeyCode::KeyL) {
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
