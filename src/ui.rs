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

use crate::io::{AVOID_MISTAKE_INPUT_DUR, ScreenWriter};

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
          s.lines(format!("\n{f}{p}")).flush();
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
  fn draw_entries(&mut self, s: &mut ScreenWriter);
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn: FnMut(&StrT) -> StrT, EnterFn> DrawEntries
  for DisplayList<TitleT, StrT, SelectedFn, EnterFn>
{
  fn draw_entries(&mut self, s: &mut ScreenWriter) {
    for (i, DataItem { text, selected, .. }) in self.data.iter_mut().enumerate() {
      if i == self.selecting {
        write!(s, "{}", (selected)(text).as_ref().attribute(Bold)).unwrap();
        s.endl();
      } else {
        s.lines(text);
      }
    }
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, EnterFn> DrawEntries
  for DisplayList<TitleT, StrT, (), EnterFn>
{
  fn draw_entries(&mut self, s: &mut ScreenWriter) {
    for DataItem { text, .. } in self.data.iter_mut() {
      s.lines(text);
    }
  }
}
pub trait Enter {
  type ReT;
  fn enter(&mut self, s: &mut ScreenWriter) -> Option<Self::ReT>;
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

  fn enter(&mut self, s: &mut ScreenWriter) -> Option<ReT> {
    let f = &mut self.data[self.selecting].enter;
    match (f)(self.selecting) {
    Ok(r) => return Some(r),
    Err(e) => {
      s.message(e.to_string());
    }
    }
    None
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn> Enter
  for DisplayList<TitleT, StrT, SelectedFn, ()>
{
  type ReT = ();

  fn enter(&mut self, _s: &mut ScreenWriter) -> Option<()> { None }
}
pub struct DisplayList<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn> {
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
  pub fn run(&mut self, s: &mut ScreenWriter) -> Option<<Self as Enter>::ReT> {
    'io: loop {
      queue!(s.out, MoveTo(self.origin_row, 0)).unwrap();
      if let Some(t) = self.title.as_ref() {
        s.lines(t);
      }
      if self.data.is_empty() {
        s.message(t!("ui.display_list.empty"));
        return None;
      }
      self.draw_entries(s);
      s.end();
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
          if let Some(r) = self.enter(s) {
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
impl ScreenWriter {
  pub fn list<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn>(
    &mut self, mut display_list: DisplayList<TitleT, StrT, SelectedFn, EnterFn>,
  ) -> Option<<DisplayList<TitleT, StrT, SelectedFn, EnterFn> as Enter>::ReT>
  where
    DisplayList<TitleT, StrT, SelectedFn, EnterFn>: DrawEntries + Enter,
  {
    display_list.run(self)
  }

  pub fn message(&mut self, msg: impl AsRef<str>) -> &mut Self {
    self.lines(msg).end();
    sleep(AVOID_MISTAKE_INPUT_DUR);
    self.lines(t!("ui.continue_prompt")).flush();
    loop {
      if let Event::Key(key_event) = read().unwrap()
        && key_event.is_press()
      {
        return self;
      }
    }
  }
}
