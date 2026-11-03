//! Critical ownership: a former input owner/life/connection cannot release or renew the next one.
use super::*;
use crate::{
    engine::{Command, Engine},
    network::{content::Content, controls::Controls, replica::Frame, sessions::Authority},
    physics::Input,
};
mod support {
    use crate as ignis;
    include!("../../tests/support/network.rs");
}

fn refresh(state: &mut State, e: &Engine, a: &Authority, w: &Welcome) {
    let owner = a.owner_view(e, w.session, w.connection).unwrap();
    state.actor = owner.actor;
    state.owner = Some(owner);
    state
        .replica
        .apply(
            &Frame::capture(e, w.world, None)
                .unwrap()
                .encode(None)
                .unwrap(),
        )
        .unwrap();
}
fn step(e: &mut Engine, a: &mut Authority) {
    a.before_tick(e, e.simulation.tick * 17).unwrap();
    e.advance(1).unwrap();
    a.after_tick(e).unwrap();
}
#[test]
fn lease_ownership_life_connection_and_admission_use_only_the_current_character() {
    let mut e = Engine::new(support::scene(2)).unwrap();
    let mut a = Authority::new(&mut e, 2).unwrap();
    let w = a.join(&mut e, 0).unwrap();
    let mut state = State::connected(&w);
    state.content = Some(Arc::new(Content::capture(&e, w.world)));
    refresh(&mut state, &e, &a, &w);
    let mut controls = Controls::default();
    let (old, hold, rev) = controls
        .prepare_hold(
            &state,
            Input {
                x: 1.,
                ..Default::default()
            },
        )
        .unwrap();
    controls.adopt_hold(old.clone(), hold, rev);
    let (new, next, rev) = controls
        .prepare_hold(
            &state,
            Input {
                z: 1.,
                ..Default::default()
            },
        )
        .unwrap();
    controls.adopt_hold(new.clone(), next, rev);
    assert_ne!(old, new);
    assert!(controls.release(&state, &old).is_none());
    assert_eq!(controls.mode(), crate::model::MovementMode::Walk);
    assert_eq!(
        controls.release(&state, &new).unwrap().input,
        Input::default()
    );
    let (t, held, r) = controls
        .prepare_hold(
            &state,
            Input {
                x: 1.,
                ..Default::default()
            },
        )
        .unwrap();
    controls.adopt_hold(t.clone(), held, r);
    let mut pending = Controls::default();
    let unadmitted = pending
        .use_ability(&state, 1, Some(state.actor), None)
        .unwrap();
    pending.issued(unadmitted.id);
    e.execute(
        Command::ActivateAbility {
            actor: "p0001".into(),
            slot: 3,
            target: Some("p0000".into()),
        }
        .into(),
    )
    .unwrap();
    step(&mut e, &mut a);
    refresh(&mut state, &e, &a, &w);
    controls.sync(&state);
    assert!(!held.context.valid(&state));
    assert!(controls.prepare_hold(&state, Input::default()).is_err());
    assert!(controls.release(&state, &t).is_none());
    e.execute(
        Command::ActivateAbility {
            actor: "p0001".into(),
            slot: 4,
            target: Some("p0000".into()),
        }
        .into(),
    )
    .unwrap();
    step(&mut e, &mut a);
    refresh(&mut state, &e, &a, &w);
    assert_eq!(state.actor.life, 1);
    // A new life must not reuse an ID whose admission is still unknown.
    assert!(
        pending
            .use_ability(&state, 1, Some(state.actor), None)
            .is_err()
    );
    assert!(!held.context.valid(&state));
    let (fresh, h, r) = controls
        .prepare_hold(
            &state,
            Input {
                x: -1.,
                ..Default::default()
            },
        )
        .unwrap();
    assert_ne!(fresh, t);
    controls.adopt_hold(fresh.clone(), h, r);
    assert!(controls.release(&state, &t).is_none());
    let target = state
        .replica
        .actors()
        .values()
        .find(|p| p.appearance.label == "p0001")
        .map(|p| Target {
            actor: p.id,
            life: p.life,
        })
        .unwrap();
    let op = controls.use_ability(&state, 1, Some(target), None).unwrap();
    assert_eq!(op.id, 1);
    controls.issued(op.id);
    assert!(controls.use_ability(&state, 1, Some(target), None).is_err());
    a.operation(&e, w.session, w.connection, op).unwrap();
    step(&mut e, &mut a);
    for result in a.take_results(w.session, w.connection).unwrap() {
        state.result(result);
    }
    assert_eq!(state.last_operation, 1);
    let next = controls.use_ability(&state, 1, Some(target), None).unwrap();
    assert_eq!(next.id, 2);
    assert_eq!(
        a.owner_view(&e, w.session, w.connection).unwrap().resources[0].1,
        99
    );
    let resumed = a.resume(&mut e, w.session, &w.secret, 100).unwrap();
    let mut replacement = State::connected(&resumed);
    replacement.content = state.content.clone();
    refresh(&mut replacement, &e, &a, &resumed);
    assert_eq!(
        pending
            .use_ability(&replacement, 1, Some(replacement.actor), None)
            .unwrap()
            .id,
        resumed.last_operation + 1
    );
    assert!(!h.context.valid(&replacement));
    controls.sync(&replacement);
    assert!(controls.release(&replacement, &fresh).is_none());
    let (last, _, _) = controls
        .prepare_hold(&replacement, Input::default())
        .unwrap();
    assert_ne!(last, fresh);
    let mut other = Controls::default();
    let (other_token, h, r) = other.prepare_hold(&replacement, Input::default()).unwrap();
    other.adopt_hold(other_token.clone(), h, r);
    assert_ne!(last, other_token);
    assert!(other.release(&replacement, &last).is_none());
    a.close(&mut e);
}
