use super::*;

impl Game {
    pub fn time_pass(&mut self, time: Time) {
        for _ in 0..=time.0 {
            self.tick_ms();
        }
    }

    pub fn action_time_pass(&mut self, time: Time) -> u64 {
        let mut actual_ms = (time.as_f64() / self.player.efficiency()) as u64;
        while actual_ms != 0 {
            self.tick_ms();
            actual_ms = (actual_ms as f64 / self.player.efficiency()) as u64;
            actual_ms -= 1;
        }
        return 0;
    }
}
