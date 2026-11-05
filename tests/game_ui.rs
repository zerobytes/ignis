use ignis::{
    engine::Command,
    game_ui::{Document, Edit, Kind, Node, Operation, Runtime, State, Text, Value},
    model::RoomRecipe,
    service::Service,
};
use serde_json::json;
fn fixture() -> Document {
    serde_json::from_str(include_str!("../examples/ui-workbench.ignis-ui")).unwrap()
}
fn command(op: Operation, revision: Option<u64>) -> Command {
    Command::Ui {
        operation: Box::new(op),
        expected_ui_revision: revision,
    }
}
#[test]
fn ui_transactions_preserve_live_world_and_reject_partial_or_stale_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Service::new(dir.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    s.execute(Command::Step { ticks: 8 }.into()).unwrap();
    s.execute(Command::Play.into()).unwrap();
    let scene = serde_json::to_value(&s.engine.scene).unwrap();
    let replay = serde_json::to_value(s.engine.replay()).unwrap();
    let revision = s.engine.revision;
    s.execute(
        command(
            Operation::Replace {
                document: Box::new(fixture()),
            },
            Some(0),
        )
        .into(),
    )
    .unwrap();
    let before = s.game_ui.inspect();
    let mut node = s.game_ui.document().root.find("title").unwrap().clone();
    node.text = Text::Literal("New title".into());
    let valid = Edit::ReplaceNode {
        id: node.id.clone(),
        node: Box::new(node),
    };
    for edits in [
        vec![valid.clone(), Edit::Remove { id: "root".into() }],
        vec![Edit::Move {
            id: "actions".into(),
            parent: "increment".into(),
            index: 0,
        }],
        vec![Edit::Insert {
            parent: "root".into(),
            index: 0,
            node: Box::new(Node::new("title", Kind::Label)),
        }],
    ] {
        assert!(
            s.execute(command(Operation::Edit { edits }, Some(1)).into())
                .is_err()
        );
        assert_eq!(before, s.game_ui.inspect());
    }
    assert!(
        s.execute(
            command(
                Operation::Edit {
                    edits: vec![valid.clone()]
                },
                Some(0)
            )
            .into()
        )
        .is_err()
    );
    assert_eq!(before, s.game_ui.inspect());
    s.execute(command(Operation::Edit { edits: vec![valid] }, Some(1)).into())
        .unwrap();
    s.execute(
        command(
            Operation::Save {
                path: "ui/current.ignis-ui".into(),
            },
            Some(2),
        )
        .into(),
    )
    .unwrap();
    let edited = s.game_ui.document().clone();
    s.execute(command(Operation::Undo, Some(2)).into()).unwrap();
    assert_eq!(s.game_ui.document(), &fixture());
    s.execute(command(Operation::Redo, Some(3)).into()).unwrap();
    assert_eq!(s.game_ui.document(), &edited);
    s.execute(command(Operation::Undo, Some(4)).into()).unwrap();
    s.execute(
        command(
            Operation::Load {
                path: "ui/current.ignis-ui".into(),
            },
            Some(5),
        )
        .into(),
    )
    .unwrap();
    assert_eq!(s.game_ui.document(), &edited);
    let before = s.game_ui.inspect();
    std::fs::write(dir.path().join("bad.ignis-ui"), b"{\"version\":999}").unwrap();
    assert!(
        s.execute(
            command(
                Operation::Load {
                    path: "bad.ignis-ui".into()
                },
                None
            )
            .into()
        )
        .is_err()
    );
    assert_eq!(before, s.game_ui.inspect());
    assert!(
        s.execute(
            command(
                Operation::Save {
                    path: "../escape".into()
                },
                None
            )
            .into()
        )
        .is_err()
    );
    assert_eq!(scene, serde_json::to_value(&s.engine.scene).unwrap());
    assert_eq!(replay, serde_json::to_value(s.engine.replay()).unwrap());
    assert_eq!(
        (
            s.engine.revision,
            s.engine.simulation.tick,
            s.engine.playing
        ),
        (revision, 8, true)
    );
}
#[test]
fn ui_document_and_runtime_are_bounded_with_independent_component_identity() {
    let mut d = fixture();
    d.validate().unwrap();
    let mut state = State::default();
    state.replace(d.clone()).unwrap();
    let before = state.inspect();
    d.theme.font_size = f32::NAN;
    assert!(state.replace(d).is_err());
    assert_eq!(before, state.inspect());
    let mut d = fixture();
    d.version = 2;
    assert!(d.validate().is_err());
    let mut d = fixture();
    for i in 0..260 {
        d.root
            .children
            .push(Node::new(format!("n{i}"), Kind::Label));
    }
    assert!(d.validate().is_err());
    let mut d = fixture();
    let mut n = Node::new("deep", Kind::Column);
    for i in 0..18 {
        let mut p = Node::new(format!("p{i}"), Kind::Column);
        p.children.push(n);
        n = p;
    }
    d.root.children.push(n);
    assert!(d.validate().is_err());
    let component = fixture()
        .root
        .find("actions")
        .unwrap()
        .instantiate("inventory")
        .unwrap();
    assert!(component.find("inventory.increment").is_some());
    let mut runtime = Runtime::new("consumer").unwrap();
    runtime.set_value("progress", Value::Number(0.5)).unwrap();
    let before = runtime.data().clone();
    assert!(
        runtime
            .set_value("progress", Value::Number(f32::INFINITY))
            .is_err()
    );
    assert_eq!(&before, runtime.data());
}
#[test]
fn mcp_ui_edits_use_the_same_service_history_and_schema() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let mut session = ignis::mcp::Session::default();
    session.handle(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),&mut service).unwrap();
    session.handle(
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        &mut service,
    );
    let list = session
        .handle(
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            &mut service,
        )
        .unwrap();
    assert!(list.to_string().contains("expected_ui_revision"));
    for (i, op) in [
        Operation::Replace {
            document: Box::new(fixture()),
        },
        Operation::Edit {
            edits: vec![Edit::Move {
                id: "name".into(),
                parent: "root".into(),
                index: 0,
            }],
        },
        Operation::Undo,
    ]
    .into_iter()
    .enumerate()
    {
        let result=session.handle(json!({"jsonrpc":"2.0","id":i+3,"method":"tools/call","params":{"name":"engine_command","arguments":{"command":command(op,Some(i as u64))}}}),&mut service).unwrap();
        assert_eq!(result["result"]["isError"], false, "{result}");
    }
    assert_eq!(service.game_ui.document(), &fixture());
}
