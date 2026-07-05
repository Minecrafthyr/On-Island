use super::*;
pub struct Player {
    pub attrs: Attributes,
    pub location: Location,
    pub inventory: ItemStacks,
}

impl Player {
    pub const fn new() -> Self {
        Player {
            attrs: Attributes::new(),
            location: Location::StrandedShip,
            inventory: ItemStacks::new(),
        }
    }

    pub fn efficiency(&self) -> f64 {
        use Attribute::*;
        let mut base = 1.0;
        if self.attrs[Energy] < Time::h(24) {
            base *= self.attrs[Energy].as_f64() / Time::h(24).as_f64();
        }
        if self.attrs[Water] < Time::h(24) {
            base *= self.attrs[Water].as_f64() / Time::h(24).as_f64();
        }
        if self.attrs[Health] < Time::h(24) {
            base *= self.attrs[Health].as_f64() / Time::h(24).as_f64();
        }
        base
    }

    pub fn tick(&mut self) {
        self.attrs[Energy] -= Time(1);
        self.attrs[Water] -= Time(1);

        if self.attrs[Energy] <= Time::h(24) {
            self.attrs[Health] -= Time(1);
        }
        if self.attrs[Water] <= Time::h(24) {
            self.attrs[Health] -= Time(1);
        }

        if self.attrs[Energy] > Time::h(48)
            && self.attrs[Water] > Time::h(48)
            && self.attrs[Health] < Time::h(72)
        {
            self.attrs[Energy] -= Time(1);
            self.attrs[Health] += Time(1);
        }
    }
}
