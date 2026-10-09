use enumflags2::BitFlags;

use crate::location::WaterYield;

#[derive(PartialEq)]
pub enum EnvCondition {
  AlwaysTrue,
  AirExposure(bool),
  Gather { dirt: Option<bool>, sand: Option<bool>, water: Option<BitFlags<WaterYield>> },
  AllOf(&'static [Self]),
  AnyOf(&'static [Self]),
}
#[derive(Clone, Copy)]
pub struct EnvContext {
  pub air: bool,
  pub dirt: bool,
  pub sand: bool,
  pub water: WaterYield,
}
