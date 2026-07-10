use crossterm::{
  cursor::MoveTo,
  execute,
  terminal::{Clear, ClearType::FromCursorDown},
};
use itertools::Itertools;
use rand::RngExt;
use time::Duration;

use super::*;
use crate::{
  crafting::CRAFTING_RECIPES,
  statics::stdout,
  ui::{NumberRequester, popup_message},
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
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    queue_lines(&t!("action.pickup.title"));
    for (i, (_, item, count, dur, activity)) in options.iter().enumerate() {
      queue_lines(&t!(
        "action.pickup.entry",
        index = i,
        name = item.name(),
        count = count,
        duration = dur,
        activity = activity
      ));
    }
    execute!(stdout(), Clear(FromCursorDown)).unwrap();
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
      self.player.inventory.insert(item);
      self.locations[location_index].pickup_stacks.remove_item(item, 1);
    }
  }

  pub fn gather(&mut self) {
    let location_data = &self.locations[self.player.location];
    if location_data.gather_stacks.is_empty() {
      return;
    }
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    queue_lines(&t!("action.gather.title"));
    for (i, item_stack) in location_data.gather_stacks.iter().enumerate() {
      queue_lines(&t!(
        "action.gather.entry",
        index = i,
        name = item_stack.item.name(),
        count = item_stack.count,
        description = item_stack.item.description(),
      ));
    }
    execute!(stdout(), Clear(FromCursorDown)).unwrap();
    let Some(choice) = NumberRequester::new(t!("action.gather.choose"))
      .range(0..=location_data.gather_stacks.len() - 1)
      .request()
    else {
      return;
    };
    let Some(count) = NumberRequester::new(t!("action.gather.how_many"))
      .range(1..=location_data.gather_stacks[choice].count)
      .default(1)
      .request()
    else {
      return;
    };
    let item = location_data.gather_stacks[choice].item;
    let Some((gather_time, activity, item_stacks)) = (match item.get_id() {
    "raw_fish" => Some((
      Duration::seconds(self.rng.random_range(120..=2000)),
      1.2,
      ItemStacks::from([(Item::new(RAW_FISH), 1)]),
    )),
    "tree" => Some((Duration::minutes(30), 2.0, [(Item::new(TREE), 2)].into())),
    _ => None,
    }) else {
      return;
    };
    for _ in 0..count {
      self.action_time_pass(gather_time, activity);
      for item_stack in item_stacks.iter().cloned() {
        self.player.inventory.insert_stack(item_stack);
      }
      self.locations[self.player.location].gather_stacks.remove_item(item, 1);
    }
  }

  pub fn rest(&mut self) {
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    execute!(stdout(), Clear(FromCursorDown)).unwrap();
    let Some(s) =
      NumberRequester::new(t!("action.rest.prompt")).range(0..=10000).default(10).request()
    else {
      return;
    };
    self.time_pass(Duration::seconds(s));
  }

  pub fn craft(&mut self) {
    let options: Vec<_> = CRAFTING_RECIPES
      .iter()
      .copied()
      .filter(|recipe| recipe.can_apply(&self.player.inventory))
      .collect();
    if options.is_empty() {
      popup_message(&t!("action.craft.none"));
      return;
    }

    execute!(stdout(), MoveTo(0, 0)).unwrap();
    queue_lines(&t!("action.craft.title"));
    for (i, recipe) in options.iter().enumerate() {
      queue_lines(&t!(
        "action.craft.entry",
        index = i,
        title = recipe.name(),
        inputs = recipe.inputs_text(),
        outputs = recipe.outputs_text(),
        duration = recipe.time,
        activity = recipe.activity
      ));
    }
    execute!(stdout(), Clear(FromCursorDown)).unwrap();

    let Some(choice) =
      NumberRequester::new(t!("action.craft.choose")).range(0..=options.len() - 1).request()
    else {
      return;
    };

    let recipe = options[choice];
    let Some(batch_count) = NumberRequester::new(t!("action.craft.how_many"))
      .range(1..=recipe.max_batch_count(&self.player.inventory))
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..batch_count {
      self.action_time_pass(recipe.time, recipe.activity);
      recipe.apply(&mut self.player.inventory);
    }
  }

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

    execute!(stdout(), MoveTo(0, 0)).unwrap();
    queue_lines(&t!("action.use_item.title"));
    for (i, (inventory_index, item, use_time, activity, attrs)) in options.iter().enumerate() {
      queue_lines(&t!(
        "action.use_item.entry",
        index = i,
        name = item.name(),
        count = self.player.inventory[*inventory_index].count,
        duration = use_time,
        activity = activity,
        attrs = attrs
          .iter()
          .map(|(a, t)| t!("action.use_item.effect", attr = a.name(), value = t))
          .join(" "),
        description = item.description()
      ));
    }
    execute!(stdout(), Clear(FromCursorDown)).unwrap();

    let Some(choice) =
      NumberRequester::new(t!("action.use_item.choose")).range(0..=options.len() - 1).request()
    else {
      return;
    };

    let (inventory_index, _, use_time, activity, attrs) = options[choice].clone();
    self.action_time_pass(use_time, activity);

    let item_stack = &mut self.player.inventory[inventory_index];
    for (attr, value) in attrs {
      self.player.attrs[attr] += value;
    }
    item_stack.count -= 1;
    if item_stack.count == 0 {
      self.player.inventory.remove(inventory_index);
    }
  }

  pub fn inventory(&self) {
    popup_message(
      self
        .player
        .inventory
        .iter()
        .map(|is| {
          t!(
            "action.inventory.entry",
            name = is.item.name(),
            count = is.count,
            description = is.item.description()
          )
        })
        .join("\n"),
    );
  }

  pub fn travel(&mut self) {
    let options = &self.locations[self.player.location].can_go;
    if options.is_empty() {
      return;
    }
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    queue_lines(&t!("action.travel.title"));

    for (i, (location, time)) in options.iter().enumerate() {
      queue_lines(&t!(
        "action.travel.entry",
        index = i,
        name = location.name(),
        time = time,
        description = location.description()
      ));
    }

    execute!(stdout(), Clear(FromCursorDown)).unwrap();
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
