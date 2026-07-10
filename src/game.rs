use std::{fmt::Display, thread::sleep};

use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, read},
    execute,
    terminal::{
        Clear, ClearType::FromCursorDown, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode,
    },
};
use rand::rngs::ThreadRng;
use strum::IntoEnumIterator;
use time::Duration;

use super::*;
use crate::statics::stdout;

pub mod actions;
pub mod helper;
pub struct Game {
    pub rng: ThreadRng,
    pub time: Duration,
    pub locations: Locations,
    pub player: Player,
}

impl Default for Game {
    fn default() -> Self { Self::new() }
}

impl Game {
    pub fn new() -> Self {
        let g = Game {
            rng: rand::rng(),
            time: Duration::ZERO,
            locations: Locations::new(),
            player: Player::new(),
        };
        execute!(stdout(), EnterAlternateScreen).unwrap();
        g
    }

    pub fn tick_ms(&mut self) {
        use Attribute::*;
        self.time += Duration::milliseconds(1);

        self.player.tick(1.0);

        if self.player.attrs[Energy] <= Duration::ZERO {
            self.end_game(t!("game.starved"));
        }
        if self.player.attrs[Water] <= Duration::ZERO {
            self.end_game(t!("game.dehydrated"));
        }
        if self.player.attrs[Health] <= Duration::ZERO {
            self.end_game(t!("game.injured"));
        }

        for ld in &mut self.locations.0 {
            ld.tick(self.time, &mut self.rng);
        }
    }

    fn render(&mut self) {
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        write_lines(&t!("game.title"));
        for attr in Attribute::iter() {
            write_lines(&t!("game.stats", name = attr.name(), value = self.player.attrs[attr]));
        }
        write_lines(&t!("game.status_line", location = self.player.location.name()));
        execute!(stdout(), Clear(FromCursorDown)).unwrap();
    }

    pub fn end_game(&mut self, message: impl Display) {
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        write_lines(&t!("game.over", message = message));
        sleep(std::time::Duration::from_secs(3));

        execute!(stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        println!("{}", t!("game.over_message", message = message));
        std::process::exit(0);
    }

    pub fn run(&mut self) {
        loop {
            self.render();
            match read().unwrap() {
            Event::Key(key_event) if let KeyCode::Char(ch) = key_event.code => match ch {
            't' => self.travel(),
            'p' => self.pickup(),
            'g' => self.gather(),
            'c' => self.craft(),
            'u' => self.use_item(),
            'i' => self.inventory(),
            'r' => self.rest(),
            'q' => self.end_game(t!("game.quit")),
            _ => {}
            },
            _ => {}
            }
        }
    }
}
