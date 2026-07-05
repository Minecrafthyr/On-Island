use super::*;
impl Game {
    pub fn gather(&mut self) {
        if self.locations[self.player.location].item_stacks.is_empty() {
            return;
        }
        execute!(self.stdout, MoveTo(0, 0)).unwrap();
        write_lines("有以下物品可供采集：\n");
        let location_data = &self.locations[self.player.location];
        for (i, item_stack) in location_data.item_stacks.iter().enumerate() {
            write_lines(&format!("[{}] {} x{}\n", i, item_stack.item.name(), item_stack.count));
        }
        execute!(self.stdout, Clear(FromCursorDown)).unwrap();
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
        execute!(self.stdout, MoveTo(0, 0)).unwrap();
        execute!(self.stdout, Clear(FromCursorDown)).unwrap();
        let s = match NumberRequester::new("请选择要休息的时间（分钟）")
            .range(0..=10000)
            .default(10)
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
        };
        self.time_pass(Time::s(s));
    }

    pub fn use_item(&mut self) {
        if self.player.inventory.is_empty() {
            return;
        };
        execute!(self.stdout, MoveTo(0, 0)).unwrap();
        write_lines("有以下物品可供使用：\n");
        for (i, item_stack, (use_time, attrs)) in
            self.player.inventory.iter().enumerate().filter_map(|(i, is)| {
                if let Some(ud) = is.item.use_data() { Some((i, is, ud)) } else { None }
            })
        {
            write_lines(&format!(
                "[{}] {} x{}（{}）{}\n",
                i,
                item_stack.item.name(),
                item_stack.count,
                use_time,
                attrs.iter().map(|(a, t)| format!("{}+{}", a.name(), t)).join(" ")
            ));
        }
        execute!(self.stdout, Clear(FromCursorDown)).unwrap();
        let choice = match NumberRequester::new("请选择要使用的物品编号")
            .range(0..=self.player.inventory.len() - 1)
            .check(&|choice: usize| {
                if self.player.inventory[choice].item.use_data().is_none() {
                    Some("此物品无法使用。".to_string())
                } else {
                    None
                }
            })
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
        };
        let (use_time, attrs) = self.player.inventory[choice].item.use_data().unwrap();
        self.action_time_pass(use_time);

        let item_stack = &mut self.player.inventory[choice];
        for (attr, value) in attrs {
            self.player.attrs[attr] += value;
        }
        item_stack.count -= 1;
        if item_stack.count == 0 {
            self.player.inventory.remove(choice);
        }
    }

    pub fn travel(&mut self) {
        execute!(self.stdout, MoveTo(0, 0)).unwrap();
        write_lines("可以前往以下地点：\n");
        let can_go: Vec<_> = self
            .connections
            .iter()
            .filter_map(|c| {
                if let Some(other_side) = c.other_side(self.player.location) {
                    Some(other_side)
                } else {
                    None
                }
            })
            .collect();
        let could_go: Vec<_> = Location::iter()
            .enumerate()
            .filter(|(_, l)| can_go.iter().find(|(cl, _)| l == cl).is_some())
            .collect();
        for (i, l) in could_go.iter() {
            write_lines(&format!("[{}] {} - {}\n", i, l.name(), l.description()));
        }

        execute!(self.stdout, Clear(FromCursorDown)).unwrap();
        let choice = match NumberRequester::new("请选择要前往的地点")
            .range(0..=Location::COUNT - 1)
            .check(&|choice: usize| {
                if could_go.iter().find(|(i, _)| *i == choice).is_none() {
                    Some("你不知道要怎么去那里。".to_string())
                } else {
                    None
                }
            })
            .request()
        {
        NumberResult::Number(n) => n,
        NumberResult::Cancel => return,
        };
        let new_loc: Location = unsafe { transmute(choice as u8) };
        self.action_time_pass(
            can_go
                .into_iter()
                .find_map(|(l, d)| if l == new_loc { Some(d) } else { None })
                .unwrap(),
        );
        self.player.location = unsafe { transmute(choice as u8) };
    }
}
