use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub enum WorkerReport {
    Progress {
        worker_id: u64,
        op: super::Op,
        ms: u64,
    },
    Fault {
        worker_id: u64,
        op: super::Op,
        kind: super::FaultKind,
    },
    Completed {
        job_id: u64,
    },
}
