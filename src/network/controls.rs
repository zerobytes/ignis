//! Client-local ownership, intent IDs and public selection. Never authority or socket I/O.
use super::{client::State, protocol::*};
use crate::{
    model::{Result, invalid},
    physics::Input,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub world: u64,
    pub session: u64,
    pub connection: u64,
    pub actor: Target,
    pub alive: bool,
}
impl Context {
    pub fn current(state: &State) -> Option<Self> {
        if !state.connected || state.content.is_none() || state.replica.tick().is_none() {
            return None;
        }
        let owner = state.owner.as_ref()?;
        let actor = state.replica.actors().get(&owner.actor.actor);
        if actor.is_some_and(|a| a.life != owner.actor.life) {
            return None;
        }
        let alive = owner.alive && actor.is_none_or(|a| a.health.is_none_or(|h| h.units > 0));
        Some(Self {
            world: state.world,
            session: state.session,
            connection: state.connection,
            actor: owner.actor,
            alive,
        })
    }
    pub fn valid(self, state: &State) -> bool {
        Self::current(state) == Some(self) && self.alive
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Hold {
    pub context: Context,
    pub input: Input,
}
#[derive(Debug, Clone, Copy)]
pub enum InputUpdate {
    Hold(Hold),
    Raw(Movement),
}
pub struct Controls {
    scope: String,
    connection: Option<(u64, u64, u64)>,
    revision: u64,
    pub context: Option<Context>,
    held: Option<(String, Hold)>,
    pub selected: Option<Target>,
    issued: u64,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            scope: crate::identity::fresh_scope(),
            connection: None,
            revision: 0,
            context: None,
            held: None,
            selected: None,
            issued: 0,
        }
    }
}
impl Controls {
    pub fn sync(&mut self, state: &State) {
        let connection = (state.world, state.session, state.connection);
        if self.connection != Some(connection) {
            self.connection = Some(connection);
            self.issued = state.last_operation;
        }
        let context = Context::current(state);
        if self.context != context {
            self.context = context;
            self.held = None;
            self.selected = None;
        }
        if self
            .selected
            .is_some_and(|t| Self::target(state, t).is_err())
        {
            self.selected = None;
        }
    }
    pub fn target(state: &State, target: Target) -> Result<()> {
        if state
            .replica
            .actors()
            .get(&target.actor)
            .is_none_or(|a| !a.appearance.kind.actor() || a.life != target.life)
        {
            return Err(invalid("Selected remote actor/life is not current"));
        }
        Ok(())
    }
    pub fn prepare_hold(&mut self, state: &State, input: Input) -> Result<(String, Hold, u64)> {
        crate::engine::Engine::validate_input(input)?;
        self.sync(state);
        let context = Context::current(state)
            .filter(|c| c.alive)
            .ok_or_else(|| invalid("Remote character is not ready/alive"))?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Remote input identity exhausted"))?;
        Ok((
            format!("ignis-remote-input:{}:{revision:016x}", self.scope),
            Hold { context, input },
            revision,
        ))
    }
    pub fn adopt_hold(&mut self, token: String, hold: Hold, revision: u64) {
        self.context = Some(hold.context);
        self.revision = revision;
        self.held = Some((token, hold));
    }
    pub fn release(&mut self, state: &State, token: &str) -> Option<Hold> {
        self.sync(state);
        let (t, hold) = self.held.as_ref()?;
        if t != token || !hold.context.valid(state) {
            return None;
        }
        let neutral = Hold {
            input: Input::default(),
            ..*hold
        };
        self.held = None;
        Some(neutral)
    }
    pub fn clear_input(&mut self) {
        self.held = None;
    }
    pub fn mode(&self) -> crate::model::MovementMode {
        self.held
            .as_ref()
            .map_or(Default::default(), |(_, h)| h.input.mode)
    }
    pub fn prepare_operation(&mut self, state: &State, intent: Intent) -> Result<Operation> {
        self.sync(state);
        let context = self
            .context
            .filter(|c| c.alive)
            .ok_or_else(|| invalid("Remote character is not ready/alive"))?;
        if self.issued > state.last_operation {
            return Err(invalid(
                "Previous operation awaits admission; reconnect to resync if it was rejected",
            ));
        }
        let id = state
            .last_operation
            .checked_add(1)
            .ok_or_else(|| invalid("Remote operation identity exhausted"))?;
        Ok(Operation {
            id,
            life: context.actor.life,
            intent,
        })
    }
    pub fn issued(&mut self, id: u64) {
        self.issued = self.issued.max(id);
    }
    pub fn use_ability(
        &mut self,
        state: &State,
        slot: u8,
        target: Option<Target>,
        aim: Option<crate::abilities::Aim>,
    ) -> Result<Operation> {
        self.sync(state);
        let grant = state
            .grants
            .iter()
            .find(|g| g.slot == slot)
            .ok_or_else(|| invalid("Unknown remote granted slot"))?;
        use crate::abilities::Selection;
        let intent = match grant.selection {
            Selection::SelfActor => {
                if target.is_some() || aim.is_some() {
                    return Err(invalid("Self ability takes no external target or aim"));
                }
                Intent::Activate { slot, target: None }
            }
            Selection::Actor => {
                if aim.is_some() {
                    return Err(invalid("Actor ability takes no spatial aim"));
                }
                let target = target
                    .or(self.selected)
                    .ok_or_else(|| invalid("Select a remote character"))?;
                Self::target(state, target)?;
                Intent::Activate {
                    slot,
                    target: Some(target),
                }
            }
            Selection::Spatial => {
                if target.is_some() {
                    return Err(invalid("Spatial ability takes no actor target"));
                }
                let aim = aim.ok_or_else(|| invalid("Spatial ability requires explicit aim"))?;
                aim.validate()?;
                Intent::Aim { slot, aim }
            }
        };
        self.prepare_operation(state, intent)
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"held":self.held.is_some(),"revision":self.revision,"selected":self.selected,"last_issued_operation":self.issued,"mode":self.mode()})
    }
}
