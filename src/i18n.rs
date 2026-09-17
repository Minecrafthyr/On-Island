use std::{
  collections::HashMap,
  sync::{LazyLock, Mutex},
};

pub struct I18n {
  translations: HashMap<&'static str, HashMap<&'static str, &'static str>>,
  current_langs: Vec<&'static str>,
}

static I18N: LazyLock<Mutex<I18n>> = LazyLock::new(|| Mutex::new(I18n::new()));

impl I18n {
  pub fn new() -> Self { Self { translations: HashMap::new(), current_langs: Vec::new() } }

  pub fn set_langs(&mut self, langs: impl IntoIterator<Item = &'static str>) {
    self.current_langs = langs.into_iter().collect();
  }

  pub fn get_lang(&self, lang: &str, key: &str) -> Option<&'static str> {
    self.translations.get(lang).and_then(|m| m.get(key).map(|k| *k))
  }

  pub fn tr(&self, key: &str) -> Option<&'static str> {
    self.current_langs.iter().find_map(|lang| self.get_lang(lang, key))
  }

  pub fn get(&self, lang: &str, key: &str) -> Option<&'static str> {
    if let x @ Some(_) = self.get_lang(lang, key) { x } else { self.tr(key) }
  }

  pub fn register<IntoStr: Into<&'static str>>(
    &mut self, lang: &'static str, entries: impl IntoIterator<Item = (IntoStr, IntoStr)>,
  ) {
    self
      .translations
      .entry(lang)
      .or_default()
      .extend(entries.into_iter().map(|(k, v)| (k.into(), v.into())));
  }
}

pub fn tr(key: &'static str) -> Option<&'static str> { I18N.lock().ok()?.tr(key) }
pub fn get(lang: &str, key: &'static str) -> Option<&'static str> {
  I18N.lock().ok()?.get(lang, key)
}
#[macro_export]
macro_rules! register_lang {
  ($lang:expr, { $($key:literal: $val:expr),* $(,)? }) => {
    if let Ok(locked) = I18N.lock() {
      let mut m = locked.translations.entry($lang).or_default();
      $(m.insert($key, $val);)*

    }
  };
}

#[macro_export]
macro_rules! get {
  ($lang:expr, $inst:expr) => {
    $crate::i18n::get($lang, $inst)
  };
  ($i18n:expr; $lang:expr, $inst:expr) => {{
    $crate::i18n::get($lang, $inst)
  }};
  ($lang:expr, $inst:expr, $($values:tt)*) => {{
    $crate::i18n::get($lang, $inst).and_then(|template|::strfmt::strfmt!(template, $($values)*).ok())
  }};
  ($i18n:expr; $lang:expr, $inst:expr, $($values:tt)*) => {{
    $i18n.get($lang, $inst).and_then(|template|::strfmt::strfmt!(template, $($values)*).ok())
  }};
}
#[macro_export]
macro_rules! tr {
  ($inst:expr) => {
    $crate::i18n::tr($inst).unwrap_or($inst)
  };
  ($i18n:expr; $inst:expr) => {
    $i18n.tr($inst).unwrap_or($inst)
  };
  ($inst:expr, $($values:tt)*) => {{
    use ::std::borrow::Cow;
    let template = $crate::i18n::tr($inst).unwrap_or($inst);
    match ::strfmt::strfmt!(template, $($values)*) {
      Ok(ok) => Cow::Owned(ok),
      Err(_) => Cow::Borrowed($inst),
    }
  }};
  ($i18n:expr; $inst:expr, $($values:tt)*) => {{
    use ::std::borrow::Cow;
    let template = $i18n.tr($inst).unwrap_or($inst);
    match ::strfmt::strfmt!(template, $($values)*) {
      Ok(ok) => Cow::Owned(ok),
      Err(_) => Cow::Borrowed($inst),
    }
  }};
}
#[macro_export]
macro_rules! dict {
  {$($key:literal : $value:expr),*$(,)?} => {
    [$(($key, $value),)*]
  };
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_i18n() -> Result<(), Box<dyn std::error::Error>> {
    let mut locked = I18N.lock()?;
    locked.set_langs(["en", "zh"]);

    locked.register("en", dict! {
      "greeting": "Hello, {name}!",
      "farewell": "Goodbye!",
    });
    locked.register("zh", dict! {
      "greeting": "你好，{name}！",
      "areyouok": "{pron}还好吗，{name}？",
      "xiaoming": "小明"
    });

    assert_eq!(locked.get("en", "greeting"), Some("Hello, {name}!"));
    assert_eq!(locked.get("zh", "greeting"), Some("你好，{name}！"));
    assert_eq!(locked.get("fr", "greeting"), Some("Hello, {name}!"));
    assert_eq!(locked.get("en", "none"), None);

    locked.set_langs(["zh", "en"]);
    assert_eq!(tr!(locked; "farewell"), "Goodbye!");
    assert_eq!(get!(locked; "en", "greeting", name => "Alice"), Some("Hello, Alice!".to_string()));
    let name = tr("xiaoming").unwrap();
    assert_eq!(tr!(locked; "greeting", name), "你好，小明！");
    assert_eq!(tr!(locked; "areyouok", name, pron => "你"), "你还好吗，小明？");
    Ok(())
  }
}
