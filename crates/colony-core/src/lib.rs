pub mod components;
pub mod resources;
pub mod events;
pub mod scheduler;
pub mod time;
pub mod systems;
pub mod maintenance;
pub mod config;
pub mod pipelines;
pub mod io_bridge;
pub mod corruption;
pub mod faults;
pub mod queue;
pub mod gpu;
pub mod gpu_dispatch;
pub mod debts;
pub mod black_swan;
pub mod mutation;
pub mod research;
pub mod game_config;
pub mod victory;
pub mod session;
pub mod save;
pub mod mod_loader;
// pub mod hotreload; // TODO: Implement hotreload functionality
pub mod script;

#[cfg(test)]
mod tests;

pub use components::*;
pub use resources::*;
pub use events::*;
pub use scheduler::*;
pub use time::*;
pub use systems::*;
pub use maintenance::*;
pub use config::*;
pub use pipelines::*;
pub use io_bridge::*;
pub use corruption::*;
pub use faults::*;
pub use queue::*;
pub use gpu::*;
pub use gpu_dispatch::*;
pub use debts::*;
pub use black_swan::*;
pub use mutation::*;
pub use research::*;
pub use game_config::*;
pub use victory::*;
pub use session::*;
pub use save::*;
// pub use mod_loader::*; // TODO: Implement mod_loader functionality
// pub use hotreload::*; // TODO: Implement hotreload functionality
pub use script::*;

use bevy::prelude::*;

pub struct ColonyPlugin;

impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        app        .insert_resource(Colony {
            power_cap_kw: 1000.0,
            bandwidth_total_gbps: 32.0,
            corruption_field: 0.0,
            target_uptime_days: 365,
            meters: GlobalMeters::new(),
            tunables: ResourceTunables::default(),
            corruption_tun: CorruptionTunables::default(),
            seed: 42,
        })
        .insert_resource(DispatchScale(1.0))
        .insert_resource(IoRolling::default())
        .insert_resource(IoRuntime::default())
        .insert_resource(CorruptionField::new())
        .insert_resource(FaultKpi::new())
        .insert_resource(ActiveScheduler::default())
        .insert_resource(JobQueue::new())
        .insert_resource(GpuBatchQueues::new())
        .insert_resource(Debts::new())
        .insert_resource(BlackSwanIndex::new())
        .insert_resource(KpiRingBuffer::new())
        .insert_resource(ResearchState::new())
        .insert_resource(create_default_tech_tree())
        .insert_resource(SessionCtl::new())
        .insert_resource(ReplayLog::new())
        .insert_resource(WinLossState::new())
        .insert_resource(SlaTracker::new(7, 86400000 / 16))
        .insert_resource(WasmHost::new())
        // .insert_resource(LuaHost::new()) // TODO: Fix thread safety issues
        // .insert_resource(ModLoader::new(std::path::PathBuf::from("mods"))) // TODO: Implement
        // .insert_resource(HotReloadManager::new()) // TODO: Implement
        .insert_resource(SimClock {
            tick_scale: TickScale::RealTime,
            now: chrono::Utc::now(),
        })
        .add_event::<WorkerReport>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            time_system,
            power_bandwidth_system,
            heat_system,
            corruption_system,
            dispatch_system,
            gpu_dispatch_system,
            report_ingest_system,
            maintenance_system,
            update_fault_kpis,
            apply_debts_system,
            update_kpi_buffer_system,
            black_swan_scan_system,
            mutation_commit_system,
            research_progress_system,
            update_sla_window,
            win_loss_system,
            session_control_system,
            update_wasm_host_system,
            // TODO: Re-enable when Lua host thread safety is resolved
            // update_lua_host_system,
            // execute_lua_events_system,
            // initialize_mod_loader_system,
            // process_hot_reload_system,
            // update_shadow_world_system,
        ));
    }
}

