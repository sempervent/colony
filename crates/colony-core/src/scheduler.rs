use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use super::{Job, Workyard, Worker};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SchedPolicy { 
    Fcfs, 
    Sjf, 
    Edf 
}

impl std::fmt::Display for SchedPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedPolicy::Fcfs => write!(f, "FCFS"),
            SchedPolicy::Sjf => write!(f, "SJF"),
            SchedPolicy::Edf => write!(f, "EDF"),
        }
    }
}

pub trait Scheduler: Send + Sync {
    fn pick(&self, yard: &Workyard, queue: &[Job], workers: &[(Entity, &Worker)]) -> Vec<(Entity, Job)>;
    fn name(&self) -> &'static str;
}

pub struct Fcfs;
pub struct Sjf;
pub struct Edf;

impl Scheduler for Fcfs {
    fn pick(&self, _y: &Workyard, q: &[Job], w: &[(Entity, &Worker)]) -> Vec<(Entity, Job)> {
        let mut out = Vec::new();
        for ((we, _), j) in w.iter().zip(q.iter()) { 
            out.push((*we, j.clone())); 
        }
        out
    }
    
    fn name(&self) -> &'static str { "FCFS" }
}

impl Scheduler for Sjf {
    fn pick(&self, _y: &Workyard, q: &[Job], w: &[(Entity, &Worker)]) -> Vec<(Entity, Job)> {
        let mut jobs = q.to_vec();
        jobs.sort_by_key(|j| j.pipeline.ops.iter().map(|op| op.cost_ms() as u32).sum::<u32>());
        let mut out = Vec::new();
        for ((we, _), j) in w.iter().zip(jobs.into_iter()) { 
            out.push((*we, j)); 
        }
        out
    }
    
    fn name(&self) -> &'static str { "SJF" }
}

impl Scheduler for Edf {
    fn pick(&self, _y: &Workyard, q: &[Job], w: &[(Entity, &Worker)]) -> Vec<(Entity, Job)> {
        let mut jobs = q.to_vec();
        jobs.sort_by_key(|j| j.deadline_ms);
        let mut out = Vec::new();
        for ((we, _), j) in w.iter().zip(jobs.into_iter()) { 
            out.push((*we, j)); 
        }
        out
    }
    
    fn name(&self) -> &'static str { "EDF" }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ActiveScheduler { 
    pub policy: SchedPolicy 
}

impl Default for ActiveScheduler {
    fn default() -> Self {
        Self { policy: SchedPolicy::Fcfs }
    }
}

impl ActiveScheduler {
    pub fn get_scheduler(&self) -> Box<dyn Scheduler> {
        match self.policy {
            SchedPolicy::Fcfs => Box::new(Fcfs),
            SchedPolicy::Sjf => Box::new(Sjf),
            SchedPolicy::Edf => Box::new(Edf),
        }
    }
    
    pub fn new_fcfs() -> Self {
        Self { policy: SchedPolicy::Fcfs }
    }
    
    pub fn new_sjf() -> Self {
        Self { policy: SchedPolicy::Sjf }
    }
    
    pub fn new_edf() -> Self {
        Self { policy: SchedPolicy::Edf }
    }
    
    pub fn get_name(&self) -> &'static str {
        match self.policy {
            SchedPolicy::Fcfs => "FCFS",
            SchedPolicy::Sjf => "SJF",
            SchedPolicy::Edf => "EDF",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Pipeline, QoS};

    fn create_test_job(id: u64, cost_ms: u32, deadline_ms: u64) -> Job {
        Job {
            id,
            pipeline: Pipeline {
                ops: vec![Op::Decode], // Simple op for testing
                mutation_tag: None,
            },
            qos: QoS::Balanced,
            deadline_ms,
            payload_sz: 1024,
        }
    }

    fn create_test_worker(id: u64) -> Worker {
        Worker {
            id,
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
        }
    }

    #[test]
    fn test_fcfs_preserves_order() {
        let fcfs = Fcfs;
        let yard = Workyard {
            kind: WorkyardKind::CpuArray,
            slots: 4,
            heat: 50.0,
            heat_cap: 100.0,
            power_draw_kw: 200.0,
            bandwidth_share: 0.1,
            isolation_domain: 0,
        };
        
        let jobs = vec![
            create_test_job(1, 5, 100),
            create_test_job(2, 1, 50),
            create_test_job(3, 3, 200),
        ];
        
        let workers = vec![
            (Entity::from_raw(1), &create_test_worker(1)),
            (Entity::from_raw(2), &create_test_worker(2)),
        ];
        
        let picks = fcfs.pick(&yard, &jobs, &workers);
        
        // FCFS should preserve original order
        assert_eq!(picks[0].1.id, 1);
        assert_eq!(picks[1].1.id, 2);
    }

    #[test]
    fn test_sjf_sorts_by_cost() {
        let sjf = Sjf;
        let yard = Workyard {
            kind: WorkyardKind::CpuArray,
            slots: 4,
            heat: 50.0,
            heat_cap: 100.0,
            power_draw_kw: 200.0,
            bandwidth_share: 0.1,
            isolation_domain: 0,
        };
        
        let jobs = vec![
            create_test_job(1, 5, 100),
            create_test_job(2, 1, 50),
            create_test_job(3, 3, 200),
        ];
        
        let workers = vec![
            (Entity::from_raw(1), &create_test_worker(1)),
            (Entity::from_raw(2), &create_test_worker(2)),
        ];
        
        let picks = sjf.pick(&yard, &jobs, &workers);
        
        // SJF should sort by cost (1, 3, 5)
        assert_eq!(picks[0].1.id, 2); // cost 1
        assert_eq!(picks[1].1.id, 3); // cost 3
    }

    #[test]
    fn test_edf_sorts_by_deadline() {
        let edf = Edf;
        let yard = Workyard {
            kind: WorkyardKind::CpuArray,
            slots: 4,
            heat: 50.0,
            heat_cap: 100.0,
            power_draw_kw: 200.0,
            bandwidth_share: 0.1,
            isolation_domain: 0,
        };
        
        let jobs = vec![
            create_test_job(1, 5, 100),
            create_test_job(2, 1, 50),
            create_test_job(3, 3, 200),
        ];
        
        let workers = vec![
            (Entity::from_raw(1), &create_test_worker(1)),
            (Entity::from_raw(2), &create_test_worker(2)),
        ];
        
        let picks = edf.pick(&yard, &jobs, &workers);
        
        // EDF should sort by deadline (50, 100, 200)
        assert_eq!(picks[0].1.id, 2); // deadline 50
        assert_eq!(picks[1].1.id, 1); // deadline 100
    }
}