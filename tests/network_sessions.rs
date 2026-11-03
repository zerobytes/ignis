//! Critical authorization, at-most-once invocation and atomic replica boundaries.
use ignis::{
    abilities::Outcome,
    engine::Engine,
    network::{
        protocol::*,
        replica::{Applied, Frame, Replica},
        sessions::Authority,
    },
    physics::Input,
};
#[path = "support/network.rs"]
mod support;
use serde_json::json;
use support::scene;

fn step(e: &mut Engine, a: &mut Authority, now: u64) {
    a.before_tick(e, now).unwrap();
    e.advance(1).unwrap();
    a.after_tick(e).unwrap();
}
fn op(id: u64, life: u64, slot: u8, target: Target) -> Operation {
    Operation {
        id,
        life,
        intent: Intent::Activate {
            slot,
            target: Some(target),
        },
    }
}
#[test]
fn thousand_sessions_keep_ownership_dedup_and_results_beyond_global_ring() {
    let mut e = Engine::new(scene(1000)).unwrap();
    let mut a = Authority::new(&mut e, 1000).unwrap();
    let users: Vec<_> = (0..1000).map(|_| a.join(&mut e, 0).unwrap()).collect();
    let ids: std::collections::BTreeSet<_> = users.iter().map(|w| w.actor.actor).collect();
    assert_eq!(ids.len(), 1000);
    assert!(a.join(&mut e, 0).is_err());
    for (i, w) in users.iter().enumerate() {
        a.movement(
            &e,
            w.session,
            w.connection,
            Movement {
                sequence: 1,
                life: w.actor.life,
                input: Input {
                    x: 0.25,
                    ..Default::default()
                },
            },
            0,
        )
        .unwrap();
        a.operation(
            &e,
            w.session,
            w.connection,
            op(1, w.actor.life, 1, users[i ^ 1].actor),
        )
        .unwrap();
    }
    for id in 2..=16 {
        a.operation(&e, users[0].session, 1, op(id, 0, 1, users[1].actor))
            .unwrap();
    }
    assert_eq!(
        a.operation(&e, users[0].session, 1, op(17, 0, 1, users[1].actor))
            .unwrap_err(),
        Rejection::Capacity
    );
    // Same operation ID with a different payload never becomes a second invocation.
    a.operation(
        &e,
        users[0].session,
        users[0].connection,
        op(1, 0, 3, users[1].actor),
    )
    .unwrap();
    step(&mut e, &mut a, 0);
    assert_eq!(e.simulation.abilities.state.receipts.len(), 256);
    assert_eq!(e.simulation.abilities.request_results().len(), 2015);
    for w in &users {
        let results = a.take_results(w.session, w.connection).unwrap();
        assert!(results.iter().any(|r| r.id == 1
            && matches!(
                r.status,
                Status::Resolved {
                    outcome: Outcome::Completed,
                    ..
                }
            )));
        assert_eq!(
            a.owner_view(&e, w.session, w.connection).unwrap().resources[0].1,
            99
        );
    }
    assert!(
        e.simulation
            .vitality
            .state
            .actors
            .iter()
            .all(|h| h.unwrap().units == 999)
    );
    let before = e.simulation.abilities.state.balances.clone();
    let retry = a
        .operation(&e, users[0].session, 1, op(1, 0, 1, users[1].actor))
        .unwrap();
    assert!(retry.status.terminal());
    step(&mut e, &mut a, 20);
    assert_eq!(e.simulation.abilities.state.balances, before);
    assert!(serde_json::from_value::<Operation>(json!({"id":2,"life":0,"actor":users[1].actor.actor,"intent":{"kind":"activate","slot":1,"target":users[0].actor}})).is_err());
    for id in 17..=82 {
        a.operation(&e, users[0].session, 1, op(id, 0, 1, users[1].actor))
            .unwrap();
        step(&mut e, &mut a, 20 + id * 20);
        a.take_results(users[0].session, 1).unwrap();
    }
    let before = e.simulation.abilities.state.balances[0];
    assert_eq!(
        a.operation(&e, users[0].session, 1, op(1, 0, 1, users[1].actor))
            .unwrap_err(),
        Rejection::ExpiredResult
    );
    assert_eq!(e.simulation.abilities.state.balances[0], before);
    a.close(&mut e);
    assert!(!e.simulation.abilities.request_results_enabled());
    assert!(
        e.inspect_execution()["retained"]
            .as_object()
            .unwrap()
            .values()
            .all(|v| v == 0)
    );
}
#[test]
fn resume_timeout_cancel_and_life_fences_never_reassign_old_input_or_charge_retry() {
    let mut e = Engine::new(scene(2)).unwrap();
    let mut a = Authority::new(&mut e, 2).unwrap();
    let u = a.join(&mut e, 0).unwrap();
    let v = a.join(&mut e, 0).unwrap();
    assert_eq!(
        a.resume(&mut e, u.session, &v.secret, 0).unwrap_err(),
        Rejection::Unavailable
    );
    a.movement(
        &e,
        u.session,
        1,
        Movement {
            sequence: 1,
            life: 0,
            input: Input {
                x: 1.,
                ..Default::default()
            },
        },
        0,
    )
    .unwrap();
    step(&mut e, &mut a, 0);
    let moving = e.simulation.position("p0000").unwrap()[0];
    step(&mut e, &mut a, 300);
    let stopped = e.simulation.position("p0000").unwrap()[0];
    for n in 1..5 {
        step(&mut e, &mut a, 300 + n);
    }
    assert!((e.simulation.position("p0000").unwrap()[0] - stopped).abs() < 0.001);
    assert!(moving > -23.25);
    let resumed = a.resume(&mut e, u.session, &u.secret, 400).unwrap();
    assert_eq!(resumed.connection, 2);
    assert_eq!(
        a.movement(
            &e,
            u.session,
            1,
            Movement {
                sequence: 2,
                life: 0,
                input: Input {
                    x: 1.,
                    ..Default::default()
                }
            },
            400
        )
        .unwrap_err(),
        Rejection::OldConnection
    );
    a.operation(&e, u.session, 2, op(1, 0, 2, v.actor)).unwrap();
    step(&mut e, &mut a, 400);
    let result = a.take_results(u.session, 2).unwrap();
    let execution = result
        .iter()
        .find_map(|r| {
            if let Status::Started { execution } = r.status {
                execution
            } else {
                None
            }
        })
        .unwrap();
    a.operation(
        &e,
        v.session,
        1,
        Operation {
            id: 1,
            life: 0,
            intent: Intent::Cancel { execution },
        },
    )
    .unwrap();
    step(&mut e, &mut a, 410);
    assert!(
        a.take_results(v.session, 1)
            .unwrap()
            .iter()
            .any(|r| matches!(
                r.status,
                Status::Resolved {
                    outcome: Outcome::NoActiveExecution,
                    ..
                }
            ))
    );
    a.operation(
        &e,
        u.session,
        2,
        Operation {
            id: 2,
            life: 0,
            intent: Intent::Cancel { execution },
        },
    )
    .unwrap();
    step(&mut e, &mut a, 420);
    let results = a.take_results(u.session, 2).unwrap();
    for id in [1, 2] {
        assert!(results.iter().any(|r| r.id == id
            && matches!(
                r.status,
                Status::Resolved {
                    outcome: Outcome::Cancelled,
                    ..
                }
            )));
    }
    assert_eq!(a.owner_view(&e, u.session, 2).unwrap().resources[0].1, 100);
    a.operation(&e, v.session, 1, op(2, 0, 3, u.actor)).unwrap();
    step(&mut e, &mut a, 440);
    a.operation(&e, v.session, 1, op(3, 0, 4, u.actor)).unwrap();
    step(&mut e, &mut a, 460);
    let life = a.owner_view(&e, u.session, 2).unwrap().actor.life;
    assert_eq!(life, 1);
    assert_eq!(
        a.movement(
            &e,
            u.session,
            2,
            Movement {
                sequence: 1,
                life: 0,
                input: Input {
                    x: 1.,
                    ..Default::default()
                }
            },
            470
        )
        .unwrap_err(),
        Rejection::StaleLife
    );
    a.operation(&e, v.session, 1, op(4, 0, 1, u.actor)).unwrap();
    step(&mut e, &mut a, 480);
    assert!(
        a.take_results(v.session, 1)
            .unwrap()
            .iter()
            .any(|r| r.id == 4
                && r.status
                    == Status::Rejected {
                        reason: Rejection::Target
                    })
    );
    a.disconnect(&mut e, u.session, 2, 500);
    let again = a.resume(&mut e, u.session, &u.secret, 501).unwrap();
    assert_eq!(again.connection, 3);
    let duplicate = a
        .operation(&e, u.session, 3, op(1, life, 1, v.actor))
        .unwrap();
    assert!(duplicate.status.terminal());
    a.disconnect(&mut e, u.session, 3, 502);
    let fresh = a.join(&mut e, 502 + RESUME_MS + 1).unwrap();
    assert_ne!(fresh.session, u.session);
    assert_eq!(
        a.resume(&mut e, u.session, &u.secret, 20_000).unwrap_err(),
        Rejection::UnknownSession
    );
    e.reset();
    assert_eq!(
        a.before_tick(&mut e, 20_000).unwrap_err(),
        Rejection::WorldChanged
    );
    a.close(&mut e);
}
#[test]
fn replica_thousand_full_delta_loss_lifecycle_and_malformed_payloads_are_atomic() {
    let e = Engine::new(scene(1000)).unwrap();
    let f0 = Frame::capture(&e, 42, None).unwrap();
    let full = f0.encode(None).unwrap();
    assert!(!full.windows(6).any(|b| b == b"energy"));
    let mut replica = Replica::new(42);
    assert_eq!(replica.apply(&full).unwrap(), Applied::Current);
    assert_eq!(replica.actor_count(), 1000);
    let mut f1 = f0.clone();
    f1.tick = 1;
    let last_actor = f1
        .actors
        .iter()
        .rposition(|a| a.appearance.kind.actor())
        .unwrap();
    f1.actors[last_actor].millimeters[0] += 1234;
    f1.actors[last_actor].health.as_mut().unwrap().units = 0;
    let delta = f1.encode(Some(&f0)).unwrap();
    assert!(delta.len() < 100);
    for end in [0, 4, 12, delta.len() - 1] {
        let old = replica.clone();
        assert!(replica.apply(&delta[..end]).is_err());
        assert_eq!(replica, old);
    }
    let mut bad = delta.clone();
    bad.push(0);
    let old = replica.clone();
    assert!(replica.apply(&bad).is_err());
    assert_eq!(replica, old);
    let mut wrong = Replica::new(43);
    assert!(wrong.apply(&full).is_err());
    assert!(wrong.actors().is_empty());
    let mut f2 = f1.clone();
    f2.tick = 2;
    f2.actors[last_actor].health.as_mut().unwrap().units = 500;
    let illegal = f2.encode(Some(&f1)).unwrap();
    let mut lost = replica.clone();
    f2.actors[last_actor].life = 1;
    let next = f2.encode(Some(&f1)).unwrap();
    assert_eq!(lost.apply(&next).unwrap(), Applied::NeedBaseline);
    assert_eq!(lost, replica);
    assert_eq!(replica.apply(&delta).unwrap(), Applied::Current);
    let old = replica.clone();
    assert!(replica.apply(&illegal).is_err());
    assert_eq!(replica, old);
    assert_eq!(replica.apply(&next).unwrap(), Applied::Current);
    assert_eq!(
        lost.apply(&f2.encode(None).unwrap()).unwrap(),
        Applied::Current
    );
    assert_eq!(lost, replica);
    assert_eq!(replica.apply(&delta).unwrap(), Applied::Stale);
    let mut f3 = f2.clone();
    f3.tick = 3;
    let mut born = f3.actors.remove(last_actor);
    let retired = born.id;
    born.id = u64::MAX;
    f3.actors.push(born);
    assert_eq!(
        replica.apply(&f3.encode(Some(&f2)).unwrap()).unwrap(),
        Applied::Current
    );
    assert!(!replica.actors().contains_key(&retired));
    assert!(replica.actors().contains_key(&u64::MAX));
    assert_eq!(replica.actor_count(), 1000);
}
