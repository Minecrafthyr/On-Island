use crate::{
  game::Game,
  io::NewScreenWriter,
  ui::{NumberRequester, popup_message},
  utils::NameAndDesc,
};

impl Game {
  pub fn pickup(&mut self) {
    let location_index = self.player.location;
    if self.locations[location_index].pickup_stacks.is_empty() {
      return;
    }
    let options: Vec<_> = self.locations[location_index]
      .pickup_stacks
      .iter()
      .enumerate()
      .filter_map(|(i, item_stack)| {
        let item = item_stack.item;
        self
          .player
          .pickup_time(item)
          .map(|(dur, activity)| (i, item, item_stack.count, dur, activity))
      })
      .collect();

    let mut s = NewScreenWriter::new();
    s.lines(t!("action.pickup.title"));
    for (i, (_, item, count, dur, activity)) in options.iter().enumerate() {
      s.lines(t!(
        "action.pickup.entry",
        index = i,
        name = item.name(),
        count = count,
        duration = dur.as_seconds_f64() : {:.2},
        activity = activity : {:.2}
      ));
    }
    s.end();
    let Some(choice) =
      NumberRequester::new(t!("action.pickup.choose")).range(0..=options.len() - 1).request()
    else {
      return;
    };
    let (_, item, stack_count, _, _) = options[choice];
    let Some(count) = NumberRequester::new(t!("action.pickup.how_many"))
      .range(1..=stack_count)
      .default(1)
      .request()
    else {
      return;
    };
    for i in 0..count {
      let Some((pick_time, activity)) = self.player.pickup_time(item) else {
        popup_message(t!("action.pickup.remaining", count = count - i));
        return;
      };
      self.action_time_pass(pick_time, activity);
      if let Some(_left) = self.player.insert_item(item) {
        popup_message(t!("action.pickup.remaining", count = count - i));
        return;
      };
      self.locations[location_index].pickup_stacks.remove_item(item);
    }
  }
}
