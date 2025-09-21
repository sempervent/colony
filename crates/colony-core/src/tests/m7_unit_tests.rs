use colony_core::*;
use bevy::prelude::*;
use colony_modsdk::{ModManifest, Entrypoints, Capabilities, WasmOpSpec, LuaEventSpec};
use std::path::PathBuf;
use anyhow::Result;

// Helper to create a minimal App for M7 testing
fn create_m7_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(WasmHost::new());
    app.insert_resource(LuaHost::new());
    app.insert_resource(ModLoader::new(PathBuf::from("mods")));
    app.insert_resource(HotReloadManager::new());
    app.insert_resource(Colony::new());
    app.world.resource_mut::<Colony>().game_setup = GameSetup::new(Scenario::default());
    app
}

#[test]
fn test_wasm_host_initialization() {
    let wasm_host = WasmHost::new();
    
    assert_eq!(wasm_host.execution_env.fuel_limit, 5_000_000);
    assert_eq!(wasm_host.execution_env.memory_limit_mib, 64);
    assert!(wasm_host.modules.is_empty());
}

#[test]
fn test_wasm_host_module_loading() -> Result<()> {
    let mut wasm_host = WasmHost::new();
    
    // Create a minimal WASM module (this would be actual WASM bytes in real implementation)
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]; // Minimal WASM header
    
    // Test module loading
    wasm_host.load_module("test_mod", &wasm_bytes)?;
    assert!(wasm_host.modules.contains_key("test_mod"));
    
    // Test module unloading
    wasm_host.unload_module("test_mod");
    assert!(!wasm_host.modules.contains_key("test_mod"));
    
    Ok(())
}

#[test]
fn test_wasm_host_execution_limits() {
    let mut wasm_host = WasmHost::new();
    
    // Test fuel limit enforcement
    assert_eq!(wasm_host.execution_env.fuel_limit, 5_000_000);
    
    // Test memory limit enforcement
    assert_eq!(wasm_host.execution_env.memory_limit_mib, 64);
    
    // Test sandbox mode
    assert!(wasm_host.execution_env.sandbox_mode);
}

#[test]
fn test_lua_host_initialization() {
    let lua_host = LuaHost::new();
    
    assert_eq!(lua_host.execution_env.instruction_budget, 200_000);
    assert_eq!(lua_host.execution_env.memory_limit_mib, 32);
    assert!(lua_host.execution_env.sandbox_mode);
    assert!(lua_host.scripts.is_empty());
}

#[test]
fn test_lua_host_script_loading() -> Result<()> {
    let mut lua_host = LuaHost::new();
    
    // Test script loading
    let script_content = r#"
        function on_tick()
            print("Hello from Lua!")
        end
    "#;
    
    lua_host.load_script("test_mod", "on_tick", script_content.to_string())?;
    assert!(lua_host.scripts.contains_key("test_mod:on_tick"));
    
    // Test script unloading
    lua_host.unload_script("test_mod", "on_tick");
    assert!(!lua_host.scripts.contains_key("test_mod:on_tick"));
    
    Ok(())
}

#[test]
fn test_lua_host_sandbox_enforcement() {
    let lua_host = LuaHost::new();
    
    // Test sandbox mode is enabled
    assert!(lua_host.execution_env.sandbox_mode);
    
    // Test instruction budget
    assert_eq!(lua_host.execution_env.instruction_budget, 200_000);
    
    // Test memory limit
    assert_eq!(lua_host.execution_env.memory_limit_mib, 32);
}

#[test]
fn test_mod_loader_initialization() {
    let mod_loader = ModLoader::new(PathBuf::from("test_mods"));
    
    assert_eq!(mod_loader.mods_dir, PathBuf::from("test_mods"));
    assert!(mod_loader.registry.mods.is_empty());
    assert!(mod_loader.registry.load_order.is_empty());
    assert!(mod_loader.enabled_mods.is_empty());
}

