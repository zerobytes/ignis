use ignis::model::{BodyKind, RoomRecipe, Scene};
use serde_json::json;
pub fn scene(players: usize) -> Scene {
    let mut s = RoomRecipe {
        width: 63.,
        depth: 63.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut prototype = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    prototype.footsteps = None;
    prototype.vitality =
        Some(serde_json::from_value(json!({"initial":1000,"maximum":1000,"scale":1})).unwrap());
    prototype.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["hit","long","kill","revive"]})).unwrap());
    for a in 0..players {
        let mut p = prototype.clone();
        p.id = format!("p{a:04}");
        p.name = p.id.clone();
        p.transform.position = [
            -23.25 + (a % 32) as f32 * 1.5,
            0.8,
            -23.25 + (a / 32) as f32 * 1.5,
        ];
        s.entities.push(p);
    }
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
  {"id":"hit","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":1}],"effects":[{"kind":"damage","recipient":"target","amount":1}]},
  {"id":"long","selection":"actor","range":100,"cast_ticks":3,"cooldown_ticks":0,"costs":[{"resource":"energy","units":3}],"effects":[{"kind":"damage","recipient":"target","amount":7}]},
  {"id":"kill","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":1000}]},
  {"id":"revive","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":500}]}
 ]})).unwrap();
    s
}
