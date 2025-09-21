use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SlaWindow {
    pub window_days: u32,
    pub hits: u64,
    pub total: u64,
    pub miss_pct: f32, // computed
}

impl SlaWindow {
    pub fn new(window_days: u32) -> Self {
        Self {
            window_days,
            hits: 0,
            total: 0,
            miss_pct: 0.0,
        }
    }

    pub fn add_result(&mut self, hit: bool) {
        self.total += 1;
        if hit {
            self.hits += 1;
        }
        self.miss_pct = if self.total > 0 {
            ((self.total - self.hits) as f32 / self.total as f32) * 100.0
        } else {
            0.0
        };
    }

    pub fn hit_rate(&self) -> f32 {
        if self.total > 0 {
            (self.hits as f32 / self.total as f32) * 100.0
        } else {
            100.0
        }
    }

    pub fn meets_threshold(&self, min_hit_pct: f32) -> bool {
        self.hit_rate() >= min_hit_pct
    }
}

#[derive(bevy::prelude::Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct WinLossState {
    pub achieved_days: u32,             // consecutive days meeting SLA
    pub doom: bool,
    pub victory: bool,
    pub score: i64,                     // composite metric
    pub doom_reason: Option<String>,    // reason for loss
    pub victory_time: Option<u64>,      // tick when victory achieved
    pub doom_time: Option<u64>,         // tick when doom occurred
}

impl WinLossState {
    pub fn new() -> Self {
        Self {
            achieved_days: 0,
            doom: false,
            victory: false,
            score: 0,
            doom_reason: None,
            victory_time: None,
            doom_time: None,
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.doom || self.victory
    }
}

#[derive(bevy::prelude::Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct SlaTracker {
    pub windows: VecDeque<SlaWindow>,
    pub current_window: SlaWindow,
    pub window_size_days: u32,
    pub ticks_per_day: u64,
}

impl SlaTracker {
    pub fn new(window_size_days: u32, ticks_per_day: u64) -> Self {
        Self {
            windows: VecDeque::new(),
            current_window: SlaWindow::new(window_size_days),
            window_size_days,
            ticks_per_day,
        }
    }

    pub fn add_deadline_result(&mut self, hit: bool, current_tick: u64) {
        self.current_window.add_result(hit);
        
        // Check if we need to advance to a new window
        let current_day = current_tick / self.ticks_per_day;
        if current_day >= self.window_size_days as u64 {
            // Move current window to history and start new one
            self.windows.push_back(self.current_window.clone());
            if self.windows.len() > 10 { // Keep only last 10 windows
                self.windows.pop_front();
            }
            self.current_window = SlaWindow::new(self.window_size_days);
        }
    }

    pub fn get_recent_hit_rate(&self) -> f32 {
        if self.windows.is_empty() {
            self.current_window.hit_rate()
        } else {
            let total_hits: u64 = self.windows.iter().map(|w| w.hits).sum();
            let total_attempts: u64 = self.windows.iter().map(|w| w.total).sum();
            if total_attempts > 0 {
                (total_hits as f32 / total_attempts as f32) * 100.0
            } else {
                100.0
            }
        }
    }

    pub fn get_consecutive_good_days(&self, min_hit_pct: f32) -> u32 {
        let mut consecutive = 0;
        for window in self.windows.iter().rev() {
            if window.meets_threshold(min_hit_pct) {
                consecutive += 1;
            } else {
                break;
            }
        }
        consecutive
    }
}

pub fn update_sla_window(
    mut sla_tracker: ResMut<SlaTracker>,
    clock: Res<super::SimClock>,
    // TODO: Add event reader for deadline hits/misses
) {
    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    
    // For now, simulate some deadline results
    // In a real implementation, this would read from actual job completion events
    let simulated_hit = (current_tick % 10) != 0; // 90% hit rate
    sla_tracker.add_deadline_result(simulated_hit, current_tick);
}

pub fn eval_victory(
    victory_rules: &super::game_config::VictoryRules,
    sla_tracker: &SlaTracker,
    corruption_field: f32,
    current_tick: u64,
    ticks_per_day: u64,
) -> bool {
    // Check corruption threshold
    if corruption_field > victory_rules.max_corruption_field {
        return false;
    }

    // Check SLA threshold
    let recent_hit_rate = sla_tracker.get_recent_hit_rate();
    if recent_hit_rate < victory_rules.min_deadline_hit_pct {
        return false;
    }

    // Check consecutive good days
    let consecutive_days = sla_tracker.get_consecutive_good_days(victory_rules.min_deadline_hit_pct);
    consecutive_days >= victory_rules.target_uptime_days
}

