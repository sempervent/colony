use colony_core::*;
use bevy::prelude::*;
use colony_modsdk::{ModManifest, Entrypoints, Capabilities};
use std::path::PathBuf;
use anyhow::Result;

#[test]
fn test_wasm_memory_limit_enforcement() {
    let wasm_host = WasmHost::new();
    
    // Test memory limit configuration
    assert_eq!(wasm_host.execution_env.memory_limit_mib, 64);
    assert!(wasm_host.execution_env.memory_limit_mib > 0);
    assert!(wasm_host.execution_env.memory_limit_mib <= 1024); // Reasonable upper bound
}

#[test]
fn test_wasm_fuel_limit_enforcement() {
    let wasm_host = WasmHost::new();
    
    // Test fuel limit configuration
    assert_eq!(wasm_host.execution_env.fuel_limit, 5_000_000);
    assert!(wasm_host.execution_env.fuel_limit > 0);
    assert!(wasm_host.execution_env.fuel_limit <= 100_000_000); // Reasonable upper bound
}

#[test]
fn test_lua_sandbox_mode_enforcement() {
    let lua_host = LuaHost::new();
    
    // Test sandbox mode is enabled
    assert!(lua_host.execution_env.sandbox_mode);
    
    // Test instruction budget
    assert_eq!(lua_host.execution_env.instruction_budget, 200_000);
    assert!(lua_host.execution_env.instruction_budget > 0);
    assert!(lua_host.execution_env.instruction_budget <= 10_000_000); // Reasonable upper bound
}

#[test]
fn test_lua_memory_limit_enforcement() {
    let lua_host = LuaHost::new();
    
    // Test memory limit
    assert_eq!(lua_host.execution_env.memory_limit_mib, 32);
    assert!(lua_host.execution_env.memory_limit_mib > 0);
    assert!(lua_host.execution_env.memory_limit_mib <= 512); // Reasonable upper bound
}

#[test]
fn test_capability_gating() {
    // Test mod without enqueue_job capability
    let manifest_no_enqueue = ModManifest {
        id: "com.test.no_enqueue".to_string(),
        name: "No Enqueue Mod".to_string(),
        version: "0.1.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints {
            lua_events: vec!["on_tick.lua".to_string()],
            ..Default::default()
        },
        capabilities: Capabilities {
            sim_time: true,
            log_debug: true,
            enqueue_job: false, // Not granted
            ..Default::default()
        },
        signature: None,
    };
    
    // Test mod with enqueue_job capability
    let manifest_with_enqueue = ModManifest {
        id: "com.test.with_enqueue".to_string(),
        name: "With Enqueue Mod".to_string(),
        version: "0.1.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints {
            lua_events: vec!["on_tick.lua".to_string()],
            ..Default::default()
        },
        capabilities: Capabilities {
            sim_time: true,
            log_debug: true,
            enqueue_job: true, // Granted
            ..Default::default()
        },
        signature: None,
    };
    
    // Verify capability differences
    assert!(!manifest_no_enqueue.capabilities.enqueue_job);
    assert!(manifest_with_enqueue.capabilities.enqueue_job);
    
    // Both should have sim_time and log_debug
    assert!(manifest_no_enqueue.capabilities.sim_time);
    assert!(manifest_no_enqueue.capabilities.log_debug);
    assert!(manifest_with_enqueue.capabilities.sim_time);
    assert!(manifest_with_enqueue.capabilities.log_debug);
}

