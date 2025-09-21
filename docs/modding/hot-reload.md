# Modding: Hot Reload

Hot reload functionality allows you to update and reload mods at runtime without restarting the game. This guide explains how hot reload works, how to use it, and how to create hot-reloadable mods.

## Overview

Hot reload provides:

- **Runtime Updates**: Update mods without restarting the game
- **Fast Iteration**: Rapid development and testing cycles
- **State Preservation**: Preserve game state during reloads
- **Dependency Management**: Handle mod dependencies during reloads
- **Error Recovery**: Graceful error handling during reloads
- **Version Management**: Manage mod versions during reloads

## Hot Reload System

### Hot Reload Manager

The hot reload system is managed by the `HotReloadManager`:

```rust
pub struct HotReloadManager {
    pub mod_registry: ModRegistry,
    pub file_watcher: FileWatcher,
    pub reload_queue: ReloadQueue,
    pub state_manager: StateManager,
    pub dependency_resolver: DependencyResolver,
}

pub struct ReloadQueue {
    pub pending_reloads: Vec<ReloadRequest>,
    pub reload_in_progress: bool,
    pub reload_timeout: Duration,
}

pub struct ReloadRequest {
    pub mod_id: ModId,
    pub reload_type: ReloadType,
    pub priority: ReloadPriority,
    pub dependencies: Vec<ModId>,
    pub timestamp: u64,
}
```

### Reload Types

```rust
pub enum ReloadType {
    Full,                        // Complete mod reload
    Partial,                     // Partial mod reload
    Config,                      // Configuration only
    Scripts,                     // Scripts only
    Assets,                      // Assets only
    Dependencies,                // Dependencies only
}

pub enum ReloadPriority {
    Low,                         // Low priority reload
    Normal,                      // Normal priority reload
    High,                        // High priority reload
    Critical,                    // Critical reload
}
```

## File Watching

### File Watcher Configuration

```rust
pub struct FileWatcher {
    pub watch_paths: Vec<PathBuf>,
    pub ignore_patterns: Vec<String>,
    pub debounce_delay: Duration,
    pub max_file_size: u64,
    pub supported_extensions: Vec<String>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watch_paths: vec![],
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.bak".to_string(),
                "*.swp".to_string(),
                ".git/*".to_string(),
                "target/*".to_string(),
            ],
            debounce_delay: Duration::from_millis(500),
            max_file_size: 10 * 1024 * 1024, // 10MB
            supported_extensions: vec![
                "lua".to_string(),
                "wasm".to_string(),
                "toml".to_string(),
                "json".to_string(),
            ],
        }
    }
}
```

### File Change Detection

```rust
impl FileWatcher {
    pub fn watch_mod_directory(&mut self, mod_path: &Path) -> Result<(), FileWatcherError> {
        // Watch the mod directory
        self.watch_paths.push(mod_path.to_path_buf());
        
        // Watch subdirectories
        for entry in std::fs::read_dir(mod_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.watch_paths.push(path);
            }
        }
        
        Ok(())
    }
    
    pub fn handle_file_change(&mut self, event: FileChangeEvent) -> Result<(), FileWatcherError> {
        let file_path = event.file_path;
        let change_type = event.change_type;
        
        // Check if file should be ignored
        if self.should_ignore_file(&file_path) {
            return Ok(());
        }
        
        // Determine mod ID from file path
        let mod_id = self.get_mod_id_from_path(&file_path)?;
        
        // Determine reload type based on file extension
        let reload_type = self.get_reload_type_from_extension(&file_path)?;
        
        // Create reload request
        let reload_request = ReloadRequest {
            mod_id,
            reload_type,
            priority: ReloadPriority::Normal,
            dependencies: vec![],
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        };
        
        // Add to reload queue
        self.reload_queue.pending_reloads.push(reload_request);
        
        Ok(())
    }
    
    fn should_ignore_file(&self, file_path: &Path) -> bool {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        
        for pattern in &self.ignore_patterns {
            if glob::Pattern::new(pattern).unwrap().matches(&file_name) {
                return true;
            }
        }
        
        false
    }
    
    fn get_reload_type_from_extension(&self, file_path: &Path) -> Result<ReloadType, FileWatcherError> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or(FileWatcherError::InvalidFileExtension)?;
        
        match extension {
            "lua" => Ok(ReloadType::Scripts),
            "wasm" => Ok(ReloadType::Full),
            "toml" => Ok(ReloadType::Config),
            "json" => Ok(ReloadType::Config),
            _ => Ok(ReloadType::Partial),
        }
    }
}
```

