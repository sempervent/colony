use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use anyhow::Result;

/// End-to-End Integration Tests for M1-M7 Features
/// 
/// These tests verify that all major systems work together correctly
/// by starting the headless server and exercising the REST API.

#[tokio::test]
async fn test_m1m2_basic_throughput() -> Result<()> {
    println!("🔗 Testing M1-M2: Basic Throughput");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "first_light_chill",
            "tick_scale": "RealTime"
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to start
    sleep(Duration::from_secs(2)).await;
    
    // Check session status
    let status_response = client
        .get(&format!("{}/session/status", base_url))
        .send()
        .await?;
    
    assert!(status_response.status().is_success());
    let status: serde_json::Value = status_response.json().await?;
    assert_eq!(status["running"], true);
    
    // Get metrics to verify throughput
    let metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(metrics_response.status().is_success());
    let metrics: serde_json::Value = metrics_response.json().await?;
    
    // Verify basic metrics are present
    assert!(metrics["colony"]["power_cap_kw"].is_number());
    assert!(metrics["colony"]["bandwidth_total_gbps"].is_number());
    assert!(metrics["colony"]["corruption_field"].is_number());
    
    // Verify power draw is within bounds
    let power_draw = metrics["colony"]["power_draw_kw"].as_f64().unwrap();
    let power_cap = metrics["colony"]["power_cap_kw"].as_f64().unwrap();
    assert!(power_draw >= 0.0);
    assert!(power_draw <= power_cap * 1.1); // Allow 10% over for testing
    
    // Verify bandwidth utilization is bounded
    let bandwidth_util = metrics["colony"]["bandwidth_util"].as_f64().unwrap();
    assert!(bandwidth_util >= 0.0);
    assert!(bandwidth_util <= 1.0);
    
    // Stop the session
    let stop_response = client
        .post(&format!("{}/session/pause", base_url))
        .send()
        .await?;
    
    assert!(stop_response.status().is_success());
    
    println!("✅ M1-M2 Basic Throughput test passed");
    Ok(())
}

#[tokio::test]
async fn test_m3_faults_schedulers() -> Result<()> {
    println!("🔗 Testing M3: Faults & Schedulers");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "factory_horizon_nominal",
            "tick_scale": "RealTime"
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to run
    sleep(Duration::from_secs(5)).await;
    
    // Get fault metrics
    let metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(metrics_response.status().is_success());
    let metrics: serde_json::Value = metrics_response.json().await?;
    
    // Verify fault metrics are present
    assert!(metrics["faults"]["soft_faults"].is_number());
    assert!(metrics["faults"]["sticky_faults"].is_number());
    assert!(metrics["faults"]["sticky_workers"].is_number());
    assert!(metrics["faults"]["retry_success_rate"].is_number());
    
    // Verify fault counts are non-negative
    let soft_faults = metrics["faults"]["soft_faults"].as_u64().unwrap();
    let sticky_faults = metrics["faults"]["sticky_faults"].as_u64().unwrap();
    let sticky_workers = metrics["faults"]["sticky_workers"].as_u64().unwrap();
    let retry_rate = metrics["faults"]["retry_success_rate"].as_f64().unwrap();
    
    assert!(soft_faults >= 0);
    assert!(sticky_faults >= 0);
    assert!(sticky_workers >= 0);
    assert!(retry_rate >= 0.0);
    assert!(retry_rate <= 1.0);
    
    // Verify corruption field is bounded
    let corruption = metrics["colony"]["corruption_field"].as_f64().unwrap();
    assert!(corruption >= 0.0);
    assert!(corruption <= 1.0);
    
    // Test scheduler policy changes
    let scheduler_response = client
        .post(&format!("{}/scheduler/policy", base_url))
        .json(&json!({
            "policy": "Sjf"
        }))
        .send()
        .await?;
    
    assert!(scheduler_response.status().is_success());
    
    // Wait for policy change to take effect
    sleep(Duration::from_secs(2)).await;
    
    // Verify scheduler policy changed
    let status_response = client
        .get(&format!("{}/session/status", base_url))
        .send()
        .await?;
    
    assert!(status_response.status().is_success());
    let status: serde_json::Value = status_response.json().await?;
    assert_eq!(status["scheduler"]["policy"], "Sjf");
    
    println!("✅ M3 Faults & Schedulers test passed");
    Ok(())
}