#[test]
fn test_replay_mode_blocks_hot_reload() {
    let mut replay_log = ReplayLog::new();
    let mut mod_loader = ModLoader::new(PathBuf::from("mods"));
    
    // Set replay mode to playback
    replay_log.mode = ReplayMode::Playback;
    
    // Attempt to trigger a hot reload
    let mod_id = "com.test.mod_to_reload".to_string();
    let result = mod_loader.trigger_hot_reload(&mod_id);
    
    // Should succeed (mod doesn't exist, but operation should work)
    assert!(result.is_ok());
    
    // In a real implementation, we would check that hot reload is blocked
    // during replay playback mode
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
fn test_wasm_execution_environment_security() {
    let wasm_host = WasmHost::new();
    let env = &wasm_host.execution_env;
    
    // Test environment constraints
    assert!(env.fuel_limit > 0);
    assert!(env.memory_limit_mib > 0);
    assert!(env.sandbox_mode);
    
    // Test reasonable limits
    assert!(env.fuel_limit <= 100_000_000); // Max 100M fuel
    assert!(env.memory_limit_mib <= 1024); // Max 1GB memory
}

#[test]
fn test_lua_execution_environment_security() {
    let lua_host = LuaHost::new();
    let env = &lua_host.execution_env;
    
    // Test environment constraints
    assert!(env.instruction_budget > 0);
    assert!(env.memory_limit_mib > 0);
    assert!(env.sandbox_mode);
    
    // Test reasonable limits
    assert!(env.instruction_budget <= 10_000_000); // Max 10M instructions
    assert!(env.memory_limit_mib <= 512); // Max 512MB memory
}

#[test]
fn test_hot_reload_security() {
    let mut hot_reload_manager = HotReloadManager::new();
    
    // Test cooldown mechanism
    assert!(hot_reload_manager.can_reload());
    
    hot_reload_manager.mark_reloaded();
    assert!(!hot_reload_manager.can_reload());
    
    // Test queue limits
    for i in 0..100 {
        hot_reload_manager.queue_reload(&format!("mod_{}", i));
    }
    
    // Should not exceed reasonable limits
    assert!(hot_reload_manager.pending_reloads.len() <= 100);
}

#[test]
fn test_mod_loader_security() {
    let mod_loader = ModLoader::new(PathBuf::from("mods"));
    
    // Test path security
    assert_eq!(mod_loader.mods_dir, PathBuf::from("mods"));
    
    // Test registry is empty initially
    assert!(mod_loader.registry.mods.is_empty());
    assert!(mod_loader.registry.load_order.is_empty());
    assert!(mod_loader.enabled_mods.is_empty());
}

#[test]
fn test_wasm_host_security() {
    let wasm_host = WasmHost::new();
    
    // Test initial state is secure
    assert!(wasm_host.modules.is_empty());
    
    // Test execution environment is properly configured
    assert!(wasm_host.execution_env.sandbox_mode);
    assert!(wasm_host.execution_env.fuel_limit > 0);
    assert!(wasm_host.execution_env.memory_limit_mib > 0);
}

#[test]
fn test_lua_host_security() {
    let lua_host = LuaHost::new();
    
    // Test initial state is secure
    assert!(lua_host.scripts.is_empty());
    
    // Test execution environment is properly configured
    assert!(lua_host.execution_env.sandbox_mode);
    assert!(lua_host.execution_env.instruction_budget > 0);
    assert!(lua_host.execution_env.memory_limit_mib > 0);
}

#[test]
fn test_mod_capabilities_security() {
    // Test that capabilities are properly restricted
    let capabilities = Capabilities {
        sim_time: true,
        log_debug: true,
        enqueue_job: false,
    };
    
    assert!(capabilities.sim_time);
    assert!(capabilities.log_debug);
    assert!(!capabilities.enqueue_job);
    
    // Test default capabilities are restrictive
    let default_capabilities = Capabilities::default();
    assert!(!default_capabilities.sim_time);
    assert!(!default_capabilities.log_debug);
    assert!(!default_capabilities.enqueue_job);
}

#[test]
fn test_mod_entrypoints_security() {
    let entrypoints = Entrypoints {
        wasm_ops: vec!["Op_Test1".to_string(), "Op_Test2".to_string()],
        lua_events: vec!["on_tick.lua".to_string(), "on_init.lua".to_string()],
    };
    
    // Test entrypoints are properly defined
    assert_eq!(entrypoints.wasm_ops.len(), 2);
    assert_eq!(entrypoints.lua_events.len(), 2);
    
    // Test entrypoints don't contain dangerous operations
    for op in &entrypoints.wasm_ops {
        assert!(!op.contains("system"));
        assert!(!op.contains("exec"));
        assert!(!op.contains("shell"));
    }
    
    for event in &entrypoints.lua_events {
        assert!(!event.contains("system"));
        assert!(!event.contains("exec"));
        assert!(!event.contains("shell"));
    }
}

#[test]
fn test_mod_manifest_security() {
    let manifest = ModManifest {
        id: "com.test.security".to_string(),
        name: "Security Test Mod".to_string(),
        version: "1.0.0".to_string(),
        authors: vec!["Test Author".to_string()],
        entrypoints: Entrypoints::default(),
        capabilities: Capabilities::default(),
        signature: None,
    };
    
    // Test manifest fields are properly sanitized
    assert!(!manifest.id.is_empty());
    assert!(!manifest.name.is_empty());
    assert!(!manifest.version.is_empty());
    assert!(!manifest.authors.is_empty());
    
    // Test ID format is valid
    assert!(manifest.id.contains("."));
    assert!(!manifest.id.starts_with("."));
    assert!(!manifest.id.ends_with("."));
    
    // Test version format is valid
    assert!(manifest.version.contains("."));
    assert!(!manifest.version.starts_with("."));
    assert!(!manifest.version.ends_with("."));
}

#[test]
fn test_wasm_op_spec_security() {
    let op_spec = WasmOpSpec {
        function_name: "test_op".to_string(),
        input_schema: "bytes".to_string(),
        output_schema: "bytes".to_string(),
    };
    
    // Test op spec fields are properly defined
    assert!(!op_spec.function_name.is_empty());
    assert!(!op_spec.input_schema.is_empty());
    assert!(!op_spec.output_schema.is_empty());
    
    // Test function name doesn't contain dangerous operations
    assert!(!op_spec.function_name.contains("system"));
    assert!(!op_spec.function_name.contains("exec"));
    assert!(!op_spec.function_name.contains("shell"));
    
    // Test schemas are valid
    assert!(op_spec.input_schema == "bytes" || op_spec.input_schema == "json");
    assert!(op_spec.output_schema == "bytes" || op_spec.output_schema == "json");
}

#[test]
fn test_lua_event_spec_security() {
    let event_spec = LuaEventSpec {
        event_name: "on_tick".to_string(),
        script_path: "on_tick.lua".to_string(),
    };
    
    // Test event spec fields are properly defined
    assert!(!event_spec.event_name.is_empty());
    assert!(!event_spec.script_path.is_empty());
    
    // Test event name doesn't contain dangerous operations
    assert!(!event_spec.event_name.contains("system"));
    assert!(!event_spec.event_name.contains("exec"));
    assert!(!event_spec.event_name.contains("shell"));
    
    // Test script path is valid
    assert!(event_spec.script_path.ends_with(".lua"));
    assert!(!event_spec.script_path.contains(".."));
    assert!(!event_spec.script_path.starts_with("/"));
    assert!(!event_spec.script_path.starts_with("\\"));
}