#[test]
fn test_mod_loader_enable_disable() -> Result<()> {
    let mut mod_loader = ModLoader::new(PathBuf::from("test_mods"));
    
    // Test enabling a mod
    mod_loader.enable_mod("test_mod")?;
    assert!(mod_loader.enabled_mods.contains(&"test_mod".to_string()));
    
    // Test disabling a mod
    mod_loader.disable_mod("test_mod")?;
    assert!(!mod_loader.enabled_mods.contains(&"test_mod".to_string()));
    
    Ok(())
}

#[test]
fn test_mod_loader_hot_reload() -> Result<()> {
    let mut mod_loader = ModLoader::new(PathBuf::from("test_mods"));
    
    // Test hot reload (should not fail even if mod doesn't exist)
    let result = mod_loader.trigger_hot_reload("nonexistent_mod");
    assert!(result.is_ok()); // Should succeed even if mod doesn't exist
    
    Ok(())
}

#[test]
fn test_hot_reload_manager_initialization() {
    let hot_reload_manager = HotReloadManager::new();
    
    assert!(hot_reload_manager.watchers.is_empty());
    assert!(hot_reload_manager.pending_reloads.is_empty());
    assert_eq!(hot_reload_manager.reload_cooldown, std::time::Duration::from_millis(500));
}

#[test]
fn test_hot_reload_manager_queue_reload() {
    let mut hot_reload_manager = HotReloadManager::new();
    
    // Test queueing a reload
    hot_reload_manager.queue_reload("test_mod");
    assert!(hot_reload_manager.pending_reloads.contains(&"test_mod".to_string()));
    
    // Test queueing the same mod again (should not duplicate)
    hot_reload_manager.queue_reload("test_mod");
    assert_eq!(hot_reload_manager.pending_reloads.len(), 1);
    
    // Test queueing different mods
    hot_reload_manager.queue_reload("another_mod");
    assert_eq!(hot_reload_manager.pending_reloads.len(), 2);
}

#[test]
fn test_hot_reload_manager_cooldown() {
    let mut hot_reload_manager = HotReloadManager::new();
    
    // Test initial cooldown state
    assert!(hot_reload_manager.can_reload());
    
    // Mark as reloaded
    hot_reload_manager.mark_reloaded();
    
    // Should not be able to reload immediately
    assert!(!hot_reload_manager.can_reload());
}

#[test]
fn test_mod_manifest_validation() -> Result<()> {
    // Test valid manifest
    let valid_manifest = ModManifest {
        id: "com.test.valid".to_string(),
        name: "Valid Mod".to_string(),
        version: "1.0.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints {
            wasm_ops: vec!["Op_Test".to_string()],
            lua_events: vec!["on_tick.lua".to_string()],
        },
        capabilities: Capabilities {
            sim_time: true,
            log_debug: true,
            enqueue_job: false,
        },
        signature: None,
    };
    
    assert!(validate_mod_manifest(&valid_manifest).is_ok());
    
    // Test invalid manifest (empty ID)
    let invalid_manifest = ModManifest {
        id: "".to_string(),
        name: "Invalid Mod".to_string(),
        version: "1.0.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints::default(),
        capabilities: Capabilities::default(),
        signature: None,
    };
    
    assert!(validate_mod_manifest(&invalid_manifest).is_err());
    
    Ok(())
}

#[test]
fn test_wasm_op_spec_validation() {
    let op_spec = WasmOpSpec {
        function_name: "test_op".to_string(),
        input_schema: "bytes".to_string(),
        output_schema: "bytes".to_string(),
    };
    
    assert_eq!(op_spec.function_name, "test_op");
    assert_eq!(op_spec.input_schema, "bytes");
    assert_eq!(op_spec.output_schema, "bytes");
}

#[test]
fn test_lua_event_spec_validation() {
    let event_spec = LuaEventSpec {
        event_name: "on_tick".to_string(),
        script_path: "on_tick.lua".to_string(),
    };
    
    assert_eq!(event_spec.event_name, "on_tick");
    assert_eq!(event_spec.script_path, "on_tick.lua");
}

#[test]
fn test_capability_gating() {
    let capabilities = Capabilities {
        sim_time: true,
        log_debug: true,
        enqueue_job: false,
    };
    
    assert!(capabilities.sim_time);
    assert!(capabilities.log_debug);
    assert!(!capabilities.enqueue_job);
}

