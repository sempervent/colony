use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use colony_core::{Colony, SimClock, TickScale, ActiveScheduler, SchedPolicy, enqueue_maintenance, JobQueue, Worker, Workyard, YardWorkload, GpuFarm, GpuBatchQueues, BlackSwanIndex, Debts, ResearchState, TechTree, FaultKpi, CorruptionField, IoRolling};
use colony_io::IoSimulatorConfig;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

#[derive(Default, Resource)]
pub struct UiCache {
    pub intents: Vec<UiIntent>,
    pub selected_tab: UiTab,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum UiTab {
    #[default]
    Dashboard,
    Pipelines,
    Workers,
    Yards,
    Io,
    Gpu,
    Scheduler,
    Corruption,
    Events,
    Research,
    Mods,
    Replay,
}

#[derive(Debug, Clone)]
pub enum UiIntent {
    TogglePause,
    SetTickScale(TickScale),
    StartUdp(IoSimulatorConfig),
    StartHttp(IoSimulatorConfig),
    Enqueue { pipeline: String, payload: usize },
    SwitchSched(SchedPolicy),
    Maintenance(Entity), // yard
    RunRitual(String),
    StartReplay(String),
    StopReplay,
    SwitchTab(UiTab),
    StartGame,
    LoadGame,
    SaveGame,
}

// UI Events that will be processed by the simulation
#[derive(Event)]
pub struct JobSubmitted(pub colony_core::Job);

#[derive(Event)]
pub struct StartUdpSim(pub IoSimulatorConfig);

#[derive(Event)]
pub struct StartHttpSim(pub IoSimulatorConfig);

#[derive(Event)]
pub struct SwitchScheduler(pub SchedPolicy);

#[derive(Event)]
pub struct StartRitual(pub String);

#[derive(Event)]
pub struct StartReplay(pub String);

#[derive(Event)]
pub struct StopReplay;

#[derive(Event)]
pub struct StartGame;

#[derive(Event)]
pub struct LoadGame;

#[derive(Event)]
pub struct SaveGame;

// UI Snapshot Resources for efficient display
#[derive(Resource, Default)]
pub struct UiMeters {
    pub power_draw: f32,
    pub power_cap: f32,
    pub bw_util: f32,
    pub corruption_global: f32,
    pub sla_percent: f32,
}

#[derive(Resource, Default)]
pub struct UiPipelines {
    pub rows: Vec<PipelineRow>,
}

#[derive(Debug, Clone)]
pub struct PipelineRow {
    pub id: String,
    pub qos: String,
    pub deadline_ms: u32,
    pub throughput: f32,
    pub miss_pct: f32,
    pub queue_depth: usize,
    pub default_payload: usize,
}

#[derive(Resource, Default)]
pub struct UiWorkers {
    pub rows: Vec<WorkerRow>,
}

#[derive(Debug, Clone)]
pub struct WorkerRow {
    pub id: String,
    pub class: String,
    pub state: String,
    pub skill_cpu: f32,
    pub skill_gpu: f32,
    pub skill_io: f32,
    pub corruption: f32,
    pub retries: u32,
}

#[derive(Resource, Default)]
pub struct UiYards {
    pub rows: Vec<YardRow>,
}

#[derive(Debug, Clone)]
pub struct YardRow {
    pub entity: Entity,
    pub kind: String,
    pub heat: f32,
    pub heat_cap: f32,
    pub throttle: f32,
    pub power_draw: f32,
    pub slots_used: usize,
    pub slots_total: usize,
}

#[derive(Resource, Default)]
pub struct UiGpu {
    pub util: f32,
    pub vram_used: f32,
    pub vram_total: f32,
    pub batch_latency: f32,
    pub batches_inflight: usize,
    pub queues: Vec<(String, usize)>,
}

#[derive(Resource, Default)]
pub struct UiEvents {
    pub eligible: Vec<String>,
    pub active: Vec<String>,
    pub recent: Vec<(String, u64)>,
    pub debts: Vec<String>,
}

#[derive(Resource, Default)]
pub struct UiResearch {
    pub points: u32,
    pub available_techs: Vec<String>,
    pub researched_techs: Vec<String>,
    pub available_rituals: Vec<String>,
}

pub struct DesktopUiPlugin;

impl Plugin for DesktopUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .insert_resource(UiCache::default())
           .insert_resource(UiMeters::default())
           .insert_resource(UiPipelines::default())
           .insert_resource(UiWorkers::default())
           .insert_resource(UiYards::default())
           .insert_resource(UiGpu::default())
           .insert_resource(UiEvents::default())
           .insert_resource(UiResearch::default())
           .add_event::<JobSubmitted>()
           .add_event::<StartUdpSim>()
           .add_event::<StartHttpSim>()
           .add_event::<SwitchScheduler>()
           .add_event::<StartRitual>()
           .add_event::<StartReplay>()
           .add_event::<StopReplay>()
           .add_event::<StartGame>()
           .add_event::<LoadGame>()
           .add_event::<SaveGame>()
           .add_systems(Startup, ui_setup)
           .add_systems(Update, update_ui_snapshots)
           .add_systems(Update, ui_frame_system)
           .add_systems(Update, ui_command_flush)
           .add_systems(Update, crate::handle_legacy_keyboard_input);
    }
}

