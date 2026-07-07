use time::Duration;

use super::*;

impl Game {
    pub fn time_pass(&mut self, time: Duration) {
        if time.is_zero() {
            return;
        }
        for _ in 0..=time.whole_milliseconds() {
            self.tick_ms();
        }
    }

    pub fn action_time_pass(&mut self, time: Duration, activity: f64) -> Duration {
        let mut progress = Duration::ZERO;
        if time.is_zero() {
            return time;
        }
        self.player.activity = activity;
        while progress < time {
            self.tick_ms();
            let step = Duration::MILLISECOND * self.player.efficiency();
            if step.is_zero() {
                return progress;
            }
            progress += step;
        }
        self.player.activity = 1.0;
        Duration::ZERO
    }
}
