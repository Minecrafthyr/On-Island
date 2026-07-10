use std::ops::{Index, IndexMut};

use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

use super::*;
#[derive(Debug, EnumIter, EnumCount, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Attribute {
    Health,
    Energy,
    Water,
}
pub use Attribute::*;

impl NameAndDesc for Attribute {
    const PREFIX: &str = "attribute";

    fn get_id(&self) -> &str { self.into() }
}

#[derive(Debug)]
pub struct Attributes {
    pub health: Duration,
    pub energy: Duration,
    pub water: Duration,
}
impl Index<Attribute> for Attributes {
    type Output = Duration;

    fn index(&self, index: Attribute) -> &Self::Output {
        match index {
        Attribute::Health => &self.health,
        Attribute::Energy => &self.energy,
        Attribute::Water => &self.water,
        }
    }
}
impl IndexMut<Attribute> for Attributes {
    fn index_mut(&mut self, index: Attribute) -> &mut Self::Output {
        match index {
        Attribute::Health => &mut self.health,
        Attribute::Energy => &mut self.energy,
        Attribute::Water => &mut self.water,
        }
    }
}
impl Default for Attributes {
    fn default() -> Self { Self::new() }
}

impl Attributes {
    pub const fn new() -> Self {
        Self {
            health: Duration::hours(72),
            energy: Duration::hours(72),
            water: Duration::hours(72),
        }
    }
}