fn ui_setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn update_ui_snapshots(
    colony: Res<Colony>,
    workers: Query<(Entity, &Worker)>,
    yards: Query<(Entity, &Workyard, &YardWorkload)>,
    gpu_farms: Query<&GpuFarm>,
    batch_queues: Res<GpuBatchQueues>,
    black_swan_index: Res<BlackSwanIndex>,
    debts: Res<Debts>,
    research_state: Res<ResearchState>,
    tech_tree: Res<TechTree>,
    fault_kpis: Res<FaultKpi>,
    corruption_field: Res<CorruptionField>,
    mut ui_meters: ResMut<UiMeters>,
    mut ui_pipelines: ResMut<UiPipelines>,
    mut ui_workers: ResMut<UiWorkers>,
    mut ui_yards: ResMut<UiYards>,
    mut ui_gpu: ResMut<UiGpu>,
    mut ui_events: ResMut<UiEvents>,
    mut ui_research: ResMut<UiResearch>,
) {
    // Update meters
    ui_meters.power_draw = colony.meters.power_draw_kw;
    ui_meters.power_cap = colony.power_cap_kw;
    ui_meters.bw_util = colony.meters.bandwidth_util;
    ui_meters.corruption_global = corruption_field.global;
    ui_meters.sla_percent = fault_kpis.deadline_hit_rate * 100.0;

    // Update pipelines (placeholder - would need actual pipeline data)
    ui_pipelines.rows.clear();
    ui_pipelines.rows.push(PipelineRow {
        id: "udp_pipeline".to_string(),
        qos: "High".to_string(),
        deadline_ms: 100,
        throughput: 1000.0,
        miss_pct: 0.05,
        queue_depth: 5,
        default_payload: 1024,
    });
    ui_pipelines.rows.push(PipelineRow {
        id: "http_pipeline".to_string(),
        qos: "Medium".to_string(),
        deadline_ms: 500,
        throughput: 500.0,
        miss_pct: 0.02,
        queue_depth: 3,
        default_payload: 2048,
    });

    // Update workers
    ui_workers.rows.clear();
    for (entity, worker) in workers.iter() {
        ui_workers.rows.push(WorkerRow {
            id: format!("worker_{}", entity.index()),
            class: format!("{:?}", worker.class),
            state: format!("{:?}", worker.state),
            skill_cpu: worker.skill_cpu,
            skill_gpu: worker.skill_gpu,
            skill_io: worker.skill_io,
            corruption: worker.corruption,
            retries: worker.retry.max_retries as u32,
        });
    }

    // Update yards
    ui_yards.rows.clear();
    for (entity, yard, workload) in yards.iter() {
        let throttle = colony_core::thermal_throttle(
            yard.heat,
            yard.heat_cap,
            colony.tunables.thermal_throttle_knee,
            colony.tunables.thermal_min_throttle,
        );
        
        ui_yards.rows.push(YardRow {
            entity,
            kind: format!("{:?}", yard.kind),
            heat: yard.heat,
            heat_cap: yard.heat_cap,
            throttle,
            power_draw: yard.power_draw_kw,
            slots_used: workload.units_this_tick as usize,
            slots_total: yard.slots as usize,
        });
    }

    // Update GPU
    if let Some(gpu_farm) = gpu_farms.iter().next() {
        ui_gpu.util = gpu_farm.meters.util;
        ui_gpu.vram_used = gpu_farm.meters.vram_used_gb;
        ui_gpu.vram_total = gpu_farm.per_gpu.vram_gb;
        ui_gpu.batch_latency = gpu_farm.meters.batch_latency_ms;
        ui_gpu.batches_inflight = gpu_farm.meters.batches_inflight as usize;
        
        ui_gpu.queues.clear();
        for (pipeline_id, buffer) in &batch_queues.buffers {
            ui_gpu.queues.push((pipeline_id.clone(), buffer.items.len()));
        }
    }

    // Update events
    ui_events.eligible.clear();
    ui_events.active = black_swan_index.meters.active.clone();
    ui_events.recent = black_swan_index.meters.recently_fired.clone();
    
    ui_events.debts.clear();
    for debt in &debts.active {
        match debt {
            colony_core::Debt::PowerMult { mult, until_tick } => {
                ui_events.debts.push(format!("Power Mult: {:.2}x (until {})", mult, until_tick));
            }
            colony_core::Debt::HeatAdd { celsius, until_tick } => {
                ui_events.debts.push(format!("Heat Add: +{:.1}°C (until {})", celsius, until_tick));
            }
            colony_core::Debt::BandwidthTax { mult, until_tick } => {
                ui_events.debts.push(format!("Bandwidth Tax: {:.2}x (until {})", mult, until_tick));
            }
            colony_core::Debt::VramLeak { mb_per_tick, until_tick } => {
                ui_events.debts.push(format!("VRAM Leak: {:.1} MB/tick (until {})", mb_per_tick, until_tick));
            }
            colony_core::Debt::FaultBias { kind, weight_mult, until_tick } => {
                ui_events.debts.push(format!("Fault Bias: {} {:.2}x (until {})", kind, weight_mult, until_tick));
            }
            colony_core::Debt::Illusion { metric, delta, until_tick } => {
                ui_events.debts.push(format!("Illusion: {} {:.2} (until {})", metric, delta, until_tick));
            }
        }
    }

    // Update research
    ui_research.points = research_state.pts;
    ui_research.available_techs = tech_tree.get_available_techs(&research_state)
        .iter()
        .map(|t| t.name.clone())
        .collect();
    ui_research.researched_techs = tech_tree.get_researched_techs(&research_state)
        .iter()
        .map(|t| t.name.clone())
        .collect();
    ui_research.available_rituals = research_state.get_available_rituals()
        .iter()
        .map(|r| r.name.clone())
        .collect();
}

