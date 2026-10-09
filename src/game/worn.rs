use crate::{
  game::Game,
  io::ScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};
impl Game {
  // TODO: implement equip/unequip
  pub fn worn(&self) {
    ScreenWriter::new_screen().list(
      DisplayList::new(
        Some(t!("action.worn.title")),
        self
          .player
          .worn
          .iter()
          .map(|item| DataItem {
            text: t!(
              "action.worn.entry",
              name = item.name(),
              volume = item.get_volume(),
              weight = item.get_weight(),
              description = item.description()
            ),
            selected: (),
            enter: (),
          })
          .collect(),
      )
      .indexing(),
    );
  }
}
