use std::{
  fmt::{Display, Write},
  io::{ErrorKind::Other, Stdout, Write as _, stdout},
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
    let mut out = stdout();
    queue!(out, MoveTo(0, 0)).unwrap();
    Self { out }
  }

  pub fn write(&mut self, s: impl Display) -> Result<&mut Self, std::io::Error> {
    self
      .write_fmt(format_args!("{s}"))
      .map_err(|_| std::io::Error::new(Other, "Failed to write to screen"))?;
    Ok(self)
  }

  pub fn endl(&mut self) -> Result<&mut Self, std::io::Error> {
    queue!(self.out, Clear(UntilNewLine), MoveToNextLine(1))?;
    Ok(self)
  }

  pub fn writeln(&mut self, s: impl Display) -> Result<&mut Self, std::io::Error> {
    self.write(s)?.endl()
  }

  pub fn flush(&mut self) -> Result<&mut Self, std::io::Error> {
    self.out.flush()?;
    Ok(self)
  }

  pub fn end(&mut self) -> Result<(), std::io::Error> {
    queue!(self.out, Clear(FromCursorDown))?;
    self.flush()?;
    Ok(())
  }
}
impl std::fmt::Write for ScreenWriter {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    let mut iter = s.lines().peekable();
    while let Some(line) = iter.next() {
      queue!(self.out, Print(line)).map_err(|_| std::fmt::Error)?;
      if iter.peek().is_some() {
        self.endl().map_err(|_| std::fmt::Error)?;
      }
    }
    Ok(())
  }
}