fn ui_frame_system(
    mut egui_ctx: EguiContexts,
    mut cache: ResMut<UiCache>,
    app_state: Res<State<AppState>>,
    clock: Res<SimClock>,
    ui_meters: Res<UiMeters>,
    ui_pipelines: Res<UiPipelines>,
    ui_workers: Res<UiWorkers>,
    ui_yards: Res<UiYards>,
    ui_gpu: Res<UiGpu>,
    ui_events: Res<UiEvents>,
    ui_research: Res<UiResearch>,
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };

    // Top bar (always visible)
    egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("⏯").clicked() {
                cache.intents.push(UiIntent::TogglePause);
            }
            
            ui.separator();
            
            ui.label(format!("Sim: {}", clock.now));
            
            // Tick scale selector
            egui::ComboBox::from_label("Tick")
                .selected_text(format!("{:?}", clock.tick_scale))
                .show_ui(ui, |cb| {
                    for scale in [TickScale::RealTime, TickScale::Seconds(1), TickScale::Days(1), TickScale::Years(1)] {
                        if cb.selectable_label(false, format!("{:?}", scale)).clicked() {
                            cache.intents.push(UiIntent::SetTickScale(scale));
                        }
                    }
                });
            
            ui.separator();
            
            ui.label(format!("SLA: {:.2}%", ui_meters.sla_percent));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Save").clicked() {
                    cache.intents.push(UiIntent::SaveGame);
                }
                if ui.button("Load").clicked() {
                    cache.intents.push(UiIntent::LoadGame);
                }
            });
        });
    });

    match app_state.get() {
        AppState::MainMenu => {
            draw_setup_wizard(ctx, &mut cache);
        }
        AppState::InGame | AppState::Paused => {
            // Left navigation
            egui::SidePanel::left("nav").resizable(true).show(ctx, |ui| {
                ui.heading("Console");
                ui.separator();
                
                for tab in [
                    UiTab::Dashboard,
                    UiTab::Pipelines,
                    UiTab::Workers,
                    UiTab::Yards,
                    UiTab::Io,
                    UiTab::Gpu,
                    UiTab::Scheduler,
                    UiTab::Corruption,
                    UiTab::Events,
                    UiTab::Research,
                    UiTab::Mods,
                    UiTab::Replay,
                ] {
                    let label = match tab {
                        UiTab::Dashboard => "📊 Dashboard",
                        UiTab::Pipelines => "🔧 Pipelines",
                        UiTab::Workers => "👷 Workers",
                        UiTab::Yards => "🏭 Yards",
                        UiTab::Io => "🌐 I/O",
                        UiTab::Gpu => "🎮 GPU",
                        UiTab::Scheduler => "⏰ Scheduler",
                        UiTab::Corruption => "⚠️ Corruption",
                        UiTab::Events => "🎭 Events",
                        UiTab::Research => "🔬 Research",
                        UiTab::Mods => "🔌 Mods",
                        UiTab::Replay => "📼 Replay",
                    };
                    
                    if ui.selectable_label(cache.selected_tab == tab, label).clicked() {
                        cache.intents.push(UiIntent::SwitchTab(tab));
                    }
                }
            });

            // Main content area
            egui::CentralPanel::default().show(ctx, |ui| {
                match cache.selected_tab {
                    UiTab::Dashboard => draw_dashboard(ui, &ui_meters, &ui_pipelines, &ui_workers, &ui_yards, &ui_gpu, &mut cache),
                    UiTab::Pipelines => draw_pipelines(ui, &ui_pipelines, &mut cache),
                    UiTab::Workers => draw_workers(ui, &ui_workers, &mut cache),
                    UiTab::Yards => draw_yards(ui, &ui_yards, &mut cache),
                    UiTab::Io => draw_io_panel(ui, &mut cache),
                    UiTab::Gpu => draw_gpu_panel(ui, &ui_gpu, &mut cache),
                    UiTab::Scheduler => draw_scheduler_panel(ui, &mut cache),
                    UiTab::Corruption => draw_corruption_panel(ui, &mut cache),
                    UiTab::Events => draw_events_panel(ui, &ui_events, &mut cache),
                    UiTab::Research => draw_research_panel(ui, &ui_research, &mut cache),
                    UiTab::Mods => draw_mods_panel(ui, &mut cache),
                    UiTab::Replay => draw_replay_panel(ui, &mut cache),
                }
            });

            // Right meters
            egui::SidePanel::right("meters").show(ctx, |ui| {
                draw_meters(ui, &ui_meters);
            });
        }
    }

    // Bottom status bar
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.label("Ready");
    });
}