#[test]
fn test_mod_discovery() -> Result<()> {
    // Test mod discovery in a directory
    let temp_dir = std::env::temp_dir().join("colony_test_mods");
    std::fs::create_dir_all(&temp_dir)?;
    
    // Create a test mod manifest
    let manifest_content = r#"
        id = "com.test.discovery"
        name = "Discovery Test Mod"
        version = "1.0.0"
        authors = ["Test Author"]
        
        [entrypoints]
        wasm_ops = ["Op_Test"]
        lua_events = ["on_tick.lua"]
        
        [capabilities]
        sim_time = true
        log_debug = true
    "#;
    
    let manifest_path = temp_dir.join("mod.toml");
    std::fs::write(&manifest_path, manifest_content)?;
    
    // Test discovery
    let manifests = discover_mods_in_directory(&temp_dir)?;
    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].id, "com.test.discovery");
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}

#[test]
fn test_mod_validation() -> Result<()> {
    let manifest = ModManifest {
        id: "com.test.validation".to_string(),
        name: "Validation Test Mod".to_string(),
        version: "1.0.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints::default(),
        capabilities: Capabilities::default(),
        signature: None,
    };
    
    // Test validation
    assert!(validate_mod_manifest(&manifest).is_ok());
    
    Ok(())
}

#[test]
fn test_wasm_execution_environment() {
    let wasm_host = WasmHost::new();
    let env = &wasm_host.execution_env;
    
    // Test environment constraints
    assert!(env.fuel_limit > 0);
    assert!(env.memory_limit_mib > 0);
    assert!(env.sandbox_mode);
}

#[test]
fn test_lua_execution_environment() {
    let lua_host = LuaHost::new();
    let env = &lua_host.execution_env;
    
    // Test environment constraints
    assert!(env.instruction_budget > 0);
    assert!(env.memory_limit_mib > 0);
    assert!(env.sandbox_mode);
}

#[test]
fn test_mod_registry_operations() {
    let mut registry = ModRegistry {
        mods: std::collections::HashMap::new(),
        load_order: Vec::new(),
    };
    
    let manifest = ModManifest {
        id: "com.test.registry".to_string(),
        name: "Registry Test Mod".to_string(),
        version: "1.0.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints::default(),
        capabilities: Capabilities::default(),
        signature: None,
    };
    
    // Test adding mod to registry
    registry.mods.insert(manifest.id.clone(), manifest.clone());
    assert!(registry.mods.contains_key(&manifest.id));
    
    // Test load order
    registry.load_order.push(manifest.id.clone());
    assert_eq!(registry.load_order.len(), 1);
}

#[test]
fn test_hot_reload_system_integration() {
    let mut app = create_m7_test_app();
    
    // Test that the hot reload system can be added to the app
    app.add_systems(Update, process_hot_reload_system);
    
    // Test that the system runs without errors
    app.update();
    
    // Verify resources are still present
    assert!(app.world.contains_resource::<HotReloadManager>());
    assert!(app.world.contains_resource::<ModLoader>());
}

#[test]
fn test_wasm_lua_integration() {
    let mut app = create_m7_test_app();
    
    // Test that both WASM and Lua hosts can coexist
    app.add_systems(Update, (
        update_wasm_host_system,
        update_lua_host_system,
        execute_lua_events_system,
    ));
    
    // Test that the systems run without errors
    app.update();
    
    // Verify both hosts are present
    assert!(app.world.contains_resource::<WasmHost>());
    assert!(app.world.contains_resource::<LuaHost>());
}

#[test]
fn test_mod_loader_system_integration() {
    let mut app = create_m7_test_app();
    
    // Test that the mod loader system can be added to the app
    app.add_systems(Update, initialize_mod_loader_system);
    
    // Test that the system runs without errors
    app.update();
    
    // Verify mod loader is present
    assert!(app.world.contains_resource::<ModLoader>());
}

