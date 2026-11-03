use ignis::model::{BodyKind, RoomRecipe, Scene};
use serde_json::json;
pub fn player(i: usize) -> String {
    format!("a_p{i:03}")
}
pub fn npc(i: usize) -> String {
    format!("z_n{i:03}")
}
pub fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 63.,
        depth: 63.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.npc_capacity = 128;
    let mut p = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    p.footsteps = Some(ignis::model::FootstepConfig {
        surface: "floor".into(),
        stride: 0.5,
        radius: 8.,
        min_speed: 0.2,
    });
    p.character_controller = Some(Default::default());
    p.blocks_vision = Some(false);
    p.faction = Some("players".into());
    p.vitality = Some(
        serde_json::from_value(json!({"initial":1000000,"maximum":1000000,"scale":1})).unwrap(),
    );
    p.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000000,"capacity":1000000}],"grants":["hit","kill"]})).unwrap());
    let mut pi = 0;
    let mut ni = 0;
    for cell in 0..600 {
        let position = [
            -22.5 + (cell % 20) as f32 * 1.5,
            0.81,
            -22.5 + (cell / 20) as f32 * 1.5,
        ];
        if cell % 6 != 0 {
            let mut e = p.clone();
            e.id = player(pi);
            e.name = e.id.clone();
            e.transform.position = position;
            s.entities.push(e);
            pi += 1;
        } else {
            let name = npc(ni);
            let e=serde_json::from_value(json!({"id":name,"name":name,"body":"npc","color":[0.6,0.2,0.8],"blocks_vision":false,"faction":"enemies","transform":{"position":position,"size":[0.6,1.6,0.6]},"character_controller":{},"vitality":{"initial":1000000,"maximum":1000000,"scale":1},"abilities":{"resources":[{"resource":"energy","initial":1000000,"capacity":1000000}],"grants":["hit"]},"npc":{"surface":"floor","speed":2,"stop_distance":0.5,"enabled":true,"mind":{"intelligence":(ni%3)as f32*0.5,"personality":{"courage":(ni%2)as f32,"fear":1.-(ni%2)as f32}},"awareness":{"mode":"perceptive","vision_range":if ni%2==0{40}else{1},"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":30,"search_turn_degrees_per_second":0,"hearing":{"range":25}},"targeting":{"kind":"hostile_nearest","switch_margin":0.25},"communication":{"mode":"radio","channel":1,"range":100,"cooldown_ticks":10,"max_age_ticks":60,"send":true,"receive":true},"objectives":[{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":1}},{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap();
            s.entities.push(e);
            ni += 1;
        }
    }
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[{"id":"hit","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":1}],"effects":[{"kind":"damage","recipient":"target","amount":1}]},{"id":"kill","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":1000000}]}]})).unwrap();
    s.faction_relations =
        serde_json::from_value(json!([{"from":"enemies","to":"players","attitude":"hostile"}]))
            .unwrap();
    let mut t = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Npc))
        .unwrap()
        .clone();
    t.id = "npc_template".into();
    t.npc.as_mut().unwrap().objectives = Some(
        serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"wait"}}]))
            .unwrap(),
    );
    s.npc_templates.push(t);
    s
}
