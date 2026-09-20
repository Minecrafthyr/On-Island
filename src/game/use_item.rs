use crate::{
  game::Game,
  io::ScreenWriter,
  player::{Action, ActionContent, Effect},
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};

impl Game {
  pub fn use_item(&mut self) {
    let options: Vec<_> = self
      .player
      .worn
      .iter_mut()
      .flat_map(|c| c.pockets.iter())
      .flat_map(|p| p.stacks.iter())
      .filter(|is| is.item.use_data.is_some())
      .collect();
    if options.is_empty() {
      return;
    }
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.use_item.title")),
      options
        .iter()
        .enumerate()
        .map(|(i, is)| DataItem {
          text: t!(
            "action.use_item.entry",
            index = i,
            name = is.item.name(),
            count = is.count,
            duration = is.item.use_data.unwrap().dur.as_seconds_f64() : {:.2},
            activity = is.item.use_data.unwrap().activity: {:.2},
            description = is.item.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let item = options[choice].item.clone();
    let use_data = item.use_data.unwrap();

    match self.player_action(Action::no_progress(
      "use_item",
      move |_| ActionContent {
        body_parts: vec![],
        effects: vec![Effect::ActivityMul(use_data.activity)],
      },
      use_data.dur,
    )) {
    Ok(()) => {
      let (_removed, mismatch) = self.player.take_items_matching(|i| *i == item, 1);
      if mismatch == 0 {
        (use_data.on_use)(&mut self.player);
        return;
      }
    }
    Err(not_ok) => {
      s.message(not_ok.to_string());
      return;
    }
    }
  }
}
