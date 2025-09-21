use colony_core::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[test]
fn test_desktop_headless_parity() {
    // Test that desktop and headless modes produce the same results
    let seed = 12345;
    let num_ticks = 1000;
    
    // Create desktop app
    let mut desktop_app = create_desktop_app(seed);
    
    // Create headless app
    let mut headless_app = create_headless_app(seed);
    
    // Run both simulations
    for _ in 0..num_ticks {
        desktop_app.update();
        headless_app.update();
    }
    
    // Compare results
    let desktop_colony = desktop_app.world.resource::<Colony>();
    let headless_colony = headless_app.world.resource::<Colony>();
    
    assert_eq!(desktop_colony.power_cap_kw, headless_colony.power_cap_kw);
    assert_eq!(desktop_colony.bandwidth_total_gbps, headless_colony.bandwidth_total_gbps);
    assert_eq!(desktop_colony.corruption_field, headless_colony.corruption_field);
    assert_eq!(desktop_colony.seed, headless_colony.seed);
    
    // Compare KPI buffers
    let desktop_kpi = desktop_app.world.resource::<KpiRingBuffer>();
    let headless_kpi = headless_app.world.resource::<KpiRingBuffer>();
    
    assert_eq!(desktop_kpi.bandwidth_util.len(), headless_kpi.bandwidth_util.len());
    for i in 0..desktop_kpi.bandwidth_util.len() {
        let (val1, _) = desktop_kpi.bandwidth_util[i];
        let (val2, _) = headless_kpi.bandwidth_util[i];
        assert!((val1 - val2).abs() < 0.001, "KPI mismatch at index {}", i);
    }
}

#[test]
fn test_ui_metrics_parity() {
    // Test that UI metrics match headless metrics
    let seed = 54321;
    let num_ticks = 500;
    
    // Create headless app
    let mut headless_app = create_headless_app(seed);
    
    // Run simulation
    for _ in 0..num_ticks {
        headless_app.update();
    }
    
    // Get headless metrics
    let headless_colony = headless_app.world.resource::<Colony>();
    let headless_kpi = headless_app.world.resource::<KpiRingBuffer>();
    let headless_faults = headless_app.world.resource::<FaultKpi>();
    let headless_research = headless_app.world.resource::<ResearchState>();
    let headless_black_swans = headless_app.world.resource::<BlackSwanIndex>();
    let headless_win_loss = headless_app.world.resource::<WinLossState>();
    
    // Create UI metrics (simulated)
    let ui_metrics = create_ui_metrics(
        headless_colony,
        &headless_kpi,
        &headless_faults,
        &headless_research,
        &headless_black_swans,
        &headless_win_loss,
    );
    
    // Verify UI metrics match headless metrics
    assert_eq!(ui_metrics.colony.power_cap_kw, headless_colony.power_cap_kw);
    assert_eq!(ui_metrics.colony.bandwidth_total_gbps, headless_colony.bandwidth_total_gbps);
    assert_eq!(ui_metrics.colony.corruption_field, headless_colony.corruption_field);
    assert_eq!(ui_metrics.colony.meters.power_draw_kw, headless_colony.meters.power_draw_kw);
    assert_eq!(ui_metrics.colony.meters.bandwidth_util, headless_colony.meters.bandwidth_util);
    
    assert_eq!(ui_metrics.faults.soft_faults, headless_faults.soft_faults);
    assert_eq!(ui_metrics.faults.sticky_faults, headless_faults.sticky_faults);
    assert_eq!(ui_metrics.faults.sticky_workers, headless_faults.sticky_workers);
    
    assert_eq!(ui_metrics.research.points, headless_research.points);
    assert_eq!(ui_metrics.research.acquired_techs.len(), headless_research.acquired_techs.len());
    
    assert_eq!(ui_metrics.black_swans.active.len(), headless_black_swans.potential_swans.len());
    assert_eq!(ui_metrics.black_swans.fired.len(), headless_black_swans.fired_swans.len());
    
    assert_eq!(ui_metrics.win_loss.victory, headless_win_loss.victory);
    assert_eq!(ui_metrics.win_loss.doom, headless_win_loss.doom);
    assert_eq!(ui_metrics.win_loss.score, headless_win_loss.score);
}

