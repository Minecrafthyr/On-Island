use std::{
  fmt::Display,
  io::{Stdout, Write as _, stdout},
};

use crossterm::{
  cursor::{MoveTo, MoveToNextLine},
  queue,
  style::Print,
  terminal::{
    Clear,
    ClearType::{FromCursorDown, UntilNewLine},
  },
};

pub struct ScreenWriter {
  pub out: Stdout,
}
impl Default for ScreenWriter {
  fn default() -> Self { Self::new() }
}
pub const AVOID_MISTAKE_INPUT_DUR: std::time::Duration = std::time::Duration::from_millis(400);

impl ScreenWriter {
  pub fn new() -> Self { Self { out: stdout() } }

  pub fn new_screen() -> Self {
    let mut s = Self { out: stdout() };
    queue!(s.out, MoveTo(0, 0)).unwrap();
    s
  }

  pub fn line(&mut self, line: impl Display) -> &mut Self {
    write!(self.out, "{line}").unwrap();
    self.endl()
  }

  pub fn lines(&mut self, s: impl AsRef<str>) -> &mut Self {
    for line in s.as_ref().lines() {
      self.line(line);
    }
    self
  }

  pub fn endl(&mut self) -> &mut Self {
    queue!(self.out, Clear(UntilNewLine), MoveToNextLine(1)).unwrap();
    self
  }

  pub fn flush(&mut self) { self.out.flush().unwrap() }

  pub fn end(&mut self) {
    queue!(self.out, Clear(FromCursorDown)).unwrap();
    self.flush();
  }
}
impl std::fmt::Write for ScreenWriter {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    let mut iter = s.lines().peekable();
    while let Some(line) = iter.next() {
      queue!(self.out, Print(line)).map_err(|_| std::fmt::Error)?;
      if iter.peek().is_some() {
        queue!(self.out, Clear(UntilNewLine), MoveToNextLine(1)).map_err(|_| std::fmt::Error)?;
      }
    }
    Ok(())
  }
}
