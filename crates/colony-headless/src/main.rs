use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use colony_core::{SimClock, TickScale, Colony, Job, Pipeline, Op, QoS, SchedPolicy, CorruptionTunables, FaultKpi, GpuTunables, BlackSwanIndex, Debts, ResearchState, TechTree, GameSetup, WinLossState, SlaTracker, SessionCtl, ReplayLog, ReplayMode};
use colony_io::{IoSimulatorConfig, CanSimConfig, ModbusSimConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let app_state = AppState {
        clock: Arc::new(RwLock::new(SimClock {
            tick_scale: TickScale::RealTime,
            now: chrono::Utc::now(),
        })),
        colony: Arc::new(RwLock::new(Colony {
            power_cap: 1000.0,
            heat_budget: 120.0,
            bandwidth: 1.0,
            corruption_field: 0.0,
            target_uptime_days: 365,
        })),
    };

    let app = Router::new()
        .route("/state/summary", get(get_summary))
        .route("/clock/scale", put(set_scale))
        .route("/job", post(create_job))
        .route("/clock", get(get_clock))
        .route("/scheduler", put(set_scheduler))
        .route("/io/udp/sim", put(set_udp_sim))
        .route("/io/http/sim", put(set_http_sim))
        .route("/pipeline/:id/enqueue", post(enqueue_pipeline))
        .route("/metrics/io", get(get_io_metrics))
        .route("/sched/policy", put(set_scheduler_policy))
        .route("/metrics/faults", get(get_fault_metrics))
        .route("/corruption/tunables", put(set_corruption_tunables))
        .route("/workers/:id/reimage", post(reimage_worker))
        .route("/io/can/sim", put(set_can_sim))
        .route("/io/modbus/sim", put(set_modbus_sim))
        .route("/metrics/gpu", get(get_gpu_metrics))
        .route("/gpu/tunables", put(set_gpu_tunables))
        .route("/gpu/flags", put(set_gpu_flags))
        .route("/events", get(get_events))
        .route("/events/:id/fire", post(fire_event))
        .route("/debts", get(get_debts))
        .route("/research", get(get_research))
        .route("/research/unlock/:tech_id", post(unlock_tech))
        .route("/rituals/:id/start", post(start_ritual))
        .route("/session/start", post(start_session))
        .route("/session/pause", post(pause_session))
        .route("/session/resume", post(resume_session))
        .route("/session/ffwd", put(set_fast_forward))
        .route("/session/status", get(get_session_status))
        .route("/session/autosave", put(set_autosave_interval))
        .route("/save/manual", post(save_manual))
        .route("/load/manual", post(load_manual))
        .route("/replay/start", post(start_replay))
        .route("/replay/stop", post(stop_replay))
        .route("/metrics/summary", get(get_metrics_summary))
        .route("/mods", get(get_mods))
        .route("/mods/reload", post(reload_mod))
        .route("/mods/enable", post(enable_mod))
        .route("/mods/dryrun", post(dryrun_mod))
        .route("/mods/docs", get(get_mod_docs))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Headless server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct AppState {
    clock: Arc<RwLock<SimClock>>,
    colony: Arc<RwLock<Colony>>,
}

#[derive(Serialize)]
struct SummaryResponse {
    clock: SimClock,
    colony: Colony,
    workers: Vec<WorkerStatus>,
    yards: Vec<YardStatus>,
}

#[derive(Serialize)]
struct YardStatus {
    kind: String,
    heat: f32,
    heat_cap: f32,
    throttle: f32,
    power_draw_kw: f32,
}

#[derive(Serialize)]
struct WorkerStatus {
    id: u64,
    state: String,
    skill_cpu: f32,
    corruption: f32,
}

#[derive(Deserialize)]
struct TimeScaleRequest {
    scale: String,
    value: Option<u64>,
}

#[derive(Deserialize)]
struct JobRequest {
    pipeline: Vec<String>,
    qos: String,
    deadline_ms: u64,
    payload_sz: usize,
}

#[derive(Deserialize)]
struct SchedulerRequest {
    scheduler: String,
}

async fn get_summary(State(state): State<AppState>) -> Result<Json<SummaryResponse>, StatusCode> {
    let clock = state.clock.read().await;
    let colony = state.colony.read().await;
    
    // Mock workers for now
    let workers = vec![
        WorkerStatus {
            id: 0,
            state: "Idle".to_string(),
            skill_cpu: 0.8,
            corruption: 0.0,
        },
        WorkerStatus {
            id: 1,
            state: "Idle".to_string(),
            skill_cpu: 0.85,
            corruption: 0.0,
        },
        WorkerStatus {
            id: 2,
            state: "Running".to_string(),
            skill_cpu: 0.9,
            corruption: 0.1,
        },
        WorkerStatus {
            id: 3,
            state: "Idle".to_string(),
            skill_cpu: 0.75,
            corruption: 0.0,
        },
    ];

    // Mock yards for now
    let yards = vec![
        YardStatus {
            kind: "CpuArray".to_string(),
            heat: 45.0,
            heat_cap: 100.0,
            throttle: 1.0,
            power_draw_kw: 200.0,
        },
    ];

    Ok(Json(SummaryResponse {
        clock: clock.clone(),
        colony: colony.clone(),
        workers,
        yards,
    }))
}

async fn set_scale(
    State(state): State<AppState>,
    Json(request): Json<TimeScaleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut clock = state.clock.write().await;
    
    clock.tick_scale = match request.scale.as_str() {
        "realtime" => TickScale::RealTime,
        "seconds" => TickScale::Seconds(request.value.unwrap_or(1)),
        "days" => TickScale::Days(request.value.unwrap_or(1) as u16),
        "years" => TickScale::Years(request.value.unwrap_or(1) as u8),
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    Ok(Json(serde_json::json!({
        "status": "ok",
        "scale": request.scale,
        "value": request.value
    })))
}

async fn create_job(
    State(_state): State<AppState>,
    Json(request): Json<JobRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ops: Result<Vec<Op>, _> = request.pipeline
        .iter()
        .map(|op_str| match op_str.as_str() {
            "Decode" => Ok(Op::Decode),
            "Fft" => Ok(Op::Fft),
            "Kalman" => Ok(Op::Kalman),
            "Yolo" => Ok(Op::Yolo),
            "Crc" => Ok(Op::Crc),
            "CanParse" => Ok(Op::CanParse),
            "UdpDemux" => Ok(Op::UdpDemux),
            "TcpSessionize" => Ok(Op::TcpSessionize),
            "ModbusMap" => Ok(Op::ModbusMap),
            "HttpParse" => Ok(Op::HttpParse),
            _ => Err("Unknown operation"),
        })
        .collect();

    let ops = ops.map_err(|_| StatusCode::BAD_REQUEST)?;

    let qos = match request.qos.as_str() {
        "Throughput" => QoS::Throughput,
        "Latency" => QoS::Latency,
        "Balanced" => QoS::Balanced,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let pipeline = Pipeline {
        ops,
        mutation_tag: None,
    };

    let job = Job {
        id: chrono::Utc::now().timestamp_millis() as u64,
        pipeline,
        qos,
        deadline_ms: request.deadline_ms,
        payload_sz: request.payload_sz,
    };

    Ok(Json(serde_json::json!({
        "status": "created",
        "job_id": job.id,
        "deadline_ms": job.deadline_ms
    })))
}

async fn get_clock(State(state): State<AppState>) -> Result<Json<SimClock>, StatusCode> {
    let clock = state.clock.read().await;
    Ok(Json(clock.clone()))
}

async fn set_scheduler(
    State(_state): State<AppState>,
    Json(request): Json<SchedulerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would update the active scheduler
    // For now, just validate the scheduler name
    let valid_schedulers = ["FCFS", "SJF", "EDF", "HeteroAware"];
    
    if !valid_schedulers.contains(&request.scheduler.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "scheduler": request.scheduler
    })))
}

