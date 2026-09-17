#![feature(
  const_trait_impl,
  const_default,
  derive_const,
  const_convert,
  inherent_associated_types,
  generic_const_exprs,
  const_precise_live_drops,
  const_ops,
  // transmute_neo
  // mut_restriction,
  // const_array,
  // const_heap,
  // const_closures
)]
#![allow(incomplete_features, clippy::missing_transmute_annotations)]

use crate::game::Game;
pub mod crafting_recipe;
pub mod game;
pub mod i18n;
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
fn main() -> Result<(), Box<dyn std::error::Error>> { Game::new()?.run() }
