use std::thread::sleep;

use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, read},
    execute,
    terminal::{
        Clear, ClearType::FromCursorDown, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
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
    pub connections: Vec<Connection>,
    pub player: Player,
}

impl Default for Game {
    fn default() -> Self { Self::new() }
}

impl Game {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        let g = Game {
            rng: rand::rng(),
            time: Duration::ZERO,
            locations: Locations::new(),
            connections: vec![
                Connection {
                    a: Location::StrandedShip,
                    b: Location::Beach,
                    time_a_to_b: Duration::seconds(30),
                    time_b_to_a: Duration::seconds(40),
                },
                Connection {
                    a: Location::Beach,
                    b: Location::Forest,
                    time_a_to_b: Duration::minutes(3),
                    time_b_to_a: Duration::minutes(3),
                },
            ],
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
            self.end_game("你被饿死了！");
        }
        if self.player.attrs[Water] <= Duration::ZERO {
            self.end_game("你被渴死了！");
        }
        if self.player.attrs[Health] <= Duration::ZERO {
            self.end_game("你伤重而死！");
        }

        for ld in &mut self.locations.0 {
            ld.tick(self.time, &mut self.rng);
        }
    }

    fn render(&mut self) {
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        write_lines("=== On Island ===\n");
        for attr in Attribute::iter() {
            write_lines(&format!("{}: {}\n", attr.name(), self.player.attrs[attr]));
        }
        write_lines(&format!(
            "位置: {}\n[t]旅行 [p]拾取 [g]采集 [u]使用物品 [r]休息 [q]退出",
            self.player.location.name()
        ));
        execute!(stdout(), Clear(FromCursorDown)).unwrap();
    }

    pub fn end_game(&mut self, message: &str) {
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        write_lines(&format!("游戏结束！\n{message}"));
        sleep(std::time::Duration::from_secs(3));

        execute!(stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        println!("游戏结束：{message}");
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
            'u' => self.use_item(),
            'i' => self.inventory(),
            'r' => self.rest(),
            'q' => self.end_game("你关闭了游戏。"),
            _ => {}
            },
            _ => {}
            }
        }
    }
}
