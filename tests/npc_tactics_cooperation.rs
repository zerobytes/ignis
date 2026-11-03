//! Critical integration: private cues, atomic group admission and life-scoped support/commands.
use ignis::{
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
};
use serde_json::{Value, json};
fn base() -> Scene {
    let mut s = RoomRecipe {
        width: 30.,
        depth: 24.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 0.8, 0.];
    p.footsteps = None;
    s
}
fn actor(id: &str, x: f32, z: f32) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[x,0.81,z],"size":[0.6,1.6,0.6]},"character_controller":{"snap_distance":0.3},"npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,"mind":{"intelligence":1},"cooperation":{"group":7},"mobility":{"walk":true},"local_connections":{"walk":true,"fly":false,"max_distance":8},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap()
}
fn command(e: &mut Engine, v: Value) -> Value {
    e.execute(serde_json::from_value::<Command>(v).unwrap().into())
        .unwrap()
}
fn hp(e: &Engine) -> i64 {
    let a = e
        .simulation
        .actors
        .order()
        .iter()
        .copied()
        .find(|&a| {
            e.simulation.definitions().entities[e.simulation.actors.entity(a)].id == "player"
        })
        .unwrap();
    e.simulation.vitality.state.actors[a as usize]
        .unwrap()
        .units
}
fn support(projectile: bool) -> Scene {
    let mut s = base();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.faction = Some("team".into());
    p.vitality = Some(ignis::vitality::Config {
        initial: 20,
        maximum: 100,
        scale: 1,
    });
    let mut definition = json!({"id":"heal","selection":"actor","range":8,"relation":"allied","cast_ticks":if projectile {0}else{30},"cooldown_ticks":90,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"heal","recipient":"target","amount":80}]});
    if projectile {
        definition["delivery"] =
            json!({"kind":"projectile","speed":10,"half_extent":0.05,"max_distance":20});
    }
    s.ability_catalog = serde_json::from_value(
        json!({"resources":[{"id":"energy","scale":1}],"abilities":[definition]}),
    )
    .unwrap();
    for (id, x, concern) in [("a", -2., 0.), ("b", 2., 1.)] {
        let mut n = actor(id, x, -2.);
        n.faction = Some("team".into());
        n.vitality = Some(ignis::vitality::Config {
            initial: 100,
            maximum: 100,
            scale: 1,
        });
        n.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["heal"]})).unwrap());
        let c = n.npc.as_mut().unwrap();
        c.cooperation.as_mut().unwrap().reserve_support = true;
        c.mind.as_mut().unwrap().personality.concern = concern;
        c.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0})).unwrap());
        c.objectives=Some(serde_json::from_value(json!([{"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":10,"policy":{"allies":{"ranking":"lowest_health_ratio","include_self":false},"vitality_below":{"subject":"target","basis_points":9000}}}},{"priority":0,"when":"always","action":{"kind":"wait"}}])).unwrap());
        s.entities.push(n);
    }
    s
}
#[test]
fn tactics_orders_and_support_preserve_private_evidence_identity_atomicity_and_replay() {
    let mut s = base();
    let mut n = actor("tactician", -8., 0.);
    let c = n.npc.as_mut().unwrap();
    c.target = Some("player".into());
    c.mind.as_mut().unwrap().personality.caution = 1.;
    c.mind.as_mut().unwrap().personality.fear = 1.;
    c.mind.as_mut().unwrap().personality.courage = 0.;
    c.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":30,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":120,"search_ticks":60,"search_turn_degrees_per_second":0})).unwrap());
    c.objectives=Some(serde_json::from_value(json!([{"priority":1,"when":"evidence","action":{"kind":"tactical","configuration":{"range":5,"cover":1,"flank":1,"max_candidates":64}}}])).unwrap());
    s.entities.push(n);
    s.entities.push(serde_json::from_value(json!({"id":"cover","name":"cover","body":"fixed","color":[0.2,0.8,0.5],"transform":{"position":[-2,1.5,2],"size":[0.5,3,2]}})).unwrap());
    s.spatial_network.nodes = vec![[-8., 0., 0.], [-4., 0., 0.], [-4., 0., 4.]];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let t = e.simulation.npcs["tactician"].tactics.unwrap();
    assert!(t.choice.unwrap().covered, "{t:?}");
    assert_eq!(t.evidence.position, [0., 0.8, 0.]);
    e.advance(100).unwrap();
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut hidden = e.scene.clone();
    hidden
        .entities
        .iter_mut()
        .find(|e| e.id == "cover")
        .unwrap()
        .transform
        .position = [-2., 1.5, 0.];
    let mut hidden = Engine::new(hidden).unwrap();
    hidden.advance(30).unwrap();
    assert!(hidden.simulation.npcs["tactician"].memory.is_none());
    assert!(hidden.simulation.npcs["tactician"].tactics.is_none());
    let mut old = e.scene.clone();
    old.version = 92;
    assert!(old.validate().is_err());
    let mut s = base();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-10., 0.8, 8.];
    let a = actor("a", -4., -2.);
    let mut b = actor("b", -4., 2.);
    b.npc.as_mut().unwrap().cooperation.as_mut().unwrap().role = 1;
    s.entities.extend([a.clone(), b]);
    let mut template = a;
    template.id = "template".into();
    s.npc_templates.push(template);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let order = json!({"kind":"move","position":[2,0.81,0],"arrival_distance":0.2,"for_ticks":900,"formation":"line","spacing":2,"facing_degrees":0});
    assert!(
        e.execute(
            serde_json::from_value::<Command>(
                json!({"op":"order_npc_group","group":7,"order":order.clone()})
            )
            .unwrap()
            .into()
        )
        .is_ok()
    );
    // Admission is pending; both frozen recipients are committed in the same command phase.
    e.advance(1).unwrap();
    let frozen = e.simulation.npcs["b"].order.unwrap();
    assert_eq!(frozen.order.members, 2);
    e.advance(360).unwrap();
    assert!(e.simulation.positions()["a"][0] > 0.5);
    assert!(e.simulation.positions()["b"][0] > 2.5);
    command(&mut e, json!({"op":"despawn_actor","actor":"a"}));
    command(
        &mut e,
        json!({"op":"spawn_actor","template":"template","position":[-4,0.81,-2]}),
    );
    command(
        &mut e,
        json!({"op":"order_npc","actor":"a","order":{"kind":"hold","for_ticks":600}}),
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.npcs["b"].order.unwrap().order, frozen.order);
    let born = e
        .simulation
        .npcs
        .iter()
        .find(|(id, _)| id.as_str() != "b")
        .unwrap();
    assert!(born.1.order.is_none());
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        ignis::population::Outcome::InstanceUnavailable
    );
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::OrderNpc {
                actor: "player".into(),
                order: ignis::npc_cooperation::Order::Hold { for_ticks: 60 }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    for _ in 0..63 {
        command(
            &mut e,
            json!({"op":"order_npc","actor":"b","order":{"kind":"hold","for_ticks":60}}),
        );
    }
    let before = command(&mut e, json!({"op":"inspect_population"}));
    assert!(
        e.execute(
            serde_json::from_value::<Command>(
                json!({"op":"order_npc_group","group":7,"order":order})
            )
            .unwrap()
            .into()
        )
        .is_err()
    );
    assert_eq!(before, command(&mut e, json!({"op":"inspect_population"})));
    e.advance(1).unwrap();
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut life_scene = e.scene.clone();
    life_scene.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
        {"id":"revive","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":40}]}
    ]})).unwrap();
    life_scene
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["kill","revive"]})).unwrap());
    life_scene
        .entities
        .iter_mut()
        .find(|e| e.id == "b")
        .unwrap()
        .vitality = Some(ignis::vitality::Config {
        initial: 100,
        maximum: 100,
        scale: 1,
    });
    let mut life_engine = Engine::new(life_scene).unwrap();
    command(
        &mut life_engine,
        json!({"op":"order_npc","actor":"b","order":{"kind":"hold","for_ticks":600}}),
    );
    life_engine.advance(1).unwrap();
    let instance = life_engine.simulation.character_controllers["b"].instance;
    command(
        &mut life_engine,
        json!({"op":"activate_ability","actor":"player","slot":1,"target":"b"}),
    );
    life_engine.advance(1).unwrap();
    assert!(life_engine.simulation.npcs["b"].order.is_none());
    command(
        &mut life_engine,
        json!({"op":"activate_ability","actor":"player","slot":2,"target":"b"}),
    );
    life_engine.advance(1).unwrap();
    assert_eq!(
        life_engine.simulation.character_controllers["b"].instance,
        instance
    );
    assert!(life_engine.simulation.npcs["b"].order.is_none());
    assert_eq!(
        Engine::verify_replay(&life_engine.replay()).unwrap()["matched"],
        true
    );
    for projectile in [false, true] {
        let mut e = Engine::new(support(projectile)).unwrap();
        e.advance(1).unwrap();
        let cooperation = command(&mut e, json!({"op":"inspect_npc_cooperation"}));
        assert_eq!(
            cooperation["support_claims"].as_array().unwrap().len(),
            1,
            "{cooperation}"
        );
        assert_eq!(
            cooperation["support_decisions"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|d| d["allowed"] == true)
                .count(),
            1
        );
        let b = e.simulation.character_controllers["b"].instance;
        assert_eq!(
            cooperation["support_claims"][0]["source"],
            serde_json::to_value(b).unwrap()
        );
        let mut state = serde_json::to_value(&e.simulation.abilities.state).unwrap();
        let claim = state["support_claims"][0].clone();
        state["support_claims"].as_array_mut().unwrap().push(claim);
        assert!(serde_json::from_value::<ignis::abilities::State>(state).is_err());
        e.advance(90).unwrap();
        assert_eq!(hp(&e), 100);
        assert!(
            command(&mut e, json!({"op":"inspect_npc_cooperation"}))["support_claims"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            e.simulation
                .abilities
                .state
                .receipts
                .iter()
                .filter(|r| r.outcome == ignis::abilities::Outcome::Started)
                .count(),
            1
        );
        assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    }
    let mut e = Engine::new(support(false)).unwrap();
    e.advance(1).unwrap();
    command(
        &mut e,
        json!({"op":"activate_ability","actor":"a","slot":1,"target":"player"}),
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.abilities.state.casts.iter().flatten().count(),
        2
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
