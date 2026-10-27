use std::fmt::Display;

use itertools::Itertools;

use super::*;
use crate::builder_method;
#[derive(Clone)]
pub enum ItemReq {
  Def(&'static ItemDef),
  All(&'static [Self]),
  Fn(&'static dyn Fn(&Item) -> bool),
}
impl Display for ItemReq {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
    Self::Def(item_def) => write!(f, "{}", item_def.name()),
    Self::All(reqs) => write!(f, "({})", reqs.iter().join(" | ")),
    Self::Fn(_) => write!(f, "???"),
    }
  }
}
impl ItemReq {
  pub fn matches(&self, i: &Item) -> bool {
    match self {
    Self::Def(item_def) if item_def == &i.def => true,
    Self::All(reqs) if reqs.iter().any(|d| d.matches(i)) => true,
    Self::Fn(f) => f(i),
    _ => false,
    }
  }
}

const impl From<&'static ItemDef> for ItemReq {
  fn from(value: &'static ItemDef) -> Self { Self::Def(value) }
}
const impl From<&'static [ItemReq]> for ItemReq {
  fn from(value: &'static [ItemReq]) -> Self { Self::All(value) }
}
const impl From<&'static dyn Fn(&Item) -> bool> for ItemReq {
  fn from(value: &'static dyn Fn(&Item) -> bool) -> Self { Self::Fn(value) }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsumeType {
  None,
  ForEach { return_on_break: bool },
  ForEachResult { return_on_break: bool },
  PerRecipe { return_on_break: bool },
  BatchAll { return_on_break: bool },
}
impl ConsumeType {
  pub fn returns_on_break(&self) -> bool {
    use ConsumeType::*;
    match *self {
    None => false,
    ForEach { return_on_break }
    | ForEachResult { return_on_break }
    | PerRecipe { return_on_break }
    | BatchAll { return_on_break } => return_on_break,
    }
  }
}
impl Display for ConsumeType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ConsumeType::*;
    match self {
    None => write!(f, "{}", t!("consume_type.none")),
    ForEach { return_on_break } =>
      write!(f, "{}", t!("consume_type.for_each", return_on_break = return_on_break)),
    ForEachResult { return_on_break } =>
      write!(f, "{}", t!("consume_type.for_each_result", return_on_break = return_on_break)),
    PerRecipe { return_on_break } =>
      write!(f, "{}", t!("consume_type.per_recipe", return_on_break = return_on_break)),
    BatchAll { return_on_break } =>
      write!(f, "{}", t!("consume_type.batch_all", return_on_break = return_on_break)),
    }
  }
}
#[derive(Clone)]
pub struct Req {
  pub item: ItemReq,
  pub count: u64 = 1,
  pub consume: ConsumeType = ConsumeType::PerRecipe { return_on_break: false },
}
const impl<I: const Into<ItemReq>> From<I> for Req {
  fn from(value: I) -> Self {
    Self {
      item: value.into(),
      count: 1,
      consume: ConsumeType::PerRecipe { return_on_break: false },
    }
  }
}
impl Display for Req {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} ×{} ({})", self.item, self.count, self.consume)
  }
}

const impl From<ItemDefStack> for Req {
  fn from(value: ItemDefStack) -> Self {
    Self {
      item: value.item.into(),
      count: value.count,
      consume: ConsumeType::PerRecipe { return_on_break: false },
    }
  }
}
impl Req {
  builder_method! {
    const;
    count: u64,
    consume: ConsumeType
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
impl Display for Depends {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
    Self::Req(req) => write!(f, "{req}"),
    Self::AllOf(reqs) => write!(f, "({})", reqs.iter().join(" & ")),
    Self::AnyOf(reqs) => write!(f, "({})", reqs.iter().join(" | ")),
    }
  }
}
