use itertools::Itertools;

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
      .filter_map(|(i, item_stack)| {
        item_stack.item.use_data.and_then(|ud| Some((i, item_stack, ud)))
      })
      .collect();
    if options.is_empty() {
      return;
    }
    NewScreenWriter::new();
    let Some(choice) = DisplayList::new(
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
            attrs = use_data.attrs.iter().map(|m| t!("action.use_item.effect", attr = m)).join(" "),
            description = item_stack.item.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )
    .run() else {
      return;
    };

    let (inventory_index, _, use_data) = options[choice].clone();
    self.action_time_pass(use_data.dur, use_data.activity);

    let item_stack = &mut self.player.inventory[inventory_index];
    for modifier in use_data.attrs {
      self.player.attrs.apply_modifier(&modifier);
    }
    item_stack.count -= 1;
    if item_stack.count == 0 {
      self.player.inventory.swap_remove(inventory_index);
    }
    self.player.recalc_volume_and_size();
  }
}