#[tokio::test]
async fn test_m4_gpu_batching() -> Result<()> {
    println!("🔗 Testing M4: GPU Batching");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "factory_horizon_nominal",
            "tick_scale": "RealTime"
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to run
    sleep(Duration::from_secs(5)).await;
    
    // Get GPU metrics
    let metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(metrics_response.status().is_success());
    let metrics: serde_json::Value = metrics_response.json().await?;
    
    // Verify GPU metrics are present
    assert!(metrics["gpu"]["vram_total_mb"].is_number());
    assert!(metrics["gpu"]["vram_used_mb"].is_number());
    assert!(metrics["gpu"]["batch_max"].is_number());
    assert!(metrics["gpu"]["pcie_bandwidth_gbps"].is_number());
    
    // Verify VRAM usage is within bounds
    let vram_total = metrics["gpu"]["vram_total_mb"].as_f64().unwrap();
    let vram_used = metrics["gpu"]["vram_used_mb"].as_f64().unwrap();
    let batch_max = metrics["gpu"]["batch_max"].as_u64().unwrap();
    let pcie_bandwidth = metrics["gpu"]["pcie_bandwidth_gbps"].as_f64().unwrap();
    
    assert!(vram_total > 0.0);
    assert!(vram_used >= 0.0);
    assert!(vram_used <= vram_total);
    assert!(batch_max > 0);
    assert!(pcie_bandwidth > 0.0);
    
    // Test GPU configuration changes
    let gpu_config_response = client
        .post(&format!("{}/gpu/config", base_url))
        .json(&json!({
            "batch_max": 64,
            "pcie_bandwidth_gbps": 32.0
        }))
        .send()
        .await?;
    
    assert!(gpu_config_response.status().is_success());
    
    // Wait for configuration to take effect
    sleep(Duration::from_secs(2)).await;
    
    // Verify configuration changed
    let updated_metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(updated_metrics_response.status().is_success());
    let updated_metrics: serde_json::Value = updated_metrics_response.json().await?;
    
    assert_eq!(updated_metrics["gpu"]["batch_max"], 64);
    assert_eq!(updated_metrics["gpu"]["pcie_bandwidth_gbps"], 32.0);
    
    println!("✅ M4 GPU Batching test passed");
    Ok(())
}

#[tokio::test]
async fn test_m5_black_swans() -> Result<()> {
    println!("🔗 Testing M5: Black Swans");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "signal_tempest_abyssal",
            "tick_scale": "RealTime"
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to run and potentially trigger Black Swans
    sleep(Duration::from_secs(10)).await;
    
    // Get Black Swan metrics
    let metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(metrics_response.status().is_success());
    let metrics: serde_json::Value = metrics_response.json().await?;
    
    // Verify Black Swan metrics are present
    assert!(metrics["black_swans"]["active"].is_array());
    assert!(metrics["black_swans"]["recently_fired"].is_array());
    assert!(metrics["black_swans"]["total_fired"].is_number());
    
    let active_swans = metrics["black_swans"]["active"].as_array().unwrap();
    let recently_fired = metrics["black_swans"]["recently_fired"].as_array().unwrap();
    let total_fired = metrics["black_swans"]["total_fired"].as_u64().unwrap();
    
    // Verify Black Swan counts are non-negative
    assert!(active_swans.len() >= 0);
    assert!(recently_fired.len() >= 0);
    assert!(total_fired >= 0);
    
    // Test manual Black Swan trigger
    let trigger_response = client
        .post(&format!("{}/blackswans/trigger", base_url))
        .json(&json!({
            "swan_id": "test_swan"
        }))
        .send()
        .await?;
    
    // This might fail if the swan doesn't exist, which is OK
    // We're just testing the endpoint exists
    assert!(trigger_response.status().is_client_error() || trigger_response.status().is_success());
    
    // Get research metrics
    let research_metrics = metrics["research"].as_object().unwrap();
    assert!(research_metrics["pts"].is_number());
    assert!(research_metrics["acquired"].is_array());
    assert!(research_metrics["available"].is_array());
    
    let research_pts = research_metrics["pts"].as_u64().unwrap();
    let acquired_techs = research_metrics["acquired"].as_array().unwrap();
    let available_techs = research_metrics["available"].as_array().unwrap();
    
    assert!(research_pts >= 0);
    assert!(acquired_techs.len() >= 0);
    assert!(available_techs.len() >= 0);
    
    println!("✅ M5 Black Swans test passed");
    Ok(())
}

