use super::*;
use crate::meshes::character::{Configuration, Placement};
#[path = "editor_character_eyes.rs"]
mod eyes;

pub(super) struct State {
    key: Option<(String, u64)>,
    draft: Configuration,
    source_path: String,
    prefix: String,
    preset_path: String,
    confirm_regenerate: bool,
    eyes_linked: bool,
    eye_side: usize,
}
impl Default for State {
    fn default() -> Self {
        Self {
            key: None,
            draft: Default::default(),
            source_path: "artifacts/character-base/human-eyes.ignis-character".into(),
            prefix: "female".into(),
            preset_path: "artifacts/character-base/female.ignis-character-config".into(),
            confirm_regenerate: false,
            eyes_linked: true,
            eye_side: 0,
        }
    }
}
fn gesture(ui: &egui::Ui) -> String {
    let epoch = ui.ctx().data_mut(|d| {
        d.get_temp::<u64>(egui::Id::new("live_edit_epoch"))
            .unwrap_or(0)
    });
    let focus = ui.memory(|m| m.focused());
    format!("ui/{epoch}/{focus:?}")
}
fn face_properties(ui: &mut egui::Ui, service: &Service, commands: &mut Vec<Command>) {
    let editor = &service.character_editor;
    if editor.handles.is_empty() {
        return;
    }
    let mut view = editor.handle_view.clone();
    let mut view_changed = false;
    ui.separator();
    let old_group = view.group.clone();
    let groups: std::collections::BTreeSet<_> = editor
        .handles
        .iter()
        .filter_map(|h| h.group.as_ref())
        .collect();
    if !groups.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut view.group, None, "Todos");
            for group in groups {
                ui.selectable_value(&mut view.group, Some(group.clone()), group);
            }
        });
    }
    if editor.handles.iter().any(|h| h.name == "cupid_valley")
        && ui.small_button("Selecionar V central do lábio").clicked()
    {
        view.group = Some("Lábios".into());
        view.selected = Some("cupid_valley".into());
        view.enabled = true;
        view_changed = true;
    }
    if editor.handles.iter().any(|h| h.name == "nose_alar_base_R") {
        ui.horizontal_wrapped(|ui| {
            ui.label("Base da asa do nariz:");
            for (label, side) in [("E", "L"), ("D", "R")] {
                if ui.small_button(label).clicked() {
                    view.group = Some("Rosto".into());
                    view.selected = Some(format!("nose_alar_base_{side}"));
                    view.enabled = true;
                    view_changed = true;
                }
            }
        });
    }
    if old_group != view.group {
        if !editor.handles.iter().any(|h| {
            view.selected.as_deref() == Some(h.name.as_str())
                && (view.group.is_none() || h.group == view.group)
        }) {
            view.selected = editor
                .handles
                .iter()
                .find(|h| view.group.is_none() || h.group == view.group)
                .map(|h| h.name.clone());
        }
        view_changed = true;
    }
    ui.horizontal_wrapped(|ui| {
        view_changed |= ui
            .checkbox(&mut view.enabled, "Editar rosto em 3D")
            .changed();
        view_changed |= ui.checkbox(&mut view.symmetry, "Simetria").changed();
        if ui
            .button(if view.group.is_some() {
                "Enquadrar região"
            } else {
                "Enquadrar rosto"
            })
            .clicked()
        {
            let mut low = glam::Vec3::splat(f32::INFINITY);
            let mut high = glam::Vec3::splat(f32::NEG_INFINITY);
            for h in editor
                .handles
                .iter()
                .filter(|h| view.group.is_none() || h.group == view.group)
            {
                low = low.min(h.position.into());
                high = high.max(h.position.into());
            }
            let center = (low + high) * 0.5;
            let basis = editor.handles[0].basis;
            let distance =
                ((high - low).length() * 2.2).max(if view.group.is_some() { 0.14 } else { 0.4 });
            let mut camera = service.engine.scene.camera.clone();
            camera.target = center.to_array();
            camera.position = (center - glam::Vec3::from(basis[2]) * distance).to_array();
            camera.fov_degrees = 32.;
            edit(
                commands,
                Action::Camera {
                    camera: Some(camera),
                },
            );
        }
    });
    if view.enabled {
        ui.small("Selecione ou arraste um ponto. Setas X/Y/Z movem em um eixo; o ponto move no plano da tela. Esc cancela a prévia. Botão direito gira a câmera.");
        egui::ComboBox::from_id_salt("face_handle_selector")
            .selected_text(
                view.selected
                    .as_ref()
                    .and_then(|name| editor.handles.iter().find(|h| h.name == *name))
                    .map_or("Escolha um ponto", |h| h.label.as_str()),
            )
            .show_ui(ui, |ui| {
                for h in editor
                    .handles
                    .iter()
                    .filter(|h| view.group.is_none() || h.group == view.group)
                {
                    if ui
                        .selectable_label(
                            view.selected.as_deref() == Some(h.name.as_str()),
                            &h.label,
                        )
                        .clicked()
                    {
                        view.selected = Some(h.name.clone());
                        view_changed = true;
                    }
                }
            });
        if let Some(h) = view
            .selected
            .as_ref()
            .and_then(|name| editor.handles.iter().find(|h| h.name == *name))
        {
            if h.name == "cupid_valley" {
                ui.small("Seta azul: recuar/projetar o centro. Seta verde: altura do V. Os dois picos têm pontos próprios. Zero mantém sua forma anterior.");
            }
            if h.name.starts_with("nose_alar_base_") {
                ui.small("Seta verde: subir/descer a base inferior da asa. Simetria também move o outro lado.");
            }
            let mut offset = h.offset;
            let mut changed = false;
            ui.add_enabled_ui(editor.ready && !service.engine.playing, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for i in 0..3 {
                        ui.label(["X", "Y", "Z"][i]);
                        let mut cm = offset[i] * 100.;
                        if ui
                            .add_enabled(
                                !(i == 0 && h.midline && view.symmetry),
                                egui::DragValue::new(&mut cm)
                                    .speed(0.025)
                                    .range(-h.limits[i] * 100. ..=h.limits[i] * 100.)
                                    .suffix(" cm")
                                    .max_decimals(2),
                            )
                            .changed()
                        {
                            offset[i] = (cm * 0.01).clamp(-h.limits[i], h.limits[i]);
                            changed = true;
                        }
                    }
                });
                if ui.small_button("Restaurar ponto").clicked() {
                    offset = [0.; 3];
                    changed = true;
                }
            });
            if changed && let Some(binding) = editor.binding(&service.engine.scene) {
                commands.push(Command::SetCharacterHandle {
                    prefix: binding.recipe.placement.prefix.clone(),
                    handle: h.name.clone(),
                    offset,
                    symmetry: view.symmetry,
                    reference: service.engine.mesh_reference(),
                    gesture: Some(gesture(ui)),
                });
            }
        }
    }
    if view_changed {
        commands.push(Command::SetCharacterHandleView { view });
    }
    ui.separator();
}
pub(super) fn properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    ui.horizontal(|ui| {
        ui.heading("Personagem");
        if ui.small_button("Fechar").clicked() {
            commands.push(Command::OpenCharacterPanel {
                open: false,
                prefix: None,
            });
        }
    });
    let scene = &service.engine.scene;
    let editor = &service.character_editor;
    egui::ScrollArea::vertical().id_salt("character_parameters").show(ui,|ui| {
        let Some(binding)=editor.binding(scene) else {
            ui.label("Conecte uma base paramétrica para editar o corpo nesta cena.");
            ui.label("Fonte da base");ui.text_edit_singleline(&mut state.source_path);
            ui.label("Nome na cena");ui.text_edit_singleline(&mut state.prefix);
            let placement=Placement {prefix:state.prefix.clone(),position:[0.,0.85,0.],rotation:[0.,0.,0.,1.],scale:1.};
            ui.add_enabled_ui(!service.engine.playing,|ui| {
                if ui.button("Conectar à base da cena").on_hover_text("Reconhece a malha existente sem substituí-la. A fonte e os parâmetros precisam corresponder à base.").clicked() {
                    commands.push(Command::ConnectCharacterSource {path:state.source_path.clone(),configuration:Default::default(),placement:placement.clone()});
                }
                if ui.button("Criar nova base").clicked() {
                    commands.push(Command::PublishCharacter {path:state.source_path.clone(),configuration:Default::default(),placement});
                }
            });
            return;
        };
        let prefix=&binding.recipe.placement.prefix;
        egui::ComboBox::from_id_salt("character_recipe").selected_text(prefix).show_ui(ui,|ui| {
            for b in &scene.characters {
                let name=&b.recipe.placement.prefix;
                if ui.selectable_label(name==prefix,name).clicked() {commands.push(Command::OpenCharacterPanel {open:true,prefix:Some(name.clone())});}
            }
        });
        ui.small("Altere as proporções e acompanhe o corpo na viewport.");
        if service.engine.playing {ui.label("Pause a cena para editar o corpo.");}
        if let Some(reason)=&editor.reason {ui.colored_label(Color32::from_rgb(230,180,100),reason);}
        if !editor.has_source(binding) {
            if ui.button("Carregar fonte").clicked() {commands.push(Command::LoadCharacterEditorSource {prefix:prefix.clone()});}
        } else if let Some(source)=&editor.source {
            let key=(prefix.clone(),service.engine.revision);
            if state.key.as_ref()!=Some(&key) {state.key=Some(key);state.draft=binding.recipe.configuration.clone();state.confirm_regenerate=false;}
            let mut changed=false;
            if !source.eyes.is_empty() && ui.button("Enquadrar olhos").clicked() {
                let mut center=glam::Vec3::ZERO;let mut count=0.;
                for eye in &source.eyes {
                    let id=format!("{}_{}",prefix,eye.iris);
                    if let (Ok(asset),Some(instance))=(scene.meshes.asset(&id),scene.meshes.instances.iter().find(|i|i.id==id)) {
                        let matrix=glam::Mat4::from_scale_rotation_translation(glam::Vec3::splat(instance.scale),glam::Quat::from_array(instance.rotation),instance.position.into());
                        for p in &asset.geometry.positions {center+=matrix.transform_point3((*p).into());count+=1.;}
                    }
                }
                if count>0. {
                    center/=count;
                    let forward=editor.handles.first().map_or(glam::Vec3::Z,|h|h.basis[2].into());
                    let mut camera=scene.camera.clone();camera.target=center.to_array();camera.position=(center-forward*0.18).to_array();camera.fov_degrees=30.;
                    edit(commands,Action::Camera {camera:Some(camera)});
                }
            }
            face_properties(ui, service, commands);
            ui.add_enabled_ui(editor.ready&&!service.engine.playing,|ui| {
                if source.parameters.iter().any(|p|p.name=="body_masculinity") {
                    ui.horizontal(|ui| {
                        ui.label("Base corporal:");
                        for (label,value) in [("Feminina",0.),("Masculina",1.)] {
                            if ui.small_button(label).clicked() {state.draft.parameters.insert("body_masculinity".into(),value);changed=true;}
                        }
                    });
                }
                if source.parameters.iter().any(|p|p.name=="posture_thoracic") {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Postura:");
                        for (label,values) in [("Base",[0.;4]),("Curvada",[0.1,-0.25,-0.6,-0.55]),("Natural",[0.,0.1,0.15,0.2]),("Ereta",[-0.35,0.5,-0.05,0.35]),("Heroica",[-0.53,0.69,-0.15,0.44])] {
                            if ui.small_button(label).clicked() {
                                for (name,value) in ["posture_pelvis","posture_lumbar","posture_thoracic","posture_shoulders"].into_iter().zip(values) {state.draft.parameters.insert(name.into(),value);}
                                // A posture preset never applies the intentional cranial form fields.
                                for name in ["neck_pitch","head_pitch"] {if source.parameters.iter().any(|p|p.name==name) {state.draft.parameters.insert(name.into(),0.);}}
                                changed=true;
                            }
                        }
                    });
                }
                changed |= eyes::controls(ui, source, &mut state.draft, &mut state.eyes_linked, &mut state.eye_side);
                for (group,names) in [
                    ("Corpo",&["body_masculinity","adiposity","musculature","muscle_definition","muscle_striation","stature"][..]),
                    ("Cintura",&["waist_width"][..]),
                    ("Postura",&["posture_pelvis","posture_lumbar","posture_thoracic","posture_shoulders"][..]),
                    ("Pescoço e cabeça",&["neck_pitch","head_pitch"][..]),
                    ("Forma da cabeça e humanoides",&["posture_head_pitch","posture_head_retraction","head_lower_pitch","head_height"][..]),
                    ("Lábios",&["lip_upper_height","lip_lower_height","lip_upper_volume","lip_lower_volume","lip_upper_width","lip_lower_width","lip_cupid_shape","lip_cupid_width","lip_philtrum","lip_upper_middle","lip_lower_middle","lip_corners","lip_upper_outer","lip_lower_outer"][..]),
                    ("Orelhas",&["ear_size_L","ear_size_R","ear_lobe_size_L","ear_lobe_size_R","ear_pointed_L","ear_pointed_R","ear_height_L","ear_height_R","ear_depth_L","ear_depth_R","ear_flare_L","ear_flare_R"][..]),
                    ("Tronco e quadril",&["shoulder_width","torso_width","torso_depth","hip_width","buttocks_volume"][..]),
                    ("Seios",&["breast_volume","breast_shape_firmness","breast_projection","breast_separation"][..]),
                    ("Membros",&["upperarm_length","lowerarm_length","upperleg_length","lowerleg_length"][..]),
                    ("Rosto",&["nose_width","nose_length","jaw_width","chin_projection","mouth_width"][..]),
                    ("Outros",&[][..]),
                ] {
                    let params:Vec<_>=source.parameters.iter().filter(|p|!p.name.starts_with("face_") && !source.eyes.iter().any(|eye| p.name.starts_with(&format!("{}_",eye.prefix))) && !(source.version>=4 && p.name=="muscle_striation") && if group=="Outros" {label(&p.name).is_none()} else {names.contains(&p.name.as_str())}).collect();
                    if params.is_empty() {continue;}
                    egui::CollapsingHeader::new(group).default_open(true).show(ui,|ui| {
                        for p in params {
                            ui.push_id(&p.name,|ui| {
                                ui.label(label(&p.name).unwrap_or(&p.name));
                                let value=state.draft.parameters.entry(p.name.clone()).or_insert(p.default);
                                let width=(ui.available_width()-35.).max(45.);
                                ui.horizontal(|ui| {
                                    let old=ui.spacing().slider_width;ui.spacing_mut().slider_width=width.min(180.);
                                    changed|=ui.add(egui::Slider::new(value,p.minimum..=p.maximum).max_decimals(3)).changed();
                                    ui.spacing_mut().slider_width=old;
                                    if ui.small_button("↺").on_hover_text("Restaurar este parâmetro").clicked() {*value=p.default;changed=true;}
                                });
                            });
                        }
                    });
                }
                if ui.button("Restaurar proporções da base").clicked() {
                    state.draft.parameters=source.parameters.iter().map(|p|(p.name.clone(),p.default)).collect();changed=true;
                }
            });
            if changed {commands.push(Command::SetCharacterParameters {prefix:prefix.clone(),configuration:state.draft.clone(),gesture:Some(gesture(ui))});}
            ui.small("Os valores são coeficientes da base. A firmeza altera a forma; movimento físico secundário ainda não está disponível.");
            if !editor.ready {
                if ui.button("Regenerar pelos parâmetros…").clicked() {state.confirm_regenerate=true;}
                if state.confirm_regenerate {
                    ui.label("Regenerar substitui a malha, materiais, transformações e animações editados destas partes. Você pode desfazer.");
                    if ui.add_enabled(!service.engine.playing,egui::Button::new("Regenerar corpo")).clicked() {commands.push(Command::RegenerateCharacter {prefix:prefix.clone()});state.confirm_regenerate=false;}
                    if ui.small_button("Cancelar").clicked() {state.confirm_regenerate=false;}
                }
            }
        }
        ui.separator();
        egui::CollapsingHeader::new("Configuração e fonte").show(ui,|ui| {
            ui.small("Os parâmetros acompanham Salvar cena e Desfazer/Refazer.");
            ui.text_edit_singleline(&mut state.preset_path);
            if ui.button("Exportar configuração").clicked() {commands.push(Command::SaveCharacterConfiguration {prefix:prefix.clone(),path:state.preset_path.clone()});}
            if ui.add_enabled(editor.ready&&!service.engine.playing,egui::Button::new("Importar configuração")).clicked() {commands.push(Command::LoadCharacterConfiguration {path:state.preset_path.clone()});}
            ui.label(format!("Fonte: {}",binding.recipe.source_path));
            ui.text_edit_singleline(&mut state.source_path);
            if ui.add_enabled(editor.ready&&!service.engine.playing,egui::Button::new("Atualizar fonte da base")).on_hover_text("Preserva os parâmetros e substitui a malha derivada pela nova fonte. Pode desfazer.").clicked() {
                commands.push(Command::UpgradeCharacterSource {prefix:prefix.clone(),path:state.source_path.clone(),reference:service.engine.mesh_reference()});
            }
            ui.small(format!("{} partes · parâmetros da fonte{}",binding.parts.len(),if editor.has_source(binding) {" carregados"} else {" indisponíveis"}));
        });
    });
}
fn label(name: &str) -> Option<&'static str> {
    Some(match name {
        "ear_size_L" => "Tamanho da orelha · E",
        "ear_size_R" => "Tamanho da orelha · D",
        "ear_lobe_size_L" => "Tamanho do lóbulo · E",
        "ear_lobe_size_R" => "Tamanho do lóbulo · D",
        "ear_pointed_L" => "Ponta da orelha (elfo / ET) · E",
        "ear_pointed_R" => "Ponta da orelha (elfo / ET) · D",
        "ear_height_L" => "Altura da orelha · E",
        "ear_height_R" => "Altura da orelha · D",
        "ear_depth_L" => "Profundidade da orelha · E",
        "ear_depth_R" => "Profundidade da orelha · D",
        "ear_flare_L" => "Abertura da orelha · E",
        "ear_flare_R" => "Abertura da orelha · D",
        "adiposity" => "Gordura corporal",
        "musculature" => "Musculatura",
        "muscle_definition" => "Definição muscular",
        "muscle_striation" => "Estrias musculares",
        "body_masculinity" => "Base feminina / masculina",
        "neck_pitch" => "Inclinação do pescoço inteiro (eixo na coluna)",
        "head_pitch" => "Inclinação da cabeça inteira (eixo no pescoço)",
        "head_lower_pitch" => "Inclinação da parte inferior da cabeça (forma)",
        "head_height" => "Altura da cabeça (forma)",
        "lip_upper_height" => "Espessura do lábio superior",
        "lip_lower_height" => "Espessura do lábio inferior",
        "lip_upper_volume" => "Volume do lábio superior",
        "lip_lower_volume" => "Volume do lábio inferior",
        "lip_upper_width" => "Largura do lábio superior",
        "lip_lower_width" => "Largura do lábio inferior",
        "lip_cupid_shape" => "Curvatura do arco do cupido",
        "lip_cupid_width" => "Largura do arco do cupido",
        "lip_philtrum" => "Volume do filtro labial",
        "lip_upper_middle" => "Centro do lábio superior",
        "lip_lower_middle" => "Centro do lábio inferior",
        "lip_corners" => "Altura dos cantos da boca",
        "lip_upper_outer" => "Laterais do lábio superior",
        "lip_lower_outer" => "Laterais do lábio inferior",
        "waist_width" => "Espessura da cintura (sem ampliar as costelas)",
        "posture_pelvis" => "Inclinação da pelve",
        "posture_lumbar" => "Curvatura lombar",
        "posture_thoracic" => "Extensão do tórax",
        "posture_head_pitch" => "Inclinação da parte superior da cabeça (forma)",
        "posture_head_retraction" => "Recuo da parte superior da cabeça (forma)",
        "posture_shoulders" => "Abertura dos ombros",
        "stature" => "Altura",
        "breast_volume" => "Volume dos seios",
        "breast_shape_firmness" => "Firmeza da forma",
        "hip_width" => "Largura do quadril",
        "shoulder_width" => "Largura dos ombros",
        "torso_width" => "Largura do tronco",
        "torso_depth" => "Profundidade do tronco",
        "buttocks_volume" => "Volume dos glúteos",
        "breast_projection" => "Projeção dos seios",
        "breast_separation" => "Separação dos seios",
        "upperarm_length" => "Comprimento do braço",
        "lowerarm_length" => "Comprimento do antebraço",
        "upperleg_length" => "Comprimento da coxa",
        "lowerleg_length" => "Comprimento da perna",
        "nose_width" => "Largura do nariz",
        "nose_length" => "Comprimento do nariz",
        "jaw_width" => "Largura da mandíbula",
        "chin_projection" => "Projeção do queixo",
        "mouth_width" => "Largura da boca",
        _ => return None,
    })
}
