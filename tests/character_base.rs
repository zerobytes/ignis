//! Critical risks: joint/weight truncation corrupts identity and skinning; invalid
//! parameter/source publication must not alter a document or its undo history.
use ignis::{
    engine::Command,
    meshes::{self, character::*},
    model::RoomRecipe,
    service::Service,
};
use std::collections::BTreeMap;

fn source() -> Source {
    let mut sums = [0.; meshes::MAX_BONES];
    for i in [0, 1, 2, 3, 4, 100, 162, 255] {
        sums[i] = 0.125;
    }
    let weight = meshes::Weight::from_sums(&sums).unwrap();
    Source {
        version: VERSION,
        id: "test_body".into(),
        provenance: "Synthetic integration fixture".into(),
        handles: Vec::new(),
        warps: Vec::new(),
        surfaces: Vec::new(),
        refinements: Vec::new(),
        eyes: Vec::new(),
        positions: vec![[0., 0., 0.], [0., 1., 0.], [1., 0., 0.]],
        parameters: vec![Parameter {
            name: "width".into(),
            minimum: 0.,
            maximum: 1.,
            default: 0.,
        }],
        targets: vec![Target {
            name: "shape".into(),
            gain: 1.,
            factors: vec![Factor {
                parameter: "width".into(),
                curve: vec![[0., 0.], [1., 1.]],
            }],
            deltas: (0..3)
                .map(|vertex| Delta {
                    vertex,
                    offset: [1., 0., 0.],
                })
                .collect(),
        }],
        joints: (0..256)
            .map(|i| Joint {
                name: format!("bone_{i}"),
                parent: None,
                head: Anchor { vertices: vec![0] },
                tail: Anchor { vertices: vec![1] },
            })
            .collect(),
        parts: vec![Part {
            name: "skin".into(),
            vertices: vec![0, 1, 2],
            triangles: vec![[0, 1, 2]],
            weights: vec![weight; 3],
            socket: None,
            color: [0.6, 0.5, 0.4],
            material: Default::default(),
        }],
    }
}
fn placement() -> Placement {
    Placement {
        prefix: "test".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
    }
}

fn handled_source() -> Source {
    let mut s = source();
    for (side, vertex) in [("left", 0), ("right", 2)] {
        let parameters = ["x", "y", "z"].map(|axis| format!("{side}_{axis}"));
        for (axis, name) in parameters.iter().enumerate() {
            s.parameters.push(Parameter {
                name: name.clone(),
                minimum: -0.02,
                maximum: 0.02,
                default: 0.,
            });
            let mut offset = [0.; 3];
            offset[axis] = 0.02;
            s.targets.push(Target {
                name: format!("move_{name}"),
                gain: 1.,
                factors: vec![Factor {
                    parameter: name.clone(),
                    curve: vec![[-0.02, -1.], [0., 0.], [0.02, 1.]],
                }],
                deltas: vec![Delta { vertex, offset }],
            });
        }
        s.handles.push(Handle {
            group: None,
            name: side.into(),
            label: side.into(),
            part: "skin".into(),
            anchor: Anchor {
                vertices: vec![vertex],
            },
            parameters,
            limits: [0.02; 3],
            mirror: Some(if side == "left" { "right" } else { "left" }.into()),
            frame_joint: 0,
            midline: false,
        });
    }
    s.parameters.push(Parameter {
        name: "posture".into(),
        minimum: -1.,
        maximum: 1.,
        default: 0.,
    });
    s.warps.push(Warp {
        rigid_sockets: Vec::new(),
        name: "posture".into(),
        parameter: "posture".into(),
        origin: Anchor { vertices: vec![0] },
        axis: [1., 0., 0.],
        angle_degrees: 20.,
        weights: vec![WarpWeight {
            vertex: 1,
            weight: 1.,
        }],
    });
    s
}

