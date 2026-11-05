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
    organ_revision: Option<u64>,
    physics: dynamics::Settings,
    root_stem: String,
    roots: crate::botany::roots::Settings,
    roots_enabled: bool,
    living_definition: crate::botany::growth::Definition,
    living_stature: crate::botany::growth::Stature,
    living_continuous: crate::botany::growth::continuous::Profile,
    living_volume: crate::botany::growth::woody_volume::Profile,
    wood_details: Option<crate::botany::wood_details::Details>,
    loss_branch: String,
    living_age_years: f64,
    living_observed: Option<(String, u64, u64, Option<u64>)>,
    living_horizon_years: f64,
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
            organ_revision: None,
            physics: Default::default(),
            root_stem: String::new(),
            roots: Default::default(),
            roots_enabled: false,
            living_definition: Default::default(),
            living_stature: Default::default(),
            living_continuous: Default::default(),
            living_volume: Default::default(),
            wood_details: None,
            loss_branch: String::new(),
            living_age_years: 0.,
            living_observed: None,
            living_horizon_years: 2.,
        }
    }
}
fn send(commands: &mut Vec<Command>, op: Op) {
    commands.push(Command::Botany {
        operation: Box::new(op),
    });
}
fn stature_controls(ui: &mut egui::Ui, stature: &mut crate::botany::growth::Stature) {
    ui.label("O diâmetro adulto define o porte proporcional desta gramática.");
    for (label, value) in [
        ("Diâmetro adulto (m) ", &mut stature.adult_diameter_m),
        (
            "Diâmetro inicial da muda (m) ",
            &mut stature.seedling_diameter_m,
        ),
        ("Idade de maturidade (anos) ", &mut stature.maturity_years),
    ] {
        ui.add(
            egui::DragValue::new(value)
                .range(f64::MIN_POSITIVE..=f64::MAX)
                .speed(0.01)
                .prefix(label),
        );
    }
}
fn continuous_controls(ui: &mut egui::Ui, p: &mut crate::botany::growth::continuous::Profile) {
    ui.label("Altura e diâmetro independentes. Referência = 95% do incremento até o limite.");
    for (label, value) in [
        ("Altura inicial (m) ", &mut p.seedling_height_m),
        ("Limite suave de altura (m) ", &mut p.height_limit_m),
        (
            "Referência de altura (anos) ",
            &mut p.height_reference_years,
        ),
        ("Diâmetro inicial (m) ", &mut p.seedling_diameter_m),
        ("Limite suave de diâmetro (m) ", &mut p.diameter_limit_m),
        (
            "Referência de diâmetro (anos) ",
            &mut p.diameter_reference_years,
        ),
        ("Distância entre nós (m) ", &mut p.internode_m),
    ] {
        ui.add(
            egui::DragValue::new(value)
                .range(f64::MIN_POSITIVE..=f64::MAX)
                .speed(0.01)
                .prefix(label),
        );
    }
    for (label, value) in [
        ("Dias por nó foliar ", &mut p.internode_days),
        ("Expansão do broto (dias) ", &mut p.shoot_expansion_days),
        ("Renovação de folhas (dias) ", &mut p.leaf_turnover_days),
    ] {
        ui.add(
            egui::DragValue::new(value)
                .range(1..=crate::botany::growth::MAX_TIME)
                .prefix(label),
        );
    }
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
            if ui.button("Sequoia-gigante realista").clicked(){
                send(commands,Op::Load{path:"assets/botany/sequoia-realistic.ignis-botany".into(),publish:true});
                for (part,name) in [("wood","bark"),("leaves","foliage")] {
                    commands.push(Command::Material{operation:Box::new(crate::materials::library::Operation::Load{path:format!("assets/materials/sequoia/{name}.ignis-material"),asset:Some(format!("sequoia_real_{part}"))})});
                }
            }
            if ui.button("Sequoia-gigante viva").clicked(){send(commands,Op::CreateLiving{definition:crate::botany::growth::sequoia::definition("sequoia".into(),83),born:0});}
            ui.text_edit_singleline(&mut state.path);
            if ui.button("Carregar fonte").clicked(){send(commands,Op::Load{path:state.path.clone(),publish:true});}
        });
        ui.collapsing("Nova planta viva · experimental",|ui|{
            let d=&mut state.living_definition;
            ui.text_edit_singleline(&mut d.prefix);
            ui.add(egui::DragValue::new(&mut d.seed).prefix("Semente "));
            egui::ComboBox::from_id_salt("growth_grammar").selected_text(match d.version {1=>"Planta de diagnóstico",2=>"Árvore viva",3=>"Árvore · porte legado",_=>"Árvore · crescimento contínuo"}).show_ui(ui,|ui|{
                ui.selectable_value(&mut d.version,1,"Planta de diagnóstico");ui.selectable_value(&mut d.version,2,"Árvore viva");ui.selectable_value(&mut d.version,3,"Árvore · porte legado");ui.selectable_value(&mut d.version,4,"Árvore · crescimento contínuo");
            });
            if d.version>=2 {
                let p=d.foliage.get_or_insert_with(Default::default);
                ui.collapsing("Folhagem desde a muda",|ui|{
                    ui.add(egui::DragValue::new(&mut p.expansion_days).range(1..=100000).prefix("Dias para expandir folhas "));
                    ui.add(egui::Slider::new(&mut p.apical_leaves,2..=64).text("Folhas no broto inicial"));
                    ui.add(egui::Slider::new(&mut p.primary_leaves,2..=64).text("Folhas por galho principal"));
                    ui.add(egui::Slider::new(&mut p.secondary_leaves,2..=32).text("Folhas por ramo secundário"));
                    ui.add(egui::DragValue::new(&mut p.length_m).range(0.02..=1.).speed(0.01).prefix("Comprimento da folha (m) "));
                    ui.add(egui::DragValue::new(&mut p.width_m).range(0.005..=p.length_m.min(0.5)).speed(0.005).prefix("Largura da folha (m) "));
                });
            }else{d.foliage=None;}
            if d.version==4 {continuous_controls(ui,d.continuous.get_or_insert_with(Default::default));}else{d.continuous=None;}
            if d.version==3 {stature_controls(ui,d.stature.get_or_insert_with(Default::default));}else {d.stature=None;}

            ui.add(egui::DragValue::new(&mut d.interval).range(1..=100000).prefix("Dias por novo ramo "));
            ui.add(egui::Slider::new(&mut d.maximum_nodes,1..=32).text("Máximo de ramos"));
            ui.add(egui::Slider::new(&mut d.internode_length,0.3..=1.).text("Distância entre ramos (m)"));
            for (label,value,minimum) in [("Período (dias) ",&mut d.schedule.period,1),("Fase (dia) ",&mut d.schedule.phase,0),("Maturidade (dias) ",&mut d.schedule.maturity,0),("Flor aberta (dias) ",&mut d.schedule.flower_duration,0),("Fruto em crescimento (dias) ",&mut d.schedule.fruit_duration,0),("Fruto maduro preso (dias) ",&mut d.schedule.ripe_duration,0)] {
                ui.add(egui::DragValue::new(value).range(minimum..=100000).prefix(label));
            }
            if ui.button("Criar planta viva").clicked(){send(commands,Op::CreateLiving{definition:d.clone(),born:0});}
        });
        let Some(recipe)=service.engine.scene.botany.iter().find(|r|Some(r.prefix.as_str())==selected)else{return;};
        let key=Some((recipe.prefix.clone(),service.engine.revision));
        if state.prefix.as_deref()!=Some(recipe.prefix.as_str()) {
            state.path=format!("artifacts/botany-native/{}.ignis-botany",recipe.prefix);
            state.prefix=Some(recipe.prefix.clone());state.organ=None;
        }
        if state.key!=key{state.draft=Some(recipe.clone());state.key=key;
            state.wood_details=recipe.wood_details.clone();
            state.living_volume=recipe.living.as_ref().and_then(|p|p.definition.wood_volume.clone()).unwrap_or_default();
            state.living_stature=recipe.living.as_ref().and_then(|p|p.definition.stature.clone()).unwrap_or_default();
            state.living_continuous=recipe.living.as_ref().and_then(|p|p.definition.continuous.clone()).unwrap_or_default();
            if !recipe.source.organs.iter().any(|o|o.id==state.root_stem&&o.parent.is_none()){state.root_stem=recipe.source.organs.iter().find(|o|o.parent.is_none()).map(|o|o.id.clone()).unwrap_or_default();}
            let system=recipe.source.root_systems.iter().find(|r|r.stem==state.root_stem);state.roots_enabled=system.is_some();state.roots=system.map(|s|s.settings.clone()).unwrap_or_default();
        }
        if recipe.living.is_none() && ui.button("Adaptar fonte adulta ao desenvolvimento vivo").clicked() {send(commands,Op::AdoptLiving{prefix:recipe.prefix.clone(),timing:Default::default(),born:0});}
        let (detail_action,loss)=crate::botany::wood_details::ui::editor(ui,recipe,&mut state.wood_details,&mut state.loss_branch);
        if let Some(action)=detail_action {use crate::botany::wood_surface::ui::Action;
            if matches!(action,Action::Cancel){state.wood_details=recipe.wood_details.clone();send(commands,Op::CancelPreview);}else{send(commands,Op::SetWoodDetails{prefix:recipe.prefix.clone(),details:state.wood_details.clone(),preview:matches!(action,Action::Preview),policy:ManualPolicy::Preserve});}
        }
        if let Some(cause)=loss {send(commands,Op::LoseBranch{prefix:recipe.prefix.clone(),organ:state.loss_branch.clone(),cause,policy:ManualPolicy::Preserve});}
        if let Some(plant)=&recipe.living {
            ui.separator();ui.heading("Desenvolvimento vivo · experimental");
            ui.label(format!("Dia {} · {} ramos produzidos",plant.production.plant.time,plant.production.sites.len()));
            ui.label("Avanço explícito; não acompanha automaticamente o relógio de gameplay.");
            let born=plant.production.plant.born;
            let current_age=plant.production.plant.time.saturating_sub(born) as f64/365.;
            let preview_time=service.botany.preview.as_ref().filter(|r|r.prefix==recipe.prefix && service.botany.revision==Some(service.engine.revision)).and_then(|r|r.living.as_ref()).map(|p|p.production.plant.time);
            let observed=(recipe.prefix.clone(),born,plant.production.plant.time,preview_time);
            if state.living_observed.as_ref()!=Some(&observed) {
                state.living_age_years=preview_time.unwrap_or(plant.production.plant.time).saturating_sub(born) as f64/365.;
                if plant.definition.version>=2 {state.living_horizon_years=state.living_horizon_years.max(if (5..=8).contains(&plant.definition.version) {3000.}else{30.});}
                state.living_observed=Some(observed);
            }
            let maximum_age=(crate::botany::growth::MAX_TIME-born) as f64/365.;
            ui.label(format!("Idade salva: {current_age:.3} anos · 1 ano = 365 dias de jogo"));
            ui.label("Arraste nos dois sentidos; atualiza a prévia ao soltar.");
            state.living_horizon_years=state.living_horizon_years.max(current_age).min(maximum_age);
            ui.add(egui::DragValue::new(&mut state.living_horizon_years).speed(0.1).range(current_age..=maximum_age).prefix("Alcance do slider (anos) "));
            let response=ui.add(egui::Slider::new(&mut state.living_age_years,0. ..=state.living_horizon_years).step_by(1./365.).fixed_decimals(3).text("Prévia de idade (anos)"));
            if response.drag_stopped() || (response.changed()&&!response.dragged()) {
                let time=born.saturating_add((state.living_age_years*365.).round() as u64).clamp(born,crate::botany::growth::MAX_TIME);
                state.living_age_years=(time-born) as f64/365.;
                if time==plant.production.plant.time {send(commands,Op::CancelPreview);}else{send(commands,Op::PreviewLivingAge{prefix:recipe.prefix.clone(),time});}
            }
            if ui.button("+10 dias na prévia").clicked() {
                let time=born.saturating_add((state.living_age_years*365.).round() as u64).saturating_add(10).min(crate::botany::growth::MAX_TIME);
                send(commands,Op::PreviewLivingAge{prefix:recipe.prefix.clone(),time});
            }
            if let Some(time)=preview_time {
                ui.label(format!("PRÉVIA: {:.3} anos · a árvore salva permanece com {current_age:.3} anos",time.saturating_sub(born) as f64/365.));
                if recipe.wood_details.is_some(){ui.label("Inclui marcas e perdas registradas até a idade da prévia; outras podas/colheitas não são reproduzidas.");}else{ui.label("Comparação de crescimento sem podas, colheitas ou escultura. Não é reprodução do histórico.");}
                if ui.button("Voltar à árvore salva").clicked(){state.living_age_years=current_age;send(commands,Op::CancelPreview);}
                if time>plant.production.plant.time && ui.button("Confirmar avanço na árvore salva").clicked(){send(commands,Op::Living{prefix:recipe.prefix.clone(),action:crate::botany::growth::Action::Advance{time}});}
                return;
            }
            if let Ok(fruits)=plant.production.attached(&plant.definition.schedule,crate::botany::growth::MAX_ATTACHED) {
                ui.label(format!("{} unidades reprodutivas ativas",fruits.len()));
                for (fruit,stage) in fruits {ui.horizontal(|ui|{
                    ui.label(format!("{} · safra {} · {:?}",fruit.site,fruit.cohort,stage));
                    if stage==crate::botany::lifecycle::Stage::RipeFruit && ui.button("Colher").clicked(){send(commands,Op::Living{prefix:recipe.prefix.clone(),action:crate::botany::growth::Action::Harvest{fruit}});}
                });}
            }
            ui.collapsing("Poda persistente",|ui|{for site in plant.production.sites.keys().filter(|s|!plant.production.removed_sites.contains(*s)){
                if ui.button(format!("Podar {site}")).clicked(){send(commands,Op::Living{prefix:recipe.prefix.clone(),action:crate::botany::growth::Action::Prune{site:site.clone()}});}
            }});
            if (6..=8).contains(&plant.definition.version) {ui.collapsing("Volumes reais do tronco",|ui|{
                let p=&mut state.living_volume;
                ui.checkbox(&mut p.continuous,"Cordões contínuos · afinar com o tronco");
                for (label,v) in [("Saliências / raio",&mut p.bulge_ratio),("Altura dos cordões / raio",&mut p.cord_height_ratio),("Profundidade dos vales / raio",&mut p.valley_depth_ratio)] {ui.add(egui::Slider::new(v,0. ..=0.45).text(label));}
                ui.add(egui::Slider::new(&mut p.bulge_count,0..=8).text("Saliências locais"));
                ui.add(egui::Slider::new(&mut p.cord_count,3..=9).text("Cordões"));
                ui.add(egui::Slider::new(&mut p.cord_width_ratio,0.15..=0.5).text("Largura / raio"));
                ui.add(egui::Slider::new(&mut p.valley_width_ratio,0.1..=0.45).text("Largura dos vales / raio"));
                ui.add(egui::Slider::new(&mut p.meander,0. ..=0.8).text("Desvio angular"));
                if !p.continuous {ui.add(egui::Slider::new(&mut p.extent,0.3..=0.7).text("Alcance no tronco"));}
                if ui.button("Aplicar volumes ao mesh").clicked(){send(commands,Op::Living{prefix:recipe.prefix.clone(),action:crate::botany::growth::Action::SetWoodVolume{profile:p.clone()}});}
            });}
            if (2..=4).contains(&plant.definition.version) {ui.collapsing("Crescimento contínuo e nós foliares",|ui|{
                continuous_controls(ui,&mut state.living_continuous);
                if ui.button("Aplicar matemática contínua").clicked(){send(commands,Op::Living{prefix:recipe.prefix.clone(),action:crate::botany::growth::Action::SetContinuous{profile:state.living_continuous.clone()}});}
            });}
            if (2..=3).contains(&plant.definition.version) {ui.collapsing("Porte e crescimento do tronco",|ui|{
                stature_controls(ui,&mut state.living_stature);
                if let Some(stature)=&plant.definition.stature {ui.label(format!("Diâmetro salvo de base: {:.3} m antes de escala manual",stature.diameter(plant.production.plant.time-born)));}
                if ui.button("Aplicar perfil de porte").clicked(){send(commands,Op::Living{prefix:recipe.prefix.clone(),action:crate::botany::growth::Action::SetStature{stature:state.living_stature.clone()}});}
            });}
            ui.text_edit_singleline(&mut state.path);
            if ui.button("Salvar fonte viva").clicked(){send(commands,Op::Save{prefix:recipe.prefix.clone(),path:state.path.clone()});}
            return;
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
        changed|=ui.add(egui::DragValue::new(&mut draft.settings.radius).range(f64::MIN_POSITIVE..=f64::MAX).speed(0.02).prefix("Multiplicador de espessura dos eixos ")).changed();
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
            if state.organ.as_ref()!=Some(id)||state.organ_revision!=Some(service.engine.revision){state.organ=Some(id.clone());state.organ_revision=Some(service.engine.revision);state.edit=recipe.edits.get(id).cloned().unwrap_or_default();}
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
            ui.add(egui::DragValue::new(&mut state.edit.radius).range(f64::MIN_POSITIVE..=f64::MAX).speed(0.02).prefix("Multiplicador de raio / espessura "));
            if let Some(crate::botany::Shape::Axis{radius,tip_ratio,..})=recipe.source.organs.iter().find(|o|o.id==*id).map(|o|&o.shape)
                && let Some(action)=crate::botany::wood_surface::ui::editor(ui,&mut state.edit,*radius,*tip_ratio) {
                use crate::botany::wood_surface::ui::Action;
                if matches!(action,Action::Cancel){state.edit=recipe.edits.get(id).cloned().unwrap_or_default();send(commands,Op::CancelPreview);}else {
                    let mut r=recipe.clone();r.version=r.version.max(if state.edit.junction_flow.is_some(){13}else{10});r.edits.insert(id.clone(),state.edit.clone());
                    send(commands,match action {Action::Preview=>Op::Preview{recipe:Box::new(r),policy:ManualPolicy::Preserve},Action::Apply=>Op::Publish{recipe:Box::new(r),policy:ManualPolicy::Preserve},Action::Cancel=>unreachable!()});
                }
            }
            if recipe.source.organs.iter().any(|o|o.id==*id && matches!(o.shape,crate::botany::Shape::Axis{..}))
                && let Some(action)=crate::botany::junction_quality::editor(ui,&mut state.edit) {
                    use crate::botany::wood_surface::ui::Action;
                    if matches!(action,Action::Cancel){state.edit=recipe.edits.get(id).cloned().unwrap_or_default();send(commands,Op::CancelPreview);}else{
                        let mut r=recipe.clone();r.version=r.version.max(if state.edit.junction_flow.is_some(){13}else{11});r.edits.insert(id.clone(),state.edit.clone());
                        send(commands,match action{Action::Preview=>Op::Preview{recipe:Box::new(r),policy:ManualPolicy::Preserve},Action::Apply=>Op::Publish{recipe:Box::new(r),policy:ManualPolicy::Preserve},Action::Cancel=>unreachable!()});
                    }
            }
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

#[cfg(test)]
mod living_slider_tests {
    use super::*;

    #[test]
    fn living_age_draft_tracks_plant_time_not_unrelated_revisions() {
        let root = tempfile::tempdir().unwrap();
        let mut scene = crate::model::RoomRecipe::default().generate().unwrap();
        scene.entities.clear();
        let mut service = Service::new(root.path(), scene).unwrap();
        service
            .execute(
                Command::Botany {
                    operation: Box::new(Op::CreateLiving {
                        definition: Default::default(),
                        born: 0,
                    }),
                }
                .into(),
            )
            .unwrap();
        service
            .execute(
                Command::Botany {
                    operation: Box::new(Op::Open {
                        open: true,
                        prefix: Some("living".into()),
                    }),
                }
                .into(),
            )
            .unwrap();
        let ctx = egui::Context::default();
        let mut state = State::default();
        let frame = |state: &mut State, service: &Service| {
            ctx.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000., 1200.),
                )),
                ..Default::default()
            });
            egui::CentralPanel::default()
                .show(&ctx, |ui| properties(ui, service, state, &mut Vec::new()));
            let _ = ctx.end_pass();
        };
        frame(&mut state, &service);
        state.living_age_years = 0.8;
        service.engine.revision += 1;
        frame(&mut state, &service);
        assert_eq!(state.living_age_years, 0.8);
        service
            .execute(
                Command::Botany {
                    operation: Box::new(Op::Living {
                        prefix: "living".into(),
                        action: crate::botany::growth::Action::Advance { time: 190 },
                    }),
                }
                .into(),
            )
            .unwrap();
        frame(&mut state, &service);
        assert_eq!(state.living_age_years, 190. / 365.);
        service.execute(Command::Undo.into()).unwrap();
        frame(&mut state, &service);
        assert_eq!(state.living_age_years, 0.);
    }
}