pub fn eval_loss(
    loss_rules: &super::game_config::LossRules,
    colony: &super::Colony,
    fault_kpis: &super::FaultKpi,
    black_swan_index: &super::BlackSwanIndex,
    current_tick: u64,
    ticks_per_day: u64,
) -> (bool, Option<String>) {
    // Check power deficit
    if colony.meters.power_draw_kw > colony.power_cap_kw {
        // TODO: Track consecutive power deficit ticks
        // For now, just check current state
        if colony.meters.power_draw_kw > colony.power_cap_kw * 1.1 {
            return (true, Some("Power deficit exceeded".to_string()));
        }
    }

    // Check sticky workers
    if fault_kpis.sticky_workers > loss_rules.max_sticky_workers {
        return (true, Some("Too many sticky workers".to_string()));
    }

    // Check Black Swan chain length
    if black_swan_index.meters.active.len() > loss_rules.black_swan_chain_len as usize {
        return (true, Some("Black Swan chain too long".to_string()));
    }

    // Check time limit
    if let Some(time_limit_days) = loss_rules.time_limit_days {
        let current_day = current_tick / ticks_per_day;
        if current_day > time_limit_days as u64 {
            return (true, Some("Time limit exceeded".to_string()));
        }
    }

    (false, None)
}

pub fn compute_score(
    victory_rules: &super::game_config::VictoryRules,
    sla_tracker: &SlaTracker,
    research_state: &super::ResearchState,
    current_tick: u64,
    ticks_per_day: u64,
) -> i64 {
    let mut score = 0i64;

    // Base score for victory
    score += 1000;

    // Bonus for high SLA
    let hit_rate = sla_tracker.get_recent_hit_rate();
    score += (hit_rate * 10.0) as i64;

    // Bonus for research
    score += research_state.acquired.len() as i64 * 100;

    // Bonus for speed (faster victory = higher score)
    let days_taken = current_tick / ticks_per_day;
    if days_taken < victory_rules.target_uptime_days as u64 {
        let speed_bonus = (victory_rules.target_uptime_days as u64 - days_taken) * 10;
        score += speed_bonus as i64;
    }

    // Penalty for corruption
    // TODO: Add corruption penalty when corruption field is available

    score
}

pub fn win_loss_system(
    mut win_loss_state: ResMut<WinLossState>,
    mut sla_tracker: ResMut<SlaTracker>,
    colony: Res<super::Colony>,
    fault_kpis: Res<super::FaultKpi>,
    black_swan_index: Res<super::BlackSwanIndex>,
    research_state: Res<super::ResearchState>,
    clock: Res<super::SimClock>,
    // TODO: Add game setup resource to get victory/loss rules
) {
    if win_loss_state.is_game_over() {
        return;
    }

    let current_tick = clock.now.timestamp_millis() as u64 / 16;
    let ticks_per_day = 86400000 / 16; // 1 day in 16ms ticks

    // For now, use default victory/loss rules
    // In a real implementation, these would come from the game setup
    let victory_rules = super::game_config::VictoryRules::default();
    let loss_rules = super::game_config::LossRules::default();

    // Check for victory
    if eval_victory(&victory_rules, &sla_tracker, colony.corruption_field, current_tick, ticks_per_day) {
        win_loss_state.victory = true;
        win_loss_state.victory_time = Some(current_tick);
        win_loss_state.score = compute_score(&victory_rules, &sla_tracker, &research_state, current_tick, ticks_per_day);
        println!("VICTORY! Score: {}", win_loss_state.score);
    }

    // Check for loss
    let (is_doom, doom_reason) = eval_loss(&loss_rules, &colony, &fault_kpis, &black_swan_index, current_tick, ticks_per_day);
    if is_doom {
        win_loss_state.doom = true;
        win_loss_state.doom_time = Some(current_tick);
        win_loss_state.doom_reason = doom_reason;
        println!("DOOM! Reason: {:?}", win_loss_state.doom_reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_window() {
        let mut window = SlaWindow::new(7);
        assert_eq!(window.hit_rate(), 100.0);
        
        window.add_result(true);
        window.add_result(false);
        window.add_result(true);
        
        assert_eq!(window.hits, 2);
        assert_eq!(window.total, 3);
        assert_eq!(window.hit_rate(), 66.66667);
    }

    #[test]
    fn test_sla_tracker() {
        let mut tracker = SlaTracker::new(7, 1000);
        
        for i in 0..10 {
            tracker.add_deadline_result(i % 3 != 0, i); // ~67% hit rate
        }
        
        assert!(tracker.current_window.total > 0);
    }

    #[test]
    fn test_win_loss_state() {
        let mut state = WinLossState::new();
        assert!(!state.is_game_over());
        
        state.victory = true;
        assert!(state.is_game_over());
    }

    #[test]
    fn test_victory_evaluation() {
        let victory_rules = super::super::game_config::VictoryRules::default();
        let mut tracker = SlaTracker::new(7, 1000);
        
        // Add some good results
        for _ in 0..100 {
            tracker.add_deadline_result(true, 0);
        }
        
        let corruption_field = 0.2; // Below threshold
        let current_tick = 0;
        let ticks_per_day = 1000;
        
        // Should not be victory yet (need consecutive days)
        assert!(!eval_victory(&victory_rules, &tracker, corruption_field, current_tick, ticks_per_day));
    }
}
