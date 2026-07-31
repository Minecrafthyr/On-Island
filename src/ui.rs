use std::{
  fmt::{Display, Write as _},
  ops::{Bound, RangeBounds},
  str::FromStr,
  thread::sleep,
};

use crossterm::{
  cursor::{MoveLeft, MoveRight, MoveTo, position},
  event::{Event, KeyCode, read},
  execute, queue,
  style::{Attribute::Bold, Print, Stylize},
  terminal::{Clear, ClearType::UntilNewLine},
};

use crate::io::{NewScreenWriter, ScreenWriter};

const AVOID_MISTAKE_INPUT_DUR: std::time::Duration = std::time::Duration::from_millis(200);
pub fn popup_message(msg: impl AsRef<str>) {
  let mut s = NewScreenWriter::new();
  s.lines(msg).end();
  sleep(AVOID_MISTAKE_INPUT_DUR);
  s.lines(t!("ui.continue_prompt")).flush();
  loop {
    if let Event::Key(key_event) = read().unwrap()
      && key_event.is_press()
    {
      return;
    }
  }
}
pub fn message(msg: impl AsRef<str>) {
  let mut s = ScreenWriter::new();
  s.lines(msg).end();
  sleep(AVOID_MISTAKE_INPUT_DUR);
  s.lines(t!("ui.continue_prompt")).flush();
  loop {
    if let Event::Key(key_event) = read().unwrap()
      && key_event.is_press()
    {
      return;
    }
  }
}

pub struct NumberRequester<
  'a,
  PromptT: Display,
  T: std::cmp::PartialOrd + FromStr + Display + Copy,
  RangeT: RangeBounds<T>,
> {
  prompt: PromptT,
  range: Option<RangeT>,
  check: Option<&'a dyn Fn(T) -> Option<String>>,
  default: Option<T>,
}
impl<
  'a,
  PromptT: Display,
  T: std::cmp::PartialOrd + FromStr + Display + Copy,
  RangeT: RangeBounds<T>,
> NumberRequester<'a, PromptT, T, RangeT>
{
  pub fn new(prompt: PromptT) -> Self { Self { prompt, range: None, check: None, default: None } }

  pub fn range(mut self, range: RangeT) -> Self {
    self.range = Some(range);
    self
  }

  pub fn check(mut self, check: &'a dyn Fn(T) -> Option<String>) -> Self {
    self.check = Some(check);
    self
  }

  pub fn default(mut self, default: T) -> Self {
    self.default = Some(default);
    self
  }

  pub fn request(self) -> Option<T>
  where
    <T as FromStr>::Err: std::fmt::Debug,
  {
    let mut s = ScreenWriter::new();
    let mut p = format!("\n{}", self.prompt);
    self.default.map(|d| write!(p, "{}", t!("ui.get_number.default", default = d)));
    if let Some(r) = &self.range {
      let _ = match r.start_bound() {
      Bound::Included(v) => write!(p, "[{}", v),
      Bound::Excluded(v) => write!(p, "({}", v),
      Bound::Unbounded => p.write_str("(-∞"),
      };
      let _ = p.write_str(", ");
      let _ = match r.end_bound() {
      Bound::Included(v) => write!(p, "{}]", v),
      Bound::Excluded(v) => write!(p, "{})", v),
      Bound::Unbounded => p.write_str("+∞)"),
      };
    }
    write!(p, "：").unwrap();
    s.lines(&p).end();

    let mut buf = String::new();
    loop {
      // let (x, y) = position().unwrap();
      let Event::Key(key_event) = read().unwrap() else { continue };
      if key_event.code == KeyCode::Esc {
        return None;
      }
      if let KeyCode::Char(ch) = key_event.code
        && ch.is_ascii_digit()
      {
        buf.push(ch);
        execute!(s.out, MoveRight(1), Print(&buf), Clear(UntilNewLine)).unwrap();
      }
      if key_event.code == KeyCode::Backspace {
        buf.pop();
        execute!(s.out, MoveLeft(1), Print(&buf), Clear(UntilNewLine)).unwrap();
      }
      if key_event.code == KeyCode::Enter {
        let number = if buf.is_empty() {
          if let Some(v) = self.default {
            v
          } else {
            s.endl().lines(t!("ui.get_number.empty", prompt = p)).flush();
            continue;
          }
        } else {
          buf.parse().unwrap()
        };
        if self.range.as_ref().map(|r| !r.contains(&number)).unwrap_or(false) {
          s.endl().lines(t!("ui.get_number.invalid", prompt = p)).flush();
          buf.clear();
          continue;
        }
        if let Some(c) = self.check
          && let Some(f) = c(number)
        {
          {
            let this = &mut s;
            let s = format!("\n{f}{p}");
            this.lines(s).flush();
          };
          buf.clear();
          continue;
        }
        return Some(number);
      }
    }
  }
}
pub struct DataItem<
  StrT: AsRef<str>,
  SelectedFn, //FnMut(&StrT) -> StrT
  EnterFn,    //FnMut(usize) -> Result<ReT, Box<dyn ToString>>
