use colony_core::*;
use bevy::prelude::*;
use std::collections::HashMap;
use anyhow::Result;

#[test]
fn test_save_file_v1_serialization() {
    let mut app = create_test_app();
    let game_setup = GameSetup::new(Scenario::default());
    let colony = app.world.resource::<Colony>();
    let research_state = app.world.resource::<ResearchState>();
    let black_swan_index = app.world.resource::<BlackSwanIndex>();
    let debts = app.world.resource::<Debts>();
    let winloss = app.world.resource::<WinLossState>();
    let session_ctl = app.world.resource::<SessionCtl>();
    let replay_log = app.world.resource::<ReplayLog>();
    let kpi_summary = KpiSummary::default();

    let save_data = SaveFileV1::new(
        game_setup,
        colony,
        research_state,
        black_swan_index,
        debts,
        winloss,
        session_ctl,
        replay_log,
        kpi_summary,
    );

    // Test serialization
    let encoded = bincode::serialize(&save_data);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: SaveFileV1 = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.colony_state.power_cap_kw, colony.power_cap_kw);
    assert_eq!(decoded.colony_state.bandwidth_total_gbps, colony.bandwidth_total_gbps);
    assert_eq!(decoded.colony_state.corruption_field, colony.corruption_field);
}

#[test]
fn test_save_file_v1_migration() {
    // Test migration from hypothetical V0 to V1
    let v0_data = create_mock_v0_save_data();
    let v1_data = migrate_v0_to_v1(v0_data);
    
    assert_eq!(v1_data.version, 1);
    assert!(v1_data.colony_state.power_cap_kw > 0.0);
    assert!(v1_data.colony_state.bandwidth_total_gbps > 0.0);
    assert!(v1_data.colony_state.corruption_field >= 0.0);
    assert!(v1_data.colony_state.corruption_field <= 1.0);
}

#[test]
fn test_save_load_cycle() {
    let mut app1 = create_test_app();
    
    // Run simulation for a bit
    for _ in 0..100 {
        app1.update();
    }
    
    // Save state
    let game_setup = app1.world.resource::<Colony>().game_setup.clone();
    let colony = app1.world.resource::<Colony>().clone();
    let research_state = app1.world.resource::<ResearchState>().clone();
    let black_swan_index = app1.world.resource::<BlackSwanIndex>().clone();
    let debts = app1.world.resource::<Debts>().clone();
    let winloss = app1.world.resource::<WinLossState>().clone();
    let session_ctl = app1.world.resource::<SessionCtl>().clone();
    let replay_log = app1.world.resource::<ReplayLog>().clone();
    let kpi_summary = KpiSummary::default();
    
    let save_data = SaveFileV1::new(
        game_setup,
        &colony,
        &research_state,
        &black_swan_index,
        &debts,
        &winloss,
        &session_ctl,
        &replay_log,
        kpi_summary,
    );
    
    // Serialize
    let encoded = bincode::serialize(&save_data).unwrap();
    
    // Deserialize
    let decoded: SaveFileV1 = bincode::deserialize(&encoded).unwrap();
    
    // Create new app and load state
    let mut app2 = create_test_app();
    load_save_data(&mut app2, &decoded);
    
    // Verify state is preserved
    let colony2 = app2.world.resource::<Colony>();
    assert_eq!(colony.power_cap_kw, colony2.power_cap_kw);
    assert_eq!(colony.bandwidth_total_gbps, colony2.bandwidth_total_gbps);
    assert_eq!(colony.corruption_field, colony2.corruption_field);
    assert_eq!(colony.seed, colony2.seed);
}

#[test]
fn test_kpi_summary_serialization() {
    let mut kpi_summary = KpiSummary::default();
    kpi_summary.bandwidth_util_history = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    kpi_summary.power_draw_history = vec![100.0, 200.0, 300.0, 400.0, 500.0];
    kpi_summary.corruption_history = vec![0.01, 0.02, 0.03, 0.04, 0.05];
    kpi_summary.fault_rate_history = vec![0.001, 0.002, 0.003, 0.004, 0.005];
    kpi_summary.black_swan_events = vec![
        "swan1".to_string(),
        "swan2".to_string(),
        "swan3".to_string(),
    ];
    kpi_summary.research_completed = vec![
        "tech1".to_string(),
        "tech2".to_string(),
    ];
    
    // Test serialization
    let encoded = bincode::serialize(&kpi_summary);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: KpiSummary = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(decoded.bandwidth_util_history, kpi_summary.bandwidth_util_history);
    assert_eq!(decoded.power_draw_history, kpi_summary.power_draw_history);
    assert_eq!(decoded.corruption_history, kpi_summary.corruption_history);
    assert_eq!(decoded.fault_rate_history, kpi_summary.fault_rate_history);
    assert_eq!(decoded.black_swan_events, kpi_summary.black_swan_events);
    assert_eq!(decoded.research_completed, kpi_summary.research_completed);
}

