use super::*;
#[derive(Clone)]
pub enum ItemReq {
  Def(&'static ItemDef),
  Reqs(&'static [Self]),
  Fn(&'static dyn Fn(&Item) -> bool),
}
impl ItemReq {
  pub fn matches(&self, i: &Item) -> bool {
    match self {
    Self::Def(item_def) if item_def == &i.def => true,
    Self::Reqs(reqs) if reqs.iter().any(|d| d.matches(i)) => true,
    Self::Fn(f) => f(i),
    _ => false,
    }
  }
}

const impl From<&'static ItemDef> for ItemReq {
  fn from(value: &'static ItemDef) -> Self { Self::Def(value) }
}
const impl From<&'static [ItemReq]> for ItemReq {
  fn from(value: &'static [ItemReq]) -> Self { Self::Reqs(value) }
}
const impl From<&'static dyn Fn(&Item) -> bool> for ItemReq {
  fn from(value: &'static dyn Fn(&Item) -> bool) -> Self { Self::Fn(value) }
}

#[derive(Clone)]
pub struct Req {
  pub item: ItemReq,
  pub count: u64,
  pub consume: bool,
  pub one_by_one: bool,
}
const impl<I: const Into<ItemReq>> From<I> for Req {
  fn from(value: I) -> Self {
    Self { item: value.into(), count: 1, one_by_one: true, consume: true }
  }
}
const impl From<ItemDefStack> for Req {
  fn from(value: ItemDefStack) -> Self {
    Self { item: value.item.into(), count: value.count, one_by_one: true, consume: true }
  }
}
impl Req {
  pub const fn count(mut self, count: u64) -> Self {
    self.count = count;
    self
  }

  pub const fn no_consume(mut self) -> Self {
    self.consume = false;
    self
  }

  pub const fn consume_once(mut self) -> Self {
    self.one_by_one = false;
    self
  }
}
#[derive(Clone)]
pub enum Depends {
  Req(Req),
  AllOf(&'static [Self]),
  AnyOf(&'static [Self]),
}

const impl<R: const Into<Req>> From<R> for Depends {
  fn from(value: R) -> Self { Self::Req(value.into()) }
}