async fn set_udp_sim(
    State(_state): State<AppState>,
    Json(config): Json<IoSimulatorConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would start/restart the UDP simulator
    Ok(Json(serde_json::json!({
        "status": "ok",
        "config": config
    })))
}

async fn set_http_sim(
    State(_state): State<AppState>,
    Json(config): Json<IoSimulatorConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would start/restart the HTTP simulator
    Ok(Json(serde_json::json!({
        "status": "ok",
        "config": config
    })))
}

async fn enqueue_pipeline(
    State(_state): State<AppState>,
    axum::extract::Path(pipeline_id): axum::extract::Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let payload_sz = request.get("payload_sz")
        .and_then(|v| v.as_u64())
        .unwrap_or(1024) as usize;
    
    // In a real implementation, this would enqueue a job for the specified pipeline
    Ok(Json(serde_json::json!({
        "status": "enqueued",
        "pipeline_id": pipeline_id,
        "payload_sz": payload_sz
    })))
}

async fn get_io_metrics(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock I/O metrics for now
    Ok(Json(serde_json::json!({
        "pps_udp": 100.0,
        "pps_http": 50.0,
        "gbps": 0.5,
        "backlog_queue": 5,
        "deadlines": {
            "udp_telemetry_ingest": 0.95,
            "http_ingest": 0.98
        }
    })))
}

