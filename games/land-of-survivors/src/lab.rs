use super::combat::*;
use ignis::model::{Result, RoomRecipe, Scene};
use std::collections::BTreeMap;
/// Technical primitives authored for this game; no prior game world or asset is loaded.
pub fn game(class_id: &str, specialization: &str, focus: Focus) -> Result<Game> {
    let mut scene: Scene = RoomRecipe {
        width: 64.,
        depth: 64.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()?;
    scene.name = "Land of Survivors - functional ability lab".into();
    let mut player = scene
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    scene.entities.retain(|e| {
        !matches!(
            e.body,
            Some(ignis::model::BodyKind::Player | ignis::model::BodyKind::Npc)
        )
    });
    player.transform.size = [0.6, 1.6, 0.6];
    player.footsteps = None;
    player.avatar = None;
    player.abilities = None;
    player.character_controller = Some(Default::default());
    player.vitality = Some(ignis::vitality::Config {
        initial: 1_000_000,
        maximum: 1_000_000,
        scale: 1000,
    });
    let mut fighters = BTreeMap::new();
    for (id, team, pos) in [
        ("player", "allies", [0., 0.81, 0.]),
        ("ally", "allies", [1.5, 0.81, -3.]),
        ("ally2", "allies", [-1.5, 0.81, -3.]),
        ("enemy", "hostiles", [0., 0.81, -4.]),
        ("enemy2", "hostiles", [1.3, 0.81, -5.5]),
        ("enemy3", "hostiles", [-1.3, 0.81, -5.5]),
        ("enemy4", "hostiles", [0., 0.81, -7.]),
    ] {
        let mut e = player.clone();
        e.id = id.into();
        e.name = id.into();
        e.transform.position = pos;
        e.faction = Some(team.into());
        e.color = if team == "allies" {
            [0.2, 0.5, 0.9]
        } else {
            [0.8, 0.2, 0.1]
        };
        scene.entities.push(e);
        let mut f = Fighter::new(class_id, specialization, focus, team, pos);
        f.armor = 0.;
        f.health = 500.;
        f.mana = 300.;
        fighters.insert(id.into(), f);
    }
    // Explicit lab relations; species do not imply hostility in the actual game.
    Game::new(scene, Catalog::bundled()?, fighters)
}
pub fn aim(g: &Game, a: &Ability) -> Aim {
    let target = if a.effects.iter().any(|e| e.relation == Relation::Enemy) {
        "enemy"
    } else {
        "ally"
    };
    let p = g.position(target);
    Aim {
        target: Some(target.into()),
        point: Some([p.x, 0., p.z]),
        direction: Some([0., 0., -1.]),
    }
}
/// Put the technical recipient within melee range when exercising a short-range kit.
pub fn place_for_ability(g: &mut Game, a: &Ability) -> Result<()> {
    if matches!(a.delivery, Delivery::Target | Delivery::Cone) && a.range < 4. {
        let delta =
            glam::Vec3::new(0., 0., -(a.range * 0.7).max(1.)) - g.position("enemy").with_y(0.);
        g.engine
            .simulation
            .displace_character("enemy", delta.to_array())?;
    }
    Ok(())
}
