use crate::{
  game::Game,
  io::ScreenWriter,
  player::{Action, ActionContent, Effect},
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};

impl Game {
  pub fn use_item(&mut self) {
    let mut s = ScreenWriter::new_screen();
    let (ci, pi, ii, _, use_data) = {
      let mut options = Vec::new();
      for (ci, c) in self.player.worn.iter_mut().enumerate() {
        for (pi, p) in c.pockets.iter().enumerate() {
          for (ii, is) in p.stacks.iter().enumerate() {
            if let Some(use_data) = is.item.use_data {
              options.push((ci, pi, ii, is, use_data));
            }
          }
        }
      }
      if options.is_empty() {
        return;
      }
      let Some(choice) = s.list(DisplayList::new(
        Some(t!("action.use_item.title")),
        options
          .iter()
          .enumerate()
          .map(|(i, (_, _, _, item_stack, use_data))| DataItem {
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
      options[choice]
    };
    match self.player_action(Action::no_progress(
      "use_item",
      move |_| ActionContent {
        body_parts: vec![],
        effects: vec![Effect::ActivityMul(use_data.activity)],
      },
      use_data.dur,
    )) {
    Ok(()) => {
      let mut containers: Vec<_> = self.player.worn.iter_mut().collect();
      let pocket = &mut containers[ci].pockets[pi];
      let new_count = pocket.stacks[ii].count.saturating_sub(1);
      if new_count == 0 {
        pocket.stacks.remove(ii);
      } else {
        pocket.stacks[ii].count = new_count;
      }
      (use_data.on_use)(&mut self.player);
    }
    Err(not_ok) => {
      s.message(not_ok.to_string());
      return;
    }
    }
  }
}
