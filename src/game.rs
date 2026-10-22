use std::{error::Error, fmt::Display, sync::Arc};

use crossterm::event::{Event, KeyCode, KeyEvent, read};
use rand::rngs::ThreadRng;
use strum_macros::{EnumCount, IntoStaticStr};

pub use crate::preclude::*;
use crate::{
  io::ScreenWriter,
  item::ItemStacks,
  location::{EnvData, Locations},
  player::{
    Effect, Endgame, Player,
    action::{Action, ActionContent, PushActionError},
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
#[derive(Default)]
pub struct Game {
  pub rng: ThreadRng,
  pub time: Duration,
  pub locations: Locations,
  pub player: Player,
  pub endgame_message: Option<String>,
}

impl Game {
  // pub fn new() -> Self {
  //   Self {
  //     rng: rand::rng(),
  //     time: Duration::ZERO,
  //     locations: Locations::new(),
  //     player: Player::new(),
  //     endgame_message: None,
  //   }
  // }

  /// ms
  pub fn tick(&mut self) -> Result<(), Endgame> {
    self.time += 1.ms();

    self.player.tick()?;
    for ld in &mut self.locations.0 {
      ld.tick(self.time, &mut self.rng);
    }
    use EnvData::*;
    let env = self.locations[self.player.location]
      .env
      .iter()
      .filter(|env_bit| matches!(env_bit, NaturalAir | CaveAir | NoAir))
      .collect();
    let mut converted = ItemStacks::new();
    for mut s in
      self.player.worn.iter().flat_map(|i| i.pockets.iter()).flat_map(|p| p.stacks.iter()).cloned()
    {
      s.item.tick(env).map(|ss| converted.insert_stacks(ss));
    }
    if let Some(left) = self.player.insert_stacks(converted) {
      self.locations[self.player.location].pickup_stacks.insert_stacks(left);
    }
    Ok(())
  }
}
#[derive(Debug, Clone, EnumCount, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum WaitActionError {
  LowEfficiency,
  Death(Endgame),
}
impl From<Endgame> for WaitActionError {
  fn from(value: Endgame) -> Self { Self::Death(value) }
}
impl NameAndDesc for WaitActionError {
  const PREFIX: &str = "wait_action_result";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.into()) }
}
impl Error for WaitActionError {}
impl Display for WaitActionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name()) }
}
#[derive(thiserror::Error, Debug)]
pub enum ActionError {
  #[error("Push action failed: {0}")]
  Push(#[from] PushActionError),
  #[error("Wait action failed: {0}")]
  Wait(#[from] WaitActionError),
}
impl Game {
  pub fn wait_player_action(&mut self, id: &'static str) -> Result<(), WaitActionError> {
    while let Some(_) = self.player.actions.iter().find(|a| a.id == id) {
      self.tick()?;
      if self.player.get_efficiency() < 0.25 {
        return Err(WaitActionError::LowEfficiency);
      }
    }
    Ok(())
  }

  pub fn player_action(&mut self, action: Action) -> Result<(), ActionError> {
    let id = action.id;
    self.player.try_push_action(action)?;
    self.wait_player_action(id)?;
    Ok(())
  }

  pub fn time_pass(&mut self, time: Duration) -> Result<(), Endgame> {
    if time == Duration::ZERO {
      return Ok(());
    }
    for _ in 0..=time.as_ms() {
      self.tick()?;
    }
    Ok(())
  }

  fn render(&mut self) -> Result<(), std::io::Error> {
    ScreenWriter::new_screen()
      .writeln(t!("game.title"))?
      .endl()?
      .writeln(t!(
        "game.stats",
        name = t!("attribute.health.name"),
        value = format!("{:.2}%", self.player.health)
      ))?
      .writeln(t!("game.stats", name = t!("attribute.energy.name"), value = self.player.energy))?
      .writeln(t!("game.stats", name = t!("attribute.water.name"), value = self.player.water))?
      .writeln(t!("game.status_line", location = self.player.location.name()))?
      .end()?;
    Ok(())
  }

  pub fn run(&mut self) -> Result<!, Box<dyn std::error::Error>> {
    loop {
      self.render()?;
      match read()? {
      Event::Key(KeyEvent { code, .. }) => match code {
      KeyCode::Char(ch) => match ch {
      't' => self.travel()?,
      'p' => self.pickup()?,
      'g' => self.gather()?,
      'c' => self.craft()?,
      'u' => self.use_item()?,
      'i' => self.inventory()?,
      'w' => self.worn()?,
      'r' => self.rest()?,
      'q' => {
        return Err(Box::new(Endgame::from(t!("game.quit"))));
      }
      _ => {}
      },
      KeyCode::Esc => self.menu()?,
      _ => {}
      },
      _ => {}
      }
    }
  }

  pub fn menu(&mut self) -> Result<(), Box<dyn Error>> {
    let mut s = ScreenWriter::new_screen();
    s.write(t!("menu.title"))?;
    Ok(())
  }

  pub fn rest(&mut self) -> Result<(), Box<dyn Error>> {
    let mut s = ScreenWriter::new_screen();
    s.end()?;
    let Some(sec) = NumberRequester {
      prompt: t!("action.rest.prompt"),
      range: Some(0..=10000),
      default: Some(10),
      ..
    }
    .request()?
    else {
      return Ok(());
    };
    match self.player_action(Action {
      id: "rest",
      f: Arc::new(|_| ActionContent::new(vec![], vec![Effect::HealthRegenMul(1.2)])),
      total_dur: sec.s(),
      ..
    }) {
    Ok(()) => Ok(()),
    Err(not_ok) => {
      s.message(not_ok.to_string())?;
      return Ok(());
    }
    }
  }
}