fn setup(mut commands: Commands, mut jobq: ResMut<queue::JobQueue>) {
    // Create a basic CPU workyard
    commands.spawn((
        Workyard {
            kind: WorkyardKind::CpuArray,
            slots: 4,
            heat: 20.0,
            heat_cap: 100.0,
            power_draw_kw: 200.0,
            bandwidth_share: 0.3,
            isolation_domain: 0,
        },
        YardWorkload::default(),
    ));

    // Create a GPU farm
    commands.spawn((
        Workyard {
            kind: WorkyardKind::GpuFarm,
            slots: 2,
            heat: 25.0,
            heat_cap: 85.0,
            power_draw_kw: 300.0,
            bandwidth_share: 0.4,
            isolation_domain: 1,
        },
        YardWorkload::default(),
        GpuFarm::new(),
    ));

    // Create some CPU workers
    for i in 0..4 {
        commands.spawn(Worker {
            id: i,
            class: WorkClass::Cpu,
            skill_cpu: 0.8 + (i as f32 * 0.05),
            skill_gpu: 0.3,
            skill_io: 0.6,
            discipline: 0.7,
            focus: 0.8,
            corruption: 0.0,
            state: WorkerState::Idle,
            retry: RetryPolicy::default(),
            sticky_faults: 0,
        });
    }

    // Create some GPU workers
    for i in 4..6 {
        commands.spawn(Worker {
            id: i,
            class: WorkClass::Gpu,
            skill_cpu: 0.4,
            skill_gpu: 0.9 + ((i - 4) as f32 * 0.05),
            skill_io: 0.3,
            discipline: 0.8,
            focus: 0.9,
            corruption: 0.0,
            state: WorkerState::Idle,
            retry: RetryPolicy::default(),
            sticky_faults: 0,
        });
    }

    // Add some sample jobs to the queue
    let now_tick = chrono::Utc::now().timestamp_millis() as u64 / 16;
    jobq.push(Job {
        id: 1,
        pipeline: Pipeline {
            ops: vec![Op::UdpDemux, Op::Decode, Op::Kalman],
            mutation_tag: None,
        },
        qos: QoS::Balanced,
        deadline_ms: 50,
        payload_sz: 4096,
    }, now_tick);

    jobq.push(Job {
        id: 2,
        pipeline: Pipeline {
            ops: vec![Op::HttpParse, Op::Decode, Op::Fft],
            mutation_tag: None,
        },
        qos: QoS::Latency,
        deadline_ms: 100,
        payload_sz: 8192,
    }, now_tick);

    jobq.push(Job {
        id: 3,
        pipeline: Pipeline {
            ops: vec![Op::CanParse, Op::Crc, Op::Kalman],
            mutation_tag: None,
        },
        qos: QoS::Throughput,
        deadline_ms: 10,
        payload_sz: 64,
    }, now_tick);

    // Add GPU jobs
    jobq.push(Job {
        id: 4,
        pipeline: Pipeline {
            ops: vec![Op::Decode, Op::Kalman, Op::GpuPreprocess, Op::Yolo, Op::GpuExport],
            mutation_tag: None,
        },
        qos: QoS::Balanced,
        deadline_ms: 40,
        payload_sz: 256,
    }, now_tick);

    jobq.push(Job {
        id: 5,
        pipeline: Pipeline {
            ops: vec![Op::GpuPreprocess, Op::Yolo, Op::GpuExport],
            mutation_tag: None,
        },
        qos: QoS::Latency,
        deadline_ms: 20,
        payload_sz: 512,
    }, now_tick);
}

fn time_system(
    mut clock: ResMut<SimClock>,
    time: Res<Time>,
) {
    // Only advance time if not in real-time mode
    if !clock.is_paused() {
        clock.advance_time();
    }
}

