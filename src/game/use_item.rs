use itertools::Itertools;

use crate::{game::Game, io::NewScreenWriter, ui::NumberRequester, utils::NameAndDesc};

impl Game {
  pub fn use_item(&mut self) {
    let options: Vec<_> = self
      .player
      .inventory
      .iter()
      .enumerate()
      .filter_map(|(i, item_stack)| {
        item_stack
          .item
          .use_data()
          .map(|(use_time, activity, attrs)| (i, item_stack.item, use_time, activity, attrs))
      })
      .collect();
    if options.is_empty() {
      return;
    }

    let mut s = NewScreenWriter::new();
    s.queue_lines(t!("action.use_item.title"));
    for (i, (inventory_index, item, use_time, activity, attrs)) in options.iter().enumerate() {
      s.queue_lines(t!(
        "action.use_item.entry",
        index = i,
        name = item.name(),
        count = self.player.inventory[*inventory_index].count,
        duration = use_time,
        activity = activity,
        attrs = attrs.iter().map(|m| t!("action.use_item.effect", attr = m)).join(" "),
        description = item.description()
      ));
    }
    s.end();

    let Some(choice) =
      NumberRequester::new(t!("action.use_item.choose")).range(0..=options.len() - 1).request()
    else {
      return;
    };

    let (inventory_index, _, use_time, activity, attrs) = options[choice].clone();
    self.action_time_pass(use_time, activity);

    let item_stack = &mut self.player.inventory[inventory_index];
    for modifier in attrs {
      self.player.attrs.apply_modifier(&modifier);
    }
    item_stack.count -= 1;
    if item_stack.count == 0 {
      self.player.inventory.remove(inventory_index);
    }
    self.player.recalc_volume_and_size();
  }
}
