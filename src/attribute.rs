use super::*;
#[derive(Debug, EnumIter, EnumCount, Clone, Copy)]
pub enum Attribute {
    Health,
    Energy,
    Water,
}
pub use Attribute::*;

impl Attribute {
    pub fn name(&self) -> &'static str {
        use Attribute::*;
        match self {
        Health => "血量",
        Energy => "能量",
        Water => "水分",
        }
    }
}

#[derive(Debug)]
pub struct Attributes([Time; Attribute::COUNT]);
impl Index<Attribute> for Attributes {
    type Output = Time;

    fn index(&self, index: Attribute) -> &Self::Output { &self.0[index as usize] }
}
impl IndexMut<Attribute> for Attributes {
    fn index_mut(&mut self, index: Attribute) -> &mut Self::Output { &mut self.0[index as usize] }
}
impl Attributes {
    pub const fn new() -> Self { Self([Time::h(72), Time::h(72), Time::h(72)]) }
}
