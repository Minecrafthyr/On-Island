use std::{
  error::Error,
  fmt::{Display, Write as _},
  ops::RangeInclusive,
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

pub fn popup_message(msg: impl AsRef<str>) {
  let mut s = NewScreenWriter::new();
  s.queue_lines(msg);
  s.end();
  sleep(std::time::Duration::from_millis(200));
  s.write_lines(t!("ui.continue_prompt"));
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
  s.queue_lines(msg);
  s.end();
  sleep(std::time::Duration::from_millis(200));
  s.write_lines(t!("ui.continue_prompt"));
  loop {
    if let Event::Key(key_event) = read().unwrap()
      && key_event.is_press()
    {
      return;
    }
  }
}

pub struct NumberRequester<'a, PromptT: Display, T: std::cmp::PartialOrd + FromStr + Display + Copy>
{
  prompt: PromptT,
  range: Option<std::range::RangeInclusive<T>>,
  check: Option<&'a dyn Fn(T) -> Option<String>>,
  default: Option<T>,
}
impl<'a, PromptT: Display, T: std::cmp::PartialOrd + FromStr + Display + Copy>
  NumberRequester<'a, PromptT, T>
{
  pub fn new(prompt: PromptT) -> Self { Self { prompt, range: None, check: None, default: None } }

  pub fn range(mut self, range: RangeInclusive<T>) -> Self {
    self.range = Some(std::range::RangeInclusive::<T>::from(range));
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
    self.range.map(|r| {
      p.write_str(&if r.start == r.last {
        t!("ui.get_number.value", value = r.start)
      } else {
        t!("ui.get_number.range", start = r.start, last = r.last)
      })
    });
    write!(p, "：").unwrap();
    s.queue_lines(&p);
    s.end();

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
            s.queueln();
            s.write_lines(t!("ui.get_number.empty", prompt = p));
            continue;
          }
        } else {
          buf.parse().unwrap()
        };
        if let Some(r) = self.range
          && !r.contains(&number)
        {
          s.queueln();
          s.write_lines(t!("ui.get_number.invalid", prompt = p));
          buf.clear();
          continue;
        }
        if let Some(c) = self.check
          && let Some(f) = c(number)
        {
          s.write_lines(format!("\n{f}{p}"));
          buf.clear();
          continue;
        }
        return Some(number);
      }
    }
  }
}
pub type DisplayListEnter = Box<dyn Fn(usize) -> Result<(), Box<dyn Error>>>;
pub struct DisplayList<TitleT: AsRef<str>, StrT: AsRef<str>> {
  s: ScreenWriter,
  origin: u16,
  title: Option<TitleT>,
  data: Vec<(StrT, Option<StrT>, Option<DisplayListEnter>)>,
  selecting: usize,
  flow: bool,
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>> DisplayList<TitleT, StrT> {
  pub fn new(
    title: Option<TitleT>, data: Vec<(StrT, Option<StrT>, Option<DisplayListEnter>)>,
  ) -> Self {
    Self {
      s: ScreenWriter::new(),
      origin: position().unwrap().1,
      title,
      data,
      selecting: 0,
      flow: true,
    }
  }

  pub fn title(mut self, title: TitleT) -> Self {
    self.title = Some(title);
    self
  }

  pub fn select_default(mut self, index: usize) -> Self {
    self.selecting = index;
    self.recalc_select();
    self
  }

  pub fn at(mut self, row: u16) -> Self {
    self.origin = row;
    self
  }

  pub fn no_flow(mut self) -> Self {
    self.flow = false;
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

  pub fn run(&mut self) -> bool {
    'io: loop {
      queue!(self.s.out, MoveTo(self.origin, 0)).unwrap();
      if let Some(t) = self.title.as_ref() {
        self.s.queue_lines(t)
      }
      if self.data.is_empty() {
        self.s.queue_lines(t!("ui.display_list.empty"));
        self.s.flush();
        return false;
      }
      for (i, (u, s, _)) in self.data.iter().enumerate() {
        let sel = i == self.selecting;
        if sel && let Some(s) = s {
          write!(self.s, "{}", s.as_ref().attribute(Bold)).unwrap();
        } else {
          self.s.queue_lines(u)
        }
      }
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
        Enter =>
          if let Some(f) = &self.data[self.selecting].2 {
            match (f)(self.selecting) {
            Ok(_) => return true,
            Err(e) => popup_message(e.to_string()),
            }
            break;
          },
        Esc => return false,
        _ => {}
        }
      }
    }
  }
}
