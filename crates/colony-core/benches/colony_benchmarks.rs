use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use colony_core::*;
use bevy::prelude::*;
use std::time::Duration;

fn benchmark_thermal_throttle(c: &mut Criterion) {
    let mut group = c.benchmark_group("thermal_throttle");
    
    for heat in [0.0, 25.0, 50.0, 75.0, 90.0, 95.0] {
        group.bench_with_input(BenchmarkId::new("throttle", heat), &heat, |b, &heat| {
            b.iter(|| {
                thermal_throttle(
                    black_box(heat),
                    black_box(100.0),
                    black_box(0.8),
                    black_box(0.1)
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_corruption_field_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("corruption_field");
    
    for corruption in [0.0, 0.1, 0.3, 0.5, 0.7, 0.9] {
        group.bench_with_input(BenchmarkId::new("update", corruption), &corruption, |b, &corruption| {
            b.iter(|| {
                update_corruption_field(
                    black_box(corruption),
                    black_box(0.01),
                    black_box(0.001)
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_scheduler_policies(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler_policies");
    
    let policies = vec![
        SchedPolicy::Sjf,
        SchedPolicy::Fifo,
        SchedPolicy::Edf,
    ];
    
    for policy in policies {
        group.bench_with_input(BenchmarkId::new("policy", format!("{:?}", policy)), &policy, |b, &policy| {
            let mut job_queue = JobQueue::new();
            
            // Add some test jobs
            for i in 0..100 {
                job_queue.enqueue(Job {
                    id: i,
                    pipeline: Pipeline {
                        ops: vec![Op::Decode],
                        mutation_tag: None,
                    },
                    qos: QoS::Balanced,
                    deadline_ms: 1000 - (i * 10),
                    payload_sz: 1024,
                });
            }
            
            let jobs: Vec<&Job> = job_queue.peek_cpu().iter().map(|ej| &ej.job).collect();
            let workers = vec![
                (Entity::from_raw(0), &Worker {
                    id: 0,
                    class: WorkClass::Cpu,
                    skill_cpu: 1.0,
                    skill_gpu: 0.0,
                    skill_io: 0.0,
                    discipline: 1.0,
                    focus: 1.0,
                    corruption: 0.0,
                    state: WorkerState::Idle,
                    retry: RetryPolicy::default(),
                    sticky_faults: 0,
                }),
            ];
            
            b.iter(|| {
                let scheduler = policy.get_scheduler();
                scheduler.pick(
                    black_box(&Workyard {
                        kind: WorkyardKind::CpuArray,
                        slots: 4,
                        heat: 50.0,
                        heat_cap: 100.0,
                        power_draw_kw: 200.0,
                        bandwidth_share: 0.3,
                        isolation_domain: 0,
                    }),
                    black_box(&jobs),
                    black_box(&workers)
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_gpu_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_batching");
    
    for batch_size in [1, 2, 4, 8, 16, 32] {
        group.bench_with_input(BenchmarkId::new("batch", batch_size), &batch_size, |b, &batch_size| {
            let mut gpu_queues = GpuBatchQueues::new();
            
            // Add jobs to batch
            for i in 0..batch_size {
                gpu_queues.enqueue(Job {
                    id: i,
                    pipeline: Pipeline {
                        ops: vec![Op::GpuPreprocess, Op::Yolo, Op::GpuExport],
                        mutation_tag: None,
                    },
                    qos: QoS::Balanced,
                    deadline_ms: 100,
                    payload_sz: 256,
                });
            }
            
            b.iter(|| {
                gpu_queues.batch_jobs(black_box(batch_size))
            })
        });
    }
    
    group.finish();
}

fn benchmark_black_swan_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("black_swan_scan");
    
    for swan_count in [1, 5, 10, 20, 50, 100] {
        group.bench_with_input(BenchmarkId::new("scan", swan_count), &swan_count, |b, &swan_count| {
            let mut black_swan_index = BlackSwanIndex::new();
            
            // Add potential swans
            for i in 0..swan_count {
                black_swan_index.add_potential_swan(
                    format!("swan_{}", i),
                    0.1 + (i as f32 * 0.01)
                );
            }
            
            b.iter(|| {
                black_swan_scan_system(
                    black_box(&mut black_swan_index),
                    black_box(&SimClock {
                        tick_scale: TickScale::RealTime,
                        now: chrono::Utc::now(),
                    })
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_kpi_buffer_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("kpi_buffer");
    
    for buffer_size in [100, 500, 1000, 5000, 10000] {
        group.bench_with_input(BenchmarkId::new("update", buffer_size), &buffer_size, |b, &buffer_size| {
            let mut kpi_buffer = KpiRingBuffer::new();
            
            // Pre-fill buffer
            for i in 0..buffer_size {
                kpi_buffer.bandwidth_util.push((i as f32 * 0.1, i as u64));
            }
            
            b.iter(|| {
                update_kpi_buffer_system(
                    black_box(&mut kpi_buffer),
                    black_box(0.5),
                    black_box(1000)
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_wasm_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_execution");
    
    for fuel_limit in [1000, 10000, 100000, 1000000] {
        group.bench_with_input(BenchmarkId::new("fuel", fuel_limit), &fuel_limit, |b, &fuel_limit| {
            let mut wasm_host = WasmHost::new();
            wasm_host.execution_env.fuel_limit = fuel_limit;
            
            b.iter(|| {
                // Mock WASM execution
                wasm_host.execute_op(
                    black_box("test_mod"),
                    black_box(&WasmOpSpec {
                        function_name: "test_op".to_string(),
                        input_schema: "bytes".to_string(),
                        output_schema: "bytes".to_string(),
                    }),
                    black_box(&[1, 2, 3, 4])
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_lua_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("lua_execution");
    
    for instruction_budget in [1000, 10000, 100000, 200000] {
        group.bench_with_input(BenchmarkId::new("instructions", instruction_budget), &instruction_budget, |b, &instruction_budget| {
            let mut lua_host = LuaHost::new();
            lua_host.execution_env.instruction_budget = instruction_budget;
            
            b.iter(|| {
                // Mock Lua execution
                lua_host.call_event_hook(
                    black_box("test_mod"),
                    black_box("on_tick")
                )
            })
        });
    }
    
    group.finish();
}

fn benchmark_hot_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_reload");
    
    for mod_count in [1, 5, 10, 20] {
        group.bench_with_input(BenchmarkId::new("reload", mod_count), &mod_count, |b, &mod_count| {
            let mut hot_reload_manager = HotReloadManager::new();
            let mut mod_loader = ModLoader::new(std::path::PathBuf::from("mods"));
            
            // Add mods to reload
            for i in 0..mod_count {
                hot_reload_manager.queue_reload(&format!("mod_{}", i));
            }
            
            b.iter(|| {
                process_hot_reload_system(
                    black_box(&mut hot_reload_manager),
                    black_box(&mut mod_loader),
                    black_box(&Time::default())
                )
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_thermal_throttle,
    benchmark_corruption_field_update,
    benchmark_scheduler_policies,
    benchmark_gpu_batching,
    benchmark_black_swan_scan,
    benchmark_kpi_buffer_update,
    benchmark_wasm_execution,
    benchmark_lua_execution,
    benchmark_hot_reload
);

criterion_main!(benches);