fn draw_setup_wizard(ctx: &egui::Context, cache: &mut UiCache) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Compute Colony - Setup Wizard");
        ui.add_space(20.0);
        
        ui.label("Welcome to Compute Colony! Configure your simulation:");
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            if ui.button("Start New Game").clicked() {
                cache.intents.push(UiIntent::StartGame);
            }
            if ui.button("Load Game").clicked() {
                cache.intents.push(UiIntent::LoadGame);
            }
        });
        
        ui.add_space(20.0);
        
        ui.label("Game Configuration:");
        ui.label("• Scenario: Default Colony");
        ui.label("• Difficulty: Normal");
        ui.label("• Pipelines: UDP, HTTP");
        ui.label("• Events: Enabled");
        ui.label("• Tick Scale: Real-time");
    });
}

fn draw_dashboard(
    ui: &mut egui::Ui,
    meters: &UiMeters,
    pipelines: &UiPipelines,
    workers: &UiWorkers,
    yards: &UiYards,
    gpu: &UiGpu,
    cache: &mut UiCache,
) {
    ui.heading("Dashboard");
    ui.add_space(10.0);
    
    // Key metrics
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Power Usage");
            ui.add(egui::ProgressBar::new(meters.power_draw / meters.power_cap)
                .text(format!("{:.0}/{:.0} kW", meters.power_draw, meters.power_cap)));
        });
        
        ui.vertical(|ui| {
            ui.label("Bandwidth");
            ui.add(egui::ProgressBar::new(meters.bw_util)
                .text(format!("{:.1}%", meters.bw_util * 100.0)));
        });
        
        ui.vertical(|ui| {
            ui.label("Corruption");
            ui.add(egui::ProgressBar::new(meters.corruption_global)
                .text(format!("{:.1}%", meters.corruption_global * 100.0)));
        });
    });
    
    ui.add_space(20.0);
    
    // System overview
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("System Overview");
            ui.label(format!("Pipelines: {}", pipelines.rows.len()));
            ui.label(format!("Workers: {}", workers.rows.len()));
            ui.label(format!("Yards: {}", yards.rows.len()));
            ui.label(format!("GPU Utilization: {:.1}%", gpu.util * 100.0));
        });
        
        ui.vertical(|ui| {
            ui.label("Quick Actions");
            if ui.button("Emergency Maintenance").clicked() {
                // TODO: Emergency maintenance
            }
            if ui.button("Reset Scheduler").clicked() {
                cache.intents.push(UiIntent::SwitchSched(SchedPolicy::Fcfs));
            }
        });
    });
}

