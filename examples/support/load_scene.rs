//! Shared concrete scene recipe for local and real-QUIC load probes.
use anyhow::{Result, bail};
use ignis::model::{BodyKind, RoomRecipe, Scene};
use serde_json::json;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Workload {
    Direct,
    Projectile,
    Periodic,
    Area,
    Buff,
    Noise,
    Mixed,
    Boss,
}
impl Workload {
    pub fn parse(value: &str) -> Result<Self> {
        Ok(match value {
            "direct" => Self::Direct,
            "projectile" => Self::Projectile,
            "periodic" => Self::Periodic,
            "area" => Self::Area,
            "buff" => Self::Buff,
            "noise" => Self::Noise,
            "mixed" => Self::Mixed,
            "boss" => Self::Boss,
            _ => bail!("Unknown delivery {value}"),
        })
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Mixed => {
                "four delivery groups in adjacent pairs: direct buff/noise, projectile buff, periodic, area buff"
            }
            Self::Boss => "all players attack one boss: damage, refreshed neutral buff and sound",
            Self::Noise => "damage plus one sound stimulus in adjacent pairs",
            Self::Buff => "damage plus refreshed neutral buff in adjacent pairs",
            Self::Direct => "direct damage in adjacent pairs",
            Self::Projectile => "projectile damage in adjacent pairs",
            Self::Periodic => "periodic damage in adjacent pairs, one pulse after6ticks",
            Self::Area => "source-anchored area, radius0.5, one pulse after6ticks",
        }
    }
}
pub fn id(actor: usize) -> String {
    format!("p{actor:04}")
}
fn definition(kind: Workload, name: &str) -> Result<ignis::abilities::Definition> {
    let mut d: ignis::abilities::Definition = serde_json::from_value(json!({
        "id":name,"selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,
        "costs":[],"effects":[{"kind":"damage","recipient":"target","amount":1}]
    }))?;
    match kind {
        Workload::Direct | Workload::Buff | Workload::Noise | Workload::Boss => {}
        Workload::Projectile => {
            d.delivery = ignis::projectiles::Delivery::Projectile {
                speed: 100.,
                half_extent: 0.01,
                max_distance: 100.,
            }
        }
        Workload::Periodic => {
            d.periodic = Some(ignis::periodic::Schedule {
                interval_ticks: 6,
                pulses: 1,
                reapply: ignis::periodic::Reapply::Independent,
            })
        }
        Workload::Area => {
            d.selection = ignis::abilities::Selection::SelfActor;
            d.range = 0.;
            d.area = Some(serde_json::from_value(json!({
                "radius":0.5,"max_targets":1,"include_source":true,"occlusion":false,
                "persistence":{"anchor":"source","duration_ticks":7,"interval_ticks":6,
                    "apply_on_spawn":false,"reapply":"independent","end_on_source_death":false}
            }))?);
        }
        Workload::Mixed => bail!("Mixed is a group of definitions"),
    }
    if matches!(kind, Workload::Buff | Workload::Boss) {
        d.effects.push(buff_effect()?);
    }
    if matches!(kind, Workload::Noise | Workload::Boss) {
        d.effects.push(ignis::abilities::Effect::Noise {
            radius: if kind == Workload::Boss { 100. } else { 10. },
        });
    }
    Ok(d)
}
fn buff_effect() -> Result<ignis::abilities::Effect> {
    Ok(serde_json::from_value(
        json!({"kind":"apply_buff","recipient":"target","buff":"marker"}),
    )?)
}
pub fn delivery_for(kind: Workload, actor: usize) -> Workload {
    if kind == Workload::Mixed {
        [
            Workload::Direct,
            Workload::Projectile,
            Workload::Periodic,
            Workload::Area,
        ][actor / 2 % 4]
    } else {
        kind
    }
}
pub fn scene(
    players: usize,
    kind: Workload,
    ground: bool,
    steps: bool,
    listeners: usize,
) -> Result<Scene> {
    let mut s = RoomRecipe {
        width: 63.,
        depth: 63.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()?;
    let mut prototype = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    if kind == Workload::Boss {
        s.entities.retain(|e| e.id == "floor");
        s.entities[0].transform.size[0] = 64.;
        s.entities[0].transform.size[2] = 64.;
        let floor = s.entities[0].clone();
        for x in -1..=1 {
            for z in -1..=1 {
                if x == 0 && z == 0 {
                    continue;
                }
                let mut tile = floor.clone();
                tile.id = format!("arena_{x}_{z}");
                tile.name = tile.id.clone();
                tile.transform.position[0] = x as f32 * 64.;
                tile.transform.position[2] = z as f32 * 64.;
                s.entities.push(tile);
            }
        }
        prototype.transform.size = [0.35, 1.6, 0.35];
    }
    prototype.character_controller = ground.then(ignis::character_controller::Config::default);
    prototype.footsteps = if steps {
        Some(ignis::model::FootstepConfig {
            surface: "floor".into(),
            stride: 0.6,
            radius: 6.,
            min_speed: 0.1,
        })
    } else {
        None
    };
    prototype.vitality = Some(serde_json::from_value(
        json!({"initial":1000000,"maximum":1000000,"scale":1}),
    )?);
    for actor in 0..players {
        let mut player = prototype.clone();
        player.id = id(actor);
        player.name = player.id.clone();
        player.transform.position = if kind == Workload::Boss {
            let angle = actor as f32 * std::f32::consts::TAU / players as f32;
            [90. * angle.cos(), 0.8, 90. * angle.sin()]
        } else {
            [
                -23.25 + (actor % 32) as f32 * 1.5,
                0.8,
                -23.25 + (actor / 32) as f32 * 1.5,
            ]
        };
        s.entities.push(player);
    }
    s.ability_catalog = Default::default();
    if matches!(kind, Workload::Buff | Workload::Boss | Workload::Mixed) {
        s.ability_catalog.attributes.buffs.push(serde_json::from_value(json!({"id":"marker","duration_ticks":60,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","operation":{"kind":"multiply","basis_points":10000}}]}))?);
    }
    if kind == Workload::Mixed {
        for (kind, name) in [
            (Workload::Direct, "direct"),
            (Workload::Projectile, "projectile"),
            (Workload::Periodic, "periodic"),
            (Workload::Area, "area"),
        ] {
            let mut d = definition(kind, name)?;
            if kind != Workload::Periodic {
                d.effects.push(buff_effect()?);
            }
            if kind == Workload::Direct {
                d.effects
                    .push(ignis::abilities::Effect::Noise { radius: 100. });
            }
            s.ability_catalog.abilities.push(d);
        }
    } else {
        s.ability_catalog.abilities.push(definition(kind, "hit")?);
    }
    let grants: Vec<_> = s
        .ability_catalog
        .abilities
        .iter()
        .map(|d| d.id.clone())
        .collect();
    for player in s
        .entities
        .iter_mut()
        .filter(|e| e.body == Some(BodyKind::Player))
    {
        player.abilities = Some(ignis::abilities::Loadout {
            resources: vec![],
            grants: grants.clone(),
        });
    }
    for a in 0..listeners {
        let (surface, position) = if kind == Workload::Boss {
            ("arena_1_0", [95., 0.8, -21. + a as f32 * 6.])
        } else {
            ("floor", [-21. + a as f32 * 6., 0.8, 28.])
        };
        s.entities.push(serde_json::from_value(json!({"id":format!("witness{a}"),"name":"Listener","body":"npc","color":[0.2,0.7,0.4],"transform":{"position":position,"size":[0.6,1.6,0.6]},"npc":{"surface":surface,"speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":1,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":30,"search_turn_degrees_per_second":0,"hearing":{"range":100}},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}}))?);
    }
    if kind == Workload::Boss {
        s.entities.push(serde_json::from_value(json!({"id":"zz_boss","name":"Boss","body":"npc","color":[0.8,0.2,0.1],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"omniscient"},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]},"vitality":{"initial":1000000,"maximum":1000000,"scale":1}}))?);
    }
    s.camera.position = if kind == Workload::Boss {
        [0., 150., 150.]
    } else {
        [0., 60., 60.]
    };
    s.camera.target = [0., 0.8, 0.];
    s.camera.fov_degrees = 60.;
    Ok(s)
}
