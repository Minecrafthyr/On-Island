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
    pub fn gather(&mut self) {
        let location_data = &self.locations[self.player.location];
        if location_data.item_stacks.is_empty() {
            return;
        }
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        queue_lines("有以下物品可供采集：\n");
        for (i, item_stack) in location_data.item_stacks.iter().enumerate() {
            queue_lines(&format!("[{}] {} x{}\n", i, item_stack.item.name(), item_stack.count));
        }
        execute!(stdout(), Clear(FromCursorDown)).unwrap();
        let choice = match NumberRequester::new("选择要采集的物品编号")
            .range(0..=location_data.item_stacks.len() - 1)
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
        };
        let count = match NumberRequester::new("采集多少？")
            .range(1..=location_data.item_stacks[choice].count)
            .default(1)
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
        };
        let item = location_data.item_stacks[choice].item;
        for _ in 0..count {
            let t = item.gather_time(self);
            self.action_time_pass(t);
            self.player.inventory.insert(item);
            self.locations[self.player.location].item_stacks[choice].count -= 1;
        }
    }

    pub fn rest(&mut self) {
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        execute!(stdout(), Clear(FromCursorDown)).unwrap();
        let s = match NumberRequester::new("请选择要休息的时间（分钟）")
            .range(0..=10000)
            .default(10)
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
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
                item_stack
                    .item
                    .use_data()
                    .map(|(use_time, attrs)| (i, item_stack.item, use_time, attrs))
            })
            .collect();
        if options.is_empty() {
            return;
        }

        execute!(stdout(), MoveTo(0, 0)).unwrap();
        queue_lines("有以下物品可供使用：\n");
        for (i, (inventory_index, item, use_time, attrs)) in options.iter().enumerate() {
            queue_lines(&format!(
                "[{}] {} x{}（{}）{}\n",
                i,
                item.name(),
                self.player.inventory[*inventory_index].count,
                use_time,
                attrs.iter().map(|(a, t)| format!("{}+{}", a.name(), t)).join(" ")
            ));
        }
        execute!(stdout(), Clear(FromCursorDown)).unwrap();

        let choice = match NumberRequester::new("请选择要使用的物品编号")
            .range(0..=options.len() - 1)
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
        };

        let (inventory_index, _, use_time, attrs) = options[choice].clone();
        self.action_time_pass(use_time);

        let item_stack = &mut self.player.inventory[inventory_index];
        for (attr, value) in attrs {
            self.player.attrs[attr] += value;
        }
        item_stack.count -= 1;
        if item_stack.count == 0 {
            self.player.inventory.remove(inventory_index);
        }
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
        let choice =
            match NumberRequester::new("请选择要前往的地点").range(0..=options.len() - 1).request()
            {
            NumberResult::Number(n) => n,
            NumberResult::Cancel => return,
            };

        let (new_location, travel_time) = options[choice];
        self.action_time_pass(travel_time);
        self.player.location = new_location;
    }
}
