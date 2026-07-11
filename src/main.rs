#![feature(const_trait_impl, const_default, derive_const, const_convert, inherent_associated_types)]
#![allow(incomplete_features)]

use crate::{game::Game, utils::RawModeGuard};
pub mod attribute;
pub mod crafting;
pub mod game;
pub mod io;
pub mod item;
pub mod location;
pub mod player;
pub mod ui;
pub mod units;
pub mod utils;

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "zh_CN");
fn main() -> std::io::Result<()> {
  let _guard = RawModeGuard::new()?;
  Game::new().run();
  Ok(())
}
