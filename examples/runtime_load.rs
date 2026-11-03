//! Reproducible local authority workload; no renderer, transport or synthetic NPC shortcuts.
use anyhow::{Context, Result, bail, ensure};
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::BodyKind,
    physics::Input,
};
use serde_json::{Value, json};
use std::time::Instant;

#[path = "support/load_scene.rs"]
mod load_scene;
use load_scene::{Workload, delivery_for, id, scene};

fn stats(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |q: f64| sorted[(sorted.len() as f64 * q).ceil().max(1.) as usize - 1];
    json!({"min_ms":sorted[0],"mean_ms":samples.iter().sum::<f64>()/samples.len() as f64,
        "p50_ms":at(0.5),"p95_ms":at(0.95),"p99_ms":at(0.99),"max_ms":sorted.last()})
}
fn main() -> Result<()> {
    let mut players = 1000usize;
    let mut ticks = 600usize;
    let mut warmup = 60usize;
    let mut kind = Workload::Direct;
    let mut ground = false;
    let mut steps = false;
    let mut listeners = 0usize;
    let mut scene_path = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--delivery" => {
                kind = Workload::parse(&args.next().context("--delivery value")?)?;
            }
            "--controller" => ground = true,
            "--footsteps" => steps = true,
            "--listeners" => {
                listeners = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Missing listener count"))?
                    .parse()?
            }
            "--scene" => {
                scene_path = Some(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("Missing scene path"))?,
                )
            }
            "--players" => {
                players = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Missing players"))?
                    .parse()?
            }
            "--ticks" => {
                ticks = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Missing ticks"))?
                    .parse()?
            }
            "--warmup" => {
                warmup = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Missing warmup"))?
                    .parse()?
            }
            _ => bail!("Unknown argument:{arg}"),
        }
    }
    ensure!(
        (1..=1000).contains(&players) && (1..=36000).contains(&ticks) && warmup <= 36000,
        "Use players1..1000,ticks1..36000,warmup0..36000"
    );
    ensure!(
        !matches!(kind, Workload::Projectile | Workload::Mixed) || players.is_multiple_of(2),
        "Projectile pair workload requires an even player count"
    );
    ensure!(
        listeners + usize::from(kind == Workload::Boss) <= 8,
        "At most8NPCs including boss"
    );
    ensure!(
        kind != Workload::Boss || !steps || ground,
        "Boss footsteps across tiles require --controller"
    );
    ensure!(
        kind != Workload::Boss || (ticks + warmup).div_ceil(6) * players < 1_000_000,
        "Boss must survive the entire workload"
    );
    let start = Instant::now();
    let authored = scene(players, kind, ground, steps, listeners)?;
    if let Some(path) = scene_path {
        std::fs::write(path, serde_json::to_vec_pretty(&authored)?)?;
    }
    let mut e = Engine::new_with_mode(authored, ExecutionMode::Continuous)?;
    let mut controls = Vec::with_capacity(players);
    for actor in 0..players {
        controls.push(
            e.execute(Command::BindControl { actor: id(actor) }.into())?["control"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    let setup_ms = start.elapsed().as_secs_f64() * 1000.;
    let mut admission = Vec::with_capacity(ticks);
    let mut simulation = Vec::with_capacity(ticks);
    let mut total = Vec::with_capacity(ticks);
    let mut peaks = [0; 4];
    let mut peak_noise = 0;
    let mut peak_steps = 0;
    let mut heard = std::collections::BTreeSet::new();
    for frame in 0..ticks + warmup {
        let begin = Instant::now();
        for (actor, control) in controls.iter().enumerate() {
            e.execute(
                Command::SetControlInput {
                    control: control.clone(),
                    input: Input {
                        x: if frame / 120 % 2 == 0 { 0.25 } else { -0.25 },
                        ..Default::default()
                    },
                }
                .into(),
            )?;
            if frame % 6 == 0 {
                e.execute(
                    Command::ActivateAbility {
                        actor: id(actor),
                        slot: if kind == Workload::Mixed {
                            (actor / 2 % 4 + 1) as u8
                        } else {
                            1
                        },
                        target: if kind == Workload::Boss {
                            Some("zz_boss".into())
                        } else if delivery_for(kind, actor) == Workload::Area {
                            None
                        } else {
                            Some(id((actor ^ 1).min(players - 1)))
                        },
                    }
                    .into(),
                )?;
            }
        }
        let queued = Instant::now();
        e.advance(1)?;
        let finished = Instant::now();
        peaks[0] = peaks[0].max(e.simulation.abilities.state.projectiles.active.len());
        peaks[1] = peaks[1].max(e.simulation.abilities.state.periodic.active.len());
        peaks[2] = peaks[2].max(e.simulation.abilities.state.persistent_areas.active.len());
        peaks[3] = peaks[3].max(e.simulation.abilities.state.buffs.active.len());
        peak_noise = peak_noise.max(e.simulation.abilities.state.noises.len());
        peak_steps = peak_steps.max(e.simulation.footsteps.pending.len());
        for (name, npc) in &e.simulation.npcs {
            if npc.heard_this_tick && !heard.contains(name) {
                heard.insert(name.clone());
            }
        }
        if frame >= warmup {
            admission.push(queued.duration_since(begin).as_secs_f64() * 1000.);
            simulation.push(finished.duration_since(queued).as_secs_f64() * 1000.);
            total.push(finished.duration_since(begin).as_secs_f64() * 1000.);
        }
    }
    let mut drain = Vec::new();
    let pending = |e: &Engine| {
        !e.simulation.abilities.state.projectiles.active.is_empty()
            || !e.simulation.abilities.state.periodic.active.is_empty()
            || !e
                .simulation
                .abilities
                .state
                .persistent_areas
                .active
                .is_empty()
    };
    while pending(&e) && drain.len() < 120 {
        let begin = Instant::now();
        e.advance(1)?;
        drain.push(begin.elapsed().as_secs_f64() * 1000.);
    }
    ensure!(!pending(&e), "Unresolved delayed workload");
    let hits = (ticks + warmup).div_ceil(6);
    let count = |outcome| {
        e.simulation
            .abilities
            .outcome_counts()
            .iter()
            .find(|(o, _)| *o == outcome)
            .map_or(0, |(_, n)| *n)
    };
    let completed = count(ignis::abilities::Outcome::Completed);
    let occluded = count(ignis::abilities::Outcome::Occluded);
    ensure!(
        kind != Workload::Boss || peaks[3] == players,
        "Not every source contributed a boss buff"
    );
    let outcome_total: u128 = e
        .simulation
        .abilities
        .outcome_counts()
        .iter()
        .map(|(_, n)| u128::from(*n))
        .sum();
    ensure!(
        outcome_total
            == e.simulation.abilities.state.receipts.len() as u128
                + u128::from(e.simulation.abilities.state.dropped_receipts),
        "Outcome totals lost receipts"
    );
    if kind == Workload::Boss {
        ensure!(
            completed + occluded == (hits * players) as u64
                && count(ignis::abilities::Outcome::Started) == completed,
            "Unclassified boss actions"
        );
        ensure!(
            e.simulation.abilities.state.emitted == completed,
            "Sound and committed boss transactions disagree"
        );
    }

    let expected = 1_000_000 - hits as i64;
    let health_matches = e
        .simulation
        .actors
        .actors
        .iter()
        .enumerate()
        .all(|(a, &entity)| {
            let entity = &e.simulation.definitions().entities[entity as usize];
            let expected = if entity.id == "zz_boss" {
                1_000_000 - completed as i64
            } else if entity.body == Some(BodyKind::Player) {
                if kind == Workload::Boss {
                    1_000_000
                } else {
                    expected
                }
            } else {
                return true;
            };
            e.simulation.vitality.state.actors[a].is_some_and(|h| h.units == expected)
        });
    let grounded = e
        .simulation
        .character_controllers
        .values()
        .filter(|m| m.active && m.grounded)
        .count();
    let step_emitters = e
        .simulation
        .footsteps
        .states
        .values()
        .filter(|s| s.emitted > 0)
        .count();
    ensure!(
        !ground || grounded == players,
        "Not all controllers remain grounded"
    );
    ensure!(
        !steps || ticks + warmup < 120 || step_emitters == players,
        "Not all players emitted footsteps"
    );
    ensure!(
        listeners == 0
            || e.simulation.abilities.state.emitted + e.simulation.footsteps.total_emitted == 0
            || heard.len() == listeners,
        "Not all listeners heard stimuli"
    );
    let mut health_histogram = std::collections::BTreeMap::new();
    for h in e.simulation.vitality.state.actors.iter().flatten() {
        *health_histogram.entry(h.units).or_insert(0usize) += 1;
    }
    let mut receipt_outcomes = std::collections::BTreeMap::new();
    for r in &e.simulation.abilities.state.receipts {
        *receipt_outcomes
            .entry(format!("{:?}", r.outcome))
            .or_insert(0usize) += 1;
    }
    let positions = e.simulation.positions();
    let extent = positions.values().fold(
        ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]),
        |(mut min, mut max), p| {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
            (min, max)
        },
    );
    let origin_reserved_bytes: usize = e
        .simulation
        .abilities
        .state
        .origins
        .slots()
        .iter()
        .map(|slot| {
            slot.attributes.bases.capacity() * std::mem::size_of::<(u8, ignis::attributes::Value)>()
                + slot.attributes.contributions.capacity()
                    * std::mem::size_of::<ignis::attributes::Instance>()
        })
        .sum();
    let execution = e.inspect_execution();
    ensure!(
        execution["retained"]
            .as_object()
            .unwrap()
            .values()
            .all(|n| n == 0),
        "Command history accumulated"
    );
    ensure!(
        execution["admitted"]["abilities"] == players * hits,
        "Unexpected action count"
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "passed":health_matches,"health_histogram":health_histogram,"recent_receipt_outcomes":receipt_outcomes,"position_extent":extent,"build_id":env!("IGNIS_BUILD_ID"),"players":players,"measured_ticks":ticks,"warmup_ticks":warmup,
            "fixed_hz":60,"paced":false,"unmeasured_drain_ticks":drain.len(),"drain_simulation":if drain.is_empty() { Value::Null } else { stats(&drain) },"setup_ms":setup_ms,"dispatch":stats(&admission),"simulation":stats(&simulation),"total":stats(&total),
            "workload":{"movement":if ground {"ground controllers moving together with real collision"} else {"dynamic boxes moving together with floor collision"},"actions":kind.label(),"action_interval_ticks":6,"npcs":listeners+usize::from(kind == Workload::Boss),"listeners":listeners,"listeners_heard":heard.len(),"ground_controllers":e.simulation.character_controllers.len(),"grounded_controllers":grounded,"step_emitters":step_emitters,"footstep_events":e.simulation.footsteps.total_emitted,"peak_pending_footsteps":peak_steps,"peak_ability_noises":peak_noise,"peak_projectiles":peaks[0],"peak_periodics":peaks[1],"peak_persistent_areas":peaks[2],"peak_buffs":peaks[3],"ability_noise_events":e.simulation.abilities.state.emitted,"network_clients":0,"renderer":false},
            "storage":{"origin_slots":e.simulation.abilities.state.origins.slots().len(),"origin_snapshot_reserved_bytes":origin_reserved_bytes,"ability_noise_reserved_events":e.simulation.abilities.state.noises.capacity(),"footstep_reserved_events":e.simulation.footsteps.pending.capacity()},
            "buff_layout":{"instance_bytes":std::mem::size_of::<ignis::attributes::Instance>(),"index_bytes_per_entry":4*std::mem::size_of::<usize>()},
            "outcomes_since_reset":e.simulation.abilities.outcome_counts().iter().map(|(outcome,count)|json!({"outcome":outcome,"count":count})).collect::<Vec<_>>(),"boss_occluded_actions":if kind == Workload::Boss {occluded} else {0},"expected_actions_per_player":hits,"expected_hits_per_player":if kind == Workload::Boss {0} else {hits},"expected_health_per_player":if kind == Workload::Boss {1_000_000} else {expected},"expected_boss_health":if kind == Workload::Boss {Some(1_000_000-completed as i64)} else {None},"execution":execution
        }))?
    );
    ensure!(
        health_matches,
        "Combat outcomes differ from the workload contract"
    );
    Ok(())
}
