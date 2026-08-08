use std::{
  fmt::Display,
  io::{Stdout, Write as _, stdout},
  ops::{Deref, DerefMut},
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

pub struct NewScreenWriter {
  s: ScreenWriter,
}
impl Default for NewScreenWriter {
  fn default() -> Self { Self::new() }
}

impl NewScreenWriter {
  pub fn new() -> Self {
    let mut s = Self { s: ScreenWriter::new() };
    queue!(s.out, MoveTo(0, 0)).unwrap();
    s
  }
}
impl Deref for NewScreenWriter {
  type Target = ScreenWriter;

  fn deref(&self) -> &Self::Target { &self.s }
}

impl DerefMut for NewScreenWriter {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.s }
}