fn draw_pipelines(ui: &mut egui::Ui, pipelines: &UiPipelines, cache: &mut UiCache) {
    ui.heading("Pipelines");
    ui.add_space(10.0);
    
    egui::Grid::new("pipes_grid").striped(true).show(ui, |ui| {
        ui.heading("Pipeline");
        ui.heading("QoS");
        ui.heading("Deadline");
        ui.heading("Throughput");
        ui.heading("Miss %");
        ui.heading("Queue");
        ui.heading("Actions");
        ui.end_row();

        for p in &pipelines.rows {
            ui.label(&p.id);
            ui.label(&p.qos);
            ui.label(format!("{} ms", p.deadline_ms));
            ui.label(format!("{:.1}/s", p.throughput));
            ui.label(format!("{:.2}%", p.miss_pct * 100.0));
            ui.label(p.queue_depth.to_string());
            if ui.small_button("Enqueue").clicked() {
                cache.intents.push(UiIntent::Enqueue {
                    pipeline: p.id.clone(),
                    payload: p.default_payload,
                });
            }
            ui.end_row();
        }
    });
}

fn draw_workers(ui: &mut egui::Ui, workers: &UiWorkers, _cache: &mut UiCache) {
    ui.heading("Workers");
    ui.add_space(10.0);
    
    egui::Grid::new("workers_grid").striped(true).show(ui, |ui| {
        ui.heading("ID");
        ui.heading("Class");
        ui.heading("State");
        ui.heading("CPU");
        ui.heading("GPU");
        ui.heading("I/O");
        ui.heading("Corruption");
        ui.heading("Retries");
        ui.end_row();

        for w in &workers.rows {
            ui.label(&w.id);
            ui.label(&w.class);
            ui.label(&w.state);
            ui.label(format!("{:.1}", w.skill_cpu));
            ui.label(format!("{:.1}", w.skill_gpu));
            ui.label(format!("{:.1}", w.skill_io));
            ui.add(egui::ProgressBar::new(w.corruption)
                .text(format!("{:.1}%", w.corruption * 100.0)));
            ui.label(w.retries.to_string());
            ui.end_row();
        }
    });
}

