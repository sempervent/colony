use bevy::prelude::*;
use crate::{Colony, Workyard, YardWorkload, DispatchScale, Worker, WorkerState, IoRolling, CorruptionField, Debts};
use crate::queue::{JobQueue, average_starvation};

pub fn power_bandwidth_system(
    mut colony: ResMut<Colony>,
    mut dispatch_scale: ResMut<DispatchScale>,
    mut io_rolling: ResMut<IoRolling>,
    yards: Query<&Workyard>,
    debts: Res<Debts>,
    clock: Res<crate::SimClock>,
) {
    let mut draw = 0.0;

    for y in &yards {
        draw += y.power_draw_kw;
    }

    // Apply debt multipliers
    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    let power_mult = debts.get_power_multiplier(current_tick);
    let bandwidth_tax = debts.get_bandwidth_tax(current_tick);
    
    colony.meters.power_draw_kw = draw * power_mult;

    // Use rolling I/O bandwidth instead of yard bandwidth shares
    let io_gbits = io_rolling.take_and_reset();
    let util = (io_gbits / colony.bandwidth_total_gbps).clamp(0.0, 1.0);
    colony.meters.bandwidth_util = util;

    let scale = if draw * power_mult > colony.power_cap_kw { 
        colony.power_cap_kw / (draw * power_mult)
    } else { 
        1.0 
    };
    dispatch_scale.0 = scale;
}

pub fn heat_system(
    mut yards: Query<(&mut Workyard, &mut YardWorkload)>,
    colony: Res<Colony>,
    debts: Res<Debts>,
    clock: Res<crate::SimClock>,
) {
    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    let heat_addition = debts.get_heat_addition(current_tick);
    
    for (mut y, mut w) in &mut yards {
        let workload_heat = w.units_this_tick * colony.tunables.heat_generated_per_unit;
        y.heat = (y.heat + workload_heat + heat_addition - colony.tunables.heat_decay_per_tick).max(20.0);
        
        // Reset workload for next tick
        w.units_this_tick = 0.0;
    }
}

pub fn corruption_system(
    mut colony: ResMut<Colony>,
    mut corruption_field: ResMut<CorruptionField>,
    mut workers: Query<&mut Worker>,
    yards: Query<&Workyard>,
    jobq: Res<JobQueue>,
    clock: Res<crate::SimClock>,
) {
    // Decay global corruption field
    corruption_field.global = (corruption_field.global - colony.corruption_tun.decay_per_tick).max(0.0);
    
    // Calculate average queue starvation
    let now_tick = clock.now.timestamp_millis() as u64 / 16; // Convert to 16ms ticks
    let max_window = 1000; // 16 seconds in ticks
    
    let cpu_starvation = average_starvation(&jobq.cpu, now_tick, max_window);
    let gpu_starvation = average_starvation(&jobq.gpu, now_tick, max_window);
    let io_starvation = average_starvation(&jobq.io, now_tick, max_window);
    let avg_starvation = (cpu_starvation + gpu_starvation + io_starvation) / 3.0;
    
    // Calculate average heat fraction
    let avg_heat_frac = if !yards.is_empty() {
        yards.iter().map(|y| y.heat / y.heat_cap).sum::<f32>() / yards.iter().count() as f32
    } else {
        0.0
    };
    
    // Increase corruption based on stress
    let stress_contribution = (
        colony.corruption_tun.heat_weight * avg_heat_frac +
        colony.corruption_tun.bw_weight * colony.meters.bandwidth_util +
        colony.corruption_tun.starvation_weight * avg_starvation
    ) * 0.001; // Small increment per tick
    
    corruption_field.global = (corruption_field.global + stress_contribution).min(1.0);
    
    // Update worker corruption
    for mut worker in workers.iter_mut() {
        let mut decay = colony.corruption_tun.worker_decay_per_tick;
        
        // Apply recovery boost for idle workers or after maintenance
        if worker.state == WorkerState::Idle {
            decay += colony.corruption_tun.recover_boost;
        }
        
        worker.corruption = (worker.corruption - decay).max(0.0);
        
        // Add stress contribution to worker corruption
        let worker_stress = (
            colony.corruption_tun.heat_weight * avg_heat_frac +
            colony.corruption_tun.bw_weight * colony.meters.bandwidth_util
        ) * 0.0005; // Smaller increment for individual workers
        
        worker.corruption = (worker.corruption + worker_stress).min(1.0);
    }
}

pub fn maintenance_system(
    mut yards: Query<(&mut Workyard, &mut YardWorkload)>,
    mut workers: Query<&mut crate::Worker>,
    mut report_reader: EventReader<crate::WorkerReport>,
) {
    for report in report_reader.read() {
        if let crate::WorkerReport::Completed { job_id } = report {
            // Check if this was a maintenance job by looking for MaintenanceCool ops
            // For now, we'll apply maintenance effects to all completed jobs
            // In a real implementation, you'd track job types
            
            // Find the yard and apply maintenance effects
            for (mut yard, _) in yards.iter_mut() {
                // Cool the yard
                yard.heat = (yard.heat - 15.0).max(20.0);
                
                // Reduce corruption for workers in this yard's isolation domain
                for mut worker in workers.iter_mut() {
                    if worker.corruption > 0.0 {
                        worker.corruption *= 0.98;
                    }
                }
                break; // Only apply to first yard for now
            }
        }
    }
}