#[test]
fn test_replay_log_serialization() {
    let mut replay_log = ReplayLog::new();
    replay_log.start_recording();
    
    // Add various events
    replay_log.record_event(ReplayEvent::Tick { n: 1 });
    replay_log.record_event(ReplayEvent::EnqueueJob {
        pipeline_id: "Decode".to_string(),
        payload: 1024,
    });
    replay_log.record_event(ReplayEvent::FaultInjected {
        fault_kind: FaultKind::Transient,
        worker_id: 0,
        job_id: 1,
    });
    replay_log.record_event(ReplayEvent::BlackSwanFired {
        swan_id: "test_swan".to_string(),
    });
    replay_log.record_event(ReplayEvent::ResearchCompleted {
        tech_id: "test_tech".to_string(),
    });
    replay_log.record_event(ReplayEvent::VictoryAchieved {
        reason: "test_victory".to_string(),
    });
    
    // Test serialization
    let encoded = bincode::serialize(&replay_log);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: ReplayLog = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(replay_log.events.len(), decoded.events.len());
    
    // Verify events are preserved
    for (original, deserialized) in replay_log.events.iter().zip(decoded.events.iter()) {
        match (original, deserialized) {
            (ReplayEvent::Tick { n: n1 }, ReplayEvent::Tick { n: n2 }) => {
                assert_eq!(n1, n2);
            }
            (ReplayEvent::EnqueueJob { pipeline_id: id1, payload: p1 }, 
             ReplayEvent::EnqueueJob { pipeline_id: id2, payload: p2 }) => {
                assert_eq!(id1, id2);
                assert_eq!(p1, p2);
            }
            (ReplayEvent::FaultInjected { fault_kind: k1, worker_id: w1, job_id: j1 },
             ReplayEvent::FaultInjected { fault_kind: k2, worker_id: w2, job_id: j2 }) => {
                assert_eq!(k1, k2);
                assert_eq!(w1, w2);
                assert_eq!(j1, j2);
            }
            (ReplayEvent::BlackSwanFired { swan_id: s1 },
             ReplayEvent::BlackSwanFired { swan_id: s2 }) => {
                assert_eq!(s1, s2);
            }
            (ReplayEvent::ResearchCompleted { tech_id: t1 },
             ReplayEvent::ResearchCompleted { tech_id: t2 }) => {
                assert_eq!(t1, t2);
            }
            (ReplayEvent::VictoryAchieved { reason: r1 },
             ReplayEvent::VictoryAchieved { reason: r2 }) => {
                assert_eq!(r1, r2);
            }
            _ => panic!("Event type mismatch"),
        }
    }
}

#[test]
fn test_colony_state_serialization() {
    let colony = Colony {
        power_cap_kw: 1000.0,
        bandwidth_total_gbps: 32.0,
        corruption_field: 0.1,
        target_uptime_days: 365,
        meters: GlobalMeters {
            power_draw_kw: 800.0,
            bandwidth_util: 0.5,
            power_deficit_ticks: 10,
            ..Default::default()
        },
        tunables: ResourceTunables::default(),
        corruption_tun: CorruptionTunables::default(),
        seed: 42,
        game_setup: GameSetup::new(Scenario::default()),
        pending_mutations: vec![("pipeline1".to_string(), "mutation1".to_string())],
    };
    
    // Test serialization
    let encoded = bincode::serialize(&colony);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: Colony = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(colony.power_cap_kw, decoded.power_cap_kw);
    assert_eq!(colony.bandwidth_total_gbps, decoded.bandwidth_total_gbps);
    assert_eq!(colony.corruption_field, decoded.corruption_field);
    assert_eq!(colony.target_uptime_days, decoded.target_uptime_days);
    assert_eq!(colony.meters.power_draw_kw, decoded.meters.power_draw_kw);
    assert_eq!(colony.meters.bandwidth_util, decoded.meters.bandwidth_util);
    assert_eq!(colony.meters.power_deficit_ticks, decoded.meters.power_deficit_ticks);
    assert_eq!(colony.seed, decoded.seed);
    assert_eq!(colony.pending_mutations, decoded.pending_mutations);
}

#[test]
fn test_research_state_serialization() {
    let mut research_state = ResearchState::new();
    research_state.add_points(1000);
    research_state.acquired_techs.insert("tech1".to_string());
    research_state.acquired_techs.insert("tech2".to_string());
    
    // Test serialization
    let encoded = bincode::serialize(&research_state);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: ResearchState = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(research_state.points, decoded.points);
    assert_eq!(research_state.acquired_techs, decoded.acquired_techs);
}

