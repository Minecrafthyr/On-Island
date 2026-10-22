#![feature(
  const_trait_impl,
  const_default,
  derive_const,
  const_convert,
  inherent_associated_types,
  const_precise_live_drops,
  const_ops,
  default_field_values,
  // transmute_neo
  // mut_restriction,
  // const_array,
  // const_heap,
  // const_closures
)]
#![allow(incomplete_features, clippy::missing_transmute_annotations, confusable_idents)]
use std::{
  io::stdout,
  panic::{catch_unwind, panic_any},
  thread::sleep,
};

use crossterm::{
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::{game::Game, io::ScreenWriter, player::Endgame};
pub mod damage;
pub mod game;
pub mod recipe;
// pub mod i18n;
// pub mod env;
pub mod creature;
pub mod io;
pub mod item;
pub mod location;
pub mod player;
pub mod preclude;
pub mod ui;
pub mod units;
pub mod utils;
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");
fn main() -> Result<(), Box<dyn std::error::Error>> {
  if let Ok(lang) = std::env::var("LANG")
    && let Some((locale, _codec)) = lang.split_once('.')
  {
    rust_i18n::set_locale(locale);
  }

  execute!(stdout(), EnterAlternateScreen)?;
  enable_raw_mode()?;
  match catch_unwind(|| Game::default().run()) {
  Ok(Err(e)) =>
    if let Some(endgame) = e.downcast_ref::<Endgame>() {
      ScreenWriter::new_screen().writeln(endgame)?.end()?;
      sleep(std::time::Duration::from_secs(2));
      execute!(stdout(), LeaveAlternateScreen)?;
      disable_raw_mode()?;
      println!("{endgame}");
      Ok(())
    } else {
      execute!(stdout(), LeaveAlternateScreen)?;
      disable_raw_mode()?;
      Err(e)
    },
  Err(panic) => {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    panic_any(panic)
  }
  }
}
