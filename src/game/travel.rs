use std::rc::Rc;

use crate::{
  game::Game,
  io::ScreenWriter,
  player::{
    Effect,
    action::{Action, ActionContent},
  },
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};

impl Game {
  pub fn travel(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let options = &self.locations[self.player.location].can_go;
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList {
      title: Some(t!("action.travel.title", current_location = self.player.location.name())),
      data: options
        .iter()
        .map(|(location, time, _)| DataItem {
          text: t!(
            "action.travel.entry",
            name = location.name(),
            time = time,
            description = location.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
      indexed: true,
      ..
    })?
    else {
      return Ok(());
    };

    let (new_location, travel_time, _) = options[choice];
    match self.player_action(Action {
      id: "travel",
      f: Rc::new(move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(1.4)])),
      total_dur: travel_time,
      ..
    }) {
    Ok(()) => {
      self.player.location = new_location;
    }
    Err(not_ok) => {
      s.message(not_ok.to_string())?;
      return Ok(());
    }
    }
    Ok(())
  }
}
