use std::{
  error::Error,
  fmt::{Display, Write as _},
  io::Write as _,
  ops::RangeInclusive,
  str::FromStr,
  thread::sleep,
};

use crossterm::{
  cursor::{MoveLeft, MoveRight, MoveTo, MoveToRow, position},
  event::{Event, KeyCode, read},
  execute, queue,
  style::Print,
  terminal::{
    Clear,
    ClearType::{FromCursorDown, UntilNewLine},
  },
};

use crate::{
  statics::stdout,
  utils::{queue_lines, write_lines},
};

pub fn popup_message(msg: impl AsRef<str>) {
  let out = stdout();
  execute!(out, MoveTo(0, 0)).unwrap();
  write_lines(msg);
  execute!(stdout(), Clear(FromCursorDown)).unwrap();
  sleep(std::time::Duration::from_millis(200));
  write_lines("输入任意键继续……");
  loop {
    match read().unwrap() {
    Event::Key(key_event) if key_event.is_press() => return,
    _ => {}
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
    let out = stdout();
    let mut s = format!("\n{}", self.prompt);
    self.default.map(|v| write!(s, "（默认{v}）"));
    self.range.map(|r| write!(s, "（{}–{}）", r.start, r.last));
    write!(s, "：").unwrap();
    write_lines(&s);

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
        execute!(out, MoveRight(1), Print(&buf), Clear(UntilNewLine)).unwrap();
      }
      if key_event.code == KeyCode::Backspace {
        buf.pop();
        execute!(out, MoveLeft(1), Print(&buf), Clear(UntilNewLine)).unwrap();
      }
      if key_event.code == KeyCode::Enter {
        let number = if buf.is_empty() {
          if let Some(v) = self.default {
            v
          } else {
            write_lines(format!("\n输入为空。{s}"));
            continue;
          }
        } else {
          buf.parse().unwrap()
        };
        if let Some(r) = self.range
          && !r.contains(&number)
        {
          write_lines(format!("\n输入无效。{s}"));
          buf.clear();
          continue;
        }
        if let Some(c) = self.check
          && let Some(f) = c(number)
        {
          write_lines(format!("\n{f}{s}"));
          buf.clear();
          continue;
        }
        return Some(number);
      }
    }
  }
}

pub struct DisplayList<TitleT: AsRef<str>, StrT: AsRef<str>> {
  origin: u16,
  title: Option<TitleT>,
  data: Vec<(StrT, Option<StrT>, Box<dyn FnOnce(usize) -> Result<(), Box<dyn Error>>>)>,
  selecting: usize,
  flow: bool,
}
impl<TitleT: AsRef<str>, StrT: AsRef<str>> DisplayList<TitleT, StrT> {
  pub fn new(
    data: Vec<(StrT, Option<StrT>, Box<dyn FnOnce(usize) -> Result<(), Box<dyn Error>>>)>,
  ) -> Self {
    Self { origin: position().unwrap().1, title: None, data, selecting: 0, flow: true }
  }

  pub fn run(&mut self) -> bool {
    'io: loop {
      self.o();
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
          let (_, _, callback) = self.data.swap_remove(self.selecting);
          match callback(self.selecting) {
          Ok(_) => return true,
          Err(e) => popup_message(e.to_string()),
          }
          break;
        }
        Esc => return false,
        _ => {}
        }
      }
    }
  }

  fn o(&self) {
    queue!(stdout(), MoveToRow(self.origin)).unwrap();
    self.title.as_ref().map(queue_lines);
    for (i, (u, s, _)) in self.data.iter().enumerate() {
      let sel = i == self.selecting;
      queue_lines(if sel && let Some(s) = s { s } else { u })
    }
    stdout().flush().unwrap();
  }
}
