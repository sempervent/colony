use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct SimClock {
    pub tick_scale: TickScale,
    pub now: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TickScale {
    RealTime,
    Seconds(u64),
    Days(u16),
    Years(u8),
}

impl SimClock {
    pub fn advance(&self) -> Duration {
        match self.tick_scale {
            TickScale::RealTime => Duration::from_millis(16),
            TickScale::Seconds(s) => Duration::from_secs(s),
            TickScale::Days(d) => Duration::from_secs(86_400 * d as u64),
            TickScale::Years(y) => Duration::from_secs(31_557_600 * y.min(10) as u64),
        }
    }

    pub fn advance_time(&mut self) {
        let duration = self.advance();
        self.now = self.now + chrono::Duration::from_std(duration).unwrap_or_default();
    }

    pub fn get_scale_display(&self) -> String {
        match self.tick_scale {
            TickScale::RealTime => "Real Time".to_string(),
            TickScale::Seconds(s) => format!("{}s", s),
            TickScale::Days(d) => format!("{}d", d),
            TickScale::Years(y) => format!("{}y", y),
        }
    }

    pub fn get_simulation_speed(&self) -> f64 {
        match self.tick_scale {
            TickScale::RealTime => 1.0,
            TickScale::Seconds(s) => s as f64,
            TickScale::Days(d) => (d as f64) * 86_400.0,
            TickScale::Years(y) => (y as f64) * 31_557_600.0,
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.tick_scale, TickScale::RealTime)
    }
}