fn draw_yards(ui: &mut egui::Ui, yards: &UiYards, cache: &mut UiCache) {
    ui.heading("Workyards");
    ui.add_space(10.0);
    
    for yard in &yards.rows {
        ui.group(|ui| {
            ui.label(format!("{} #{}", yard.kind, yard.entity.index()));
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Heat");
                    ui.add(egui::ProgressBar::new(yard.heat / yard.heat_cap)
                        .text(format!("{:.1}°C", yard.heat)));
                });
                
                ui.vertical(|ui| {
                    ui.label("Throttle");
                    ui.label(format!("{:.2}x", yard.throttle));
                });
                
                ui.vertical(|ui| {
                    ui.label("Power");
                    ui.label(format!("{:.0} kW", yard.power_draw));
                });
                
                ui.vertical(|ui| {
                    ui.label("Slots");
                    ui.label(format!("{}/{}", yard.slots_used, yard.slots_total));
                });
            });
            
            if ui.button("Maintenance").clicked() {
                cache.intents.push(UiIntent::Maintenance(yard.entity));
            }
        });
        ui.add_space(5.0);
    }
}

fn draw_io_panel(ui: &mut egui::Ui, cache: &mut UiCache) {
    ui.heading("I/O Control Panel");
    ui.add_space(10.0);
    
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("UDP Simulator");
            if ui.button("Start UDP").clicked() {
                cache.intents.push(UiIntent::StartUdp(IoSimulatorConfig {
                    rate_hz: 100.0,
                    jitter_ms: 5,
                    burstiness: 0.1,
                    loss: 0.01,
                    payload_bytes: 1024,
                    http_paths: vec![],
                }));
            }
            if ui.button("Stop UDP").clicked() {
                // TODO: Stop UDP
            }
        });
        
        ui.vertical(|ui| {
            ui.label("HTTP Simulator");
            if ui.button("Start HTTP").clicked() {
                cache.intents.push(UiIntent::StartHttp(IoSimulatorConfig {
                    rate_hz: 50.0,
                    jitter_ms: 10,
                    burstiness: 0.2,
                    loss: 0.005,
                    payload_bytes: 2048,
                    http_paths: vec![],
                }));
            }
            if ui.button("Stop HTTP").clicked() {
                // TODO: Stop HTTP
            }
        });
    });
}

fn draw_gpu_panel(ui: &mut egui::Ui, gpu: &UiGpu, _cache: &mut UiCache) {
    ui.heading("GPU Farm Control");
    ui.add_space(10.0);
    
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("GPU Utilization");
            ui.add(egui::ProgressBar::new(gpu.util)
                .text(format!("{:.1}%", gpu.util * 100.0)));
        });
        
        ui.vertical(|ui| {
            ui.label("VRAM Usage");
            ui.add(egui::ProgressBar::new(gpu.vram_used / gpu.vram_total)
                .text(format!("{:.2}/{:.1} GB", gpu.vram_used, gpu.vram_total)));
        });
    });
    
    ui.add_space(10.0);
    
    ui.label(format!("Batch Latency: {:.1} ms", gpu.batch_latency));
    ui.label(format!("Batches In Flight: {}", gpu.batches_inflight));
    
    ui.add_space(10.0);
    
    ui.label("Batch Queues:");
    for (pipeline_id, depth) in &gpu.queues {
        ui.label(format!("{}: {} items", pipeline_id, depth));
    }
}