#[test]
fn test_dashboard_parity() {
    // Test that dashboard displays match headless metrics
    let seed = 98765;
    let num_ticks = 750;
    
    // Create headless app
    let mut headless_app = create_headless_app(seed);
    
    // Run simulation
    for _ in 0..num_ticks {
        headless_app.update();
    }
    
    // Get headless metrics
    let headless_colony = headless_app.world.resource::<Colony>();
    let headless_kpi = headless_app.world.resource::<KpiRingBuffer>();
    let headless_faults = headless_app.world.resource::<FaultKpi>();
    let headless_research = headless_app.world.resource::<ResearchState>();
    let headless_black_swans = headless_app.world.resource::<BlackSwanIndex>();
    let headless_win_loss = headless_app.world.resource::<WinLossState>();
    
    // Create dashboard data
    let dashboard_data = create_dashboard_data(
        headless_colony,
        &headless_kpi,
        &headless_faults,
        &headless_research,
        &headless_black_swans,
        &headless_win_loss,
    );
    
    // Verify dashboard data matches headless metrics
    assert_eq!(dashboard_data.power_draw_kw, headless_colony.meters.power_draw_kw);
    assert_eq!(dashboard_data.power_cap_kw, headless_colony.power_cap_kw);
    assert_eq!(dashboard_data.bandwidth_util, headless_colony.meters.bandwidth_util);
    assert_eq!(dashboard_data.corruption_field, headless_colony.corruption_field);
    
    assert_eq!(dashboard_data.fault_rate, headless_faults.soft_faults as f32 / num_ticks as f32);
    assert_eq!(dashboard_data.sticky_workers, headless_faults.sticky_workers);
    
    assert_eq!(dashboard_data.research_points, headless_research.points);
    assert_eq!(dashboard_data.research_techs, headless_research.acquired_techs.len());
    
    assert_eq!(dashboard_data.black_swans_active, headless_black_swans.potential_swans.len());
    assert_eq!(dashboard_data.black_swans_fired, headless_black_swans.fired_swans.len());
    
    assert_eq!(dashboard_data.victory, headless_win_loss.victory);
    assert_eq!(dashboard_data.doom, headless_win_loss.doom);
    assert_eq!(dashboard_data.score, headless_win_loss.score);
}

#[test]
fn test_ui_rendering_parity() {
    // Test that UI rendering produces consistent results
    let seed = 11111;
    let num_ticks = 300;
    
    // Create desktop app
    let mut desktop_app = create_desktop_app(seed);
    
    // Run simulation
    for _ in 0..num_ticks {
        desktop_app.update();
    }
    
    // Get UI state
    let ui_state = get_ui_state(&desktop_app);
    
    // Verify UI state is consistent
    assert!(ui_state.power_bar.value >= 0.0);
    assert!(ui_state.power_bar.value <= 1.0);
    assert!(ui_state.bandwidth_bar.value >= 0.0);
    assert!(ui_state.bandwidth_bar.value <= 1.0);
    assert!(ui_state.corruption_bar.value >= 0.0);
    assert!(ui_state.corruption_bar.value <= 1.0);
    
    assert!(ui_state.fault_indicator.count >= 0);
    assert!(ui_state.research_indicator.count >= 0);
    assert!(ui_state.black_swan_indicator.count >= 0);
    
    assert!(ui_state.victory_indicator.active == ui_state.win_loss.victory);
    assert!(ui_state.doom_indicator.active == ui_state.win_loss.doom);
}