> {
  pub text: StrT,
  pub selected: SelectedFn,
  pub enter: EnterFn,
}

pub trait DrawEntries {
  fn draw_entries(&mut self);
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn: FnMut(&StrT) -> StrT, EnterFn> DrawEntries
  for DisplayList<TitleT, StrT, SelectedFn, EnterFn>
{
  fn draw_entries(&mut self) {
    for (i, DataItem { text, selected, .. }) in self.data.iter_mut().enumerate() {
      if i == self.selecting {
        write!(self.s, "{}", (selected)(text).as_ref().attribute(Bold)).unwrap();
      } else {
        self.s.lines(text);
      }
    }
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, EnterFn> DrawEntries
  for DisplayList<TitleT, StrT, (), EnterFn>
{
  fn draw_entries(&mut self) {
    for DataItem { text, .. } in self.data.iter_mut() {
      self.s.lines(text);
    }
  }
}
pub trait Enter {
  type ReT;
  fn enter(&mut self) -> Option<Self::ReT>;
}

impl<
  ReT,
  TitleT: AsRef<str>,
  StrT: AsRef<str>,
  SelectedFn,
  EnterFn: FnMut(usize) -> Result<ReT, Box<dyn ToString>>,
> Enter for DisplayList<TitleT, StrT, SelectedFn, EnterFn>
{
  type ReT = ReT;

  fn enter(&mut self) -> Option<ReT> {
    let f = &mut self.data[self.selecting].enter;
    match (f)(self.selecting) {
    Ok(r) => return Some(r),
    Err(e) => message(e.to_string()),
    }
    None
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn> Enter
  for DisplayList<TitleT, StrT, SelectedFn, ()>
{
  type ReT = ();

  fn enter(&mut self) -> Option<()> { None }
}
pub struct DisplayList<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn> {
  s: ScreenWriter,
  origin_row: u16,
  title: Option<TitleT>,
  data: Vec<DataItem<StrT, SelectedFn, EnterFn>>,
  selecting: usize,
  flow: bool,
  indexed: bool,
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn>
  DisplayList<TitleT, StrT, SelectedFn, EnterFn>
{
  pub fn new(title: Option<TitleT>, data: Vec<DataItem<StrT, SelectedFn, EnterFn>>) -> Self {
    Self {
      s: ScreenWriter::new(),
      origin_row: position().unwrap().1,
      title,
      data,
      selecting: 0,
      flow: true,
      indexed: false,
    }
  }

  pub fn title(mut self, title: TitleT) -> Self {
    self.title = Some(title);
    self
  }

  /// Turn on index before display item to select instantly.
  pub fn indexing(mut self) -> Self {
    self.indexed = true;
    self
  }

  fn recalc_select(&mut self) {
    if self.flow {
      self.selecting %= self.data.len();
    } else {
      if self.selecting >= self.data.len() {
        self.selecting = self.data.len() - 1;
      }
    }
  }

  /// Select index on startup
  pub fn select_default(mut self, index: usize) -> Self {
    self.selecting = index;
    self.recalc_select();
    self
  }

  pub fn at(mut self, row: u16) -> Self {
    self.origin_row = row;
    self
  }

  /// Disable flow from max to zero index
  pub fn no_flow(mut self) -> Self {
    self.flow = false;
    self
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn>
  DisplayList<TitleT, StrT, SelectedFn, EnterFn>
where
  Self: DrawEntries + Enter,
{
  pub fn run(&mut self) -> Option<<Self as Enter>::ReT> {
    'io: loop {
      queue!(self.s.out, MoveTo(self.origin_row, 0)).unwrap();
      if let Some(t) = self.title.as_ref() {
        self.s.lines(t);
      }
      if self.data.is_empty() {
        self.s.lines(t!("ui.display_list.empty")).flush();
        return None;
      }
      self.draw_entries();
      self.s.flush();
      loop {
        let Event::Key(key_event) = read().unwrap() else { continue };
        use KeyCode::*;
        match key_event.code {
        Up =>
          if self.selecting == 0 {
            if self.flow {
              self.selecting = self.data.len() - 1;
              continue 'io;
            }
          } else {
            self.selecting -= 1;
            continue 'io;
          },
        Down =>
          if self.selecting == self.data.len() - 1 {
            if self.flow {
              self.selecting = 0;
              continue 'io;
            }
          } else {
            self.selecting += 1;
            continue 'io;
          },
        Enter => {
          if let Some(r) = self.enter() {
            return Some(r);
          }
          break;
        }
        Esc => return None,
        _ => {}
        }
      }
    }
  }
}
