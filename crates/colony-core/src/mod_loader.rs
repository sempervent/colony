use bevy::prelude::*;
use colony_modsdk::{
    ModManifest, ModRegistryEntry, ModValidationResult, HotReloadTransaction, 
    HotReloadStatus, ShadowWorldResult, KpiDeltas, ContentHashes
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;

/// Mod registry containing all loaded mods
#[derive(Resource, Default)]
pub struct ModRegistry {
    pub mods: HashMap<String, ModRegistryEntry>,
    pub hot_reload_queue: Vec<HotReloadTransaction>,
    pub file_watcher: Option<notify::RecommendedWatcher>,
    pub mod_directory: PathBuf,
    pub replay_mode: bool,
}

/// Mod loader for discovering, validating, and hot reloading mods
pub struct ModLoader {
    pub registry: ModRegistry,
    pub wasm_host: crate::script::WasmHost,
    pub lua_host: crate::script::LuaHost,
}

impl ModRegistry {
    pub fn new(mod_directory: PathBuf) -> Self {
        Self {
            mods: HashMap::new(),
            hot_reload_queue: Vec::new(),
            file_watcher: None,
            mod_directory,
            replay_mode: false,
        }
    }

    pub fn load_mod(&mut self, mod_path: &Path) -> Result<ModValidationResult> {
        let manifest_path = mod_path.join("mod.toml");
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: ModManifest = toml::from_str(&manifest_content)?;
        
        let validation = manifest.validate();
        if !validation.valid {
            return Ok(validation);
        }

        // Create mod registry entry
        let mut entry = ModRegistryEntry::new(manifest);
        
        // Load content files
        self.load_mod_content(&mut entry, mod_path)?;
        
        // Calculate content hashes
        entry.content_hashes = self.calculate_content_hashes(mod_path)?;
        
        self.mods.insert(entry.manifest.id.clone(), entry);
        
        Ok(validation)
    }

    fn load_mod_content(&self, entry: &mut ModRegistryEntry, mod_path: &Path) -> Result<()> {
        // Load pipelines if specified
        if let Some(pipelines_path) = &entry.manifest.entrypoints.pipelines {
            let full_path = mod_path.join(pipelines_path);
            if full_path.exists() {
                // In a real implementation, this would load and merge pipeline definitions
                println!("Loading pipelines from: {:?}", full_path);
            }
        }

        // Load Black Swan events if specified
        if let Some(events_path) = &entry.manifest.entrypoints.blackswans {
            let full_path = mod_path.join(events_path);
            if full_path.exists() {
                // In a real implementation, this would load and merge event definitions
                println!("Loading Black Swan events from: {:?}", full_path);
            }
        }

        // Load tech tree if specified
        if let Some(tech_path) = &entry.manifest.entrypoints.tech {
            let full_path = mod_path.join(tech_path);
            if full_path.exists() {
                // In a real implementation, this would load and merge tech definitions
                println!("Loading tech tree from: {:?}", full_path);
            }
        }

        // Load scenarios if specified
        if let Some(scenarios_path) = &entry.manifest.entrypoints.scenarios {
            let full_path = mod_path.join(scenarios_path);
            if full_path.exists() {
                // In a real implementation, this would load and merge scenario definitions
                println!("Loading scenarios from: {:?}", full_path);
            }
        }

        Ok(())
    }

    fn calculate_content_hashes(&self, mod_path: &Path) -> Result<ContentHashes> {
        let mut hashes = ContentHashes::default();
        
        // Calculate hash for pipelines.toml
        let pipelines_path = mod_path.join("pipelines.toml");
        if pipelines_path.exists() {
            let content = fs::read(&pipelines_path)?;
            hashes.pipelines = Some(self.hash_content(&content));
        }

        // Calculate hash for events.toml
        let events_path = mod_path.join("events.toml");
        if events_path.exists() {
            let content = fs::read(&events_path)?;
            hashes.blackswans = Some(self.hash_content(&content));
        }

        // Calculate hash for tech.toml
        let tech_path = mod_path.join("tech.toml");
        if tech_path.exists() {
            let content = fs::read(&tech_path)?;
            hashes.tech = Some(self.hash_content(&content));
        }

        // Calculate hash for scenarios.toml
        let scenarios_path = mod_path.join("scenarios.toml");
        if scenarios_path.exists() {
            let content = fs::read(&scenarios_path)?;
            hashes.scenarios = Some(self.hash_content(&content));
        }

        Ok(hashes)
    }

    fn hash_content(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn discover_mods(&mut self) -> Result<Vec<String>> {
        let mut discovered = Vec::new();
        
        if !self.mod_directory.exists() {
            fs::create_dir_all(&self.mod_directory)?;
            return Ok(discovered);
        }

        for entry in fs::read_dir(&self.mod_directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let mod_toml = path.join("mod.toml");
                if mod_toml.exists() {
                    discovered.push(path.to_string_lossy().to_string());
                }
            }
        }

        Ok(discovered)
    }

    pub fn enable_mod(&mut self, mod_id: &str) -> bool {
        if let Some(entry) = self.mods.get_mut(mod_id) {
            entry.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_mod(&mut self, mod_id: &str) -> bool {
        if let Some(entry) = self.mods.get_mut(mod_id) {
            entry.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn get_mod(&self, mod_id: &str) -> Option<&ModRegistryEntry> {
        self.mods.get(mod_id)
    }

    pub fn get_enabled_mods(&self) -> Vec<&ModRegistryEntry> {
        self.mods.values().filter(|entry| entry.enabled).collect()
    }

    pub fn start_file_watcher(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    if let Err(e) = tx.try_send(event) {
                        eprintln!("Failed to send file event: {}", e);
                    }
                }
                Err(e) => eprintln!("File watcher error: {}", e),
            }
        })?;

        watcher.watch(&self.mod_directory, RecursiveMode::Recursive)?;
        self.file_watcher = Some(watcher);

        // Spawn task to handle file events
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                Self::handle_file_event(event);
            }
        });

        Ok(())
    }

    fn handle_file_event(event: Event) {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                println!("File changed: {:?}", event.paths);
                // In a real implementation, this would trigger hot reload
            }
            _ => {}
        }
    }

    pub fn set_replay_mode(&mut self, enabled: bool) {
        self.replay_mode = enabled;
        if enabled {
            // Clear hot reload queue in replay mode
            self.hot_reload_queue.clear();
        }
    }

    pub fn can_hot_reload(&self) -> bool {
        !self.replay_mode
    }
}