## Mod Reloading

### Reload Process

```rust
impl HotReloadManager {
    pub fn process_reload_queue(&mut self) -> Result<(), ReloadError> {
        if self.reload_queue.reload_in_progress {
            return Ok(()); // Skip if reload already in progress
        }
        
        self.reload_queue.reload_in_progress = true;
        
        // Process pending reloads
        while let Some(reload_request) = self.reload_queue.pending_reloads.pop() {
            self.process_reload_request(reload_request)?;
        }
        
        self.reload_queue.reload_in_progress = false;
        Ok(())
    }
    
    fn process_reload_request(&mut self, request: ReloadRequest) -> Result<(), ReloadError> {
        let mod_id = request.mod_id;
        let reload_type = request.reload_type;
        
        // Check if mod is loaded
        if !self.mod_registry.is_mod_loaded(&mod_id) {
            return Err(ReloadError::ModNotLoaded);
        }
        
        // Save current state
        let current_state = self.state_manager.save_mod_state(&mod_id)?;
        
        // Unload mod
        self.unload_mod(&mod_id)?;
        
        // Reload mod
        let reloaded_mod = self.reload_mod(&mod_id, &reload_type)?;
        
        // Restore state
        self.state_manager.restore_mod_state(&mod_id, &current_state)?;
        
        // Update registry
        self.mod_registry.update_mod(reloaded_mod);
        
        Ok(())
    }
}
```

### Mod Unloading

```rust
impl HotReloadManager {
    fn unload_mod(&mut self, mod_id: &ModId) -> Result<(), ReloadError> {
        // Get mod from registry
        let mod_info = self.mod_registry.get_mod(mod_id)
            .ok_or(ReloadError::ModNotFound)?;
        
        // Unload WASM modules
        if let Some(wasm_modules) = &mod_info.wasm_modules {
            for module in wasm_modules {
                self.wasm_host.unload_module(module.id)?;
            }
        }
        
        // Unload Lua scripts
        if let Some(lua_scripts) = &mod_info.lua_scripts {
            for script in lua_scripts {
                self.lua_host.unload_script(script.id)?;
            }
        }
        
        // Unregister event handlers
        self.event_system.unregister_mod_handlers(mod_id)?;
        
        // Clean up resources
        self.resource_manager.cleanup_mod_resources(mod_id)?;
        
        Ok(())
    }
}
```

### Mod Reloading

```rust
impl HotReloadManager {
    fn reload_mod(&mut self, mod_id: &ModId, reload_type: &ReloadType) -> Result<ModInfo, ReloadError> {
        let mod_path = self.get_mod_path(mod_id)?;
        
        match reload_type {
            ReloadType::Full => self.reload_full_mod(&mod_path),
            ReloadType::Partial => self.reload_partial_mod(&mod_path),
            ReloadType::Config => self.reload_config_only(&mod_path),
            ReloadType::Scripts => self.reload_scripts_only(&mod_path),
            ReloadType::Assets => self.reload_assets_only(&mod_path),
            ReloadType::Dependencies => self.reload_dependencies_only(&mod_path),
        }
    }
    
    fn reload_full_mod(&mut self, mod_path: &Path) -> Result<ModInfo, ReloadError> {
        // Parse mod manifest
        let manifest = self.parse_mod_manifest(mod_path)?;
        
        // Load WASM modules
        let wasm_modules = self.load_wasm_modules(mod_path, &manifest)?;
        
        // Load Lua scripts
        let lua_scripts = self.load_lua_scripts(mod_path, &manifest)?;
        
        // Load assets
        let assets = self.load_assets(mod_path, &manifest)?;
        
        // Create mod info
        let mod_info = ModInfo {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            manifest,
            wasm_modules: Some(wasm_modules),
            lua_scripts: Some(lua_scripts),
            assets: Some(assets),
            dependencies: vec![],
            capabilities: vec![],
        };
        
        Ok(mod_info)
    }
    
    fn reload_scripts_only(&mut self, mod_path: &Path) -> Result<ModInfo, ReloadError> {
        // Get existing mod info
        let mut mod_info = self.mod_registry.get_mod(&self.get_mod_id_from_path(mod_path)?)
            .ok_or(ReloadError::ModNotFound)?;
        
        // Reload only Lua scripts
        let lua_scripts = self.load_lua_scripts(mod_path, &mod_info.manifest)?;
        mod_info.lua_scripts = Some(lua_scripts);
        
        Ok(mod_info)
    }
}
```

