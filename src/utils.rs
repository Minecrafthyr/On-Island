use std::io::stdout;

use crossterm::queue;

use super::*;
pub const MS_PER_S: u64 = 1000;
pub const MS_PER_MIN: u64 = 1000 * 60;
#[inline]
pub fn seconds(s: u64) -> u64 { s * MS_PER_S }
#[inline]
pub fn minutes(s: u64) -> u64 { s * MS_PER_MIN }
pub enum NumberResult<T> {
    Number(T),
    Cancel,
}
pub struct NumberRequester<'a, T: std::cmp::PartialOrd + FromStr + Display + Copy> {
    prompt: &'a str,
    range: Option<std::range::RangeInclusive<T>>,
    check: Option<&'a dyn Fn(T) -> Option<String>>,
    default: Option<T>,
}
impl<'a, T: std::cmp::PartialOrd + FromStr + Display + Copy> NumberRequester<'a, T> {
    pub fn new(prompt: &'a str) -> Self { Self { prompt, range: None, check: None, default: None } }

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

    pub fn request(self) -> NumberResult<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let out = &mut stdout();
        let default_str =
            if let Some(v) = self.default { format!("（默认{v}）") } else { "".to_string() };
        let range_str = if let Some(r) = self.range {
            format!("（{}–{}）", r.start, r.last)
        } else {
            "".to_string()
        };
        let s = format!("\n{}{range_str}{default_str}：", self.prompt);
        write_lines(&s);

        let mut buf = String::new();
        loop {
            // let (x, y) = position().unwrap();
            match read().unwrap() {
            Event::Key(key_event) => {
                if key_event.code == KeyCode::Esc {
                    return NumberResult::Cancel;
                }
                if let KeyCode::Char(ch) = key_event.code
                    && ch.is_digit(10)
                {
                    buf.push(ch);
                    execute!(out, MoveToColumn(0), Print(&buf), Clear(UntilNewLine)).unwrap();
                }
                if key_event.code == KeyCode::Backspace {
                    buf.pop();
                    execute!(out, MoveToColumn(0), Print(&buf), Clear(UntilNewLine)).unwrap();
                }
                if key_event.code == KeyCode::Enter {
                    let number = if buf.is_empty() {
                        if let Some(v) = self.default {
                            v
                        } else {
                            write_lines(&format!("\n输入为空。{s}"));
                            continue;
                        }
                    } else {
                        buf.parse().unwrap()
                    };
                    if let Some(r) = self.range
                        && !r.contains(&number)
                    {
                        write_lines(&format!("\n输入无效。{s}"));
                        buf.clear();
                        continue;
                    }
                    if let Some(c) = self.check
                        && let Some(f) = c(number)
                    {
                        write_lines(&format!("\n{f}{s}"));
                        buf.clear();
                        continue;
                    }
                    return NumberResult::Number(number);
                }
            }
            _ => {}
            }
        }
    }
}

pub fn popup_message(msg: &str) {
    let out = &mut stdout();
    execute!(out, MoveTo(0, 0)).unwrap();
    write_lines(msg);
    sleep(Duration::from_millis(200));
    loop {
        match read().unwrap() {
        Event::Key(key_event) if key_event.is_press() => return,
        _ => {}
        }
    }
}

pub fn get_stdout() {}

pub fn write_lines(s: &str) {
    let out = &mut stdout();
    for line in s.lines() {
        queue!(out, Print(line), Clear(UntilNewLine), MoveToNextLine(1)).unwrap();
    }
    out.flush().unwrap();
}
