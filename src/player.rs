use time::Duration;

use super::*;

pub struct Player {
    pub attrs: Attributes,
    pub location: Location,
    pub inventory: ItemStacks,
    pub activity: f64,
}

impl Default for Player {
    fn default() -> Self { Self::new() }
}

impl Player {
    pub const fn new() -> Self {
        Player {
            attrs: Attributes::new(),
            location: Location::StrandedShip,
            inventory: ItemStacks::new(),
            activity: 1.0,
        }
    }

    pub fn efficiency(&self) -> f64 {
        use Attribute::*;
        let mut base = 1.0;
        if self.attrs[Energy] < Duration::hours(24) {
            base *= self.attrs[Energy].as_seconds_f64() / Duration::hours(24).as_seconds_f64();
        }
        if self.attrs[Water] < Duration::hours(24) {
            base *= self.attrs[Water].as_seconds_f64() / Duration::hours(24).as_seconds_f64();
        }
        if self.attrs[Health] < Duration::hours(24) {
            base *= self.attrs[Health].as_seconds_f64() / Duration::hours(24).as_seconds_f64();
        }
        base
    }

    pub fn tick(&mut self, activity: f64) {
        let dur = Duration::MILLISECOND * activity;
        self.attrs[Energy] -= dur;
        self.attrs[Water] -= dur;

        if self.attrs[Energy] <= Duration::hours(24) {
            self.attrs[Health] -= dur;
        }
        if self.attrs[Water] <= Duration::hours(24) {
            self.attrs[Health] -= dur;
        }

        if self.attrs[Energy] > Duration::hours(48)
            && self.attrs[Water] > Duration::hours(48)
            && self.attrs[Health] < Duration::hours(72)
        {
            self.attrs[Energy] -= dur;
            self.attrs[Health] += dur;
        }
    }
}
