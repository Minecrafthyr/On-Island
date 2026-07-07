#![feature(const_trait_impl)]

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
pub mod effect;
pub mod statics;

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "zh_CN");
fn main() -> std::io::Result<()> {
    let _guard = RawModeGuard::new()?; // 守卫持有 raw mode
    Game::new().run();
    Ok(())
}
