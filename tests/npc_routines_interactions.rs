//! Critical: atomic world effects, dialogue ownership/revision and uninterrupted live physics.
use ignis::{
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    network::replica::{Frame, Kind, Replica},
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
    p.transform.position = [-6.5, 0.8, 0.];
    p.footsteps = None;
    s
}
fn actor(id: &str, position: [f32; 3]) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":"npc","blocks_vision":false,"color":[0.6,0.2,0.8],"transform":{"position":position,"size":[0.6,1.6,0.6]},"vitality":{"initial":100,"maximum":100,"scale":1},"character_controller":{"snap_distance":0.3},"npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":true,"mind":{"intelligence":1},"circulation":{},"mobility":{"walk":true},"local_connections":{"walk":true,"fly":false,"max_distance":8},"objectives":[{"priority":0,"when":"always","action":{"kind":"wait"}}]}})).unwrap()
}
fn object(id: &str, p: [f32; 3], size: [f32; 3], fixed: bool) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":if fixed {Some("fixed")}else{None},"color":[0.2,0.8,0.5],"transform":{"position":p,"size":size}})).unwrap()
}
fn command(e: &mut Engine, v: Value) -> Value {
    e.execute(serde_json::from_value::<Command>(v).unwrap().into())
        .unwrap()
}
fn slot(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .order()
        .iter()
        .copied()
        .find(|&a| e.simulation.definitions().entities[e.simulation.actors.entity(a)].id == id)
        .unwrap() as usize
}
fn level(e: &Engine, id: &str, resource: &str) -> i64 {
    let r = e
        .scene
        .ability_catalog
        .resources
        .iter()
        .position(|r| r.id == resource)
        .unwrap();
    e.simulation
        .abilities
        .resource_level(slot(e, id), r)
        .unwrap()
        .0
}
fn routines() -> Scene {
    let mut s = base();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"charge","scale":1},{"id":"bold","scale":1},{"id":"safe","scale":1},{"id":"energy","scale":1}],"abilities":[
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
        {"id":"revive","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":40}]}
    ]})).unwrap();
    s.entities.iter_mut().find(|e|e.id=="player").unwrap().abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":10,"capacity":10}],"grants":["kill","revive"]})).unwrap());
    s.entities.extend([
        object("well", [4., 0.8, 0.], [0.2, 0.2, 0.2], false),
        object("shrine", [5., 0.8, 0.], [0.2, 0.2, 0.2], false),
    ]);
    let mut guide = actor("guide", [-5., 0.81, 0.]);
    guide.npc.as_mut().unwrap().dialogue = Some("guide".into());
    let mut template = guide.clone();
    template.id = "guide_template".into();
    s.npc_templates.push(template);
    s.entities.push(guide);
    for (id, z, courage, fear) in [("brave", -2., 1., 0.), ("careful", 2., 0., 1.)] {
        let mut n = actor(id, [-3., 0.81, z]);
        n.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"charge","initial":0,"capacity":100},{"resource":"bold","initial":0,"capacity":100},{"resource":"safe","initial":0,"capacity":100}],"grants":[]})).unwrap());
        let c = n.npc.as_mut().unwrap();
        c.mind.as_mut().unwrap().personality.courage = courage;
        c.mind.as_mut().unwrap().personality.fear = fear;
        c.objectives = Some(
            serde_json::from_value(
                json!([{"priority":0,"when":"always","action":{"kind":"routine"}}]),
            )
            .unwrap(),
        );
        c.routine=Some(serde_json::from_value(json!({"auto_dialogue":true,"needs":[{"resource":"charge","tag":"recharge","action":"charge","delta":0}],"schedule":[{"start":0,"end":800,"point":"shrine","action":"speak"}]})).unwrap());
        s.entities.push(n);
    }
    s.interactions=serde_json::from_value(json!({"values":[{"id":"visits","initial":0,"minimum":0,"maximum":10},{"id":"done","initial":0,"minimum":0,"maximum":1}],
        "points":[{"id":"well","entity":"well","tags":["recharge"],"actions":["charge","fail"]},{"id":"shrine","entity":"shrine","actions":["speak"]}],
        "actions":[{"id":"charge","range":2,"cooldown_ticks":10,"effects":[{"kind":"resource","resource":"charge","delta":100}]},{"id":"speak","range":3,"dialogue":"oracle"},
            {"id":"fail","range":20,"effects":[{"kind":"resource","resource":"energy","delta":-5},{"kind":"add_value","key":"visits","delta":1000}]}],
        "dialogues":[{"id":"oracle","entry":"offer","nodes":[{"id":"offer","text":"Escolha um caminho.","choices":[
            {"id":"bold","text":"Aceitar o desafio","personality_weights":[0,0,0,-1000,1000,0,0,0,0],"effects":[{"kind":"resource","resource":"bold","delta":1}]},
            {"id":"safe","text":"Escolher segurança","personality_weights":[0,0,0,1000,-1000,0,0,0,0],"effects":[{"kind":"resource","resource":"safe","delta":1}]}
        ]}]},{"id":"guide","entry":"loop","lock_locomotion":true,"nodes":[{"id":"loop","text":"Visitar?","choices":[
            {"id":"repeat","text":"Uma visita","next":"loop","effects":[{"kind":"resource","resource":"energy","delta":-1},{"kind":"add_value","key":"visits","delta":1}]},
            {"id":"finish","text":"Terminar","next":"thanks","effects":[{"kind":"set_value","key":"done","value":1}]}
        ]},{"id":"thanks","text":"Fechado.","choices":[]}]}]})).unwrap();
    // Action bounds remain physically meaningful; this point is approached separately below.
    s.interactions
        .actions
        .iter_mut()
        .find(|a| a.id == "fail")
        .unwrap()
        .range = 8.;
    s
}
fn doors() -> Scene {
    let mut s = base();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-2., 0.8, 4.];
    s.ability_catalog =
        serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[]}))
            .unwrap();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities = Some(
        serde_json::from_value(
            json!({"resources":[{"resource":"energy","initial":10,"capacity":10}],"grants":[]}),
        )
        .unwrap(),
    );
    s.entities
        .push(object("door", [0., 1.5, 4.], [0.6, 3., 1.], true));
    let mut traveller = actor("traveller", [-4., 0.81, 0.]);
    traveller
        .npc
        .as_mut()
        .unwrap()
        .mobility
        .as_mut()
        .unwrap()
        .fly_speed = Some(3.);
    traveller.npc.as_mut().unwrap().objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[4,0.8,0],"arrival_distance":0.2}}])).unwrap());
    let mut template = traveller.clone();
    template.id = "template".into();
    s.npc_templates.push(template);
    s.entities.push(traveller);
    let mut flyer = actor("flyer", [-4., 4.81, 0.]);
    let c = flyer.npc.as_mut().unwrap();
    c.mobility.as_mut().unwrap().fly_speed = Some(3.);
    c.local_connections = None;
    c.flight=Some(serde_json::from_value(json!({"mode":"curved","acceleration":8,"deceleration":8,"turn_degrees_per_second":180,"smoothing_seconds":0.1,"minimum_speed":0,"can_hover":true,"initial_direction":[1,0,0]})).unwrap());
    c.objectives=Some(serde_json::from_value(json!([{"priority":0,"when":"always","action":{"kind":"guard_3d","point":[4,4.8,0],"arrival_distance":0.2}}])).unwrap());
    s.entities.push(flyer);
    s.navigation_generation=Some(serde_json::from_value(json!({"entity":"traveller","min":[-4,0,0],"max":[4,4,4],"cell_size":4,"connection_radius":4.1,"include_air":true})).unwrap());
    s.interactions=serde_json::from_value(json!({"values":[{"id":"closes","initial":0,"minimum":0,"maximum":10}],"points":[{"id":"door","entity":"door","mutable":true,"actions":["ghost","close"]}],"actions":[
        {"id":"ghost","range":3,"effects":[{"kind":"object","point":"door","state":{"solid":false,"visible":true,"blocks_vision":false,"sound_transmission":1}}]},
        {"id":"close","range":3,"effects":[{"kind":"resource","resource":"energy","delta":-5},{"kind":"add_value","key":"closes","delta":1},{"kind":"object","point":"door","state":{"solid":true,"visible":true,"blocks_vision":true,"sound_transmission":0}}]}
    ]})).unwrap();
    s
}
#[test]
fn routines_dialogue_and_objects_preserve_atomicity_life_revision_physics_replica_and_replay() {
    let mut e = Engine::new(routines()).unwrap();
    e.advance(600).unwrap();
    assert_eq!(
        level(&e, "brave", "charge"),
        100,
        "{:?} {:?} {:?}",
        e.simulation.positions(),
        e.simulation.npcs["brave"].routine,
        e.simulation.mobility.states["brave"]
    );
    assert_eq!(level(&e, "careful", "charge"), 100);
    assert_eq!(level(&e, "brave", "bold"), 1);
    assert_eq!(level(&e, "brave", "safe"), 0);
    assert_eq!(level(&e, "careful", "safe"), 1);
    assert_eq!(level(&e, "careful", "bold"), 0);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let dialogue = command(
        &mut e,
        json!({"op":"talk_npc","actor":"player","npc":"guide"}),
    );
    assert_eq!(dialogue["accepted"], true);
    e.advance(1).unwrap();
    let inspection = command(&mut e, json!({"op":"inspect_dialogue","actor":"player"}));
    let handle = inspection["handle"].as_str().unwrap().to_owned();
    for _ in 0..2 {
        command(
            &mut e,
            json!({"op":"choose_dialogue","actor":"player","session":handle,"choice":"repeat"}),
        );
    }
    e.advance(1).unwrap();
    assert_eq!(level(&e, "player", "energy"), 9);
    assert_eq!(e.simulation.interactions.state.values[0], 1);
    command(
        &mut e,
        json!({"op":"choose_dialogue","actor":"player","session":handle,"choice":"finish"}),
    );
    e.advance(1).unwrap();
    assert!(
        command(&mut e, json!({"op":"inspect_dialogue","actor":"player"}))["session"].is_null()
    );
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::ChooseDialogue {
                actor: "player".into(),
                session: handle.clone(),
                choice: "repeat".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    command(
        &mut e,
        json!({"op":"talk_npc","actor":"player","npc":"guide"}),
    );
    e.advance(1).unwrap();
    let stale = command(&mut e, json!({"op":"inspect_dialogue","actor":"player"}))["handle"]
        .as_str()
        .unwrap()
        .to_owned();
    let guide = e.simulation.character_controllers["guide"].instance;
    command(
        &mut e,
        json!({"op":"activate_ability","actor":"player","slot":1,"target":"guide"}),
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["guide"].phase,
        ignis::perception::Phase::Dead
    );
    command(
        &mut e,
        json!({"op":"activate_ability","actor":"player","slot":2,"target":"guide"}),
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.character_controllers["guide"].instance, guide);
    assert!(
        e.execute(
            Command::ChooseDialogue {
                actor: "player".into(),
                session: stale,
                choice: "repeat".into()
            }
            .into()
        )
        .is_err()
    );
    let bold = level(&e, "brave", "bold");
    command(
        &mut e,
        json!({"op":"activate_ability","actor":"player","slot":1,"target":"brave"}),
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["brave"].phase,
        ignis::perception::Phase::Dead
    );
    command(
        &mut e,
        json!({"op":"activate_ability","actor":"player","slot":2,"target":"brave"}),
    );
    e.advance(1).unwrap();
    e.advance(60).unwrap();
    assert_eq!(level(&e, "brave", "bold"), bold);
    // The late value failure must roll back the earlier resource debit.
    let mut failed = routines();
    failed
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-3., 0.8, 0.];
    let mut failed = Engine::new(failed).unwrap();
    command(
        &mut failed,
        json!({"op":"interact","actor":"player","point":"well","action":"fail"}),
    );
    failed.advance(1).unwrap();
    assert_eq!(level(&failed, "player", "energy"), 10);
    assert_eq!(failed.simulation.interactions.state.values[0], 0);
    assert_eq!(
        failed
            .simulation
            .interactions
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        ignis::interactions::Outcome::ValueBounds
    );
    assert_eq!(
        Engine::verify_replay(&failed.replay()).unwrap()["matched"],
        true
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut old = e.scene.clone();
    old.version = 93;
    assert!(old.validate().is_err());
    let mut e = Engine::new(doors()).unwrap();
    e.advance(120).unwrap();
    let flyer = e.simulation.character_controllers["flyer"].instance;
    assert!(e.simulation.character_controllers["flyer"].airborne);
    let before = Frame::capture(&e, 7, None).unwrap();
    let door = before
        .actors
        .iter()
        .find(|a| a.appearance.label == "door")
        .unwrap()
        .id;
    let mut replica = Replica::new(7);
    replica.apply(&before.encode(None).unwrap()).unwrap();
    let original_graph = e.simulation.definitions().spatial_network.clone();
    command(&mut e, json!({"op":"despawn_actor","actor":"traveller"}));
    command(
        &mut e,
        json!({"op":"spawn_actor","template":"template","position":[-6,0.81,6]}),
    );
    command(
        &mut e,
        json!({"op":"interact","actor":"player","point":"door","action":"ghost"}),
    );
    e.advance(1).unwrap();
    assert_ne!(e.simulation.definitions().spatial_network, original_graph);
    assert_eq!(e.simulation.character_controllers["flyer"].instance, flyer);
    assert!(e.simulation.character_controllers["flyer"].airborne);
    let after = Frame::capture(&e, 7, Some(&before)).unwrap();
    replica
        .apply(&after.encode(Some(&before)).unwrap())
        .unwrap();
    assert_eq!(replica.actors()[&door].appearance.kind, Kind::Visual);
    // Walk into the open volume; closing must reject the entire object/resource/value batch.
    command(&mut e, json!({"op":"set_input","input":{"x":1,"z":0}}));
    for _ in 0..120 {
        if e.simulation.positions()["player"][0] >= -0.05 {
            break;
        }
        e.advance(1).unwrap();
    }
    command(&mut e, json!({"op":"set_input","input":{"x":0,"z":0}}));
    let occupied_before = e.simulation.positions()["player"];
    command(
        &mut e,
        json!({"op":"interact","actor":"player","point":"door","action":"close"}),
    );
    e.advance(1).unwrap();
    assert_eq!(
        level(&e, "player", "energy"),
        10,
        "before={occupied_before:?} now={:?} {:?}",
        e.simulation.positions()["player"],
        e.simulation.interactions.state.receipts.back()
    );
    assert_eq!(e.simulation.interactions.state.values[0], 0);
    assert!(!e.simulation.interactions.state.objects[0].solid);
    command(&mut e, json!({"op":"set_input","input":{"x":-1,"z":0}}));
    for _ in 0..120 {
        if e.simulation.positions()["player"][0] <= -2. {
            break;
        }
        e.advance(1).unwrap();
    }
    command(&mut e, json!({"op":"set_input","input":{"x":0,"z":0}}));
    command(
        &mut e,
        json!({"op":"interact","actor":"player","point":"door","action":"close"}),
    );
    e.advance(1).unwrap();
    assert_eq!(level(&e, "player", "energy"), 5);
    assert!(e.simulation.interactions.state.objects[0].solid);
    command(&mut e, json!({"op":"set_input","input":{"x":1,"z":0}}));
    e.advance(90).unwrap();
    command(&mut e, json!({"op":"set_input","input":{"x":0,"z":0}}));
    assert!(e.simulation.positions()["player"][0] < -0.5);
    e.advance(300).unwrap();
    assert_eq!(e.simulation.character_controllers["flyer"].instance, flyer);
    assert_eq!(
        e.simulation.mobility.states["flyer"].phase,
        ignis::mobility::Phase::Arrived
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut initial = doors();
    initial.interactions.points[0].initial.solid = Some(false);
    let initial = Engine::new(initial).unwrap();
    assert!(!initial.simulation.interactions.state.objects[0].solid);
    assert_eq!(
        initial
            .scene
            .entities
            .iter()
            .find(|e| e.id == "door")
            .unwrap()
            .body,
        Some(ignis::model::BodyKind::Fixed)
    );
}