fn draw_scheduler_panel(ui: &mut egui::Ui, cache: &mut UiCache) {
    ui.heading("Scheduler Control");
    ui.add_space(10.0);
    
    ui.label("Select Policy:");
    ui.horizontal(|ui| {
        if ui.button("FCFS").clicked() {
            cache.intents.push(UiIntent::SwitchSched(SchedPolicy::Fcfs));
        }
        if ui.button("SJF").clicked() {
            cache.intents.push(UiIntent::SwitchSched(SchedPolicy::Sjf));
        }
        if ui.button("EDF").clicked() {
            cache.intents.push(UiIntent::SwitchSched(SchedPolicy::Edf));
        }
    });
    
    ui.add_space(10.0);
    
    ui.label("Policy Descriptions:");
    ui.label("• FCFS: First Come, First Served");
    ui.label("• SJF: Shortest Job First");
    ui.label("• EDF: Earliest Deadline First");
}

fn draw_corruption_panel(ui: &mut egui::Ui, _cache: &mut UiCache) {
    ui.heading("Corruption & Faults");
    ui.add_space(10.0);
    
    ui.label("This panel shows corruption levels and fault statistics.");
    ui.label("Use the main UI panels to monitor specific metrics.");
}

fn draw_events_panel(ui: &mut egui::Ui, events: &UiEvents, _cache: &mut UiCache) {
    ui.heading("Black Swan Events");
    ui.add_space(10.0);
    
    ui.label("Active Events:");
    for event in &events.active {
        ui.label(format!("• {}", event));
    }
    
    ui.add_space(10.0);
    
    ui.label("Recently Fired:");
    for (id, tick) in &events.recent {
        ui.label(format!("• {} (tick {})", id, tick));
    }
    
    ui.add_space(10.0);
    
    ui.label("Active Debts:");
    for debt in &events.debts {
        ui.label(format!("• {}", debt));
    }
}

fn draw_research_panel(ui: &mut egui::Ui, research: &UiResearch, cache: &mut UiCache) {
    ui.heading("Research & Tech Tree");
    ui.add_space(10.0);
    
    ui.label(format!("Research Points: {}", research.points));
    
    ui.add_space(10.0);
    
    ui.label("Available Techs:");
    for tech in &research.available_techs {
        ui.label(format!("• {}", tech));
    }
    
    ui.add_space(10.0);
    
    ui.label("Researched Techs:");
    for tech in &research.researched_techs {
        ui.label(format!("✅ {}", tech));
    }
    
    ui.add_space(10.0);
    
    ui.label("Available Rituals:");
    for ritual in &research.available_rituals {
        if ui.button(format!("Start {}", ritual)).clicked() {
            cache.intents.push(UiIntent::RunRitual(ritual.clone()));
        }
    }
}

fn draw_mods_panel(ui: &mut egui::Ui, _cache: &mut UiCache) {
    ui.heading("Mods Console");
    ui.add_space(10.0);
    
    ui.label("Installed Mods:");
    ui.label("• vanilla (enabled)");
    
    ui.add_space(10.0);
    
    ui.label("Mod Controls:");
    if ui.button("Hot Reload").clicked() {
        // TODO: Hot reload
    }
    if ui.button("Dry Run").clicked() {
        // TODO: Dry run
    }
}

fn draw_replay_panel(ui: &mut egui::Ui, cache: &mut UiCache) {
    ui.heading("Replay Control");
    ui.add_space(10.0);
    
    ui.label("Replay Status: Not Active");
    
    ui.add_space(10.0);
    
    ui.horizontal(|ui| {
        if ui.button("Start Replay").clicked() {
            cache.intents.push(UiIntent::StartReplay("replay.ron".to_string()));
        }
        if ui.button("Stop Replay").clicked() {
            cache.intents.push(UiIntent::StopReplay);
        }
    });
}