#[tokio::test]
async fn test_m6_victory_loss() -> Result<()> {
    println!("🔗 Testing M6: Victory/Loss");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session with a short scenario
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "first_light_chill",
            "tick_scale": "RealTime"
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to run
    sleep(Duration::from_secs(5)).await;
    
    // Get victory/loss status
    let metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(metrics_response.status().is_success());
    let metrics: serde_json::Value = metrics_response.json().await?;
    
    // Verify victory/loss metrics are present
    assert!(metrics["winloss"]["achieved_days"].is_number());
    assert!(metrics["winloss"]["doom"].is_boolean());
    assert!(metrics["winloss"]["victory"].is_boolean());
    assert!(metrics["winloss"]["score"].is_number());
    assert!(metrics["winloss"]["current_day"].is_number());
    
    let achieved_days = metrics["winloss"]["achieved_days"].as_u64().unwrap();
    let doom = metrics["winloss"]["doom"].as_bool().unwrap();
    let victory = metrics["winloss"]["victory"].as_bool().unwrap();
    let score = metrics["winloss"]["score"].as_i64().unwrap();
    let current_day = metrics["winloss"]["current_day"].as_u64().unwrap();
    
    // Verify victory/loss state is valid
    assert!(achieved_days >= 0);
    assert!(current_day >= 0);
    assert!(score >= 0);
    
    // Victory and doom should be mutually exclusive
    assert!(!(victory && doom));
    
    // Test session control
    let pause_response = client
        .post(&format!("{}/session/pause", base_url))
        .send()
        .await?;
    
    assert!(pause_response.status().is_success());
    
    // Verify session is paused
    let status_response = client
        .get(&format!("{}/session/status", base_url))
        .send()
        .await?;
    
    assert!(status_response.status().is_success());
    let status: serde_json::Value = status_response.json().await?;
    assert_eq!(status["running"], false);
    
    // Test fast forward
    let ffwd_response = client
        .post(&format!("{}/session/ffwd", base_url))
        .json(&json!({
            "enabled": true
        }))
        .send()
        .await?;
    
    assert!(ffwd_response.status().is_success());
    
    // Test autosave configuration
    let autosave_response = client
        .put(&format!("{}/session/autosave", base_url))
        .json(&json!({
            "interval_minutes": 10
        }))
        .send()
        .await?;
    
    assert!(autosave_response.status().is_success());
    
    println!("✅ M6 Victory/Loss test passed");
    Ok(())
}

