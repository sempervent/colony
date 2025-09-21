use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::{Job, WorkClass};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnqueuedJob {
    pub job: Job,
    pub enq_tick: u64,
}

impl EnqueuedJob {
    pub fn new(job: Job, enq_tick: u64) -> Self {
        Self { job, enq_tick }
    }
}

#[derive(Resource, Default, Debug, Serialize, Deserialize)]
pub struct JobQueue {
    pub cpu: Vec<EnqueuedJob>,
    pub gpu: Vec<EnqueuedJob>,
    pub io: Vec<EnqueuedJob>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            cpu: Vec::new(),
            gpu: Vec::new(),
            io: Vec::new(),
        }
    }

    pub fn push(&mut self, job: Job, tick: u64) {
        let enqueued = EnqueuedJob::new(job, tick);
        
        // Simple classification based on operations
        let has_gpu_ops = enqueued.job.pipeline.ops.iter().any(|op| {
            matches!(op, super::Op::Yolo | super::Op::Fft)
        });
        
        let has_io_ops = enqueued.job.pipeline.ops.iter().any(|op| {
            matches!(op, super::Op::UdpDemux | super::Op::HttpParse | super::Op::CanParse | super::Op::TcpSessionize)
        });
        
        if has_gpu_ops {
            self.gpu.push(enqueued);
        } else if has_io_ops {
            self.io.push(enqueued);
        } else {
            self.cpu.push(enqueued);
        }
    }

    pub fn pop_cpu(&mut self) -> Option<EnqueuedJob> {
        self.cpu.pop()
    }

    pub fn pop_gpu(&mut self) -> Option<EnqueuedJob> {
        self.gpu.pop()
    }

    pub fn pop_io(&mut self) -> Option<EnqueuedJob> {
        self.io.pop()
    }

    pub fn peek_cpu(&self) -> &[EnqueuedJob] {
        &self.cpu
    }

    pub fn peek_gpu(&self) -> &[EnqueuedJob] {
        &self.gpu
    }

    pub fn peek_io(&self) -> &[EnqueuedJob] {
        &self.io
    }

    pub fn len(&self) -> usize {
        self.cpu.len() + self.gpu.len() + self.io.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cpu.is_empty() && self.gpu.is_empty() && self.io.is_empty()
    }

    pub fn clear(&mut self) {
        self.cpu.clear();
        self.gpu.clear();
        self.io.clear();
    }
}

// Helper function to calculate starvation metric
pub fn starvation(now_tick: u64, enq_tick: u64, max_window: u64) -> f32 {
    if max_window == 0 {
        return 0.0;
    }
    (((now_tick - enq_tick) as f32) / max_window as f32).clamp(0.0, 1.0)
}

// Calculate average starvation for a queue
pub fn average_starvation(queue: &[EnqueuedJob], now_tick: u64, max_window: u64) -> f32 {
    if queue.is_empty() {
        return 0.0;
    }
    
    let total_starvation: f32 = queue
        .iter()
        .map(|enq_job| starvation(now_tick, enq_job.enq_tick, max_window))
        .sum();
    
    total_starvation / queue.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Pipeline, Op, QoS};

    fn create_test_job(id: u64) -> Job {
        Job {
            id,
            pipeline: Pipeline {
                ops: vec![Op::Decode],
                mutation_tag: None,
            },
            qos: QoS::Balanced,
            deadline_ms: 100,
            payload_sz: 1024,
        }
    }

    #[test]
    fn test_job_queue_push_pop() {
        let mut queue = JobQueue::new();
        let job = create_test_job(1);
        
        queue.push(job, 100);
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
        
        let enqueued = queue.pop_cpu().unwrap();
        assert_eq!(enqueued.job.id, 1);
        assert_eq!(enqueued.enq_tick, 100);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_starvation_calculation() {
        // No starvation
        assert_eq!(starvation(100, 100, 1000), 0.0);
        
        // Half starvation
        assert_eq!(starvation(600, 100, 1000), 0.5);
        
        // Full starvation
        assert_eq!(starvation(1100, 100, 1000), 1.0);
        
        // Beyond max window (clamped)
        assert_eq!(starvation(2000, 100, 1000), 1.0);
    }

    #[test]
    fn test_average_starvation() {
        let mut queue = JobQueue::new();
        queue.push(create_test_job(1), 100);
        queue.push(create_test_job(2), 200);
        queue.push(create_test_job(3), 300);
        
        let avg = average_starvation(&queue.cpu, 500, 1000);
        // Job 1: (500-100)/1000 = 0.4
        // Job 2: (500-200)/1000 = 0.3  
        // Job 3: (500-300)/1000 = 0.2
        // Average: (0.4 + 0.3 + 0.2) / 3 = 0.3
        assert!((avg - 0.3).abs() < 0.01);
    }
}