impl ModLoader {
    pub fn new(mod_directory: PathBuf) -> Self {
        Self {
            registry: ModRegistry::new(mod_directory),
            wasm_host: crate::script::WasmHost::new(),
            lua_host: crate::script::LuaHost::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        // Discover and load all mods
        let mod_paths = self.registry.discover_mods()?;
        
        for mod_path in mod_paths {
            let path = PathBuf::from(mod_path);
            if let Err(e) = self.registry.load_mod(&path) {
                eprintln!("Failed to load mod from {:?}: {}", path, e);
            }
        }

        // Start file watcher for hot reload
        if self.registry.can_hot_reload() {
            self.registry.start_file_watcher()?;
        }

        Ok(())
    }

    pub fn hot_reload_mod(&mut self, mod_id: &str) -> Result<HotReloadTransaction> {
        if !self.registry.can_hot_reload() {
            return Err(anyhow::anyhow!("Hot reload disabled in replay mode"));
        }

        let old_entry = self.registry.mods.get(mod_id).cloned();
        let mod_path = self.registry.mod_directory.join(mod_id);
        
        // Load new version
        let validation = self.registry.load_mod(&mod_path)?;
        if !validation.valid {
            return Err(anyhow::anyhow!("Mod validation failed: {:?}", validation.errors));
        }

        let new_entry = self.registry.mods.get(mod_id).unwrap().clone();
        
        // Create hot reload transaction
        let mut transaction = HotReloadTransaction {
            mod_id: mod_id.to_string(),
            old_entry,
            new_entry,
            shadow_world_result: None,
            status: HotReloadStatus::Pending,
        };

        // Validate compatibility
        if let Some(ref old) = transaction.old_entry {
            if !old.is_compatible_with(&transaction.new_entry) {
                transaction.status = HotReloadStatus::Failed;
                return Ok(transaction);
            }
        }

        transaction.status = HotReloadStatus::Validating;
        self.registry.hot_reload_queue.push(transaction.clone());
        
        Ok(transaction)
    }

    pub fn execute_shadow_world(&mut self, transaction: &mut HotReloadTransaction) -> Result<()> {
        transaction.status = HotReloadStatus::ShadowWorld;
        
        // In a real implementation, this would:
        // 1. Create a shadow world with minimal state
        // 2. Load the new mod version
        // 3. Run for a few ticks (e.g., 120)
        // 4. Compare KPI deltas
        // 5. Set the result
        
        let shadow_result = ShadowWorldResult {
            success: true,
            kpi_deltas: KpiDeltas::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
            ticks_simulated: 120,
        };
        
        transaction.shadow_world_result = Some(shadow_result);
        transaction.status = HotReloadStatus::Ready;
        
        Ok(())
    }

    pub fn apply_hot_reload(&mut self, transaction: &HotReloadTransaction) -> Result<()> {
        if transaction.status != HotReloadStatus::Ready {
            return Err(anyhow::anyhow!("Transaction not ready for application"));
        }

        // Apply the new mod version
        self.registry.mods.insert(
            transaction.mod_id.clone(),
            transaction.new_entry.clone(),
        );

        // Update WASM and Lua hosts
        // In a real implementation, this would reload the actual modules
        
        Ok(())
    }

    pub fn revert_hot_reload(&mut self, transaction: &HotReloadTransaction) -> Result<()> {
        if let Some(ref old_entry) = transaction.old_entry {
            self.registry.mods.insert(
                transaction.mod_id.clone(),
                old_entry.clone(),
            );
        } else {
            // Remove the mod if it was newly added
            self.registry.mods.remove(&transaction.mod_id);
        }

        Ok(())
    }
}

/// System to process hot reload queue
pub fn process_hot_reload_system(
    mut mod_loader: ResMut<ModLoader>,
    // TODO: Add other resources needed for shadow world simulation
) {
    let mut completed_transactions = Vec::new();
    
    for (i, transaction) in mod_loader.registry.hot_reload_queue.iter_mut().enumerate() {
        match transaction.status {
            HotReloadStatus::Pending => {
                // Start validation
                transaction.status = HotReloadStatus::Validating;
            }
            HotReloadStatus::Validating => {
                // Execute shadow world validation
                if let Err(e) = mod_loader.execute_shadow_world(transaction) {
                    transaction.status = HotReloadStatus::Failed;
                    eprintln!("Shadow world validation failed: {}", e);
                }
            }
            HotReloadStatus::Ready => {
                // Apply the hot reload
                if let Err(e) = mod_loader.apply_hot_reload(transaction) {
                    transaction.status = HotReloadStatus::Failed;
                    eprintln!("Hot reload application failed: {}", e);
                } else {
                    transaction.status = HotReloadStatus::Applied;
                }
            }
            HotReloadStatus::Applied | HotReloadStatus::Failed => {
                completed_transactions.push(i);
            }
            _ => {}
        }
    }

    // Remove completed transactions
    for &i in completed_transactions.iter().rev() {
        mod_loader.registry.hot_reload_queue.remove(i);
    }
}

/// System to initialize mod loader
pub fn initialize_mod_loader_system(
    mut mod_loader: ResMut<ModLoader>,
) {
    if let Err(e) = mod_loader.initialize() {
        eprintln!("Failed to initialize mod loader: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_mod_registry_creation() {
        let temp_dir = TempDir::new().unwrap();
        let registry = ModRegistry::new(temp_dir.path().to_path_buf());
        
        assert!(registry.mods.is_empty());
        assert!(registry.hot_reload_queue.is_empty());
        assert!(!registry.replay_mode);
    }

    #[test]
    fn test_mod_registry_replay_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = ModRegistry::new(temp_dir.path().to_path_buf());
        
        registry.set_replay_mode(true);
        assert!(registry.replay_mode);
        assert!(!registry.can_hot_reload());
        
        registry.set_replay_mode(false);
        assert!(!registry.replay_mode);
        assert!(registry.can_hot_reload());
    }

    #[test]
    fn test_mod_registry_mod_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = ModRegistry::new(temp_dir.path().to_path_buf());
        
        // Test mod retrieval for non-existent mod
        assert!(registry.get_mod("nonexistent").is_none());
        
        // Test mod enabling/disabling for non-existent mod
        assert!(!registry.enable_mod("nonexistent"));
        assert!(!registry.disable_mod("nonexistent"));
        
        // Test enabled mods
        assert!(registry.get_enabled_mods().is_empty());
    }

    #[test]
    fn test_mod_loader_creation() {
        let temp_dir = TempDir::new().unwrap();
        let loader = ModLoader::new(temp_dir.path().to_path_buf());
        
        assert!(loader.registry.mods.is_empty());
        assert!(loader.registry.hot_reload_queue.is_empty());
    }

    #[test]
    fn test_content_hashing() {
        let temp_dir = TempDir::new().unwrap();
        let registry = ModRegistry::new(temp_dir.path().to_path_buf());
        
        let content = b"test content";
        let hash = registry.hash_content(content);
        
        // Hash should be consistent
        let hash2 = registry.hash_content(content);
        assert_eq!(hash, hash2);
        
        // Different content should produce different hash
        let different_content = b"different content";
        let different_hash = registry.hash_content(different_content);
        assert_ne!(hash, different_hash);
    }
}
