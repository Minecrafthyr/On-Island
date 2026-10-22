use crate::{
  game::Game,
  io::ScreenWriter,
  item::Item,
  ui::{DataItem, DisplayList},
  utils::{IterStack, NameAndDesc, TakeItems},
};
impl Game {
  pub fn inventory(&mut self) -> Result<(), std::io::Error> {
    let items: Vec<_> = self.player.iter_stack().cloned().collect();
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(
      DisplayList::new(
        Some(t!("action.inventory.title")),
        items
          .iter()
          .map(|is| DataItem {
            text: t!(
              "action.inventory.entry",
              name = is.item.name(),
              count = is.count,
              volume = is.volume(),
              weight = is.weight(),
              description = is.item.description()
            ),
            selected: (),
            enter: |i| Ok(i),
          })
          .collect(),
      )
      .indexing(),
    )?
    else {
      return Ok(());
    };

    let item = &items[choice].item;
    if !item.wearable {
      s.message(t!("action.inventory.not_wearable", name = item.name()))?;
      return Ok(());
    }
    self.equip_item(item)?;
    s.message(t!("action.inventory.equipped", name = item.name()))?;
    Ok(())
  }

  pub(super) fn equip_item(&mut self, item: &Item) -> Result<(), std::io::Error> {
    if !item.wearable {
      return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Cannot equip an item that is not wearable",
      ));
    }
    let (mut removed, mismatch) = self.player.take_items(|candidate| candidate == item, 1);
    if mismatch != 0 {
      return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "The selected item is no longer in the inventory",
      ));
    }
    let Some(stack) = removed.pop() else {
      return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "The selected item could not be removed from the inventory",
      ));
    };
    self.player.worn.push(stack.item);
    Ok(())
  }

  pub(super) fn equip_from_inventory(
    &mut self, s: &mut ScreenWriter,
  ) -> Result<(), std::io::Error> {
    let items: Vec<_> =
      self.player.iter_stack().filter(|stack| stack.item.wearable).cloned().collect();
    let Some(choice) = s.list(DisplayList {
      title: Some(t!("action.worn.equip_title")),
      data: items
        .iter()
        .map(|stack| DataItem {
          text: t!(
            "action.worn.equip_entry",
            name = stack.item.name(),
            count = stack.count,
            description = stack.item.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
      indexed: true,
      ..
    })?
    else {
      return Ok(());
    };
    let item = items[choice].item.clone();
    self.equip_item(&item)?;
    s.message(t!("action.inventory.equipped", name = item.name()))?;
    Ok(())
  }
}
