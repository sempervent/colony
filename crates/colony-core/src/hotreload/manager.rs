use bevy::prelude::*;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Resource)]
pub struct HotReloadManager {
    pub watchers: HashMap<String, notify::RecommendedWatcher>,
    pub pending_reloads: Vec<String>,
    pub reload_cooldown: std::time::Duration,
    pub last_reload: std::time::Instant,
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotReloadManager {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
            pending_reloads: Vec::new(),
            reload_cooldown: std::time::Duration::from_millis(500),
            last_reload: std::time::Instant::now(),
        }
    }

    pub fn watch_mod(&mut self, mod_id: &str, mod_path: PathBuf) -> Result<()> {
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // File was modified, trigger reload
                        println!("Mod file modified: {:?}", event.paths);
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        })?;
        
        watcher.watch(&mod_path, RecursiveMode::Recursive)?;
        self.watchers.insert(mod_id.to_string(), watcher);
        Ok(())
    }

    pub fn unwatch_mod(&mut self, mod_id: &str) {
        self.watchers.remove(mod_id);
    }

    pub fn queue_reload(&mut self, mod_id: &str) {
        if !self.pending_reloads.contains(&mod_id.to_string()) {
            self.pending_reloads.push(mod_id.to_string());
        }
    }

    pub fn can_reload(&self) -> bool {
        self.last_reload.elapsed() >= self.reload_cooldown
    }

    pub fn mark_reloaded(&mut self) {
        self.last_reload = std::time::Instant::now();
    }
}

pub fn process_hot_reload_system(
    mut hot_reload_manager: ResMut<HotReloadManager>,
    mut mod_loader: ResMut<crate::mod_loader::ModLoader>,
    time: Res<Time>,
) {
    if !hot_reload_manager.can_reload() {
        return;
    }

    for mod_id in hot_reload_manager.pending_reloads.drain(..) {
        if let Err(e) = mod_loader.trigger_hot_reload(&mod_id) {
            println!("Failed to hot reload mod {}: {}", mod_id, e);
        }
    }
    
    hot_reload_manager.mark_reloaded();
}

pub fn update_shadow_world_system(
    time: Res<Time>,
) {
    // Update shadow world for hot reload
    // This would maintain a copy of the world state for safe reloading
}