fn dispatch_system(
    mut yards: Query<(Entity, &mut Workyard, &mut YardWorkload)>,
    mut workers: Query<(Entity, &mut Worker)>,
    mut jobq: ResMut<queue::JobQueue>,
    policy: Res<ActiveScheduler>,
    colony: Res<Colony>,
    dispatch_scale: Res<DispatchScale>,
    mut io_rolling: ResMut<IoRolling>,
    corruption_field: Res<CorruptionField>,
    clock: Res<SimClock>,
    mut report_writer: EventWriter<WorkerReport>,
) {
    for (yard_e, mut yard, mut workload) in yards.iter_mut() {
        let available_workers: Vec<Entity> = workers
            .iter()
            .filter(|(_, worker)| worker.state == WorkerState::Idle)
            .map(|(entity, _)| entity)
            .collect();

        // Get jobs from the appropriate queue based on yard type
        let jobs = match yard.kind {
            WorkyardKind::CpuArray => jobq.peek_cpu(),
            WorkyardKind::GpuFarm => jobq.peek_gpu(),
            WorkyardKind::SignalHub => jobq.peek_io(),
        };
        
        if available_workers.is_empty() || jobs.is_empty() {
            continue;
        }

        // Collect job IDs to remove after processing
        let mut completed_job_ids = Vec::new();
        
        // Use the active scheduler to pick jobs
        let scheduler = policy.get_scheduler();
        let worker_refs: Vec<(Entity, &Worker)> = workers
            .iter()
            .filter(|(_, worker)| worker.state == WorkerState::Idle)
            .map(|(entity, worker)| (entity, worker))
            .collect();
        
        let job_values: Vec<Job> = jobs.iter().map(|ej| ej.job.clone()).collect();
        let picks = scheduler.pick(&*yard, &job_values, &worker_refs);
        
        for (worker_e, job) in picks {
            if let Ok((_, mut worker)) = workers.get_mut(worker_e) {
                worker.state = WorkerState::Running;
                
                // Calculate throttling factors
                let throttle = thermal_throttle(
                    yard.heat, 
                    yard.heat_cap, 
                    colony.tunables.thermal_throttle_knee, 
                    colony.tunables.thermal_min_throttle
                );
                let power_scale = dispatch_scale.0;
                let bw_mult = bandwidth_latency_multiplier(
                    colony.meters.bandwidth_util, 
                    colony.tunables.bandwidth_tail_exp
                );

                // Calculate work units for heat generation
                let mut total_work_units = 0.0;
                for op in &job.pipeline.ops {
                    total_work_units += op.work_units();
                    // Track I/O bandwidth for UdpDemux and HttpParse
                    match op {
                        Op::UdpDemux | Op::HttpParse => {
                            io_rolling.add_bytes(job.payload_sz);
                        }
                        _ => {}
                    }
                }
                workload.units_this_tick += total_work_units;
                
                // Calculate queue starvation for fault injection
                let now_tick = clock.now.timestamp_millis() as u64 / 16;
                let enq_tick = jobs.iter().find(|ej| ej.job.id == job.id).map(|ej| ej.enq_tick).unwrap_or(now_tick);
                let queue_starvation = queue::starvation(now_tick, enq_tick, 1000);
                
                // Check for fault injection
                let fault = faults::fault_inject_on_completion(
                    &*worker,
                    &*yard,
                    &job.pipeline.ops[0], // Use first op for fault check
                    corruption_field.global,
                    colony.meters.bandwidth_util,
                    queue_starvation,
                    &colony.corruption_tun,
                    colony.seed,
                    now_tick,
                );
                
                if let Some(fault_kind) = fault {
                    // Handle fault
                    faults::handle_fault(
                        fault_kind,
                        &mut worker,
                        job.id,
                        job.pipeline.ops[0].clone(),
                        &colony.corruption_tun,
                        &mut report_writer,
                    );
                } else {
                    // Normal completion
                    report_writer.send(WorkerReport::Completed { job_id: job.id });
                }
                
                // Mark job for removal
                completed_job_ids.push(job.id);
            }
        }
        
        // Remove completed jobs from the appropriate queue
        for job_id in completed_job_ids {
            match yard.kind {
                WorkyardKind::CpuArray => { jobq.cpu.retain(|ej| ej.job.id != job_id); }
                WorkyardKind::GpuFarm => { jobq.gpu.retain(|ej| ej.job.id != job_id); }
                WorkyardKind::SignalHub => { jobq.io.retain(|ej| ej.job.id != job_id); }
            }
        }
    }
}

fn report_ingest_system(
    mut report_reader: EventReader<WorkerReport>,
    mut workers: Query<&mut Worker>,
) {
    for report in report_reader.read() {
        match report {
            WorkerReport::Completed { job_id } => {
                // Find and reset worker to idle
                for mut worker in workers.iter_mut() {
                    if worker.state == WorkerState::Running {
                        worker.state = WorkerState::Idle;
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
