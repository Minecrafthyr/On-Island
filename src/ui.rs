use std::{
  fmt::{Display, Write as _},
  io::ErrorKind::Other,
  num::NonZero,
  ops::{Bound, RangeBounds},
  str::FromStr,
  thread::sleep,
};

use crossterm::{
  cursor::{MoveLeft, MoveRight, MoveTo},
  event::{Event, KeyCode, KeyEvent, read},
  execute, queue,
  style::{Attribute::Bold, Print, Stylize},
  terminal::{Clear, ClearType::UntilNewLine},
};

use crate::io::{AVOID_MISTAKE_INPUT_DUR, ScreenWriter};
#[expect(unevaluated_default_field_value)]
pub struct NumberRequester<
  'a,
  PromptT: Display,
  T: std::cmp::PartialOrd + FromStr + Display + Copy,
  RangeT: RangeBounds<T>,
> {
  pub prompt: PromptT,
  pub range: Option<RangeT> = None,
  pub check: Option<&'a dyn Fn(T) -> Option<String>> = None,
  pub default: Option<T> = None,
}
impl<
  'a,
  PromptT: Display,
  T: std::cmp::PartialOrd + FromStr + Display + Copy,
  RangeT: RangeBounds<T>,
> NumberRequester<'a, PromptT, T, RangeT>
{
  pub fn request(self) -> Result<Option<T>, std::io::Error>
  where
    <T as FromStr>::Err: std::fmt::Debug,
  {
    let mut s = ScreenWriter::new();
    let mut p = format!("\n{}", self.prompt);
    self.default.map(|d| write!(p, "{}", t!("ui.get_number.default", default = d)));
    if let Some(r) = &self.range {
      _ = match r.start_bound() {
      Bound::Included(v) => write!(p, "[{}", v),
      Bound::Excluded(v) => write!(p, "({}", v),
      Bound::Unbounded => write!(p, "(-∞"),
      };
      _ = p.write_str(", ");
      _ = match r.end_bound() {
      Bound::Included(v) => write!(p, "{}]", v),
      Bound::Excluded(v) => write!(p, "{})", v),
      Bound::Unbounded => write!(p, "+∞)"),
      };
    }
    write!(p, "：").unwrap();
    s.writeln(&p)?.end()?;

    let mut buf = String::new();
    loop {
      let Event::Key(KeyEvent { code, .. }) = read()? else {
        continue;
      };
      // let (x, y) = position().unwrap();
      use KeyCode::*;
      match code {
      Esc => break,
      Char(ch) if ch.is_ascii_digit() => {
        buf.push(ch);
        execute!(s.out, MoveRight(1), Print(&buf), Clear(UntilNewLine)).unwrap();
      }
      Backspace => {
        buf.pop();
        execute!(s.out, MoveLeft(1), Print(&buf), Clear(UntilNewLine)).unwrap();
      }
      Enter => {
        let number = if buf.is_empty() {
          if let Some(v) = self.default {
            v
          } else {
            s.endl()?.writeln(t!("ui.get_number.empty", prompt = p))?.flush()?;
            continue;
          }
        } else {
          buf.parse().unwrap()
        };
        if self.range.as_ref().map(|r| !r.contains(&number)).unwrap_or(false) {
          s.endl()?.writeln(t!("ui.get_number.invalid", prompt = p))?.flush()?;
          buf.clear();
          continue;
        }
        if let Some(c) = self.check
          && let Some(f) = c(number)
        {
          s.endl()?.write(f)?.write(&p)?.endl()?.flush()?;
          buf.clear();
          continue;
        }
        return Ok(Some(number));
      }
      _ => {}
      }
    }
    Ok(None)
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
  fn draw_entries(&mut self, s: &mut ScreenWriter) -> Result<(), std::io::Error>;
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn: FnMut(&StrT) -> StrT, EnterFn> DrawEntries
  for DisplayList<TitleT, StrT, SelectedFn, EnterFn>
{
  fn draw_entries(&mut self, s: &mut ScreenWriter) -> Result<(), std::io::Error> {
    for (i, DataItem { text, selected, .. }) in self.data.iter_mut().enumerate() {
      if self.indexed {
        write!(s, "[{i}] ")
          .map_err(|_| std::io::Error::new(Other, "format error: cannot write to screen"))?;
      }
      if i == self.selecting {
        s.writeln((selected)(text).as_ref().attribute(Bold))?;
      } else {
        s.writeln(text.as_ref())?;
      }
    }
    Ok(())
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, EnterFn> DrawEntries
  for DisplayList<TitleT, StrT, (), EnterFn>
{
  fn draw_entries(&mut self, s: &mut ScreenWriter) -> Result<(), std::io::Error> {
    for DataItem { text, .. } in self.data.iter_mut() {
      s.writeln(text.as_ref())?;
    }
    Ok(())
  }
}
pub trait Enter {
  type ReT;
  fn enter(&mut self, s: &mut ScreenWriter) -> Result<Option<Self::ReT>, std::io::Error>;
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

  fn enter(&mut self, s: &mut ScreenWriter) -> Result<Option<ReT>, std::io::Error> {
    let f = &mut self.data[self.selecting].enter;
    match (f)(self.selecting) {
    Ok(r) => return Ok(Some(r)),
    Err(e) => {
      s.message(e.to_string())?;
    }
    }
    Ok(None)
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn> Enter
  for DisplayList<TitleT, StrT, SelectedFn, ()>
{
  type ReT = ();

  fn enter(&mut self, _s: &mut ScreenWriter) -> Result<Option<()>, std::io::Error> { Ok(None) }
}
#[expect(unevaluated_default_field_value)]
pub struct DisplayList<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn> {
  pub origin_row: Option<NonZero<u16>> = None,
  pub title: Option<TitleT> = None,
  pub data: Vec<DataItem<StrT, SelectedFn, EnterFn>>,
  pub selecting: usize = 0,
  pub overflow: bool = true,
  pub indexed: bool = false,
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn>
  DisplayList<TitleT, StrT, SelectedFn, EnterFn>
{
  pub fn new(title: Option<TitleT>, data: Vec<DataItem<StrT, SelectedFn, EnterFn>>) -> Self {
    Self { title, data, .. }
  }

  pub fn title(mut self, title: TitleT) -> Self {
    self.title = Some(title);
    self
  }

  pub fn indexing(mut self) -> Self {
    self.indexed = true;
    self
  }

  fn recalc_select(&mut self) {
    let len = self.data.len();
    if self.overflow {
      self.selecting %= len;
    } else {
      if self.selecting >= len {
        self.selecting = len - 1;
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
    self.origin_row = Some(NonZero::new(row).unwrap_or(NonZero::new(1).unwrap()));
    self
  }

  pub fn no_overflow(mut self) -> Self {
    self.overflow = false;
    self
  }
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn>
  DisplayList<TitleT, StrT, SelectedFn, EnterFn>
where
  Self: DrawEntries + Enter,
{
  pub fn run(
    &mut self, s: &mut ScreenWriter,
  ) -> Result<Option<<Self as Enter>::ReT>, std::io::Error> {
    'io: loop {
      if let Some(origin_row) = self.origin_row {
        queue!(s.out, MoveTo(origin_row.get(), 0))?;
      }
      if let Some(t) = self.title.as_ref() {
        s.writeln(t.as_ref())?;
      }
      if self.data.is_empty() {
        s.message(t!("ui.display_list.empty"))?;
        return Ok(None);
      }
      self.draw_entries(s)?;
      s.end()?;
      let mut last_press = std::time::Instant::now();
      let mut count = 0usize;

      loop {
        let Event::Key(KeyEvent { code, modifiers: _, kind, state: _ }) = read()? else {
          continue;
        };
        let now = std::time::Instant::now();
        let max_select = self.data.len() - 1;
        use KeyCode::*;
        match code {
        Up =>
          if self.selecting == 0 {
            if self.overflow {
              self.selecting = max_select;
              continue 'io;
            }
          } else {
            self.selecting -= 1;
            continue 'io;
          },
        Down =>
          if self.selecting == max_select {
            if self.overflow {
              self.selecting = 0;
              continue 'io;
            }
          } else {
            self.selecting += 1;
            continue 'io;
          },
        Enter => {
          if let r @ Some(_) = self.enter(s)? {
            return Ok(r);
          }
          break;
        }
        Esc => return Ok(None),
        Char(n) if kind.is_press() && matches!(n, '0'..'9') => {
          let n = n.to_digit(10).unwrap() as usize;
          if now - last_press < std::time::Duration::from_millis(300) {
            count = (count * 10 + n).min(max_select);
            self.selecting = count;
          } else {
            count = n;
          }
        }
        _ => {}
        }
        if kind.is_press() {
          last_press = now;
        }
      }
    }
  }
}
impl ScreenWriter {
  pub fn list<TitleT: AsRef<str>, StrT: AsRef<str>, SelectedFn, EnterFn>(
    &mut self, mut display_list: DisplayList<TitleT, StrT, SelectedFn, EnterFn>,
  ) -> Result<Option<<DisplayList<TitleT, StrT, SelectedFn, EnterFn> as Enter>::ReT>, std::io::Error>
  where
    DisplayList<TitleT, StrT, SelectedFn, EnterFn>: DrawEntries + Enter,
  {
    display_list.run(self)
  }

  pub fn message(&mut self, msg: impl AsRef<str>) -> Result<&mut Self, std::io::Error> {
    self.writeln(msg.as_ref())?;
    self.end()?;
    sleep(AVOID_MISTAKE_INPUT_DUR);
    self.writeln(t!("ui.continue_prompt"))?.flush()?;
    loop {
      if let Event::Key(key_event) = read()?
        && key_event.is_press()
      {
        return Ok(self);
      }
    }
  }
}
