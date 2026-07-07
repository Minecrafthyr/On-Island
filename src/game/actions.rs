use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType::FromCursorDown},
};
use itertools::Itertools;
use time::Duration;

use super::*;
use crate::statics::stdout;
impl Game {
    pub fn pickup(&mut self) {
        let location_data = &self.locations[self.player.location];
        if location_data.pickup_stacks.is_empty() {
            return;
        }
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        queue_lines("有以下物品可拾取：\n");
        for (i, item_stack) in location_data.pickup_stacks.iter().enumerate() {
            queue_lines(&format!("[{}] {} x{}\n", i, item_stack.item.name(), item_stack.count));
        }
        execute!(stdout(), Clear(FromCursorDown)).unwrap();
        let Some(choice) = NumberRequester::new("选择要拾取的物品编号")
            .range(0..=location_data.pickup_stacks.len() - 1)
            .request()
        else {
            return;
        };
        let Some(count) = NumberRequester::new("拾取多少？")
            .range(1..=location_data.pickup_stacks[choice].count)
            .default(1)
            .request()
        else {
            return;
        };
        let item = location_data.pickup_stacks[choice].item;
        let Some(pick_time) = item.pick_up_time() else {
            return;
        };
        for _ in 0..count {
            self.action_time_pass(pick_time, 1.1);
            self.player.inventory.insert(item);
            self.locations[self.player.location].pickup_stacks.remove_item(item, 1);
        }
    }

    pub fn gather(&mut self) {
        let location_data = &self.locations[self.player.location];
        if location_data.gather_stacks.is_empty() {
            return;
        }
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        queue_lines("有以下物品可供采集：\n");
        for (i, item_stack) in location_data.gather_stacks.iter().enumerate() {
            queue_lines(&format!(
                "[{}] {} ×{}\n- {}\n",
                i,
                item_stack.item.name(),
                item_stack.count,
                item_stack.item.description(),
            ));
        }
        execute!(stdout(), Clear(FromCursorDown)).unwrap();
        let Some(choice) = NumberRequester::new("选择要采集的物品编号")
            .range(0..=location_data.gather_stacks.len() - 1)
            .request()
        else {
            return;
        };
        let Some(count) = NumberRequester::new("采集多少？")
            .range(1..=location_data.gather_stacks[choice].count)
            .default(1)
            .request()
        else {
            return;
        };
        let item = location_data.gather_stacks[choice].item;
        let Some((gather_time, activity, item_stacks)) = item.gather_items(self) else { return };
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
        let Some(s) = NumberRequester::new("请选择要休息的时间（分钟）")
            .range(0..=10000)
            .default(10)
            .request()
        else {
            return;
        };
        self.time_pass(Duration::seconds(s));
    }

    pub fn use_item(&mut self) {
        let options: Vec<_> = self
            .player
            .inventory
            .iter()
            .enumerate()
            .filter_map(|(i, item_stack)| {
                item_stack.item.use_data().map(|(use_time, activity, attrs)| {
                    (i, item_stack.item, use_time, activity, attrs)
                })
            })
            .collect();
        if options.is_empty() {
            return;
        }

        execute!(stdout(), MoveTo(0, 0)).unwrap();
        queue_lines("有以下物品可供使用：\n");
        for (i, (inventory_index, item, use_time, activity, attrs)) in options.iter().enumerate() {
            queue_lines(&format!(
                "[{}] {} ×{} ({}) ({}活动) {}\n- {}\n",
                i,
                item.name(),
                self.player.inventory[*inventory_index].count,
                use_time,
                activity,
                attrs.iter().map(|(a, t)| format!("{}+{}", a.name(), t)).join(" "),
                item.description()
            ));
        }
        execute!(stdout(), Clear(FromCursorDown)).unwrap();

        let Some(choice) =
            NumberRequester::new("请选择要使用的物品编号").range(0..=options.len() - 1).request()
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
        let msg = self
            .player
            .inventory
            .iter()
            .map(|is| format!("{} ×{}\n- {}", is.item.name(), is.count, is.item.description()))
            .join("\n");
        popup_message(&msg);
    }

    pub fn travel(&mut self) {
        let options: Vec<_> =
            self.connections.iter().filter_map(|c| c.other_side(self.player.location)).collect();
        if options.is_empty() {
            return;
        }
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        queue_lines("可以前往以下地点：\n");

        for (i, (location, time)) in options.iter().enumerate() {
            queue_lines(&format!(
                "[{}] {} ({}) - {}\n",
                i,
                location.name(),
                time,
                location.description()
            ));
        }

        execute!(stdout(), Clear(FromCursorDown)).unwrap();
        let Some(choice) =
            NumberRequester::new("请选择要前往的地点").range(0..=options.len() - 1).request()
        else {
            return;
        };

        let (new_location, travel_time) = options[choice];
        self.action_time_pass(travel_time, 1.4);
        self.player.location = new_location;
    }
}