## State Management

### State Preservation

```rust
pub struct StateManager {
    pub mod_states: HashMap<ModId, ModState>,
    pub global_state: GlobalState,
    pub state_serializer: StateSerializer,
}

pub struct ModState {
    pub mod_id: ModId,
    pub lua_state: Option<LuaState>,
    pub wasm_state: Option<WasmState>,
    pub config_state: Option<ConfigState>,
    pub asset_state: Option<AssetState>,
    pub timestamp: u64,
}

impl StateManager {
    pub fn save_mod_state(&mut self, mod_id: &ModId) -> Result<ModState, StateError> {
        let mut mod_state = ModState {
            mod_id: mod_id.clone(),
            lua_state: None,
            wasm_state: None,
            config_state: None,
            asset_state: None,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        };
        
        // Save Lua state
        if let Some(lua_host) = &self.lua_host {
            mod_state.lua_state = Some(lua_host.save_state(mod_id)?);
        }
        
        // Save WASM state
        if let Some(wasm_host) = &self.wasm_host {
            mod_state.wasm_state = Some(wasm_host.save_state(mod_id)?);
        }
        
        // Save config state
        mod_state.config_state = Some(self.save_config_state(mod_id)?);
        
        // Save asset state
        mod_state.asset_state = Some(self.save_asset_state(mod_id)?);
        
        self.mod_states.insert(mod_id.clone(), mod_state.clone());
        Ok(mod_state)
    }
    
    pub fn restore_mod_state(&mut self, mod_id: &ModId, state: &ModState) -> Result<(), StateError> {
        // Restore Lua state
        if let Some(lua_state) = &state.lua_state {
            if let Some(lua_host) = &mut self.lua_host {
                lua_host.restore_state(mod_id, lua_state)?;
            }
        }
        
        // Restore WASM state
        if let Some(wasm_state) = &state.wasm_state {
            if let Some(wasm_host) = &mut self.wasm_host {
                wasm_host.restore_state(mod_id, wasm_state)?;
            }
        }
        
        // Restore config state
        if let Some(config_state) = &state.config_state {
            self.restore_config_state(mod_id, config_state)?;
        }
        
        // Restore asset state
        if let Some(asset_state) = &state.asset_state {
            self.restore_asset_state(mod_id, asset_state)?;
        }
        
        Ok(())
    }
}
```

## Dependency Management

### Dependency Resolution

```rust
pub struct DependencyResolver {
    pub dependency_graph: DependencyGraph,
    pub resolution_cache: HashMap<ModId, Vec<ModId>>,
}

impl DependencyResolver {
    pub fn resolve_dependencies(&mut self, mod_id: &ModId) -> Result<Vec<ModId>, DependencyError> {
        // Check cache first
        if let Some(cached) = self.resolution_cache.get(mod_id) {
            return Ok(cached.clone());
        }
        
        // Resolve dependencies
        let dependencies = self.resolve_dependencies_recursive(mod_id)?;
        
        // Cache result
        self.resolution_cache.insert(mod_id.clone(), dependencies.clone());
        
        Ok(dependencies)
    }
    
    fn resolve_dependencies_recursive(&self, mod_id: &ModId) -> Result<Vec<ModId>, DependencyError> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        
        self.resolve_dependencies_dfs(mod_id, &mut resolved, &mut visited)?;
        
        Ok(resolved)
    }
    
    fn resolve_dependencies_dfs(
        &self,
        mod_id: &ModId,
        resolved: &mut Vec<ModId>,
        visited: &mut HashSet<ModId>,
    ) -> Result<(), DependencyError> {
        if visited.contains(mod_id) {
            return Ok(()); // Already visited
        }
        
        visited.insert(mod_id.clone());
        
        // Get mod dependencies
        let dependencies = self.dependency_graph.get_dependencies(mod_id)?;
        
        // Resolve dependencies recursively
        for dep_id in dependencies {
            self.resolve_dependencies_dfs(&dep_id, resolved, visited)?;
        }
        
        // Add to resolved list
        resolved.push(mod_id.clone());
        
        Ok(())
    }
}
```