async fn set_scheduler_policy(
    State(_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let policy_str = request.get("policy")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let policy = match policy_str {
        "fcfs" => SchedPolicy::Fcfs,
        "sjf" => SchedPolicy::Sjf,
        "edf" => SchedPolicy::Edf,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    
    // In a real implementation, this would update the active scheduler
    Ok(Json(serde_json::json!({
        "status": "ok",
        "policy": policy_str
    })))
}

async fn get_fault_metrics(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock fault metrics for now
    Ok(Json(serde_json::json!({
        "last_tick_faults": 2,
        "soft_drop_rate": 0.15,
        "sticky_workers": 1,
        "deadline_hit": {
            "udp_telemetry_ingest": 0.95,
            "http_ingest": 0.98
        }
    })))
}

async fn set_corruption_tunables(
    State(_state): State<AppState>,
    Json(tunables): Json<CorruptionTunables>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would update the corruption tunables
    Ok(Json(serde_json::json!({
        "status": "ok",
        "tunables": tunables
    })))
}

async fn reimage_worker(
    State(_state): State<AppState>,
    axum::extract::Path(worker_id): axum::extract::Path<u64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would reset worker corruption and clear sticky faults
    Ok(Json(serde_json::json!({
        "status": "reimaged",
        "worker_id": worker_id
    })))
}

async fn set_can_sim(
    State(_state): State<AppState>,
    Json(config): Json<CanSimConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would start/restart the CAN simulator
    Ok(Json(serde_json::json!({
        "status": "ok",
        "config": config
    })))
}

async fn set_modbus_sim(
    State(_state): State<AppState>,
    Json(config): Json<ModbusSimConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would start/restart the Modbus simulator
    Ok(Json(serde_json::json!({
        "status": "ok",
        "config": config
    })))
}

async fn get_gpu_metrics(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock GPU metrics for now
    Ok(Json(serde_json::json!({
        "util": 0.75,
        "vram_used_gb": 8.5,
        "batches_inflight": 3,
        "batch_latency_ms": 12.3,
        "queues": {
            "can_telemetry": 5,
            "gpu_pipeline_4": 2
        }
    })))
}

async fn set_gpu_tunables(
    State(_state): State<AppState>,
    Json(tunables): Json<GpuTunables>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would update the GPU tunables
    Ok(Json(serde_json::json!({
        "status": "ok",
        "tunables": tunables
    })))
}

async fn set_gpu_flags(
    State(_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mixed_precision = request.get("mixed_precision")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // In a real implementation, this would update the GPU flags
    Ok(Json(serde_json::json!({
        "status": "ok",
        "mixed_precision": mixed_precision
    })))
}

async fn get_events(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock events data for now
    Ok(Json(serde_json::json!({
        "eligible": ["vram_ecc_propagation", "pcie_link_flap"],
        "active": [
            {
                "id": "vram_ecc_propagation",
                "effects": ["DebtPowerMult", "FaultBias"],
                "ttl": 3600000
            }
        ],
        "recent": [
            ["vram_ecc_propagation", 1000],
            ["pcie_link_flap", 500]
        ]
    })))
}

async fn fire_event(
    State(_state): State<AppState>,
    axum::extract::Path(event_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would force-fire a Black Swan event
    Ok(Json(serde_json::json!({
        "status": "fired",
        "event_id": event_id
    })))
}

async fn get_debts(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock debts data for now
    Ok(Json(serde_json::json!({
        "active": [
            {
                "type": "PowerMult",
                "mult": 1.08,
                "until_tick": 2000
            },
            {
                "type": "FaultBias",
                "kind": "StickyConfig",
                "weight_mult": 1.5,
                "until_tick": 1500
            }
        ]
    })))
}

async fn get_research(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock research data for now
    Ok(Json(serde_json::json!({
        "pts": 50,
        "acquired": ["truth_beacon"],
        "available": [
            {
                "id": "dual_run_adjudicator",
                "name": "Dual-Run Adjudicator",
                "cost_pts": 15,
                "requires": ["truth_beacon"]
            }
        ],
        "rituals": [
            {
                "id": "ecc_scrub",
                "name": "ECC Scrub",
                "time_ms": 30000,
                "parts": 1
            }
        ]
    })))
}

async fn unlock_tech(
    State(_state): State<AppState>,
    axum::extract::Path(tech_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would unlock a tech
    Ok(Json(serde_json::json!({
        "status": "unlocked",
        "tech_id": tech_id
    })))
}

async fn start_ritual(
    State(_state): State<AppState>,
    axum::extract::Path(ritual_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would start a ritual
    Ok(Json(serde_json::json!({
        "status": "started",
        "ritual_id": ritual_id,
        "eta_ms": 30000
    })))
}

async fn start_session(
    State(_state): State<AppState>,
    Json(game_setup): Json<GameSetup>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would start a new session
    Ok(Json(serde_json::json!({
        "status": "started",
        "scenario": game_setup.scenario.name,
        "tick_scale": game_setup.tick_scale
    })))
}

async fn pause_session(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would pause the session
    Ok(Json(serde_json::json!({
        "status": "paused"
    })))
}

async fn resume_session(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would resume the session
    Ok(Json(serde_json::json!({
        "status": "resumed"
    })))
}

async fn set_fast_forward(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let on = params.get("on").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);
    
    // In a real implementation, this would set fast forward mode
    Ok(Json(serde_json::json!({
        "status": "ok",
        "fast_forward": on
    })))
}

async fn get_session_status(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock session status
    Ok(Json(serde_json::json!({
        "running": true,
        "fast_forward": false,
        "sim_time": 1000,
        "day_count": 5,
        "sla_pct": 99.2,
        "victory": false,
        "doom": false
    })))
}

async fn set_autosave_interval(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let minutes = params.get("minutes").and_then(|v| v.parse::<u32>().ok()).unwrap_or(5);
    
    // In a real implementation, this would set autosave interval
    Ok(Json(serde_json::json!({
        "status": "ok",
        "autosave_interval_minutes": minutes
    })))
}

async fn save_manual(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let slot = params.get("slot").unwrap_or(&"manual_save".to_string());
    
    // In a real implementation, this would save to the specified slot
    Ok(Json(serde_json::json!({
        "status": "saved",
        "slot": slot
    })))
}

async fn load_manual(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let slot = params.get("slot").unwrap_or(&"manual_save".to_string());
    
    // In a real implementation, this would load from the specified slot
    Ok(Json(serde_json::json!({
        "status": "loaded",
        "slot": slot
    })))
}

async fn start_replay(
    State(_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = request.get("path").and_then(|v| v.as_str()).unwrap_or("replay");
    
    // In a real implementation, this would start replay from the specified path/slot
    Ok(Json(serde_json::json!({
        "status": "replay_started",
        "path": path
    })))
}

async fn stop_replay(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // In a real implementation, this would stop replay
    Ok(Json(serde_json::json!({
        "status": "replay_stopped"
    })))
}

async fn get_metrics_summary(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock comprehensive metrics summary
    Ok(Json(serde_json::json!({
        "sla": {
            "hit_rate": 99.2,
            "achieved_days": 5,
            "target_days": 365
        },
        "resources": {
            "power_draw_kw": 850.0,
            "power_cap_kw": 1000.0,
            "bandwidth_util": 0.65,
            "corruption_field": 0.12
        },
        "heat": {
            "yards": [
                {"heat": 65.0, "cap": 90.0, "throttle": 0.95},
                {"heat": 72.0, "cap": 90.0, "throttle": 0.88}
            ]
        },
        "gpu": {
            "util": 0.78,
            "vram_used_gb": 12.5,
            "vram_total_gb": 16.0,
            "batches_inflight": 3
        },
        "faults": {
            "last_tick_faults": 2,
            "soft_drop_rate": 0.01,
            "sticky_workers": 0
        },
        "black_swans": {
            "active": ["pcie_link_flap"],
            "recent": [["vram_ecc_propagation", 1000]]
        },
        "research": {
            "pts": 25,
            "acquired": ["truth_beacon"],
            "available": ["dual_run_adjudicator"]
        }
    })))
}

async fn get_mods(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock mod list
    Ok(Json(serde_json::json!({
        "mods": [
            {
                "id": "com.example.packetalchemy",
                "name": "Packet Alchemy",
                "version": "1.0.0",
                "authors": ["Example Corp"],
                "description": "Advanced packet processing operations",
                "enabled": true,
                "signature": "valid",
                "entrypoints": {
                    "wasm_ops": ["Op_AdaptiveFft", "Op_Anomaly"],
                    "lua_events": ["on_tick.lua", "on_fault.lua"],
                    "pipelines": "pipelines.toml",
                    "blackswans": "events.toml",
                    "tech": "tech.toml"
                },
                "capabilities": {
                    "sim_time": true,
                    "rng": true,
                    "metrics_read": true,
                    "enqueue_job": true,
                    "log_debug": true
                }
            },
            {
                "id": "com.example.thermalboost",
                "name": "Thermal Boost",
                "version": "0.5.0",
                "authors": ["Thermal Corp"],
                "description": "Enhanced thermal management",
                "enabled": false,
                "signature": "valid",
                "entrypoints": {
                    "wasm_ops": ["Op_ThermalOptimizer"],
                    "lua_events": ["on_heat_spike.lua"],
                    "tech": "tech.toml"
                },
                "capabilities": {
                    "sim_time": true,
                    "metrics_read": true,
                    "log_debug": true
                }
            }
        ]
    })))
}

async fn reload_mod(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mod_id = params.get("id").unwrap_or(&"unknown".to_string());
    
    // In a real implementation, this would trigger hot reload
    Ok(Json(serde_json::json!({
        "status": "reload_started",
        "mod_id": mod_id,
        "transaction_id": "tx_12345"
    })))
}

async fn enable_mod(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mod_id = params.get("id").unwrap_or(&"unknown".to_string());
    let enabled = params.get("on").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);
    
    // In a real implementation, this would enable/disable the mod
    Ok(Json(serde_json::json!({
        "status": "ok",
        "mod_id": mod_id,
        "enabled": enabled
    })))
}

async fn dryrun_mod(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mod_id = params.get("id").unwrap_or(&"unknown".to_string());
    let ticks = params.get("ticks").and_then(|v| v.parse::<u32>().ok()).unwrap_or(120);
    
    // In a real implementation, this would run a dry run simulation
    Ok(Json(serde_json::json!({
        "status": "dryrun_completed",
        "mod_id": mod_id,
        "ticks_simulated": ticks,
        "kpi_deltas": {
            "deadline_hit_rate_change": 0.5,
            "power_draw_change": 2.1,
            "bandwidth_util_change": 0.3,
            "corruption_field_change": 0.01,
            "heat_levels_change": [1.2, 0.8]
        },
        "success": true,
        "warnings": ["Minor bandwidth utilization increase"],
        "errors": []
    })))
}

async fn get_mod_docs(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mod_id = params.get("id").unwrap_or(&"all".to_string());
    
    // In a real implementation, this would return generated API docs
    Ok(Json(serde_json::json!({
        "mod_id": mod_id,
        "sdk_version": "1.0.0",
        "wasm_abi": {
            "version": 1,
            "functions": [
                {
                    "name": "colony_op_init",
                    "signature": "extern \"C\" fn colony_op_init(ctx: *mut OpCtx) -> i32",
                    "description": "Initialize the operation with the given context",
                    "return_codes": [
                        {"code": 0, "meaning": "Success", "description": "Operation initialized successfully"},
                        {"code": -1, "meaning": "Error", "description": "Initialization failed"}
                    ]
                }
            ]
        },
        "lua_api": {
            "global_functions": [
                {
                    "name": "colony.get_sim_time",
                    "signature": "get_sim_time() -> u64",
                    "description": "Get current simulation time in ticks",
                    "requires_capability": "sim_time"
                }
            ]
        }
    })))
}
