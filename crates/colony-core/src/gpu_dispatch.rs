use bevy::prelude::*;
use super::{GpuFarm, GpuBatchBuffer, GpuBatchItem, calculate_batch_timing, Worker, WorkerState, Workyard, YardWorkload, Job, Op, thermal_throttle, bandwidth_latency_multiplier, IoRolling, CorruptionField, FaultKpi, WorkerReport};
use super::corruption::CorruptionTunables;
use super::faults::{fault_inject_on_completion, handle_fault};
use super::queue::starvation;
use super::scheduler::Scheduler;
use tokio::time::Duration;

#[derive(Resource, Default)]
pub struct GpuBatchQueues {
    pub buffers: std::collections::HashMap<String, GpuBatchBuffer>,
}

impl GpuBatchQueues {
    pub fn new() -> Self {
        Self {
            buffers: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_create_buffer(&mut self, pipeline_id: &str) -> &mut GpuBatchBuffer {
        self.buffers.entry(pipeline_id.to_string()).or_insert_with(GpuBatchBuffer::new)
    }
}

pub fn gpu_dispatch_system(
    mut yards: Query<(Entity, &mut Workyard, &mut YardWorkload, &mut GpuFarm)>,
    mut workers: Query<(Entity, &mut Worker)>,
    mut jobq: ResMut<super::queue::JobQueue>,
    mut batch_queues: ResMut<GpuBatchQueues>,
    colony: Res<super::Colony>,
    dispatch_scale: Res<super::DispatchScale>,
    mut io_rolling: ResMut<IoRolling>,
    corruption_field: Res<CorruptionField>,
    clock: Res<super::SimClock>,
    mut report_writer: EventWriter<WorkerReport>,
) {
    for (yard_e, mut yard, mut workload, mut gpu_farm) in yards.iter_mut() {
        if yard.kind != super::WorkyardKind::GpuFarm {
            continue;
        }

        let available_workers: Vec<Entity> = workers
            .iter()
            .filter(|(_, worker)| worker.state == WorkerState::Idle && worker.class == super::WorkClass::Gpu)
            .map(|(entity, _)| entity)
            .collect();

        if available_workers.is_empty() {
            continue;
        }

        // Get GPU jobs from the queue
        let jobs = jobq.peek_gpu();
        if jobs.is_empty() {
            continue;
        }

        // Process jobs for batching
        for enqueued_job in jobs.iter() {
            let job = &enqueued_job.job;
            
            // Check if this job has GPU operations
            let has_gpu_ops = job.pipeline.ops.iter().any(|op| {
                matches!(op, Op::GpuPreprocess | Op::Yolo | Op::GpuExport)
            });

            if !has_gpu_ops {
                continue;
            }

            // Find the first GPU operation
            if let Some(gpu_op) = job.pipeline.ops.iter().find(|op| {
                matches!(op, Op::GpuPreprocess | Op::Yolo | Op::GpuExport)
            }) {
                let pipeline_id = format!("gpu_pipeline_{}", job.id);
                let buffer = batch_queues.get_or_create_buffer(&pipeline_id);

                // Check VRAM constraints
                let item_vram = gpu_op.vram_needed_mb(job.payload_sz);
                let current_vram = buffer.total_vram_mb();
                let vram_limit_mb = gpu_farm.per_gpu.vram_gb * 1024.0;

                if current_vram + item_vram > vram_limit_mb {
                    // VRAM limit exceeded, skip this job for now
                    continue;
                }

                // Add to batch buffer
                let now_tick = clock.now.timestamp_millis() as u64 / 16;
                buffer.add_item(GpuBatchItem {
                    job_id: job.id,
                    op: gpu_op.clone(),
                    payload_sz: job.payload_sz,
                    enqueue_tick: enqueued_job.enq_tick,
                });

                // Check if we should flush the batch
                if buffer.should_flush(&gpu_farm.per_gpu, now_tick) {
                    process_gpu_batch(
                        &mut gpu_farm,
                        &mut workers,
                        &mut workload,
                        buffer,
                        &colony,
                        &dispatch_scale,
                        &mut io_rolling,
                        &corruption_field,
                        &clock,
                        &mut report_writer,
                    );

                    // Remove the job from the queue
                    jobq.gpu.retain(|ej| ej.job.id != job.id);
                }
            }
        }
    }
}

fn process_gpu_batch(
    gpu_farm: &mut GpuFarm,
    workers: &mut Query<(Entity, &mut Worker)>,
    workload: &mut YardWorkload,
    batch: &mut GpuBatchBuffer,
    colony: &super::Colony,
    dispatch_scale: &super::DispatchScale,
    io_rolling: &mut IoRolling,
    corruption_field: &CorruptionField,
    clock: &super::SimClock,
    report_writer: &mut EventWriter<WorkerReport>,
) {
    if batch.items.is_empty() {
        return;
    }

    // Find an available GPU worker
    let worker_entity = workers
        .iter()
        .find(|(_, worker)| worker.state == WorkerState::Idle && worker.class == super::WorkClass::Gpu)
        .map(|(entity, _)| entity);

    if let Some(worker_entity) = worker_entity {
        let (_, mut worker) = workers.get_mut(worker_entity).unwrap();
        worker.state = WorkerState::Running;

        // Calculate batch timing
        let is_first_batch = gpu_farm.meters.batches_inflight == 0;
        let exec_ms = calculate_batch_timing(batch, &gpu_farm.per_gpu, &gpu_farm.flags, is_first_batch);

        // Apply thermal throttling
        let throttle = thermal_throttle(
            gpu_farm.meters.util * 100.0, // Convert util to heat-like value
            gpu_farm.per_gpu.vram_gb * 10.0, // Convert VRAM to heat cap-like value
            colony.tunables.thermal_throttle_knee,
            colony.tunables.thermal_min_throttle
        );

        // Apply power scaling
        let power_scale = dispatch_scale.0;

        // Apply bandwidth latency multiplier
        let bw_mult = bandwidth_latency_multiplier(
            colony.meters.bandwidth_util,
            colony.tunables.bandwidth_tail_exp
        );

        // Calculate final execution time with throttling
        let final_exec_ms = exec_ms * throttle * power_scale / bw_mult;

        // Calculate work units for heat generation
        let mut total_work_units = 0.0;
        for item in &batch.items {
            total_work_units += item.op.work_units();
            // Track I/O bandwidth for GPU operations
            match item.op {
                Op::GpuPreprocess | Op::GpuExport => {
                    io_rolling.add_bytes(item.payload_sz);
                }
                _ => {}
            }
        }
        workload.units_this_tick += total_work_units;

        // Calculate queue starvation for fault injection
        let now_tick = clock.now.timestamp_millis() as u64 / 16;
        let enq_tick = batch.first_enqueue_tick.unwrap_or(now_tick);
        let queue_starvation = starvation(now_tick, enq_tick, 1000);

        // Check for fault injection (batch-level)
        let fault = fault_inject_on_completion(
            worker,
            &super::Workyard {
                kind: super::WorkyardKind::GpuFarm,
                slots: 2,
                heat: gpu_farm.meters.util * 100.0,
                heat_cap: gpu_farm.per_gpu.vram_gb * 10.0,
                power_draw_kw: 300.0,
                bandwidth_share: 0.4,
                isolation_domain: 1,
            },
            &batch.items[0].op,
            corruption_field.global,
            colony.meters.bandwidth_util,
            queue_starvation,
            &colony.corruption_tun,
            colony.seed,
            now_tick,
        );

        if let Some(fault_kind) = fault {
            // Handle batch-level fault
            handle_fault(
                fault_kind,
                &mut worker,
                batch.items[0].job_id,
                batch.items[0].op.clone(),
                &colony.corruption_tun,
                report_writer,
            );
        } else {
            // Normal batch completion
            for item in &batch.items {
                report_writer.send(WorkerReport::Completed { job_id: item.job_id });
            }
        }

        // Update GPU meters
        gpu_farm.meters.util = (final_exec_ms / 16.0).min(1.0); // 16ms tick window
        gpu_farm.meters.vram_used_gb = batch.total_vram_mb() / 1024.0;
        gpu_farm.meters.batches_inflight += 1;
        
        // Update batch latency EWMA
        let alpha = 0.1; // EWMA smoothing factor
        gpu_farm.meters.batch_latency_ms = alpha * final_exec_ms + (1.0 - alpha) * gpu_farm.meters.batch_latency_ms;

        worker.state = WorkerState::Idle;
    }

    // Clear the batch buffer
    batch.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{WorkyardKind, WorkClass};

    #[test]
    fn test_gpu_batch_queues() {
        let mut queues = GpuBatchQueues::new();
        let buffer = queues.get_or_create_buffer("test_pipeline");
        assert_eq!(buffer.items.len(), 0);
    }

    #[test]
    fn test_vram_constraint_checking() {
        let mut batch = GpuBatchBuffer::new();
        batch.add_item(GpuBatchItem {
            job_id: 1,
            op: Op::Yolo,
            payload_sz: 1024,
            enqueue_tick: 100,
        });

        let vram_mb = batch.total_vram_mb();
        assert!(vram_mb > 0.0);
    }
}
