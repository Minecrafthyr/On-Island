#![feature(const_trait_impl, const_default, derive_const, const_convert, inherent_associated_types)]
#![allow(incomplete_features)]

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::game::Game;
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
  enable_raw_mode()?;

  let result = std::panic::catch_unwind(|| {
    Game::new().run();
  });

  let _ = disable_raw_mode();

  if let Err(err) = result {
    println!("游戏 panic: {:?}", err);
  }

  Ok(())
}
