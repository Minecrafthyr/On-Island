use super::*;
#[derive(
  Debug, Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum PushActionError {
  NoSuchBodyPart,
  BodyPartUsed,
}
impl NameAndDesc for PushActionError {
  const PREFIX: &str = "wait_action_result";

  fn get_id(&self) -> &str { self.into() }
}
impl Error for PushActionError {}
impl Display for PushActionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name()) }
}
pub struct ActionContent {
  pub body_parts: Vec<(&'static BodyPart, IsExclusive)>,
  pub effects: Vec<Effect>,
}

impl ActionContent {
  pub fn new(body_parts: Vec<(&'static BodyPart, IsExclusive)>, effects: Vec<Effect>) -> Self {
    Self { body_parts, effects }
  }
}
#[derive(Clone)]
pub struct Action {
  pub id: &'static str,
  pub func: Arc<dyn Fn(&mut Player) -> ActionContent>,
  pub progress: Duration,
  pub total_dur: Duration,
}
impl Action {
  pub fn no_progress(
    id: &'static str, f: impl Fn(&mut Player) -> ActionContent + 'static, total_dur: Duration,
  ) -> Self {
    Self { id, func: Arc::new(f), progress: Duration::ZERO, total_dur }
  }
}
impl Player {
  fn eval_actions(&mut self) {
    let funcs: Vec<_> = self.actions.iter().map(|a| a.func.clone()).collect();
    'func: for f in funcs {
      let ActionContent { body_parts, effects } = (f)(self);
      for (bp, is_exclusive) in body_parts {
        if let Some((_, e_is_exclusive)) =
          self.used_body_parts.iter_mut().find(|(ebp, _)| *ebp == bp)
        {
          if *e_is_exclusive {
            continue 'func;
          } else if is_exclusive {
            *e_is_exclusive = true;
          }
        } else {
          self.used_body_parts.push((bp, is_exclusive));
        }
      }
      self.effects.extend(effects);
    }
  }

  pub(super) fn tick_actions(&mut self) {
    let mut action_idx = 0;
    while action_idx < self.actions.len() {
      let efficiency = 1.ms() * self.get_efficiency();
      let action = &mut self.actions[action_idx];
      action.progress += efficiency;
      if action.progress > action.total_dur {
        self.actions.remove(action_idx);
        self.eval_actions();
      } else {
        action_idx += 1;
      }
    }
  }

  pub fn try_push_action(&mut self, action: Action) -> Result<(), PushActionError> {
    let ActionContent { body_parts, effects: _ } = (action.func)(self);
    for (bp, _is_exclusive) in body_parts {
      if self.used_body_parts.iter_mut().contains(&(bp, true)) {
        return Err(PushActionError::BodyPartUsed);
      }
    }
    Ok(())
  }
}
