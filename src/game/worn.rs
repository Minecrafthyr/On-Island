use crate::{
  game::Game,
  io::ScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};
impl Game {
  pub fn worn(&mut self) -> Result<(), std::io::Error> {
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList {
      title: Some(t!("action.worn.title")),
      data: std::iter::once(t!("action.worn.equip"))
        .chain(self.player.worn.iter().map(|item| {
          t!(
            "action.worn.entry",
            name = item.name(),
            volume = item.get_volume(),
            weight = item.get_weight(),
            description = item.description()
          )
        }))
        .map(|text| DataItem { text, selected: (), enter: |i| Ok(i) })
        .collect(),
      indexed: true,
      ..
    })?
    else {
      return Ok(());
    };

    if choice == 0 {
      self.equip_from_inventory(&mut s)?;
      return Ok(());
    }

    let index = choice - 1;
    let item = self.player.worn.remove(index);
    let name = item.name();
    if let Some(remainder) = self.player.insert_stack(item.clone() * 1) {
      self.player.worn.insert(index, remainder.item);
      s.message(t!("action.worn.unequip_failed", name = name))?;
    } else {
      s.message(t!("action.worn.unequipped", name = name))?;
    }
    Ok(())
  }
}