### Dependency Validation

```rust
impl DependencyResolver {
    pub fn validate_dependencies(&self, mod_id: &ModId) -> Result<(), DependencyError> {
        let dependencies = self.resolve_dependencies(mod_id)?;
        
        for dep_id in dependencies {
            // Check if dependency is loaded
            if !self.is_mod_loaded(&dep_id) {
                return Err(DependencyError::DependencyNotLoaded(dep_id));
            }
            
            // Check version compatibility
            if !self.is_version_compatible(mod_id, &dep_id)? {
                return Err(DependencyError::VersionIncompatible(dep_id));
            }
        }
        
        Ok(())
    }
    
    fn is_version_compatible(&self, mod_id: &ModId, dep_id: &ModId) -> Result<bool, DependencyError> {
        let mod_info = self.get_mod_info(mod_id)?;
        let dep_info = self.get_mod_info(dep_id)?;
        
        // Check version constraints
        for constraint in &mod_info.dependencies {
            if constraint.mod_id == *dep_id {
                return Ok(constraint.version_constraint.matches(&dep_info.version));
            }
        }
        
        Ok(true) // No specific constraint
    }
}
```

## Error Handling

### Reload Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReloadError {
    #[error("Mod not found: {0}")]
    ModNotFound(ModId),
    
    #[error("Mod not loaded: {0}")]
    ModNotLoaded(ModId),
    
    #[error("Dependency error: {0}")]
    DependencyError(#[from] DependencyError),
    
    #[error("State error: {0}")]
    StateError(#[from] StateError),
    
    #[error("File watcher error: {0}")]
    FileWatcherError(#[from] FileWatcherError),
    
    #[error("WASM error: {0}")]
    WasmError(#[from] WasmError),
    
    #[error("Lua error: {0}")]
    LuaError(#[from] LuaError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] SerializationError),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
}
```

### Error Recovery

```rust
impl HotReloadManager {
    pub fn handle_reload_error(&mut self, error: ReloadError, mod_id: &ModId) -> Result<(), ReloadError> {
        match error {
            ReloadError::DependencyError(_) => {
                // Try to resolve dependencies
                self.resolve_dependencies(mod_id)?;
                self.retry_reload(mod_id)?;
            },
            ReloadError::StateError(_) => {
                // Reset state and retry
                self.reset_mod_state(mod_id)?;
                self.retry_reload(mod_id)?;
            },
            ReloadError::WasmError(_) => {
                // Fall back to previous version
                self.fallback_to_previous_version(mod_id)?;
            },
            ReloadError::LuaError(_) => {
                // Reload scripts only
                self.reload_scripts_only(mod_id)?;
            },
            _ => {
                // Log error and continue
                self.log_reload_error(&error, mod_id);
            }
        }
        
        Ok(())
    }
    
    fn retry_reload(&mut self, mod_id: &ModId) -> Result<(), ReloadError> {
        let reload_request = ReloadRequest {
            mod_id: mod_id.clone(),
            reload_type: ReloadType::Full,
            priority: ReloadPriority::High,
            dependencies: vec![],
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        };
        
        self.reload_queue.pending_reloads.push(reload_request);
        Ok(())
    }
}
```

## Configuration

### Hot Reload Configuration

```toml
# In mod configuration
[hot_reload]
enabled = true
watch_mod_directory = true
watch_subdirectories = true
debounce_delay = 500
max_file_size = 10485760
supported_extensions = ["lua", "wasm", "toml", "json"]

[hot_reload.ignore_patterns]
patterns = [
    "*.tmp",
    "*.bak",
    "*.swp",
    ".git/*",
    "target/*",
    "*.log"
]

[hot_reload.reload_queue]
max_pending_reloads = 100
reload_timeout = 30
priority_handling = true

[hot_reload.state_management]
preserve_state = true
state_serialization = "bincode"
state_compression = "gzip"
max_state_size = 1048576

[hot_reload.dependencies]
auto_resolve = true
validate_versions = true
cache_resolutions = true
max_dependency_depth = 10
```

## Usage Examples

### Basic Hot Reload Usage

```lua
-- In your Lua mod
local function on_mod_loaded(event_data)
    print("Mod loaded, setting up hot reload")
    
    -- Register event handlers
    colony.events.register("tick_start", on_tick_start)
    colony.events.register("job_created", on_job_created)
end

local function on_tick_start(event_data)
    -- This function will be updated when the file is saved
    print("Tick started: " .. event_data.tick)
    
    -- Your logic here
    if event_data.tick % 100 == 0 then
        print("Reached tick milestone: " .. event_data.tick)
    end
end

local function on_job_created(event_data)
    -- This function will also be updated when the file is saved
    print("Job created: " .. event_data.job.id)
    
    -- Your logic here
    if event_data.job.priority >= 8 then
        print("High priority job detected")
    end
end

-- Register mod loaded handler
colony.events.register("mod_loaded", on_mod_loaded)
```

### Advanced Hot Reload Usage

```lua
-- Advanced hot reload with state preservation
local ModState = {
    tick_count = 0,
    job_count = 0,
    last_reload = 0
}

local function on_mod_loaded(event_data)
    print("Mod loaded, setting up advanced hot reload")
    
    -- Restore state if available
    local saved_state = colony.state.get_mod_state("my_mod")
    if saved_state then
        ModState = saved_state
        print("Restored mod state from previous session")
    end
    
    -- Register event handlers
    colony.events.register("tick_start", on_tick_start)
    colony.events.register("job_created", on_job_created)
    colony.events.register("mod_unloaded", on_mod_unloaded)
end

local function on_tick_start(event_data)
    ModState.tick_count = ModState.tick_count + 1
    
    -- Your logic here
    if ModState.tick_count % 100 == 0 then
        print("Reached tick milestone: " .. ModState.tick_count)
    end
end

local function on_job_created(event_data)
    ModState.job_count = ModState.job_count + 1
    
    -- Your logic here
    if event_data.job.priority >= 8 then
        print("High priority job detected: " .. ModState.job_count)
    end
end

local function on_mod_unloaded(event_data)
    -- Save state before unloading
    ModState.last_reload = colony.time.get_current_tick()
    colony.state.save_mod_state("my_mod", ModState)
    print("Saved mod state before unload")
end

-- Register mod loaded handler
colony.events.register("mod_loaded", on_mod_loaded)
```

## Best Practices

### Design Guidelines

1. **State Management**: Design mods to preserve state during reloads
2. **Error Handling**: Implement robust error handling for reload scenarios
3. **Dependency Management**: Manage dependencies carefully
4. **Performance**: Keep reload operations efficient
5. **Testing**: Test hot reload functionality thoroughly

### Performance Considerations

1. **File Watching**: Use efficient file watching mechanisms
2. **State Serialization**: Optimize state serialization and deserialization
3. **Dependency Resolution**: Cache dependency resolutions
4. **Resource Management**: Manage resources efficiently during reloads
5. **Error Recovery**: Implement fast error recovery mechanisms

### Security Considerations

1. **File Validation**: Validate files before reloading
2. **State Validation**: Validate state before restoration
3. **Dependency Validation**: Validate dependencies before reloading
4. **Resource Limits**: Respect resource limits during reloads
5. **Error Information**: Don't leak sensitive information in errors

---

**Hot reload provides powerful development capabilities for the Colony Simulator. Understanding these concepts is key to efficient mod development.** 🏭🔄
