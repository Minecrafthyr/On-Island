#![feature(const_trait_impl, const_default, derive_const, const_convert, inherent_associated_types)]
#![allow(incomplete_features)]
pub mod utils;
use utils::*;
pub mod item;
use item::*;
pub mod location;
use location::*;
pub mod attribute;
use attribute::*;
pub mod player;
use player::*;
pub mod game;
use game::*;
pub mod crafting;
pub mod effect;
pub mod statics;
pub mod ui;
pub mod units;

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "zh_CN");
fn main() -> std::io::Result<()> {
  let _guard = RawModeGuard::new()?;
  Game::new().run();
  Ok(())
}