// Critical: a viewport preview cannot commit or overwrite a concurrent edit;
// paired parameter/mesh updates, invalid inputs and a whole gesture share one undo.
#[test]
fn bounded_handles_preview_symmetry_and_stale_references_are_transactional() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("body.ignis-character"),
        serde_json::to_vec(&handled_source()).unwrap(),
    )
    .unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "body.ignis-character".into(),
                configuration: Default::default(),
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.character_editor.handles.len(), 2);
    let before = service.engine.scene.clone();
    let revision = service.engine.revision;
    let reference = service.engine.mesh_reference();
    let prepared = service
        .character_editor
        .prepare_handle(
            &service.engine,
            "test",
            "left",
            [0.01, 0.005, -0.004],
            true,
            &reference,
        )
        .unwrap();
    let preview = prepared.preview(&service.engine).unwrap();
    assert_ne!(preview.meshes, before.meshes);
    let values = &preview.characters[0].recipe.configuration.parameters;
    assert_eq!((values["left_x"], values["right_x"]), (0.01, -0.01));
    assert_eq!(values["left_y"], values["right_y"]);
    assert_eq!(values["left_z"], values["right_z"]);
    assert_eq!(service.engine.scene, before);
    assert_eq!(service.engine.revision, revision);
    for offset in [[0.021, 0., 0.], [f32::NAN, 0., 0.]] {
        assert!(
            service
                .execute(
                    Command::SetCharacterHandle {
                        prefix: "test".into(),
                        handle: "left".into(),
                        offset,
                        symmetry: true,
                        reference: reference.clone(),
                        gesture: None
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(service.engine.scene, before);
        assert_eq!(service.engine.revision, revision);
    }
    for offset in [[0.005, 0., 0.], [0.01, 0.005, -0.004]] {
        service
            .execute(
                Command::SetCharacterHandle {
                    prefix: "test".into(),
                    handle: "left".into(),
                    offset,
                    symmetry: true,
                    reference: service.engine.mesh_reference(),
                    gesture: Some("drag".into()),
                }
                .into(),
            )
            .unwrap();
    }
    let edited = service.engine.scene.clone();
    let revision = service.engine.revision;
    assert_eq!(edited, preview);
    assert!(
        service
            .execute(
                Command::SetCharacterHandle {
                    prefix: "test".into(),
                    handle: "left".into(),
                    offset: [0.; 3],
                    symmetry: false,
                    reference,
                    gesture: None
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.scene, edited);
    assert_eq!(service.engine.revision, revision);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, before);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, edited);
    service
        .execute(
            Command::SaveScene {
                path: "handled.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    let loaded: ignis::model::Scene =
        serde_json::from_slice(&std::fs::read(root.path().join("handled.scene.json")).unwrap())
            .unwrap();
    let mut loaded = Service::new(root.path(), loaded).unwrap();
    loaded
        .execute(
            Command::OpenCharacterPanel {
                open: true,
                prefix: None,
            }
            .into(),
        )
        .unwrap();
    assert!(loaded.character_editor.ready);
    assert_eq!(loaded.engine.scene, edited);
}

// Critical source format and rig integration: signed fields/warps must never be
// accepted as source1, and fitting a new rest pose must preserve native skinning.
#[test]
fn source2_warps_fit_the_rest_rig_and_reject_malformed_control_data() {
    let mut legacy = source();
    legacy.version = 1;
    legacy.validate().unwrap();
    assert_eq!(
        legacy.compile(&Default::default(), &placement()).unwrap().0,
        source()
            .compile(&Default::default(), &placement())
            .unwrap()
            .0
    );
    let s = handled_source();
    s.validate().unwrap();
    let mut central = s.clone();
    central.handles[0].mirror = None;
    central.handles[0].midline = true;
    central.handles[1].mirror = None;
    central.validate().unwrap();
    let symmetric = central
        .handle_configuration(&Default::default(), "left", [0.01, 0.005, 0.004], true)
        .unwrap();
    assert_eq!(symmetric.parameters["left_x"], 0.);
    let asymmetric = central
        .handle_configuration(&Default::default(), "left", [0.01, 0.005, 0.004], false)
        .unwrap();
    assert_eq!(asymmetric.parameters["left_x"], 0.01);
    let c = Configuration {
        parameters: BTreeMap::from([("posture".into(), 0.7)]),
    };
    let d = s.compile(&c, &placement()).unwrap().0;
    assert_ne!(d, s.compile(&Default::default(), &placement()).unwrap().0);
    let pose = d.pose("test_skin", 0).unwrap();
    for (a, b) in pose.positions.iter().zip(&d.assets[0].geometry.positions) {
        assert!(glam::Vec3::from(*a).distance((*b).into()) < 1e-5);
    }
    let mut bad = s.clone();
    bad.version = 1;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.warps[0].weights[0].vertex = 999;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.handles[0].mirror = None;
    assert!(bad.validate().is_err());
    let mut bad = s;
    bad.handles[0].parameters[0] = "missing".into();
    assert!(bad.validate().is_err());
}

#[test]
fn fitted_rig_preserves_extended_influences_and_version_boundaries() {
    let s = source();
    let config = Configuration {
        parameters: BTreeMap::from([("width".into(), 1.)]),
    };
    let (mut d, _) = s.compile(&config, &placement()).unwrap();
    assert_eq!(d.assets[0].rig.as_ref().unwrap().bones.len(), 256);
    assert_eq!(
        d.assets[0].rig.as_ref().unwrap().weights[0]
            .influences()
            .filter(|(_, w)| *w > 0.)
            .count(),
        8
    );
    let pose = d.pose("test_skin", 0).unwrap();
    for (a, b) in pose.positions.iter().zip(&d.assets[0].geometry.positions) {
        assert!(glam::Vec3::from(*a).distance((*b).into()) < 1e-5);
    }
    // Bone255 and the overflow channels must actually contribute, not just survive JSON.
    let rig = d.assets[0].rig.as_ref().unwrap();
    let b = rig.bones[255].clone();
    d.assets[0].clips.push(std::sync::Arc::new(meshes::Clip {
        id: "probe".into(),
        duration_ticks: 2,
        tracks: vec![meshes::Track {
            bone: 255,
            keys: vec![meshes::Keyframe {
                tick: 0,
                translation: [b.translation[0] + 8., b.translation[1], b.translation[2]],
                rotation: b.rotation,
            }],
        }],
    }));
    d.instances[0].playback = Some(meshes::Playback {
        clip: "probe".into(),
        offset_ticks: 0,
        looping: false,
    });
    let moved = d.pose("test_skin", 0).unwrap();
    assert!((moved.positions[0][0] - pose.positions[0][0] - 1.).abs() < 1e-5);
    let roundtrip: meshes::Document =
        serde_json::from_slice(&serde_json::to_vec(&d).unwrap()).unwrap();
    assert_eq!(roundtrip, d);
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.meshes = d;
    scene.validate().unwrap();
    scene.version = 104;
    assert!(scene.validate().unwrap_err().message.contains("scene105"));
    scene.version = ignis::model::SCENE_VERSION;
    let engine = ignis::engine::Engine::new(scene).unwrap();
    let content = ignis::network::content::Content::capture(&engine, 1);
    let encoded = content.encode().unwrap();
    let decoded = ignis::network::content::Content::decode(1, &encoded).unwrap();
    assert_eq!(decoded.meshes, engine.scene.meshes);
    let mut sums = [0.; meshes::MAX_BONES];
    sums[..17].fill(1. / 17.);
    assert!(
        meshes::Weight::from_sums(&sums)
            .unwrap_err()
            .message
            .contains("exceeds16")
    );
    let mut malformed = s.clone();
    malformed.targets[0].deltas[1].vertex = 0;
    assert!(malformed.validate().is_err());
}

#[test]
fn source_publication_is_atomic_reversible_and_self_contained() {
    let root = tempfile::tempdir().unwrap();
    let s = source();
    std::fs::write(
        root.path().join("body.ignis-character"),
        serde_json::to_vec(&s).unwrap(),
    )
    .unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let before = service.engine.scene.clone();
    let publish = |parameters| Command::PublishCharacter {
        path: "body.ignis-character".into(),
        configuration: Configuration { parameters },
        placement: placement(),
    };
    assert!(
        service
            .execute(publish(BTreeMap::from([("width".into(), 2.)])).into())
            .is_err()
    );
    assert_eq!(service.engine.scene, before);
    service
        .execute(publish(BTreeMap::from([("width".into(), 1.)])).into())
        .unwrap();
    let baked = service.engine.scene.clone();
    assert_eq!(baked.meshes.assets.len(), 1);
    service
        .execute(
            Command::SaveReplay {
                path: "body.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    let replay = service
        .execute(
            Command::RunReplay {
                path: "body.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(replay.data["matched"], true);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, before);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, baked);
    std::fs::remove_file(root.path().join("body.ignis-character")).unwrap();
    let scene: ignis::model::Scene =
        serde_json::from_slice(&serde_json::to_vec(&baked).unwrap()).unwrap();
    scene.validate().unwrap();
    assert_eq!(scene, baked);
}
// A slider gesture must atomically own both the recipe and the bake. These checks
// protect undo/persistence and prevent silently replacing manual artist changes.
#[test]
fn editor_character_gestures_recipes_and_manual_edit_protection_are_atomic() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("body.ignis-character"),
        serde_json::to_vec(&source()).unwrap(),
    )
    .unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "body.ignis-character".into(),
                configuration: Default::default(),
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    let initial = service.engine.scene.clone();
    // The source cache must serve edits even if the source becomes unavailable.
    std::fs::remove_file(root.path().join("body.ignis-character")).unwrap();
    for width in [0.3, 0.7] {
        service
            .execute(
                Command::SetCharacterParameters {
                    prefix: "test".into(),
                    configuration: Configuration {
                        parameters: BTreeMap::from([("width".into(), width)]),
                    },
                    gesture: Some("one-drag".into()),
                }
                .into(),
            )
            .unwrap();
    }
    let edited = service.engine.scene.clone();
    assert_ne!(initial.meshes, edited.meshes);
    assert_eq!(
        edited.characters[0].recipe.configuration.parameters["width"],
        0.7
    );
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, initial);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, edited);
    service
        .execute(
            Command::SaveScene {
                path: "body.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::SaveCharacterConfiguration {
                prefix: "test".into(),
                path: "body.ignis-character-config".into(),
            }
            .into(),
        )
        .unwrap();
    let loaded: ignis::model::Scene =
        serde_json::from_slice(&std::fs::read(root.path().join("body.scene.json")).unwrap())
            .unwrap();
    loaded.validate().unwrap();
    assert_eq!(loaded, edited);
    let mut unavailable = Service::new(root.path(), loaded).unwrap();
    unavailable
        .execute(
            Command::OpenCharacterPanel {
                open: true,
                prefix: None,
            }
            .into(),
        )
        .unwrap();
    assert!(!unavailable.character_editor.ready);
    assert_eq!(unavailable.engine.scene, edited);
    // Artist changes remain a valid baked scene but invalidate parameter replacement.
    let mut instance = service.engine.scene.meshes.instances[0].clone();
    instance.color = [0.2, 0.3, 0.4];
    service
        .execute(
            Command::EditScene {
                edits: vec![ignis::engine::SceneEdit::Mesh {
                    edit: meshes::Edit::PutInstance { instance },
                }],
            }
            .into(),
        )
        .unwrap();
    let manual = service.engine.scene.clone();
    let revision = service.engine.revision;
    for command in [
        Command::SetCharacterParameters {
            prefix: "test".into(),
            configuration: Default::default(),
            gesture: None,
        },
        Command::LoadCharacterConfiguration {
            path: "body.ignis-character-config".into(),
        },
    ] {
        assert!(service.execute(command.into()).is_err());
        assert_eq!(service.engine.scene, manual);
        assert_eq!(service.engine.revision, revision);
    }
    service
        .execute(
            Command::RegenerateCharacter {
                prefix: "test".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene, edited);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, manual);
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, edited);
    assert!(service.character_editor.ready);
    let mut legacy = edited.clone();
    legacy.version = 105;
    assert!(legacy.validate().is_err());
}

#[test]
fn character_connection_import_and_source_identity_fail_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let source = source();
    std::fs::write(
        root.path().join("body.ignis-character"),
        serde_json::to_vec(&source).unwrap(),
    )
    .unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.meshes = source.compile(&Default::default(), &placement()).unwrap().0;
    let mut service = Service::new(root.path(), scene).unwrap();
    let baked = service.engine.scene.meshes.clone();
    service
        .execute(
            Command::ConnectCharacterSource {
                path: "body.ignis-character".into(),
                configuration: Default::default(),
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene.meshes, baked);
    let connected = service.engine.scene.clone();
    let revision = service.engine.revision;
    let public = ignis::network::content::Content::capture(&service.engine, 1)
        .encode()
        .unwrap();
    let public: serde_json::Value = serde_json::from_slice(&public).unwrap();
    assert!(public.get("characters").is_none());
    assert!(
        !String::from_utf8(serde_json::to_vec(&public).unwrap())
            .unwrap()
            .contains("body.ignis-character")
    );
    let mut recipe = connected.characters[0].recipe.clone();
    let mut corrupt = connected.clone();
    corrupt.characters[0]
        .recipe
        .configuration
        .parameters
        .insert("width".into(), 0.6);
    let mut corrupt = Service::new(root.path(), corrupt).unwrap();
    corrupt
        .execute(
            Command::OpenCharacterPanel {
                open: true,
                prefix: None,
            }
            .into(),
        )
        .unwrap();
    assert!(!corrupt.character_editor.ready);
    assert!(
        corrupt
            .execute(
                Command::SetCharacterParameters {
                    prefix: "test".into(),
                    configuration: Default::default(),
                    gesture: None
                }
                .into()
            )
            .is_err()
    );
    for bad in [2., f32::INFINITY] {
        assert!(
            service
                .execute(
                    Command::SetCharacterParameters {
                        prefix: "test".into(),
                        configuration: Configuration {
                            parameters: BTreeMap::from([("width".into(), bad)])
                        },
                        gesture: None
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(service.engine.scene, connected);
        assert_eq!(service.engine.revision, revision);
    }
    recipe.source_sha256 = "0".repeat(64);
    std::fs::write(
        root.path().join("bad.ignis-character-config"),
        serde_json::to_vec(&recipe).unwrap(),
    )
    .unwrap();
    assert!(
        service
            .execute(
                Command::LoadCharacterConfiguration {
                    path: "bad.ignis-character-config".into()
                }
                .into()
            )
            .is_err()
    );
    // A replaced source cannot be bound to the saved SHA even if it is valid.
    let mut changed = source;
    changed.provenance.push_str(" changed");
    std::fs::write(
        root.path().join("body.ignis-character"),
        serde_json::to_vec(&changed).unwrap(),
    )
    .unwrap();
    assert!(
        service
            .execute(
                Command::LoadCharacterEditorSource {
                    prefix: "test".into()
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.scene, connected);
    assert_eq!(service.engine.revision, revision);
    service.execute(Command::Undo.into()).unwrap();
    assert!(service.engine.scene.characters.is_empty());
    assert_eq!(service.engine.scene.meshes, baked);
}

// Critical source3 integration: surface coordinates/materials must round-trip in
// the same recipe+bake transaction, and malformed/legacy input must not publish.
#[test]
fn source3_surface_relief_is_validated_persistent_and_atomic() {
    let mut s = source();
    s.targets.clear();
    s.surfaces.push(Surface {
        part: "skin".into(),
        factors: vec![Factor {
            parameter: "width".into(),
            curve: vec![[0., 0.], [1., 1.]],
        }],
        coordinates: s.positions.clone(),
        detail: ignis::materials::Detail {
            profile: ignis::materials::Profile::Rounded,
            repeat: None,
            triplanar: None,
            plane: ignis::materials::Plane::Xy,
            tint: [1.; 3],
            opacity: 0.,
            bump: -0.002,
            segments: vec![ignis::materials::Segment {
                start: [0., 0.],
                end: [1., 1.],
                widths: [0.01; 2],
            }],
        },
    });
    s.validate().unwrap();
    let plain = s.compile(&Default::default(), &placement()).unwrap().0;
    assert!(plain.assets[0].material.detail.is_none());
    let active = Configuration {
        parameters: BTreeMap::from([("width".into(), 1.)]),
    };
    let detailed = s.compile(&active, &placement()).unwrap().0;
    assert_eq!(
        plain.assets[0].geometry.positions,
        detailed.assets[0].geometry.positions
    );
    assert_eq!(
        detailed.assets[0].geometry.material_coordinates,
        Some(s.positions.clone())
    );
    assert_eq!(
        detailed.assets[0].material.detail.as_ref().unwrap().bump,
        -0.002
    );
    let mut legacy = s.clone();
    legacy.version = 2;
    assert!(legacy.validate().is_err());
    let mut malformed = s.clone();
    malformed.surfaces[0].coordinates.pop();
    assert!(malformed.validate().is_err());
    let mut malformed = s.clone();
    malformed.surfaces[0].factors[0].curve[1][1] = 2.;
    assert!(malformed.validate().is_err());
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("surface.ignis-character"),
        serde_json::to_vec(&s).unwrap(),
    )
    .unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "surface.ignis-character".into(),
                configuration: active,
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    let initial = service.engine.scene.clone();
    service
        .execute(
            Command::SetCharacterParameters {
                prefix: "test".into(),
                configuration: Default::default(),
                gesture: None,
            }
            .into(),
        )
        .unwrap();
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, initial);
    service
        .execute(
            Command::SaveScene {
                path: "surface.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    let saved: ignis::model::Scene =
        serde_json::from_slice(&std::fs::read(root.path().join("surface.scene.json")).unwrap())
            .unwrap();
    saved.validate().unwrap();
    assert_eq!(saved, initial);
    std::fs::write(
        root.path().join("bad.ignis-character"),
        serde_json::to_vec(&malformed).unwrap(),
    )
    .unwrap();
    assert!(
        service
            .execute(
                Command::PublishCharacter {
                    path: "bad.ignis-character".into(),
                    configuration: Default::default(),
                    placement: Placement {
                        prefix: "bad".into(),
                        ..placement()
                    }
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.scene, initial);
}

// Regression found with the real eyes: fitting each direction from global Y
// loses socket twist under rigid head motion. Transport must preserve the full
// insert transform, including composed warps; invalid masks cannot be accepted.
#[test]
fn source3_rigid_socket_transport_preserves_the_entire_insert() {
    fn weight(joint: u8) -> meshes::Weight {
        meshes::Weight {
            joints: [joint, 0, 0, 0],
            weights: [1., 0., 0., 0.],
            extra: None,
        }
    }
    let mut s = Source {
        version: 3,
        id: "socket_test".into(),
        provenance: "Synthetic rigid insert regression".into(),
        positions: vec![
            [0., 0., 0.],
            [0., 1., 0.],
            [0.02, 0.55, -0.1],
            [0.12, 0.6, -0.5],
            [0.01, 0.55, -0.11],
            [0.03, 0.55, -0.11],
            [0.02, 0.57, -0.11],
        ],
        parameters: vec![Parameter {
            name: "pitch".into(),
            minimum: -1.,
            maximum: 1.,
            default: 0.,
        }],
        targets: vec![],
        handles: vec![],
        surfaces: vec![],
        refinements: vec![],
        eyes: vec![],
        joints: vec![
            Joint {
                name: "root".into(),
                parent: None,
                head: Anchor { vertices: vec![0] },
                tail: Anchor { vertices: vec![1] },
            },
            Joint {
                name: "eye".into(),
                parent: Some(0),
                head: Anchor { vertices: vec![2] },
                tail: Anchor { vertices: vec![3] },
            },
        ],
        parts: vec![
            Part {
                name: "skin".into(),
                vertices: vec![0, 1, 2, 3],
                triangles: vec![[0, 1, 3], [0, 3, 2]],
                weights: vec![weight(0); 4],
                socket: None,
                color: [0.5; 3],
                material: Default::default(),
            },
            Part {
                name: "insert".into(),
                vertices: vec![4, 5, 6],
                triangles: vec![[0, 1, 2]],
                weights: vec![weight(1); 3],
                socket: Some(1),
                color: [1.; 3],
                material: Default::default(),
            },
        ],
        warps: vec![Warp {
            name: "pitch".into(),
            parameter: "pitch".into(),
            origin: Anchor { vertices: vec![0] },
            axis: [1., 0., 0.],
            angle_degrees: 35.,
            weights: (0..4)
                .map(|vertex| WarpWeight { vertex, weight: 1. })
                .collect(),
            rigid_sockets: vec![1],
        }],
    };
    s.validate().unwrap();
    let original = s.compile(&Default::default(), &placement()).unwrap().0;
    let active = Configuration {
        parameters: BTreeMap::from([("pitch".into(), 1.)]),
    };
    for angle in [35_f32, 25.] {
        if angle == 25. {
            let mut second = s.warps[0].clone();
            second.name = "second".into();
            second.angle_degrees = -10.;
            s.warps.push(second);
        }
        let changed = s.compile(&active, &placement()).unwrap().0;
        let rotation = glam::Quat::from_rotation_x(angle.to_radians());
        for (a, b) in original
            .asset("test_insert")
            .unwrap()
            .geometry
            .positions
            .iter()
            .zip(&changed.asset("test_insert").unwrap().geometry.positions)
        {
            assert!((rotation * glam::Vec3::from(*a)).distance((*b).into()) < 1e-5);
        }
        let pose = changed.pose("test_insert", 0).unwrap();
        for (a, b) in pose
            .positions
            .iter()
            .zip(&changed.asset("test_insert").unwrap().geometry.positions)
        {
            assert!(glam::Vec3::from(*a).distance((*b).into()) < 1e-5);
        }
    }
    let mut bad = s.clone();
    bad.warps[0].weights[3].weight = 0.9;
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
    let mut legacy = s;
    legacy.version = 2;
    for w in &mut legacy.warps {
        w.rigid_sockets.clear();
    }
    assert_eq!(
        legacy.compile(&Default::default(), &placement()).unwrap().0,
        original
    );
}

// Critical source4 risk: topology refinement must retain all skin channels and
// source upgrades must replace recipe+bake as one undoable, guarded transaction.
#[test]
fn source4_refinement_and_upgrade_preserve_channels_and_authoring_state() {
    let mut cage = source();
    cage.positions = vec![[-1., 0., 0.], [-1., 1., 0.], [1., 1., 0.], [1., 0., 0.]];
    cage.parts[0].vertices = vec![0, 1, 2, 3];
    cage.parts[0].triangles = vec![[0, 1, 2], [0, 2, 3]];
    cage.parts[0].weights = vec![cage.parts[0].weights[0].clone(); 4];
    let mut smooth = cage.clone();
    smooth.refinements = vec![Refinement::new("skin".into(), vec![vec![0, 1, 2, 3]])];
    let (bake, _) = smooth
        .compile(&Configuration::default(), &placement())
        .unwrap();
    let asset = &bake.assets[0];
    assert_eq!(asset.geometry.positions.len(), 9);
    assert_eq!(asset.geometry.triangles.len(), 8);
    assert_eq!(asset.geometry.positions[0], [-0.75, 0.125, 0.]);
    assert_eq!(asset.geometry.positions[8], [0., 0.5, 0.]);
    assert!(
        asset.rig.as_ref().unwrap().weights.iter().all(|w| w
            .influences()
            .filter(|(_, v)| *v > 0.)
            .count()
            == 8)
    );
    let encoded = serde_json::to_vec(&smooth).unwrap();
    assert_eq!(decode(&encoded).unwrap(), smooth);
    let mut wrong = smooth.clone();
    wrong.version = 3;
    assert!(wrong.validate().is_err());
    wrong = smooth.clone();
    wrong.refinements[0].faces[0].swap(1, 2);
    assert!(wrong.validate().is_err());
    // Warm caches cannot conceal later invalid cage edits.
    wrong = smooth.clone();
    wrong.parts[0].weights[3].weights[0] = 0.;
    assert!(wrong.validate().is_err());
    let mut excess = smooth.clone();
    for (i, w) in excess.parts[0].weights.iter_mut().enumerate() {
        let mut sums = [0.; meshes::MAX_BONES];
        for k in 0..8 {
            sums[i * 8 + k] = 0.125;
        }
        *w = meshes::Weight::from_sums(&sums).unwrap();
    }
    assert!(excess.validate().is_err()); // Never prune a >16-joint refined vertex.
    let root = tempfile::tempdir().unwrap();
    for (name, s) in [("cage", &cage), ("smooth", &smooth), ("invalid", &wrong)] {
        std::fs::write(
            root.path().join(format!("{name}.ignis-character")),
            serde_json::to_vec(s).unwrap(),
        )
        .unwrap();
    }
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "cage.ignis-character".into(),
                configuration: Configuration {
                    parameters: BTreeMap::from([("width".into(), 0.4)]),
                },
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    let before = service.engine.scene.clone();
    let reference = service.engine.mesh_reference();
    service
        .execute(
            Command::UpgradeCharacterSource {
                prefix: "test".into(),
                path: "smooth.ignis-character".into(),
                reference: reference.clone(),
            }
            .into(),
        )
        .unwrap();
    let after = service.engine.scene.clone();
    assert_eq!(
        after.characters[0].recipe.configuration,
        before.characters[0].recipe.configuration
    );
    assert_eq!(after.meshes.assets[0].geometry.positions.len(), 9);
    let revision = service.engine.revision;
    for (path, reference) in [
        ("smooth", reference),
        ("invalid", service.engine.mesh_reference()),
    ] {
        assert!(
            service
                .execute(
                    Command::UpgradeCharacterSource {
                        prefix: "test".into(),
                        path: format!("{path}.ignis-character"),
                        reference
                    }
                    .into()
                )
                .is_err()
        );
        assert_eq!(service.engine.scene, after);
        assert_eq!(service.engine.revision, revision);
    }
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, before);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, after);
    let saved: ignis::model::Scene =
        serde_json::from_slice(&serde_json::to_vec(&after).unwrap()).unwrap();
    assert_eq!(saved, after);
    saved.validate().unwrap();
    service
        .execute(
            Command::SaveReplay {
                path: "refinement.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::RunReplay {
                path: "refinement.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene, after);

    let mut instance = after.meshes.instances[0].clone();
    instance.color = [0.2, 0.3, 0.4];
    service
        .execute(
            Command::EditScene {
                edits: vec![ignis::engine::SceneEdit::Mesh {
                    edit: meshes::Edit::PutInstance { instance },
                }],
            }
            .into(),
        )
        .unwrap();
    let manual = service.engine.scene.clone();
    assert!(
        service
            .execute(
                Command::UpgradeCharacterSource {
                    prefix: "test".into(),
                    path: "cage.ignis-character".into(),
                    reference: service.engine.mesh_reference()
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.scene, manual);
}

// Format boundary: adding an authoring landmark must not silently reinterpret
// source1..4 budgets or change baked-scene/configuration envelopes.
#[test]
fn source5_additional_handles_keep_legacy_limits_and_parameter_budget() {
    let mut s = handled_source();
    let template = s.handles[0].clone();
    s.handles.clear();
    for i in 0..81 {
        let mut h = template.clone();
        h.name = format!("landmark_{i}");
        h.mirror = None;
        h.parameters = ["x", "y", "z"].map(|axis| format!("landmark_{i}_{axis}"));
        for name in &h.parameters {
            s.parameters.push(Parameter {
                name: name.clone(),
                minimum: -0.02,
                maximum: 0.02,
                default: 0.,
            });
        }
        s.handles.push(h);
    }
    s.version = 5;
    assert!(s.parameters.len() <= MAX_PARAMETERS);
    assert!(s.validate().is_err());
    s.handles.pop();
    s.validate().unwrap();
    assert_eq!(decode(&serde_json::to_vec(&s).unwrap()).unwrap(), s);
    s.version = 4;
    assert!(s.validate().is_err());
    s.handles.truncate(64);
    s.validate().unwrap();
    while s.parameters.len() <= 256 {
        s.parameters.push(Parameter {
            name: format!("legacy_extra_{}", s.parameters.len()),
            minimum: -1.,
            maximum: 1.,
            default: 0.,
        });
    }
    s.version = 5;
    assert!(s.validate().is_err());
    s.version = 6;
    s.validate().unwrap();
    while s.parameters.len() <= MAX_PARAMETERS {
        s.parameters.push(Parameter {
            name: format!("extra_{}", s.parameters.len()),
            minimum: -1.,
            maximum: 1.,
            default: 0.,
        });
    }
    s.version = 6;
    assert!(s.validate().is_err());
}

// Critical: expanded authoring configurations need explicit versions; neither
// old scenes/recipes nor source5 may silently accept the expanded envelope.
#[test]
fn source6_expanded_configuration_versions_are_atomic_and_durable() {
    let mut s = source();
    for i in 0..261 {
        s.parameters.push(Parameter {
            name: format!("additional_{i}"),
            minimum: -1.,
            maximum: 1.,
            default: 0.,
        });
    }
    s.validate().unwrap();
    let mut old = s.clone();
    old.version = 5;
    assert!(old.validate().is_err());
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("expanded.ignis-character"),
        serde_json::to_vec(&s).unwrap(),
    )
    .unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.version = 108;
    let mut service = Service::new(root.path(), scene).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "expanded.ignis-character".into(),
                configuration: Default::default(),
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    let after = service.engine.scene.clone();
    assert_eq!(after.version, ignis::model::SCENE_VERSION);
    assert_eq!(after.characters[0].recipe.version, 2);
    assert_eq!(
        after.characters[0].recipe.configuration.parameters.len(),
        262
    );
    let mut legacy = after.clone();
    legacy.version = 108;
    assert!(legacy.validate().is_err());
    legacy = after.clone();
    legacy.characters[0].recipe.version = 1;
    assert!(legacy.validate().is_err());
    let mut recipe = after.characters[0].recipe.clone();
    recipe.version = 1;
    recipe.configuration.parameters.retain(|k, _| k == "width");
    recipe.validate().unwrap();
    legacy.characters[0].recipe = recipe;
    legacy.version = 106;
    legacy.validate().unwrap();
    let reference = service.engine.mesh_reference();
    let mut invalid = after.characters[0].clone();
    invalid.recipe.version = 1;
    assert!(
        service
            .execute(
                Command::EditScene {
                    edits: vec![ignis::engine::SceneEdit::SetCharacterBinding {
                        binding: Box::new(invalid)
                    }]
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.scene, after);
    assert_eq!(service.engine.mesh_reference(), reference);
    service
        .execute(
            Command::SaveCharacterConfiguration {
                prefix: "test".into(),
                path: "expanded.ignis-character-config".into(),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::SavePortableSave {
                path: "expanded.ignis-portable".into(),
            }
            .into(),
        )
        .unwrap();
    service.execute(Command::Undo.into()).unwrap();
    assert!(service.engine.scene.characters.is_empty());
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, after);
    service
        .execute(
            Command::SaveReplay {
                path: "expanded.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::RunReplay {
                path: "expanded.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene, after);
}

#[test]
fn ocular_source_contract_cache_rigid_transport_and_durability() {
    let mut s = source();
    for (name, _, min, max, default) in eyes::CONTROLS {
        s.parameters.push(Parameter {
            name: format!("ocular_{name}"),
            minimum: *min,
            maximum: *max,
            default: if *name == "quality" { 0. } else { *default },
        });
    }
    for name in ["sclera", "iris", "cornea"] {
        let mut p = s.parts[0].clone();
        p.name = name.into();
        p.socket = Some(255);
        let mut sums = [0.; meshes::MAX_BONES];
        sums[255] = 1.;
        p.weights = vec![meshes::Weight::from_sums(&sums).unwrap(); 3];
        s.parts.push(p);
    }
    s.eyes.push(Eye::new(
        "sclera".into(),
        "iris".into(),
        "cornea".into(),
        Anchor { vertices: vec![0] },
        "ocular".into(),
        0.012,
    ));
    s.validate().unwrap();
    let mut bad = s.clone();
    bad.version = 6;
    assert!(bad.validate().is_err());
    bad = s.clone();
    bad.eyes[0].cornea = "iris".into();
    assert!(bad.validate().is_err());
    bad = s.clone();
    bad.parts[3].weights[0] = s.parts[0].weights[0].clone();
    assert!(bad.validate().is_err());
    let (a, _) = s.compile(&Configuration::default(), &placement()).unwrap();
    let (b, _) = s.compile(&Configuration::default(), &placement()).unwrap();
    let ai = a.asset("test_iris").unwrap();
    let bi = b.asset("test_iris").unwrap();
    assert!(std::sync::Arc::ptr_eq(
        ai.material.procedural.as_ref().unwrap(),
        bi.material.procedural.as_ref().unwrap()
    ));
    let material = ai.material.procedural.as_ref().unwrap();
    assert_eq!(material.fixed.as_ref().unwrap().resolution, 64);
    assert_eq!(material.fixed.as_ref().unwrap().samples.len(), 5461);
    assert!(material.sample([0.5, 0.5], 0.).color_roughness[0] < 0.003);
    assert!(material.sample([0.8, 0.5], 0.).color_roughness[0] > 0.003);
    assert!(
        material
            .sample([0.8, 0.5], 1.)
            .color_roughness
            .iter()
            .all(|v| v.is_finite())
    );
    let mut config = Configuration::default();
    config.parameters.insert("width".into(), 0.2);
    let (translated, _) = s.compile(&config, &placement()).unwrap();
    for id in ["test_sclera", "test_iris", "test_cornea"] {
        let p = &a.asset(id).unwrap().geometry.positions;
        let q = &translated.asset(id).unwrap().geometry.positions;
        for (p, q) in p.iter().zip(q) {
            assert!((q[0] - p[0] - 0.2).abs() < 1e-6);
            assert_eq!(p[1], q[1]);
            assert_eq!(p[2], q[2]);
        }
        assert_eq!(a.asset(id).unwrap().rig.as_ref().unwrap().bones.len(), 256);
    }
    let iris_again = translated.asset("test_iris").unwrap();
    assert!(std::sync::Arc::ptr_eq(
        material,
        iris_again.material.procedural.as_ref().unwrap()
    ));
    config.parameters.insert("ocular_seed".into(), 91.);
    let (different, _) = s.compile(&config, &placement()).unwrap();
    assert_ne!(
        material.fixed,
        different
            .asset("test_iris")
            .unwrap()
            .material
            .procedural
            .as_ref()
            .unwrap()
            .fixed
    );
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("ocular.ignis-character"),
        serde_json::to_vec(&s).unwrap(),
    )
    .unwrap();
    let mut service = Service::new(root.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    service
        .execute(
            Command::PublishCharacter {
                path: "ocular.ignis-character".into(),
                configuration: Default::default(),
                placement: placement(),
            }
            .into(),
        )
        .unwrap();
    let baseline = service.engine.scene.clone();
    assert!(baseline.version >= 112);
    service
        .execute(
            Command::OpenCharacterPanel {
                open: true,
                prefix: Some("test".into()),
            }
            .into(),
        )
        .unwrap();
    let mut invalid = Configuration::default();
    invalid.parameters.insert("ocular_pupil".into(), 0.9);
    assert!(
        service
            .execute(
                Command::SetCharacterParameters {
                    prefix: "test".into(),
                    configuration: invalid,
                    gesture: None
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(service.engine.scene, baseline);
    service
        .execute(
            Command::SetCharacterParameters {
                prefix: "test".into(),
                configuration: config,
                gesture: None,
            }
            .into(),
        )
        .unwrap();
    let changed = service.engine.scene.clone();
    service.execute(Command::Undo.into()).unwrap();
    assert_eq!(service.engine.scene, baseline);
    service.execute(Command::Redo.into()).unwrap();
    assert_eq!(service.engine.scene, changed);
    service
        .execute(
            Command::SaveReplay {
                path: "ocular.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::RunReplay {
                path: "ocular.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene, changed);
    service
        .execute(
            Command::SaveScene {
                path: "ocular.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    service
        .execute(
            Command::LoadScene {
                path: "ocular.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(service.engine.scene, changed);
    service
        .execute(
            Command::SavePortableSave {
                path: "ocular.ignis-portable".into(),
            }
            .into(),
        )
        .unwrap();
}
