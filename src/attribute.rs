use std::ops::{Index, IndexMut};

use strum::EnumCount;
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
pub struct Attributes([Duration; Attribute::COUNT]);
impl Index<Attribute> for Attributes {
    type Output = Duration;

    fn index(&self, index: Attribute) -> &Self::Output { &self.0[index as usize] }
}
impl IndexMut<Attribute> for Attributes {
    fn index_mut(&mut self, index: Attribute) -> &mut Self::Output { &mut self.0[index as usize] }
}
impl Default for Attributes {
    fn default() -> Self { Self::new() }
}

impl Attributes {
    pub const fn new() -> Self { Self([Duration::hours(72); 3]) }
}
