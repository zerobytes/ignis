//! Functional native game runner. JSON-lines control, no animations or render dependencies in the game tick.
#[path = "../games/land-of-survivors/src/combat.rs"]
mod combat;
#[path = "../games/land-of-survivors/src/lab.rs"]
mod lab;
use combat::*;
use ignis::model::{Result, invalid};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{BufRead, Write};
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    New {
        class_id: String,
        specialization: String,
        focus: Focus,
    },
    Inspect,
    Catalog,
    Cast {
        actor: String,
        ability: String,
        #[serde(default)]
        aim: Aim,
    },
    Slot {
        actor: String,
        slot: usize,
        #[serde(default)]
        aim: Aim,
    },
    Cancel {
        actor: String,
    },
    Step {
        ticks: u32,
    },
    Move {
        actor: String,
        input: [f32; 2],
    },
    Build {
        actor: String,
        specialization: String,
        focus: Focus,
    },
    Equip {
        actor: String,
        slots: Vec<String>,
    },
    Mount {
        actor: String,
        mounted: bool,
    },
    Ai {
        actor: String,
        enabled: bool,
    },
    Save,
    Load {
        save: Vec<u8>,
    },
    Verify,
}
fn inspect(g: &Game) -> Value {
    let shields = g
        .state
        .fighters
        .iter()
        .map(|(id, f)| (id, f.shield()))
        .collect::<std::collections::BTreeMap<_, _>>();
    json!({"state":g.state,"positions":g.engine.simulation.positions(),"shields":shields,"bar":g.bar("player"),"available":g.available("player"),"native_tick":g.engine.simulation.tick})
}
#[derive(Serialize)]
struct Verified {
    id: String,
    events: usize,
    projectile: bool,
    periodic: bool,
    form: Form,
}
fn verify() -> Result<Value> {
    let catalog = Catalog::bundled()?;
    let mut rows = vec![];
    for a in &catalog.abilities {
        let mut g = lab::game(&a.class_id, &a.specialization, a.focuses[0])?;
        g.state.fighters.get_mut("player").unwrap().form = a.form;
        for f in g.state.fighters.values_mut() {
            f.health = 500.;
            f.mana = 300.;
        }
        lab::place_for_ability(&mut g, a)?;
        let aim = lab::aim(&g, a);
        let old = g.position("player");
        g.cast("player", &a.id, aim)?;
        g.step((ticks(a.cast + a.duration) + 240).min(3600) as u32)?;
        let events = g.state.events.iter().filter(|e| e.ability == a.id).count();
        if !g
            .state
            .events
            .iter()
            .any(|e| e.ability == a.id && e.kind == "committed")
        {
            return Err(invalid(format!("{} never committed", a.id)));
        }
        let observed = g
            .state
            .events
            .iter()
            .any(|e| e.ability == a.id && e.target.is_some() && e.amount > 0.)
            || a.effects.iter().any(|e| e.kind == EffectKind::Form)
                && g.state.fighters["player"].form == a.effects[0].form
            || a.delivery == Delivery::Dash && g.position("player").distance(old) > 0.01;
        if !observed {
            return Err(invalid(format!("{} produced no functional result", a.id)));
        }
        rows.push(Verified {
            id: a.id.clone(),
            events,
            projectile: a.delivery == Delivery::Projectile,
            periodic: a.periodic,
            form: g.state.fighters["player"].form,
        });
    }
    Ok(
        json!({"verified":rows.len(),"abilities":rows,"native_physics":"Ignis/Rapier","animations":false}),
    )
}
fn execute(g: &mut Game, input: Input) -> Result<Value> {
    match input {
        Input::New {
            class_id,
            specialization,
            focus,
        } => {
            *g = lab::game(&class_id, &specialization, focus)?;
        }
        Input::Inspect => {}
        Input::Catalog => return Ok(serde_json::to_value(&g.catalog).unwrap()),
        Input::Cast {
            actor,
            ability,
            aim,
        } => g.cast(&actor, &ability, aim)?,
        Input::Slot { actor, slot, aim } => g.cast_slot(&actor, slot, aim)?,
        Input::Cancel { actor } => g.cancel(&actor)?,
        Input::Step { ticks } => g.step(ticks)?,
        Input::Move { actor, input } => g.movement(&actor, input)?,
        Input::Build {
            actor,
            specialization,
            focus,
        } => g.build(&actor, &specialization, focus)?,
        Input::Equip { actor, slots } => g.equip(&actor, slots)?,
        Input::Mount { actor, mounted } => g.set_mounted(&actor, mounted)?,
        Input::Ai { actor, enabled } => {
            g.state
                .fighters
                .get_mut(&actor)
                .ok_or_else(|| invalid("Unknown fighter"))?
                .ai = enabled;
        }
        Input::Save => return Ok(json!({"save":g.save()?})),
        Input::Load { save } => {
            *g = Game::load(&save)?;
        }
        Input::Verify => return verify(),
    }
    Ok(inspect(g))
}
/// Both the terminal and MCP normalize into the same typed game dispatcher above.
fn mcp(g: &mut Game, request: &Value) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => Ok(
            json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"land-of-survivors","version":"1"}}),
        ),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(
            json!({"tools":[{"name":"los_game","description":"Execute the native Land of Survivors gameplay component:284 abilities, builds, ten-slot bars, physical movement, saves and inspection.","inputSchema":schemars::schema_for!(Input)}]}),
        ),
        "tools/call" => {
            if request["params"]["name"] != "los_game" {
                Err(invalid("Unknown game tool"))
            } else {
                serde_json::from_value::<Input>(request["params"]["arguments"].clone()).map_err(|e|invalid(e.to_string()))
                .and_then(|input|execute(g,input)).map(|value|json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false}))
            }
        }
        _ => Err(invalid("Unknown MCP method")),
    };
    Some(match result {
        Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
        Err(e) => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":e.message}}),
    })
}
fn main() -> anyhow::Result<()> {
    let mut game = lab::game("mage", "fire", Focus::Dps)?;
    let mut stdout = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        if line.len() > 48 * 1024 * 1024 {
            writeln!(stdout, "{{\"error\":\"Input exceeds48MiB\"}}")?;
            continue;
        }
        if let Ok(request) = serde_json::from_str::<Value>(&line)
            && request.get("method").is_some()
        {
            if let Some(response) = mcp(&mut game, &request) {
                writeln!(stdout, "{}", response)?;
                stdout.flush()?;
            }
            continue;
        }
        let result = serde_json::from_str::<Input>(&line)
            .map_err(|e| invalid(e.to_string()))
            .and_then(|input| execute(&mut game, input));
        let response = match result {
            Ok(value) => json!({"result":value}),
            Err(e) => json!({"error":e.message}),
        };
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }
    Ok(())
}