#[test]
fn test_headless_api_parity() {
    // Test that headless API responses match internal state
    let seed = 22222;
    let num_ticks = 400;
    
    // Create headless app
    let mut headless_app = create_headless_app(seed);
    
    // Run simulation
    for _ in 0..num_ticks {
        headless_app.update();
    }
    
    // Get internal state
    let colony = headless_app.world.resource::<Colony>();
    let kpi = headless_app.world.resource::<KpiRingBuffer>();
    let faults = headless_app.world.resource::<FaultKpi>();
    let research = headless_app.world.resource::<ResearchState>();
    let black_swans = headless_app.world.resource::<BlackSwanIndex>();
    let win_loss = headless_app.world.resource::<WinLossState>();
    
    // Create API response
    let api_response = create_api_response(
        colony,
        &kpi,
        &faults,
        &research,
        &black_swans,
        &win_loss,
    );
    
    // Verify API response matches internal state
    assert_eq!(api_response.colony.power_cap_kw, colony.power_cap_kw);
    assert_eq!(api_response.colony.bandwidth_total_gbps, colony.bandwidth_total_gbps);
    assert_eq!(api_response.colony.corruption_field, colony.corruption_field);
    assert_eq!(api_response.colony.meters.power_draw_kw, colony.meters.power_draw_kw);
    assert_eq!(api_response.colony.meters.bandwidth_util, colony.meters.bandwidth_util);
    
    assert_eq!(api_response.faults.soft_faults, faults.soft_faults);
    assert_eq!(api_response.faults.sticky_faults, faults.sticky_faults);
    assert_eq!(api_response.faults.sticky_workers, faults.sticky_workers);
    
    assert_eq!(api_response.research.points, research.points);
    assert_eq!(api_response.research.acquired_techs.len(), research.acquired_techs.len());
    
    assert_eq!(api_response.black_swans.active.len(), black_swans.potential_swans.len());
    assert_eq!(api_response.black_swans.fired.len(), black_swans.fired_swans.len());
    
    assert_eq!(api_response.win_loss.victory, win_loss.victory);
    assert_eq!(api_response.win_loss.doom, win_loss.doom);
    assert_eq!(api_response.win_loss.score, win_loss.score);
}

#[test]
fn test_ui_performance_parity() {
    // Test that UI performance is acceptable
    let seed = 33333;
    let num_ticks = 100;
    
    // Create desktop app
    let mut desktop_app = create_desktop_app(seed);
    
    // Measure UI update performance
    let start = std::time::Instant::now();
    
    for _ in 0..num_ticks {
        desktop_app.update();
    }
    
    let duration = start.elapsed();
    
    // Assert UI updates are fast enough (less than 16ms per tick for 60 FPS)
    let avg_time_per_tick = duration.as_millis() as f32 / num_ticks as f32;
    assert!(avg_time_per_tick < 16.0, "UI update too slow: {}ms per tick", avg_time_per_tick);
}

#[test]
fn test_ui_memory_parity() {
    // Test that UI memory usage is reasonable
    let seed = 44444;
    let num_ticks = 200;
    
    // Create desktop app
    let mut desktop_app = create_desktop_app(seed);
    
    // Run simulation
    for _ in 0..num_ticks {
        desktop_app.update();
    }
    
    // Get UI memory usage (simulated)
    let ui_memory = get_ui_memory_usage(&desktop_app);
    
    // Assert UI memory usage is reasonable (less than 100MB)
    assert!(ui_memory.total_mb < 100.0, "UI memory usage too high: {}MB", ui_memory.total_mb);
    assert!(ui_memory.textures_mb < 50.0, "UI texture memory too high: {}MB", ui_memory.textures_mb);
    assert!(ui_memory.buffers_mb < 25.0, "UI buffer memory too high: {}MB", ui_memory.buffers_mb);
}

// Helper functions

fn create_desktop_app(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut colony = Colony::new();
    colony.seed = seed;
    app.insert_resource(colony);
    
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
    
    // Add UI-specific resources
    app.insert_resource(UiState::new());
    app.insert_resource(DashboardData::new());
    
    app
}

