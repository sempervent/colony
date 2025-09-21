use colony_core::{Colony, Worker};

pub fn update_corruption_system(colony: &mut Colony, workers: &mut [Worker], delta_time: f32) {
    // Global corruption field decay
    colony.corruption_field = (colony.corruption_field - 0.01 * delta_time).max(0.0);
    
    // Per-worker corruption updates
    for worker in workers.iter_mut() {
        // Corruption increases with thermal stress and decreases with discipline
        let thermal_stress = if worker.state == colony_core::WorkerState::Running { 0.1 } else { 0.0 };
        let discipline_factor = 1.0 - worker.discipline * 0.5;
        
        let corruption_delta = (thermal_stress * discipline_factor - worker.corruption * 0.1) * delta_time;
        worker.corruption = (worker.corruption + corruption_delta).clamp(0.0, 1.0);
    }
}

pub fn get_corruption_penalty(corruption: f32) -> f32 {
    // Corruption reduces performance and increases fault rates
    1.0 - corruption * 0.3
}