#[test]
fn test_black_swan_index_serialization() {
    let mut black_swan_index = BlackSwanIndex::new();
    black_swan_index.add_potential_swan("swan1".to_string(), 0.1);
    black_swan_index.add_potential_swan("swan2".to_string(), 0.2);
    black_swan_index.fired_swans.insert("swan1".to_string());
    
    // Test serialization
    let encoded = bincode::serialize(&black_swan_index);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: BlackSwanIndex = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(black_swan_index.potential_swans, decoded.potential_swans);
    assert_eq!(black_swan_index.fired_swans, decoded.fired_swans);
}

#[test]
fn test_debts_serialization() {
    let mut debts = Debts::new();
    debts.add_debt("power_spike".to_string(), 100);
    debts.add_debt("bandwidth_surge".to_string(), 200);
    
    // Test serialization
    let encoded = bincode::serialize(&debts);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: Debts = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(debts.active, decoded.active);
}

#[test]
fn test_win_loss_state_serialization() {
    let mut win_loss_state = WinLossState::new();
    win_loss_state.victory = true;
    win_loss_state.doom = false;
    win_loss_state.score = 10000;
    win_loss_state.achieved_days = 30;
    
    // Test serialization
    let encoded = bincode::serialize(&win_loss_state);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: WinLossState = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(win_loss_state.victory, decoded.victory);
    assert_eq!(win_loss_state.doom, decoded.doom);
    assert_eq!(win_loss_state.score, decoded.score);
    assert_eq!(win_loss_state.achieved_days, decoded.achieved_days);
}

#[test]
fn test_session_ctl_serialization() {
    let mut session_ctl = SessionCtl::new();
    session_ctl.running = true;
    session_ctl.fast_forward = false;
    session_ctl.current_tick = 1000;
    
    // Test serialization
    let encoded = bincode::serialize(&session_ctl);
    assert!(encoded.is_ok());
    
    // Test deserialization
    let decoded: SessionCtl = bincode::deserialize(&encoded.unwrap()).unwrap();
    assert_eq!(session_ctl.running, decoded.running);
    assert_eq!(session_ctl.fast_forward, decoded.fast_forward);
    assert_eq!(session_ctl.current_tick, decoded.current_tick);
}

// Helper functions

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Colony::new());
    app.insert_resource(JobQueue::new());
    app.insert_resource(CorruptionField::new());
    app.insert_resource(FaultKpi::new());
    app.insert_resource(ActiveScheduler { policy: SchedPolicy::Sjf });
    app.insert_resource(Debts::new());
    app.insert_resource(BlackSwanIndex::new());
    app.insert_resource(KpiRingBuffer::new());
    app.insert_resource(ResearchState::new());
    app.insert_resource(SessionCtl::new());
    app.insert_resource(ReplayLog::new());
    app.insert_resource(WinLossState::new());
    app.insert_resource(SlaTracker::new(7, 86400000 / 16));
    app.insert_resource(SimClock {
        tick_scale: TickScale::RealTime,
        now: chrono::Utc::now(),
    });
    app
}

fn create_mock_v0_save_data() -> MockV0SaveData {
    MockV0SaveData {
        version: 0,
        power_cap_kw: 1000.0,
        bandwidth_total_gbps: 32.0,
        corruption_field: 0.1,
        seed: 42,
    }
}

fn migrate_v0_to_v1(v0_data: MockV0SaveData) -> SaveFileV1 {
    let colony = Colony {
        power_cap_kw: v0_data.power_cap_kw,
        bandwidth_total_gbps: v0_data.bandwidth_total_gbps,
        corruption_field: v0_data.corruption_field,
        target_uptime_days: 365,
        meters: GlobalMeters::new(),
        tunables: ResourceTunables::default(),
        corruption_tun: CorruptionTunables::default(),
        seed: v0_data.seed,
        game_setup: GameSetup::new(Scenario::default()),
        pending_mutations: Vec::new(),
    };
    
    SaveFileV1::new(
        GameSetup::new(Scenario::default()),
        &colony,
        &ResearchState::new(),
        &BlackSwanIndex::new(),
        &Debts::new(),
        &WinLossState::new(),
        &SessionCtl::new(),
        &ReplayLog::new(),
        KpiSummary::default(),
    )
}

fn load_save_data(app: &mut App, save_data: &SaveFileV1) {
    *app.world.resource_mut::<Colony>() = save_data.colony_state.clone();
    *app.world.resource_mut::<ResearchState>() = save_data.research_state.clone();
    *app.world.resource_mut::<BlackSwanIndex>() = save_data.black_swan_index.clone();
    *app.world.resource_mut::<Debts>() = save_data.debts.clone();
    *app.world.resource_mut::<WinLossState>() = save_data.win_loss_state.clone();
    *app.world.resource_mut::<SessionCtl>() = save_data.session_ctl.clone();
    *app.world.resource_mut::<ReplayLog>() = save_data.replay_log.clone();
}

// Mock V0 save data structure
#[derive(Debug, Clone)]
struct MockV0SaveData {
    version: u32,
    power_cap_kw: f32,
    bandwidth_total_gbps: f32,
    corruption_field: f32,
    seed: u64,
}
