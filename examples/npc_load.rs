//! Local real NPC/Player workload. No renderer, network or frame-rate promise.
#[path = "support/npc_scene.rs"]
mod fixture;
use ignis::engine::{Command, Engine, ExecutionMode};
use serde_json::json;
use std::time::Instant;
fn main() -> anyhow::Result<()> {
    let mut ticks = 120u32;
    let mut args = std::env::args().skip(1);
    let mut save = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--ticks" => ticks = args.next().unwrap().parse()?,
            "--scene" => save = args.next(),
            _ => anyhow::bail!("Use --ticks1..600 --scene path"),
        }
    }
    anyhow::ensure!((1..=600).contains(&ticks));
    let s = fixture::scene();
    if let Some(p) = save {
        std::fs::write(p, serde_json::to_vec_pretty(&s)?)?;
    }
    let mut e = Engine::new_with_mode(s, ExecutionMode::Continuous)?;
    for p in 0..500 {
        let token = e.execute(
            Command::BindControl {
                actor: fixture::player(p),
            }
            .into(),
        )?["control"]
            .as_str()
            .unwrap()
            .to_owned();
        e.execute(
            Command::SetControlInput {
                control: token,
                input: ignis::physics::Input {
                    x: 1.,
                    z: 0.,
                    ..Default::default()
                },
            }
            .into(),
        )?;
    }
    let mut durations = Vec::with_capacity(ticks as usize);
    let mut total = Vec::with_capacity(ticks as usize);
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..ticks {
        let start = Instant::now();
        for p in 0..500 {
            e.execute(
                Command::ActivateAbility {
                    actor: fixture::player(p),
                    slot: 1,
                    target: Some(fixture::npc(p % 100)),
                }
                .into(),
            )?;
        }
        let sim = Instant::now();
        e.advance(1)?;
        durations.push(sim.elapsed().as_secs_f64() * 1000.);
        total.push(start.elapsed().as_secs_f64() * 1000.);
        for (id, s) in &e.simulation.npcs {
            if s.heard_this_tick {
                seen.insert(id.clone());
            }
        }
    }
    anyhow::ensure!(e.simulation.npcs.len() == 100);
    anyhow::ensure!(e.simulation.abilities.state.casts.len() == 628);
    anyhow::ensure!(seen.len() >= 40, "Hearing was not exercised");
    let grounded = e
        .simulation
        .character_controllers
        .values()
        .filter(|s| s.grounded)
        .count();
    anyhow::ensure!(
        grounded == 600,
        "All600controllers must keep ground contact"
    );
    let hp: i64 = e
        .simulation
        .vitality
        .state
        .actors
        .iter()
        .flatten()
        .map(|h| h.units)
        .sum();
    let energy: i64 = e
        .simulation
        .abilities
        .state
        .balances
        .iter()
        .map(|b| b[0])
        .sum();
    anyhow::ensure!(
        600000000 - hp == 600000000 - energy,
        "Every completed damage has exactly one energy cost"
    );
    let quantile = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[(v.len() as f64 * 0.95).ceil() as usize - 1]
    };
    let sim = quantile(&mut durations);
    let all = quantile(&mut total);
    println!(
        "{}",
        json!({"passed":true,"build_id":env!("IGNIS_BUILD_ID"),"ticks":ticks,"npcs":100,"players":500,"slots":628,"grounded":grounded,"npc_heard":seen.len(),"footsteps":e.simulation.footsteps.total_emitted,"damage_commits":600000000-hp,"simulation_p95_ms":sim,"total_p95_ms":all,"history":e.inspect_execution()["retained"],"renderer":false,"network":false})
    );
    Ok(())
}