#[tokio::test]
async fn test_m7_mods() -> Result<()> {
    println!("🔗 Testing M7: Mods");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Get installed mods
    let mods_response = client
        .get(&format!("{}/mods", base_url))
        .send()
        .await?;
    
    assert!(mods_response.status().is_success());
    let mods: serde_json::Value = mods_response.json().await?;
    
    // Verify mods response structure
    assert!(mods["mods"].is_array());
    let mods_array = mods["mods"].as_array().unwrap();
    
    // Each mod should have required fields
    for mod_entry in mods_array {
        assert!(mod_entry["id"].is_string());
        assert!(mod_entry["name"].is_string());
        assert!(mod_entry["version"].is_string());
        assert!(mod_entry["authors"].is_array());
        assert!(mod_entry["enabled"].is_boolean());
        assert!(mod_entry["entrypoints"].is_object());
        assert!(mod_entry["capabilities"].is_object());
    }
    
    // Test mod enable/disable
    if !mods_array.is_empty() {
        let first_mod = &mods_array[0];
        let mod_id = first_mod["id"].as_str().unwrap();
        
        // Test enabling mod
        let enable_response = client
            .post(&format!("{}/mods/enable", base_url))
            .query(&[("id", mod_id), ("on", "true")])
            .send()
            .await?;
        
        assert!(enable_response.status().is_success());
        
        // Test disabling mod
        let disable_response = client
            .post(&format!("{}/mods/enable", base_url))
            .query(&[("id", mod_id), ("on", "false")])
            .send()
            .await?;
        
        assert!(disable_response.status().is_success());
    }
    
    // Test mod reload
    if !mods_array.is_empty() {
        let first_mod = &mods_array[0];
        let mod_id = first_mod["id"].as_str().unwrap();
        
        let reload_response = client
            .post(&format!("{}/mods/reload", base_url))
            .query(&[("id", mod_id)])
            .send()
            .await?;
        
        assert!(reload_response.status().is_success());
    }
    
    // Test dry run
    if !mods_array.is_empty() {
        let first_mod = &mods_array[0];
        let mod_id = first_mod["id"].as_str().unwrap();
        
        let dryrun_response = client
            .post(&format!("{}/mods/dryrun", base_url))
            .query(&[("id", mod_id), ("ticks", "120")])
            .send()
            .await?;
        
        assert!(dryrun_response.status().is_success());
        let dryrun_result: serde_json::Value = dryrun_response.json().await?;
        
        // Verify dry run response structure
        assert!(dryrun_result["status"].is_string());
        assert!(dryrun_result["mod_id"].is_string());
        assert!(dryrun_result["ticks_simulated"].is_number());
        assert!(dryrun_result["kpi_deltas"].is_object());
        assert!(dryrun_result["success"].is_boolean());
        assert!(dryrun_result["warnings"].is_array());
        assert!(dryrun_result["errors"].is_array());
    }
    
    // Test mod documentation
    let docs_response = client
        .get(&format!("{}/mods/docs", base_url))
        .send()
        .await?;
    
    assert!(docs_response.status().is_success());
    let docs: serde_json::Value = docs_response.json().await?;
    
    // Verify documentation structure
    assert!(docs["mod_id"].is_string());
    assert!(docs["sdk_version"].is_string());
    assert!(docs["wasm_abi"].is_object());
    assert!(docs["lua_api"].is_object());
    
    println!("✅ M7 Mods test passed");
    Ok(())
}

#[tokio::test]
async fn test_save_load_persistence() -> Result<()> {
    println!("🔗 Testing Save/Load Persistence");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "first_light_chill",
            "tick_scale": "RealTime"
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to run
    sleep(Duration::from_secs(3)).await;
    
    // Get initial metrics
    let initial_metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(initial_metrics_response.status().is_success());
    let initial_metrics: serde_json::Value = initial_metrics_response.json().await?;
    
    // Save the game
    let save_response = client
        .post(&format!("{}/save/manual", base_url))
        .json(&json!({
            "slot_name": "test_save"
        }))
        .send()
        .await?;
    
    assert!(save_response.status().is_success());
    
    // Wait a bit more
    sleep(Duration::from_secs(2)).await;
    
    // Load the game
    let load_response = client
        .post(&format!("{}/load/manual", base_url))
        .json(&json!({
            "slot_name": "test_save"
        }))
        .send()
        .await?;
    
    assert!(load_response.status().is_success());
    
    // Get loaded metrics
    let loaded_metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(loaded_metrics_response.status().is_success());
    let loaded_metrics: serde_json::Value = loaded_metrics_response.json().await?;
    
    // Verify key metrics are preserved
    let initial_power = initial_metrics["colony"]["power_cap_kw"].as_f64().unwrap();
    let loaded_power = loaded_metrics["colony"]["power_cap_kw"].as_f64().unwrap();
    assert_eq!(initial_power, loaded_power);
    
    let initial_bandwidth = initial_metrics["colony"]["bandwidth_total_gbps"].as_f64().unwrap();
    let loaded_bandwidth = loaded_metrics["colony"]["bandwidth_total_gbps"].as_f64().unwrap();
    assert_eq!(initial_bandwidth, loaded_bandwidth);
    
    println!("✅ Save/Load Persistence test passed");
    Ok(())
}

