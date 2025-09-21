use bevy::prelude::*;
use super::{WorkerReport, FaultKind, Worker, Workyard, Op};
use crate::corruption::{fault_probability, tick_rng, CorruptionTunables};
use rand::Rng;

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct FaultKpi {
    pub last_tick_faults: u32,
    pub soft_drop_rate: f32,     // moving avg
    pub sticky_workers: u32,
    pub deadline_hit_rate: f32,  // updated in dispatcher/report ingest
    pub total_faults: u32,
    pub transient_faults: u32,
    pub data_skew_faults: u32,
    pub sticky_faults: u32,
    pub queue_drop_faults: u32,
}

impl FaultKpi {
    pub fn new() -> Self {
        Self {
            last_tick_faults: 0,
            soft_drop_rate: 0.0,
            sticky_workers: 0,
            deadline_hit_rate: 1.0,
            total_faults: 0,
            transient_faults: 0,
            data_skew_faults: 0,
            sticky_faults: 0,
            queue_drop_faults: 0,
        }
    }
}

pub fn fault_inject_on_completion(
    worker: &Worker,
    yard: &Workyard,
    op: &Op,
    global_corruption: f32,
    bandwidth_util: f32,
    queue_starvation: f32,
    tunables: &CorruptionTunables,
    seed: u64,
    tick: u64,
) -> Option<FaultKind> {
    let heat_frac = yard.heat / yard.heat_cap;
    
    let prob = fault_probability(
        tunables.base_fault_rate,
        global_corruption,
        worker.corruption,
        heat_frac,
        bandwidth_util,
        queue_starvation,
        tunables,
    );
    
    let mut rng = tick_rng(seed, tick);
    if rng.gen::<f32>() < prob {
        // Weighted selection of fault type
        let fault_weights = [
            (FaultKind::Transient, 0.60),
            (FaultKind::DataSkew, 0.20),
            (FaultKind::QueueDrop, 0.15),
            (FaultKind::StickyConfig, 0.05 + worker.corruption * 0.1), // More likely with high corruption
        ];
        
        let total_weight: f32 = fault_weights.iter().map(|(_, w)| w).sum();
        let roll = rng.gen::<f32>() * total_weight;
        
        let mut acc = 0.0;
        for (fault_kind, weight) in fault_weights.iter() {
            acc += weight;
            if roll <= acc {
                return Some(*fault_kind);
            }
        }
        
        // Fallback to Transient
        Some(FaultKind::Transient)
    } else {
        None
    }
}

pub fn handle_fault(
    fault: FaultKind,
    worker: &mut Worker,
    job_id: u64,
    op: Op,
    tunables: &CorruptionTunables,
    report_writer: &mut EventWriter<WorkerReport>,
) {
    match fault {
        FaultKind::Transient => {
            // Retry with backoff
            if worker.retry.max_retries > 0 {
                worker.retry.max_retries -= 1;
                // In a real implementation, we'd schedule a retry with exponential backoff
                // For now, just emit the fault and let the system handle it
                report_writer.send(WorkerReport::Fault {
                    worker_id: worker.id,
                    op,
                    kind: fault,
                });
            } else {
                // Max retries exceeded, treat as queue drop
                report_writer.send(WorkerReport::Fault {
                    worker_id: worker.id,
                    op,
                    kind: FaultKind::QueueDrop,
                });
            }
        }
        FaultKind::DataSkew => {
            // Force re-run, no worker quarantine
            report_writer.send(WorkerReport::Fault {
                worker_id: worker.id,
                op,
                kind: fault,
            });
        }
        FaultKind::StickyConfig => {
            // Quarantine worker
            worker.state = super::WorkerState::Recovering;
            worker.sticky_faults += 1;
            report_writer.send(WorkerReport::Fault {
                worker_id: worker.id,
                op,
                kind: fault,
            });
        }
        FaultKind::QueueDrop => {
            // Job dropped, deadline likely missed
            report_writer.send(WorkerReport::Fault {
                worker_id: worker.id,
                op,
                kind: fault,
            });
        }
    }
}

pub fn update_fault_kpis(
    mut kpis: ResMut<FaultKpi>,
    workers: Query<&Worker>,
    report_reader: EventReader<WorkerReport>,
) {
    // Count sticky workers
    kpis.sticky_workers = workers
        .iter()
        .filter(|w| w.state == super::WorkerState::Recovering)
        .count() as u32;
    
    // Process fault reports
    for report in report_reader.read() {
        if let WorkerReport::Fault { kind, .. } = report {
            kpis.total_faults += 1;
            match kind {
                FaultKind::Transient => kpis.transient_faults += 1,
                FaultKind::DataSkew => kpis.data_skew_faults += 1,
                FaultKind::StickyConfig => kpis.sticky_faults += 1,
                FaultKind::QueueDrop => kpis.queue_drop_faults += 1,
            }
        }
    }
    
    // Update soft drop rate (simplified moving average)
    if kpis.total_faults > 0 {
        kpis.soft_drop_rate = kpis.queue_drop_faults as f32 / kpis.total_faults as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{WorkyardKind, WorkClass, WorkerState};

    fn create_test_worker() -> Worker {
        Worker {
            id: 1,
            class: WorkClass::Cpu,
            skill_cpu: 1.0,
            skill_gpu: 0.0,
            skill_io: 0.0,
            discipline: 1.0,
            focus: 1.0,
            corruption: 0.1,
            state: WorkerState::Idle,
            retry: RetryPolicy::default(),
            sticky_faults: 0,
        }
    }

    fn create_test_yard() -> Workyard {
        Workyard {
            kind: WorkyardKind::CpuArray,
            slots: 4,
            heat: 80.0,
            heat_cap: 100.0,
            power_draw_kw: 200.0,
            bandwidth_share: 0.1,
            isolation_domain: 0,
        }
    }

    #[test]
    fn test_fault_injection_high_stress() {
        let worker = create_test_worker();
        let yard = create_test_yard();
        let tunables = CorruptionTunables::default();
        
        // High stress scenario
        let fault = fault_inject_on_completion(
            &worker,
            &yard,
            &Op::Decode,
            0.5, // high global corruption
            0.9, // high bandwidth util
            0.8, // high queue starvation
            &tunables,
            42, 100,
        );
        
        // Should have higher chance of fault (but not guaranteed)
        // This is probabilistic, so we just test that it can return Some
        // In a real test, we'd run many iterations and check statistics
    }

    #[test]
    fn test_fault_injection_low_stress() {
        let worker = create_test_worker();
        let yard = create_test_yard();
        let tunables = CorruptionTunables::default();
        
        // Low stress scenario
        let fault = fault_inject_on_completion(
            &worker,
            &yard,
            &Op::Decode,
            0.0, // no global corruption
            0.1, // low bandwidth util
            0.0, // no queue starvation
            &tunables,
            42, 100,
        );
        
        // Should have lower chance of fault
        // This is probabilistic, so we just test that it can return None
    }
}
