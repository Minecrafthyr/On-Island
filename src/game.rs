use std::{fmt::Display, io::stdout, thread::sleep};

use crossterm::{
  event::{Event, KeyCode, read},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::rngs::ThreadRng;
use time::Duration;

use crate::{
  io::ScreenWriter, location::Locations, player::Player, ui::NumberRequester,
  units::DurationDisplay, utils::NameAndDesc,
};

pub mod craft;
pub mod gather;
pub mod inventory;
pub mod pickup;
pub mod travel;
pub mod use_item;
pub mod worn;
pub struct Game {
  pub rng: ThreadRng,
  pub time: Duration,
  pub locations: Locations,
  pub player: Player,
}

impl Game {
  pub fn new() -> std::io::Result<Self> {
    let g = Game {
      rng: rand::rng(),
      time: Duration::ZERO,
      locations: Locations::new(),
      player: Player::new(),
    };
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode().unwrap();
    Ok(g)
  }

  /// ms
  pub fn tick(&mut self) {
    self.time += Duration::milliseconds(1);

    self.player.tick(1.0);

    if self.player.energy <= Duration::ZERO {
      self.end_game(t!("game.starved"));
    }
    if self.player.water <= Duration::ZERO {
      self.end_game(t!("game.dehydrated"));
    }
    if self.player.health <= 0.0 {
      self.end_game(t!("game.injured"));
    }

    for ld in &mut self.locations.0 {
      ld.tick(self.time, &mut self.rng);
    }
  }

  pub fn time_pass(&mut self, time: Duration) {
    if time.is_zero() {
      return;
    }
    for _ in 0..=time.whole_milliseconds() {
      self.tick();
    }
  }

  pub fn action_time_pass(&mut self, time: Duration, activity: f64) {
    let mut progress = Duration::ZERO;
    self.player.activity = activity;
    while progress < time {
      self.tick();
      let step = Duration::MILLISECOND * self.player.efficiency();
      progress += step;
    }
    self.player.activity = 1.0;
  }

  fn render(&mut self) {
    let mut s = ScreenWriter::new_screen();
    s.lines(t!("game.title")).endl();
    s.lines(t!(
      "game.stats",
      name = t!("attribute.health.name"),
      value = format!("{:.2}%", self.player.health)
    ));
    s.lines(t!(
      "game.stats",
      name = t!("attribute.energy.name"),
      value = DurationDisplay(self.player.energy)
    ));
    s.lines(t!(
      "game.stats",
      name = t!("attribute.water.name"),
      value = DurationDisplay(self.player.water)
    ));
    s.lines(t!("game.status_line", location = self.player.location.name())).end();
  }

  pub fn end_game(&mut self, message: impl Display) {
    ScreenWriter::new_screen().lines(t!("game.over_message", message = message)).end();
    sleep(std::time::Duration::from_secs(2));
    execute!(stdout(), LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
    println!("{}", t!("game.over_message", message = message));
    std::process::exit(0);
  }

  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
      'w' => self.worn(),
      'r' => self.rest(),
      'q' => self.end_game(t!("game.quit")),
      _ => {}
      },
      _ => {}
      }
    }
  }

  pub fn rest(&mut self) {
    ScreenWriter::new_screen().end();
    let Some(s) =
      NumberRequester::new(t!("action.rest.prompt")).range(0..=10000).default(10).request()
    else {
      return;
    };
    self.time_pass(Duration::seconds(s));
  }
}
