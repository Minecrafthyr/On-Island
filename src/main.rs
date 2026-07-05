#![feature(const_trait_impl)]

use std::{
    fmt::Display,
    io::Write,
    mem::transmute,
    ops::{Index, IndexMut, RangeInclusive},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToNextLine},
    event::{Event, KeyCode, read},
    execute,
    style::Print,
    terminal::{
        Clear,
        ClearType::{FromCursorDown, UntilNewLine},
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use itertools::Itertools;
use rand::RngExt;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount, EnumIter};

pub mod utils;
use utils::*;
pub mod time;
use time::*;

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

fn main() { Game::new().run() }
