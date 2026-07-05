use rand::Rng;

use super::*;
#[derive(Debug)]
pub struct RestorationData {
    pub item: Item,
    pub count: RangeInclusive<u64>,
    pub interval: Time,
    pub chance: f64,
    pub limit: u64,
}
#[derive(Debug)]
pub struct LocationData {
    pub item_stacks: ItemStacks,
    pub restore: Vec<RestorationData>,
}
impl LocationData {
    pub fn tick(&mut self, time: Time, rng: &mut impl Rng) {
        for r in &mut self.restore {
            if time % r.interval == Time(0) && rng.random_bool(r.chance) {
                if let Some(ei) = self.item_stacks.iter_mut().find(|is| is.item == r.item) {
                    ei.count += rng.random_range(r.count.clone());
                } else {
                    self.item_stacks
                        .push(ItemStack { item: r.item, count: rng.random_range(r.count.clone()) });
                }
            }
        }
    }
}
#[derive(Debug, Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash)]
pub enum Location {
    StrandedShip,
    Beach,
    Forest,
}
impl Location {
    pub fn name(&self) -> &'static str {
        match self {
        Self::StrandedShip => "搁浅的船",
        Self::Beach => "海滩",
        Self::Forest => "森林",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
        Self::StrandedShip => "一艘搁浅的船，周围是一片海滩。",
        Self::Beach => "一片金黄的海滩。",
        Self::Forest => "一片茂密的森林，树木高耸入云。",
        }
    }
}
#[derive(Debug)]
pub struct Locations(pub [LocationData; Location::COUNT]);
impl Index<Location> for Locations {
    type Output = LocationData;

    fn index(&self, index: Location) -> &Self::Output { &self.0[index as usize] }
}
impl IndexMut<Location> for Locations {
    fn index_mut(&mut self, index: Location) -> &mut Self::Output { &mut self.0[index as usize] }
}
impl Locations {
    pub fn new() -> Self {
        use item::Item::*;
        Self([
            LocationData {
                item_stacks: vec![ItemStack { item: Biscuit, count: 10 }, ItemStack {
                    item: Water,
                    count: 10,
                }]
                .into(),
                restore: vec![],
            },
            LocationData {
                item_stacks: vec![ItemStack { item: RawFish, count: 10 }, ItemStack {
                    item: Rock,
                    count: 50,
                }]
                .into(),
                restore: vec![RestorationData {
                    item: RawFish,
                    count: RangeInclusive::from(1..=2),
                    interval: Time::m(20),
                    chance: 30.0,
                    limit: 10,
                }],
            },
            LocationData {
                item_stacks: vec![ItemStack { item: Wood, count: 400 }].into(),
                restore: vec![RestorationData {
                    item: Wood,
                    count: RangeInclusive::from(1..=1),
                    interval: Time::d(10),
                    chance: 100.0,
                    limit: 500,
                }],
            },
        ])
    }
}

#[derive(Debug)]
pub struct Connection {
    pub a: Location,
    pub b: Location,
    pub time_a_to_b: Time,
    pub time_b_to_a: Time,
}

impl Connection {
    pub fn other_side(&self, l: Location) -> Option<(Location, Time)> {
        if l == self.a {
            Some((self.b, self.time_a_to_b))
        } else if l == self.b {
            Some((self.a, self.time_b_to_a))
        } else {
            None
        }
    }
}