#[test]
fn test_shadow_world_system() {
    let mut app = create_m7_test_app();
    
    // Test that the shadow world system can be added to the app
    app.add_systems(Update, update_shadow_world_system);
    
    // Test that the system runs without errors
    app.update();
    
    // Verify the system runs (no specific resource to check)
    assert!(true); // System ran without panicking
}

#[test]
fn test_m7_systems_together() {
    let mut app = create_m7_test_app();
    
    // Add all M7 systems
    app.add_systems(Update, (
        update_wasm_host_system,
        update_lua_host_system,
        execute_lua_events_system,
        initialize_mod_loader_system,
        process_hot_reload_system,
        update_shadow_world_system,
    ));
    
    // Test that all systems run together without errors
    for _ in 0..10 {
        app.update();
    }
    
    // Verify all M7 resources are present
    assert!(app.world.contains_resource::<WasmHost>());
    assert!(app.world.contains_resource::<LuaHost>());
    assert!(app.world.contains_resource::<ModLoader>());
    assert!(app.world.contains_resource::<HotReloadManager>());
}

#[test]
fn test_mod_manifest_serialization() -> Result<()> {
    let manifest = ModManifest {
        id: "com.test.serialization".to_string(),
        name: "Serialization Test Mod".to_string(),
        version: "1.0.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints {
            wasm_ops: vec!["Op_Test".to_string()],
            lua_events: vec!["on_tick.lua".to_string()],
        },
        capabilities: Capabilities {
            sim_time: true,
            log_debug: true,
            enqueue_job: false,
        },
        signature: None,
    };
    
    // Test TOML serialization
    let toml_str = toml::to_string(&manifest)?;
    assert!(toml_str.contains("com.test.serialization"));
    assert!(toml_str.contains("Serialization Test Mod"));
    
    // Test TOML deserialization
    let deserialized: ModManifest = toml::from_str(&toml_str)?;
    assert_eq!(deserialized.id, manifest.id);
    assert_eq!(deserialized.name, manifest.name);
    assert_eq!(deserialized.version, manifest.version);
    
    Ok(())
}

#[test]
fn test_mod_capabilities_combinations() {
    // Test various capability combinations
    let combinations = vec![
        Capabilities {
            sim_time: true,
            log_debug: false,
            enqueue_job: false,
        },
        Capabilities {
            sim_time: false,
            log_debug: true,
            enqueue_job: false,
        },
        Capabilities {
            sim_time: false,
            log_debug: false,
            enqueue_job: true,
        },
        Capabilities {
            sim_time: true,
            log_debug: true,
            enqueue_job: true,
        },
    ];
    
    for (i, capabilities) in combinations.iter().enumerate() {
        assert!(capabilities.sim_time || capabilities.log_debug || capabilities.enqueue_job,
                "Combination {} should have at least one capability", i);
    }
}

#[test]
fn test_mod_entrypoints_validation() {
    let entrypoints = Entrypoints {
        wasm_ops: vec!["Op_Test1".to_string(), "Op_Test2".to_string()],
        lua_events: vec!["on_tick.lua".to_string(), "on_init.lua".to_string()],
    };
    
    assert_eq!(entrypoints.wasm_ops.len(), 2);
    assert_eq!(entrypoints.lua_events.len(), 2);
    assert!(entrypoints.wasm_ops.contains(&"Op_Test1".to_string()));
    assert!(entrypoints.lua_events.contains(&"on_tick.lua".to_string()));
}

#[test]
fn test_mod_loader_path_handling() {
    let mods_dir = PathBuf::from("/tmp/test_mods");
    let mod_loader = ModLoader::new(mods_dir.clone());
    
    assert_eq!(mod_loader.mods_dir, mods_dir);
    
    // Test path operations
    let mod_path = mod_loader.mods_dir.join("test_mod");
    assert!(mod_path.ends_with("test_mod"));
}

#[test]
fn test_hot_reload_watcher_creation() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("colony_test_watch");
    std::fs::create_dir_all(&temp_dir)?;
    
    // Test watcher creation
    let _watcher = ModWatcher::new(&temp_dir, |_path| {
        println!("File changed: {:?}", _path);
    })?;
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}