fn draw_meters(ui: &mut egui::Ui, meters: &UiMeters) {
    ui.heading("System Meters");
    ui.add_space(10.0);
    
    ui.label("Power");
    ui.add(egui::ProgressBar::new(meters.power_draw / meters.power_cap)
        .text(format!("{:.0}/{:.0} kW", meters.power_draw, meters.power_cap)));
    
    ui.add_space(10.0);
    
    ui.label("Bandwidth");
    ui.add(egui::ProgressBar::new(meters.bw_util)
        .text(format!("{:.1}%", meters.bw_util * 100.0)));
    
    ui.add_space(10.0);
    
    ui.label("Corruption");
    ui.add(egui::ProgressBar::new(meters.corruption_global)
        .text(format!("{:.1}%", meters.corruption_global * 100.0)));
    
    ui.add_space(10.0);
    
    ui.label("SLA");
    ui.add(egui::ProgressBar::new(meters.sla_percent / 100.0)
        .text(format!("{:.1}%", meters.sla_percent)));
}

fn ui_command_flush(
    mut cache: ResMut<UiCache>,
    _ev_job: EventWriter<JobSubmitted>,
    mut ev_udp: EventWriter<StartUdpSim>,
    mut ev_http: EventWriter<StartHttpSim>,
    mut ev_sched: EventWriter<SwitchScheduler>,
    mut ev_ritual: EventWriter<StartRitual>,
    mut ev_replay_start: EventWriter<StartReplay>,
    mut ev_replay_stop: EventWriter<StopReplay>,
    mut ev_start_game: EventWriter<StartGame>,
    mut ev_load_game: EventWriter<LoadGame>,
    mut ev_save_game: EventWriter<SaveGame>,
    mut next_state: ResMut<NextState<AppState>>,
    mut scheduler: ResMut<ActiveScheduler>,
    mut clock: ResMut<SimClock>,
    _yards: Query<Entity, With<Workyard>>,
    mut jobq: ResMut<JobQueue>,
) {
    let intents = std::mem::take(&mut cache.intents);
    for intent in intents {
        match intent {
            UiIntent::TogglePause => {
                // Toggle between InGame and Paused
                next_state.set(AppState::Paused);
            }
            UiIntent::SetTickScale(scale) => {
                clock.tick_scale = scale;
            }
            UiIntent::StartUdp(config) => {
                ev_udp.write(StartUdpSim(config));
            }
            UiIntent::StartHttp(config) => {
                ev_http.write(StartHttpSim(config));
            }
            UiIntent::Enqueue { pipeline: _, payload: _ } => {
                // TODO: Create proper job from pipeline
                // ev_job.write(JobSubmitted(job));
            }
            UiIntent::SwitchSched(policy) => {
                scheduler.policy = policy;
                ev_sched.write(SwitchScheduler(policy));
            }
            UiIntent::Maintenance(yard_entity) => {
                enqueue_maintenance(yard_entity, &mut jobq);
            }
            UiIntent::RunRitual(ritual_id) => {
                ev_ritual.write(StartRitual(ritual_id));
            }
            UiIntent::StartReplay(file) => {
                ev_replay_start.write(StartReplay(file));
            }
            UiIntent::StopReplay => {
                ev_replay_stop.write(StopReplay);
            }
            UiIntent::SwitchTab(tab) => {
                // Handle tab switching after the loop
                cache.selected_tab = tab;
            }
            UiIntent::StartGame => {
                ev_start_game.write(StartGame);
                next_state.set(AppState::InGame);
            }
            UiIntent::LoadGame => {
                ev_load_game.write(LoadGame);
            }
            UiIntent::SaveGame => {
                ev_save_game.write(SaveGame);
            }
        }
    }
}
