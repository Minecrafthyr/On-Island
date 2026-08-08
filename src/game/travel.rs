use crate::{
  game::Game,
  io::NewScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};

impl Game {
  pub fn travel(&mut self) {
    let options = &self.locations[self.player.location].can_go;
    let mut s = NewScreenWriter::new();
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
    self.action_time_pass(travel_time, 1.4);
    self.player.location = new_location;
  }
}
