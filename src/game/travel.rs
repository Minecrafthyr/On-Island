use crate::{
  game::Game,
  io::ScreenWriter,
  player::{Action, ActionContent, Effect},
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};

impl Game {
  pub fn travel(&mut self) {
    let options = &self.locations[self.player.location].can_go;
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.travel.title", current_location = self.player.location.name())),
      options
        .iter()
        .enumerate()
        .map(|(i, (location, time))| DataItem {
          text: t!(
            "action.travel.entry",
            index = i,
            name = location.name(),
            time = time,
            description = location.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };

    let (new_location, travel_time) = options[choice];
    match self.player_action(Action::no_progress(
      "travel",
      move |_| ActionContent { body_parts: vec![], effects: vec![Effect::ActivityMul(1.4)] },
      travel_time,
    )) {
    Ok(()) => {
      self.player.location = new_location;
    }
    Err(not_ok) => {
      s.message(not_ok.to_string());
      return;
    }
    }
  }
}
