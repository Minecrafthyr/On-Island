use crate::{game::Game, io::NewScreenWriter, ui::NumberRequester, utils::NameAndDesc};

impl Game {
  pub fn travel(&mut self) {
    let options = &self.locations[self.player.location].can_go;
    if options.is_empty() {
      return;
    }

    let mut s = NewScreenWriter::new();
    s.lines(t!("action.travel.title"));
    for (i, (location, time)) in options.iter().enumerate() {
      s.lines(t!(
        "action.travel.entry",
        index = i,
        name = location.name(),
        time = time,
        description = location.description()
      ));
    }
    s.end();
    let Some(choice) =
      NumberRequester::new(t!("action.travel.choose")).range(0..=options.len() - 1).request()
    else {
      return;
    };

    let (new_location, travel_time) = options[choice];
    self.action_time_pass(travel_time, 1.4);
    self.player.location = new_location;
  }
}