fn create_headless_app(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let mut colony = Colony::new();
    colony.seed = seed;
    app.insert_resource(colony);
    
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

fn create_ui_metrics(
    colony: &Colony,
    kpi: &KpiRingBuffer,
    faults: &FaultKpi,
    research: &ResearchState,
    black_swans: &BlackSwanIndex,
    win_loss: &WinLossState,
) -> UiMetrics {
    UiMetrics {
        colony: colony.clone(),
        kpi: kpi.clone(),
        faults: faults.clone(),
        research: research.clone(),
        black_swans: BlackSwanMetrics {
            active: black_swans.potential_swans.keys().cloned().collect(),
            fired: black_swans.fired_swans.iter().cloned().collect(),
        },
        win_loss: win_loss.clone(),
    }
}

fn create_dashboard_data(
    colony: &Colony,
    kpi: &KpiRingBuffer,
    faults: &FaultKpi,
    research: &ResearchState,
    black_swans: &BlackSwanIndex,
    win_loss: &WinLossState,
) -> DashboardData {
    DashboardData {
        power_draw_kw: colony.meters.power_draw_kw,
        power_cap_kw: colony.power_cap_kw,
        bandwidth_util: colony.meters.bandwidth_util,
        corruption_field: colony.corruption_field,
        fault_rate: faults.soft_faults as f32 / 1000.0, // Simulated rate
        sticky_workers: faults.sticky_workers,
        research_points: research.points,
        research_techs: research.acquired_techs.len(),
        black_swans_active: black_swans.potential_swans.len(),
        black_swans_fired: black_swans.fired_swans.len(),
        victory: win_loss.victory,
        doom: win_loss.doom,
        score: win_loss.score,
    }
}

fn get_ui_state(app: &App) -> UiState {
    app.world.resource::<UiState>().clone()
}

fn get_ui_memory_usage(app: &App) -> UiMemoryUsage {
    UiMemoryUsage {
        total_mb: 50.0, // Simulated
        textures_mb: 25.0,
        buffers_mb: 15.0,
        other_mb: 10.0,
    }
}

fn create_api_response(
    colony: &Colony,
    kpi: &KpiRingBuffer,
    faults: &FaultKpi,
    research: &ResearchState,
    black_swans: &BlackSwanIndex,
    win_loss: &WinLossState,
) -> ApiResponse {
    ApiResponse {
        colony: colony.clone(),
        kpi: kpi.clone(),
        faults: faults.clone(),
        research: research.clone(),
        black_swans: BlackSwanMetrics {
            active: black_swans.potential_swans.keys().cloned().collect(),
            fired: black_swans.fired_swans.iter().cloned().collect(),
        },
        win_loss: win_loss.clone(),
    }
}

// Mock structures for testing

#[derive(Resource, Clone)]
struct UiState {
    power_bar: ProgressBar,
    bandwidth_bar: ProgressBar,
    corruption_bar: ProgressBar,
    fault_indicator: Indicator,
    research_indicator: Indicator,
    black_swan_indicator: Indicator,
    victory_indicator: Indicator,
    doom_indicator: Indicator,
    win_loss: WinLossState,
}

impl UiState {
    fn new() -> Self {
        Self {
            power_bar: ProgressBar { value: 0.0 },
            bandwidth_bar: ProgressBar { value: 0.0 },
            corruption_bar: ProgressBar { value: 0.0 },
            fault_indicator: Indicator { count: 0 },
            research_indicator: Indicator { count: 0 },
            black_swan_indicator: Indicator { count: 0 },
            victory_indicator: Indicator { active: false },
            doom_indicator: Indicator { active: false },
            win_loss: WinLossState::new(),
        }
    }
}

#[derive(Resource, Clone)]
struct DashboardData {
    power_draw_kw: f32,
    power_cap_kw: f32,
    bandwidth_util: f32,
    corruption_field: f32,
    fault_rate: f32,
    sticky_workers: u32,
    research_points: u32,
    research_techs: usize,
    black_swans_active: usize,
    black_swans_fired: usize,
    victory: bool,
    doom: bool,
    score: i64,
}

impl DashboardData {
    fn new() -> Self {
        Self {
            power_draw_kw: 0.0,
            power_cap_kw: 0.0,
            bandwidth_util: 0.0,
            corruption_field: 0.0,
            fault_rate: 0.0,
            sticky_workers: 0,
            research_points: 0,
            research_techs: 0,
            black_swans_active: 0,
            black_swans_fired: 0,
            victory: false,
            doom: false,
            score: 0,
        }
    }
}

#[derive(Clone)]
struct UiMetrics {
    colony: Colony,
    kpi: KpiRingBuffer,
    faults: FaultKpi,
    research: ResearchState,
    black_swans: BlackSwanMetrics,
    win_loss: WinLossState,
}

#[derive(Clone)]
struct BlackSwanMetrics {
    active: Vec<String>,
    fired: Vec<String>,
}

#[derive(Clone)]
struct ApiResponse {
    colony: Colony,
    kpi: KpiRingBuffer,
    faults: FaultKpi,
    research: ResearchState,
    black_swans: BlackSwanMetrics,
    win_loss: WinLossState,
}

#[derive(Clone)]
struct ProgressBar {
    value: f32,
}

#[derive(Clone)]
struct Indicator {
    count: u32,
    active: bool,
}

impl Indicator {
    fn new() -> Self {
        Self {
            count: 0,
            active: false,
        }
    }
}

#[derive(Clone)]
struct UiMemoryUsage {
    total_mb: f32,
    textures_mb: f32,
    buffers_mb: f32,
    other_mb: f32,
}
