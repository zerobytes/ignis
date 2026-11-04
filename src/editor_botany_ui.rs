use super::*;
use crate::botany::{ManualPolicy, Operation as Op, OrganEdit, Recipe, dynamics};

pub(super) struct State {
    key: Option<(String, u64)>,
    prefix: Option<String>,
    draft: Option<Recipe>,
    path: String,
    filter: String,
    edit: OrganEdit,
    organ: Option<String>,
    physics: dynamics::Settings,
    root_stem: String,
    roots: crate::botany::roots::Settings,
    roots_enabled: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            key: None,
            prefix: None,
            draft: None,
            path: "assets/botany/palmeirinha.ignis-botany".into(),
            filter: String::new(),
            edit: Default::default(),
            organ: None,
            physics: Default::default(),
            root_stem: String::new(),
            roots: Default::default(),
            roots_enabled: false,
        }
    }
}
fn send(commands: &mut Vec<Command>, op: Op) {
    commands.push(Command::Botany {
        operation: Box::new(op),
    });
}
pub(super) fn properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    egui::ScrollArea::vertical().show(ui,|ui|{
        ui.heading("Vegetação");
        ui.label("Fonte, órgãos e crescimento");
        let selected=service.botany.selected.as_deref();
        egui::ComboBox::from_id_salt("botany_choice").selected_text(selected.unwrap_or("Escolha uma planta")).show_ui(ui,|ui|{
            for r in &service.engine.scene.botany{if ui.selectable_label(selected==Some(&r.prefix),&r.source.name).clicked(){send(commands,Op::Open{open:true,prefix:Some(r.prefix.clone())});}}
        });
        ui.collapsing("Carregar exemplar / fonte",|ui|{
            for (id,name) in [("palmeirinha","Palmeirinha"),("platano","Plátano"),("eucalipto","Eucalipto"),("figueira","Figueira ornamental"),("figueira_rs","Figueira campestre · RS")] {
                if ui.button(name).clicked(){send(commands,Op::Load{path:format!("assets/botany/{id}.ignis-botany"),publish:true});}
            }
            ui.text_edit_singleline(&mut state.path);
            if ui.button("Carregar fonte").clicked(){send(commands,Op::Load{path:state.path.clone(),publish:true});}
        });
        let Some(recipe)=service.engine.scene.botany.iter().find(|r|Some(r.prefix.as_str())==selected)else{return;};
        let key=Some((recipe.prefix.clone(),service.engine.revision));
        if state.prefix.as_deref()!=Some(recipe.prefix.as_str()) {
            state.path=format!("artifacts/botany-native/{}.ignis-botany",recipe.prefix);
            state.prefix=Some(recipe.prefix.clone());state.organ=None;
        }
        if state.key!=key{state.draft=Some(recipe.clone());state.key=key;
            if !recipe.source.organs.iter().any(|o|o.id==state.root_stem&&o.parent.is_none()){state.root_stem=recipe.source.organs.iter().find(|o|o.parent.is_none()).map(|o|o.id.clone()).unwrap_or_default();}
            let system=recipe.source.root_systems.iter().find(|r|r.stem==state.root_stem);state.roots_enabled=system.is_some();state.roots=system.map(|s|s.settings.clone()).unwrap_or_default();
        }
        let draft=state.draft.as_mut().unwrap();let mut changed=false;
        ui.separator();
        ui.label(format!("{} órgãos potenciais na fonte",recipe.source.organs.len()));
        if let Some(profile)=&recipe.source.development {
            let age=draft.settings.age_years.get_or_insert(profile.reference_age_years);
            let response=ui.add(egui::Slider::new(age,0.25..=200.).text("Idade da planta (anos)"));changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
            let response=ui.add(egui::Slider::new(&mut draft.settings.vigor,0.25..=2.).text("Vigor de desenvolvimento"));changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
            ui.label("Anos regulam tronco, copa e órgãos pelo perfil da fonte.");
        } else {ui.label("Fonte sem perfil de desenvolvimento cronológico.");}
        changed|=ui.add(egui::Slider::new(&mut draft.settings.scale,0.1..=5.).text("Escala")).changed();
        changed|=ui.add(egui::Slider::new(&mut draft.settings.crown_width,0.1..=5.).text("Largura da copa")).changed();
        changed|=ui.add(egui::Slider::new(&mut draft.settings.radius,0.1..=5.).text("Espessura dos eixos")).changed();
        changed|=ui.add(egui::Slider::new(&mut draft.settings.leaf_length,0.1..=5.).text("Comprimento das folhas")).changed();
        changed|=ui.add(egui::Slider::new(&mut draft.settings.leaf_width,0.1..=5.).text("Largura das folhas")).changed();
        changed|=ui.add(egui::Slider::new(&mut draft.settings.leaf_density,0. ..=1.).text("Densidade de folhas")).changed();
        if draft.source.organs.iter().any(|o|o.life.as_ref().is_some_and(|life|life.role==crate::botany::development::Role::Frond)) {
            let response=ui.add(egui::Slider::new(&mut draft.settings.frond_growth_variation,0. ..=1.).text("Variação de crescimento das frondes"));
            let growth_changed=response.drag_stopped()||(response.changed()&&!response.dragged());
            if growth_changed {draft.version=draft.version.max(9);changed=true;}
            ui.label("0: tamanhos de referência. 1: estágios jovens e desenvolvidos da fonte.");
        }
        ui.label("Folhas");changed|=ui.color_edit_button_rgb(&mut draft.settings.leaf_color).changed();
        if changed{send(commands,Op::Preview{recipe:Box::new(draft.clone()),policy:ManualPolicy::Preserve});}
        ui.horizontal_wrapped(|ui|{
            if ui.button("Aplicar").clicked(){send(commands,Op::Publish{recipe:Box::new(draft.clone()),policy:ManualPolicy::Preserve});}
            if ui.button("Cancelar prévia").clicked(){state.key=None;send(commands,Op::CancelPreview);}
            if ui.button("Salvar fonte").clicked(){send(commands,Op::Save{prefix:recipe.prefix.clone(),path:state.path.clone()});}
        });
        ui.collapsing("Regeneração com descarte",|ui|{
            ui.label("Descarta escultura e alterações manuais feitas na malha. Use apenas para substituir esses ajustes.");
            if ui.button("Descartar ajustes e regenerar").clicked(){send(commands,Op::Publish{recipe:Box::new(draft.clone()),policy:ManualPolicy::Discard});}
        });
        ui.collapsing("Sistema de raízes",|ui|{
            let previous=state.root_stem.clone();
            egui::ComboBox::from_id_salt("root_stem").selected_text(&state.root_stem).show_ui(ui,|ui|{for o in recipe.source.organs.iter().filter(|o|o.parent.is_none()&&matches!(o.shape,crate::botany::Shape::Axis{..})){ui.selectable_value(&mut state.root_stem,o.id.clone(),&o.id);}});
            if previous!=state.root_stem {let old=recipe.source.root_systems.iter().find(|r|r.stem==state.root_stem);state.roots_enabled=old.is_some();state.roots=old.map(|r|r.settings.clone()).unwrap_or_default();}
            let mut root_changed=ui.checkbox(&mut state.roots_enabled,"Raízes basais").changed();
            let root_sides=if state.roots.buttress.is_some(){recipe.settings.sides.max(12)}else{recipe.settings.sides};
            let max_count=((160-recipe.settings.sides as usize)/root_sides as usize).min(12) as u8;
            if state.roots_enabled {
                let kind=state.roots.kind;
                egui::ComboBox::from_id_salt("root_kind").selected_text(match kind{crate::botany::roots::Kind::Radial=>"Radiais / superficiais",crate::botany::roots::Kind::Taproot=>"Pivotante",crate::botany::roots::Kind::Fibrous=>"Fibrosas"}).show_ui(ui,|ui|{for (value,label) in [(crate::botany::roots::Kind::Radial,"Radiais / superficiais"),(crate::botany::roots::Kind::Taproot,"Pivotante"),(crate::botany::roots::Kind::Fibrous,"Fibrosas")]{ui.selectable_value(&mut state.roots.kind,value,label);}});
                if kind!=state.roots.kind {root_changed=true;match state.roots.kind{crate::botany::roots::Kind::Taproot=>{state.roots.buttress=None;state.roots.count=1;state.roots.depth=8.;state.roots.exposure=0.;},crate::botany::roots::Kind::Fibrous=>{state.roots.buttress=None;state.roots.count=10.min(max_count);state.roots.radius_ratio=0.10;state.roots.depth=5.;state.roots.exposure=0.;},crate::botany::roots::Kind::Radial=>{state.roots=Default::default();}}}
                state.roots.count=state.roots.count.min(max_count);
                let response=ui.add(egui::DragValue::new(&mut state.roots.seed).speed(1.).prefix("Semente das raízes "));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
                let response=ui.add(egui::Slider::new(&mut state.roots.count,1..=if state.roots.kind==crate::botany::roots::Kind::Taproot{1}else{max_count}).text("Raízes principais"));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
                let response=ui.add(egui::Slider::new(&mut state.roots.orders,0..=3).text("Ordens de ramificação"));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
                for (label,value,lo,hi) in [("Alcance / raio do tronco",&mut state.roots.spread,2.,40.),("Profundidade / raio do tronco",&mut state.roots.depth,0.1,40.),("Raio da raiz / tronco",&mut state.roots.radius_ratio,0.03,0.9),("Altura do colo / raio",&mut state.roots.neck_height,0.5,5.),("Exposição",&mut state.roots.exposure,0.,1.),("Comprimento por ordem",&mut state.roots.length_ratio,0.3,0.85),("Ângulo dos filhos (rad)",&mut state.roots.branch_angle,0.2,1.2),("Irregularidade",&mut state.roots.irregularity,0.,0.35),("Trecho distal ancorado",&mut state.roots.anchored_fraction,0.,1.)]{let response=ui.add(egui::Slider::new(value,lo..=hi).text(label));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());}
                if state.roots.kind==crate::botany::roots::Kind::Radial {
                    let mut enabled=state.roots.buttress.is_some();
                    if ui.checkbox(&mut enabled,"Paredes / contrafortes com crista").changed(){root_changed=true;state.roots.buttress=enabled.then(Default::default);}
                    if let Some(form)=&mut state.roots.buttress {
                        for (label,value,lo,hi) in [("Altura da parede / raio do tronco",&mut form.height,0.,8.),("Alcance da parede / raio",&mut form.reach,1.,30.),("Espessura da parede",&mut form.wall_width,0.08,1.),("Raio da crista",&mut form.crest_width,0.08,1.5),("Meandro da trajetória",&mut form.meander,0.,0.35),("Nervuras no tronco",&mut form.fluting,0.,2.)]{
                            let response=ui.add(egui::Slider::new(value,lo..=hi).text(label));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
                        }
                    }
                }
                ui.label("Solo: plano local da planta");for (label,value) in [("Altura",&mut state.roots.soil_height),("Inclinação X",&mut state.roots.soil_slope[0])]{let response=ui.add(egui::DragValue::new(value).speed(0.01).range(-1. ..=1.).prefix(label));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());}
                let response=ui.add(egui::DragValue::new(&mut state.roots.soil_slope[1]).speed(0.01).range(-1. ..=1.).prefix("Inclinação Z"));root_changed|=response.drag_stopped()||(response.changed()&&!response.dragged());
            }
            let settings=state.roots_enabled.then(||state.roots.clone());
            if root_changed{send(commands,Op::SetRoots{prefix:recipe.prefix.clone(),stem:state.root_stem.clone(),settings:settings.clone(),preview:true,policy:ManualPolicy::Preserve});}
            if ui.button("Aplicar raízes").clicked(){send(commands,Op::SetRoots{prefix:recipe.prefix.clone(),stem:state.root_stem.clone(),settings,preview:false,policy:ManualPolicy::Preserve});}
            ui.collapsing("Substituir raízes e descartar ajustes",|ui|{if ui.button("Aplicar raízes descartando escultura").clicked(){send(commands,Op::SetRoots{prefix:recipe.prefix.clone(),stem:state.root_stem.clone(),settings:state.roots_enabled.then(||state.roots.clone()),preview:false,policy:ManualPolicy::Discard});}});
            if ui.button("Cancelar prévia de raízes").clicked(){state.key=None;send(commands,Op::CancelPreview);}
        });
        ui.separator();ui.label("Órgão");ui.text_edit_singleline(&mut state.filter);
        egui::ComboBox::from_id_salt("botany_organ").selected_text(service.botany.organ.as_deref().unwrap_or("Selecione um órgão")).show_ui(ui,|ui|{
            for o in recipe.source.organs.iter().filter(|o|o.id.contains(&state.filter)).take(200){
                if ui.selectable_label(service.botany.organ.as_deref()==Some(&o.id),&o.id).clicked(){send(commands,Op::SelectOrgan{prefix:recipe.prefix.clone(),organ:Some(o.id.clone())});}
            }
        });
        if let Some(id)=&service.botany.organ {
            if state.organ.as_ref()!=Some(id){state.organ=Some(id.clone());state.edit=recipe.edits.get(id).cloned().unwrap_or_default();}
            if let Some(stage)=draft.source.organs.iter().find(|o|o.id==*id).and_then(|o|o.life.as_ref()).filter(|life|life.role==crate::botany::development::Role::Frond).map(|life|life.growth_stage) {
                let mut value=stage;
                let response=ui.add(egui::Slider::new(&mut value,0.1..=1.).text("Estágio desta fronde: jovem → desenvolvida"));
                if response.changed() {
                    let source=std::sync::Arc::make_mut(&mut draft.source);source.version=source.version.max(9);
                    source.organs.iter_mut().find(|o|o.id==*id).unwrap().life.as_mut().unwrap().growth_stage=value;
                    draft.version=draft.version.max(9);
                }
                if response.drag_stopped()||(response.changed()&&!response.dragged()) {
                    send(commands,Op::Preview{recipe:Box::new(draft.clone()),policy:ManualPolicy::Preserve});
                }
            }
            ui.add(egui::Slider::new(&mut state.edit.length,0.1..=5.).text("Comprimento do órgão"));
            ui.add(egui::Slider::new(&mut state.edit.width,0.1..=5.).text("Largura do órgão"));
            ui.add(egui::Slider::new(&mut state.edit.radius,0.1..=5.).text(if recipe.source.organs.iter().any(|o|o.id==*id&&matches!(o.shape,crate::botany::Shape::Blade{..})){"Espessura da folha"}else{"Raio do órgão"}));
            ui.collapsing("Curva de crescimento do galho",|ui|{
                ui.label("Curvatura na base / ponta (rad/m), nos dois eixos locais");
                for (label,values) in [("Base",&mut state.edit.deformation.base),("Ponta",&mut state.edit.deformation.tip)] {
                    ui.horizontal(|ui|{ui.label(label);for v in values{ui.add(egui::DragValue::new(v).speed(0.02).range(-10. ..=10.));}});
                }
                ui.add(egui::Slider::new(&mut state.edit.deformation.sag,-2. ..=2.).text("Pendência pelo comprimento"));
                ui.add(egui::Slider::new(&mut state.edit.deformation.tropism,-2. ..=2.).text("Crescer para cima / baixo"));
                ui.add(egui::Slider::new(&mut state.edit.deformation.twist,-3. ..=3.).text("Torção (rad/m)"));
                ui.add(egui::Slider::new(&mut state.edit.deformation.irregularity,0. ..=1.).text("Irregularidade 3D"));
                ui.add(egui::Slider::new(&mut state.edit.deformation.frequency,0.1..=8.).text("Frequência"));
                if ui.button("Prévia da curva").clicked(){let mut r=recipe.clone();r.edits.insert(id.clone(),state.edit.clone());send(commands,Op::Preview{recipe:Box::new(r),policy:ManualPolicy::Preserve});}
            });
            ui.label("Rotação (rad) / deslocamento (m)");
            ui.horizontal(|ui|{for x in &mut state.edit.rotation{ui.add(egui::DragValue::new(x).speed(0.02));}});
            ui.horizontal(|ui|{for x in &mut state.edit.offset{ui.add(egui::DragValue::new(x).speed(0.01));}});
            if ui.button("Aplicar ajuste ao órgão").clicked(){send(commands,Op::EditOrgan{prefix:recipe.prefix.clone(),organ:id.clone(),edit:state.edit.clone()});}
            if ui.button("Podar órgão na fonte").clicked(){let mut r=recipe.clone();r.pruned.insert(id.clone());send(commands,Op::Publish{recipe:Box::new(r),policy:ManualPolicy::Preserve});}
        }
        ui.collapsing("Deformação e crescimento",|ui|{
            ui.label("Ensaio da planta; contatos por seções e chão.");
            ui.add(egui::Slider::new(&mut state.physics.age,0. ..=20.).text("Fase de expansão do ensaio"));
            ui.add(egui::Slider::new(&mut state.physics.growth_rate,0. ..=2.).text("Expansão do ensaio / s"));
            ui.horizontal(|ui|{ui.label("Vento");for x in &mut state.physics.wind{ui.add(egui::DragValue::new(x).speed(0.1).range(-30. ..=30.));}});
            ui.checkbox(&mut state.physics.use_scene_wind,"Usar campo de vento da cena");
            ui.checkbox(&mut state.physics.self_contact,"Contato entre órgãos");ui.checkbox(&mut state.physics.leaf_contact,"Incluir lâminas no contato");
            let mut action=None;
            ui.horizontal_wrapped(|ui|{
                if ui.button("Iniciar ensaio").clicked(){action=Some(dynamics::Action::Start{settings:state.physics.clone()});}
                if ui.button("120 passos").clicked(){action=Some(dynamics::Action::Step{ticks:120});}
                if ui.button("Restaurar").clicked(){action=Some(dynamics::Action::Reset);}
            });
            if let Some(session)=&service.botany.physics && service.botany.revision==Some(service.engine.revision){
                ui.label(format!("Passo {} · massa {:.3} kg",session.tick,session.mass()));
                if ui.button(if session.playing{"Pausar ensaio"}else{"Animar com a cena"}).clicked(){action=Some(dynamics::Action::Play{playing:!session.playing});}
                if ui.button("Crescer até idade escolhida").clicked(){action=Some(dynamics::Action::Grow{age:state.physics.age});}
                if let Some(id)=&service.botany.organ && ui.button("Podar durante o ensaio").clicked(){action=Some(dynamics::Action::Prune{organ:id.clone()});}
            }
            if let Some(action)=action{send(commands,Op::Physics{prefix:recipe.prefix.clone(),action});}
        });
    });
}
