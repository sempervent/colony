use colony_core::Workyard;

pub fn update_thermal_system(yards: &mut [Workyard], delta_time: f32) {
    for yard in yards.iter_mut() {
        // Heat generation based on power draw and utilization
        let utilization = 0.7; // Simplified - would come from actual worker utilization
        let heat_generation = yard.power_draw * utilization * delta_time * 0.01;
        
        // Heat decay
        let heat_decay = yard.heat * 0.1 * delta_time;
        
        // Update heat
        yard.heat = (yard.heat + heat_generation - heat_decay).max(0.0);
    }
}

pub fn get_thermal_throttle(heat: f32, heat_cap: f32) -> f32 {
    if heat < heat_cap * 0.85 {
        1.0
    } else {
        (heat_cap / heat).clamp(0.4, 1.0)
    }
}
