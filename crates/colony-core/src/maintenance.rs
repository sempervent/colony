use bevy::prelude::*;
use crate::{Job, Pipeline, Op, QoS, JobQueue};

pub fn enqueue_maintenance(yard_entity: Entity, jobq: &mut JobQueue) {
    let maintenance_job = Job {
        id: chrono::Utc::now().timestamp_millis() as u64,
        pipeline: Pipeline {
            ops: vec![Op::MaintenanceCool],
            mutation_tag: Some("maintenance".to_string()),
        },
        qos: QoS::Balanced,
        deadline_ms: 5000, // 5 second deadline for maintenance
        payload_sz: 0, // No payload for maintenance
    };
    
    jobq.push(maintenance_job, 0); // TODO: Pass actual current tick
}
