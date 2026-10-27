use inew::New;

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

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.into()) }
}
impl Error for PushActionError {}
impl Display for PushActionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name()) }
}
#[derive(New)]
pub struct ActionContent {
  pub body_parts: Vec<(&'static BodyPart, IsExclusive)>,
  pub effects: Vec<Effect>,
}

#[derive(Clone)]
pub struct Action {
  pub id: &'static str,
  pub f: Rc<dyn Fn(&mut Player) -> ActionContent>,
  pub progress: Duration = Duration::ZERO,
  pub total_dur: Duration,
}
impl Player {
  fn eval_actions(&mut self) {
    let funcs = self.actions.iter().map(|a| a.f.clone()).collect_vec();
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
    let mut idx = 0;
    while idx < self.actions.len() {
      let efficiency = 1.ms() * self.get_efficiency();
      let action = &mut self.actions[idx];
      action.progress += efficiency;
      if action.progress > action.total_dur {
        self.actions.remove(idx);
        self.eval_actions();
      } else {
        idx += 1;
      }
    }
  }

  pub fn try_push_action(&mut self, action: Action) -> Result<(), PushActionError> {
    let ActionContent { body_parts, effects: _ } = (action.f)(self);
    if body_parts.into_iter().any(|(bp, _)| self.used_body_parts.contains(&(bp, true))) {
      Err(PushActionError::BodyPartUsed)
    } else {
      self.actions.push(action);
      Ok(())
    }
  }
}
