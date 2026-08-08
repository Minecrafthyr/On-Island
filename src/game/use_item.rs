use crate::{
  game::Game,
  io::NewScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};

impl Game {
  pub fn use_item(&mut self) {
    let options: Vec<_> = self
      .player
      .inventory
      .iter()
      .enumerate()
      .filter_map(|(i, item_stack)| item_stack.item.use_data.map(|ud| (i, item_stack, ud)))
      .collect();
    if options.is_empty() {
      return;
    }
    let mut s = NewScreenWriter::new();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.use_item.title")),
      options
        .iter()
        .enumerate()
        .map(|(i, (_inventory_index, item_stack, use_data))| DataItem {
          text: t!(
            "action.use_item.entry",
            index = i,
            name = item_stack.item.name(),
            count = item_stack.count,
            duration = use_data.dur.as_seconds_f64() : {:.2},
            activity = use_data.activity: {:.2},
            description = item_stack.item.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };

    let (inventory_index, _, use_data) = options[choice];
    self.action_time_pass(use_data.dur, self.player.activity);

    let count = self.player.inventory[inventory_index].count;
    (use_data.on_use)(&mut self.player);
    let new_count = count.saturating_sub(1);
    if new_count == 0 {
      self.player.inventory.swap_remove(inventory_index);
    } else {
      self.player.inventory[inventory_index].count = new_count;
    }
    self.player.recalc_volume_and_size();
  }
}