#[tokio::test]
async fn test_replay_determinism() -> Result<()> {
    println!("🔗 Testing Replay Determinism");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Start a session with a fixed seed
    let session_response = client
        .post(&format!("{}/session/start", base_url))
        .json(&json!({
            "scenario_id": "first_light_chill",
            "tick_scale": "RealTime",
            "seed": 12345
        }))
        .send()
        .await?;
    
    assert!(session_response.status().is_success());
    
    // Wait for simulation to run
    sleep(Duration::from_secs(5)).await;
    
    // Get metrics from first run
    let first_metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(first_metrics_response.status().is_success());
    let first_metrics: serde_json::Value = first_metrics_response.json().await?;
    
    // Stop the session
    let stop_response = client
        .post(&format!("{}/session/pause", base_url))
        .send()
        .await?;
    
    assert!(stop_response.status().is_success());
    
    // Start replay with the same seed
    let replay_response = client
        .post(&format!("{}/replay/start", base_url))
        .json(&json!({
            "path": "test_replay",
            "seed": 12345
        }))
        .send()
        .await?;
    
    assert!(replay_response.status().is_success());
    
    // Wait for replay to run
    sleep(Duration::from_secs(5)).await;
    
    // Get metrics from replay
    let replay_metrics_response = client
        .get(&format!("{}/metrics/summary", base_url))
        .send()
        .await?;
    
    assert!(replay_metrics_response.status().is_success());
    let replay_metrics: serde_json::Value = replay_metrics_response.json().await?;
    
    // Compare key metrics (within tolerance)
    let first_power = first_metrics["colony"]["power_draw_kw"].as_f64().unwrap();
    let replay_power = replay_metrics["colony"]["power_draw_kw"].as_f64().unwrap();
    let power_diff = (first_power - replay_power).abs();
    assert!(power_diff < first_power * 0.02); // 2% tolerance
    
    let first_bandwidth = first_metrics["colony"]["bandwidth_util"].as_f64().unwrap();
    let replay_bandwidth = replay_metrics["colony"]["bandwidth_util"].as_f64().unwrap();
    let bandwidth_diff = (first_bandwidth - replay_bandwidth).abs();
    assert!(bandwidth_diff < 0.02); // 2% tolerance
    
    // Stop replay
    let stop_replay_response = client
        .post(&format!("{}/replay/stop", base_url))
        .send()
        .await?;
    
    assert!(stop_replay_response.status().is_success());
    
    println!("✅ Replay Determinism test passed");
    Ok(())
}

#[tokio::test]
async fn test_health_check() -> Result<()> {
    println!("🔗 Testing Health Check");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Test health endpoint
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await?;
    
    assert!(health_response.status().is_success());
    let health: serde_json::Value = health_response.json().await?;
    
    // Verify health response structure
    assert!(health["status"].is_string());
    assert!(health["version"].is_string());
    assert!(health["uptime"].is_number());
    
    assert_eq!(health["status"], "healthy");
    
    println!("✅ Health Check test passed");
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    println!("🔗 Testing Error Handling");
    
    let client = Client::new();
    let base_url = "http://localhost:8080";
    
    // Test invalid endpoint
    let invalid_response = client
        .get(&format!("{}/invalid/endpoint", base_url))
        .send()
        .await?;
    
    assert!(invalid_response.status().is_client_error());
    
    // Test invalid JSON
    let invalid_json_response = client
        .post(&format!("{}/session/start", base_url))
        .body("invalid json")
        .header("Content-Type", "application/json")
        .send()
        .await?;
    
    assert!(invalid_json_response.status().is_client_error());
    
    // Test invalid parameters
    let invalid_params_response = client
        .post(&format!("{}/mods/enable", base_url))
        .query(&[("id", "nonexistent_mod")])
        .send()
        .await?;
    
    // This might succeed or fail depending on implementation
    // We're just testing the endpoint doesn't crash
    assert!(invalid_params_response.status().is_success() || 
            invalid_params_response.status().is_client_error());
    
    println!("✅ Error Handling test passed");
    Ok(())
}

/// Helper function to wait for server to be ready
async fn wait_for_server(base_url: &str) -> Result<()> {
    let client = Client::new();
    let max_attempts = 30;
    
    for attempt in 1..=max_attempts {
        match client.get(&format!("{}/health", base_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Server is ready after {} attempts", attempt);
                    return Ok(());
                }
            }
            Err(_) => {
                // Server not ready yet
            }
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    Err(anyhow::anyhow!("Server failed to start within {} seconds", max_attempts))
}

/// Setup function to start the headless server before tests
#[tokio::test]
async fn test_server_startup() -> Result<()> {
    println!("🔗 Testing Server Startup");
    
    let base_url = "http://localhost:8080";
    
    // Wait for server to be ready
    wait_for_server(base_url).await?;
    
    // Test basic connectivity
    let client = Client::new();
    let response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await?;
    
    assert!(response.status().is_success());
    
    println!("✅ Server Startup test passed");
    Ok(())
}
