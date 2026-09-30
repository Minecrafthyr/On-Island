use std::{error::Error, fmt::Display, io::stdout, thread::sleep};

use crossterm::{
  event::{Event, KeyCode, KeyEvent, read},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::rngs::ThreadRng;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};

pub use crate::preclude::*;
use crate::{
  io::ScreenWriter,
  location::Locations,
  player::{
    Effect, Player,
    action::{Action, ActionContent},
  },
  ui::NumberRequester,
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
    self.time += 1.ms();

    self.player.tick();

    for ld in &mut self.locations.0 {
      ld.tick(self.time, &mut self.rng);
    }
  }
}
#[derive(
  Debug, Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum WaitActionError {
  LowEfficiency,
}
impl NameAndDesc for WaitActionError {
  const PREFIX: &str = "wait_action_result";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.into()) }
}
impl Error for WaitActionError {}
impl Display for WaitActionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name()) }
}
impl Game {
  pub fn wait_player_action(&mut self, id: &'static str) -> Result<(), WaitActionError> {
    while let Some(_) = self.player.actions.iter().find(|a| a.id == id) {
      self.tick();
      if self.player.get_efficiency() < 0.25 {
        return Err(WaitActionError::LowEfficiency);
      }
    }
    Ok(())
  }

  pub fn player_action(&mut self, action: Action) -> Result<(), Box<dyn Error>> {
    let id = action.id;
    self.player.try_push_action(action)?;
    self.wait_player_action(id)?;
    Ok(())
  }

  pub fn time_pass(&mut self, time: Duration) {
    if time == Duration::ZERO {
      return;
    }
    for _ in 0..=time.as_ms() {
      self.tick();
    }
  }

  fn render(&mut self) {
    let mut s = ScreenWriter::new_screen();
    s.lines(t!("game.title")).endl();
    s.lines(t!(
      "game.stats",
      name = t!("attribute.health.name"),
      value = format!("{:.2}%", self.player.health)
    ));
    s.lines(t!("game.stats", name = t!("attribute.energy.name"), value = self.player.energy));
    s.lines(t!("game.stats", name = t!("attribute.water.name"), value = self.player.water));
    s.lines(t!("game.status_line", location = self.player.location.name())).end();
  }

  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    loop {
      self.render();
      match read().unwrap() {
      Event::Key(KeyEvent { code, .. }) => match code {
      KeyCode::Char(ch) => match ch {
      't' => self.travel(),
      'p' => self.pickup(),
      'g' => self.gather(),
      'c' => self.craft(),
      'u' => self.use_item(),
      'i' => self.inventory(),
      'w' => self.worn(),
      'r' => self.rest(),
      'q' => end_game(t!("game.quit")),
      _ => {}
      },
      KeyCode::Esc => self.menu(),
      _ => {}
      },
      _ => {}
      }
    }
  }

  pub fn menu(&mut self) {}

  pub fn rest(&mut self) {
    let mut s = ScreenWriter::new_screen();
    s.end();
    let Some(sec) =
      NumberRequester::new(t!("action.rest.prompt")).range(0..=10000).default(10).request()
    else {
      return;
    };
    match self.player_action(Action::no_progress(
      "rest",
      |_| ActionContent::new(vec![], vec![Effect::HealthRegenMul(1.2)]),
      Duration::s(sec),
    )) {
    Ok(()) => {}
    Err(not_ok) => {
      s.message(not_ok.to_string());
      return;
    }
    }
    self.time_pass(Duration::s(sec));
  }
}

pub fn end_game(message: impl Display) {
  ScreenWriter::new_screen().lines(t!("game.over_message", message = message)).end();
  sleep(std::time::Duration::from_secs(2));
  execute!(stdout(), LeaveAlternateScreen).unwrap();
  disable_raw_mode().unwrap();
  println!("{}", t!("game.over_message", message = message));
  std::process::exit(0);
}
