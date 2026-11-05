//! Editor of the same persistent detail/history object accepted by SetWoodDetails.
use super::wood_surface::ui::Action;
use super::*;
pub fn editor(
    ui: &mut egui::Ui,
    recipe: &Recipe,
    details: &mut Option<Details>,
    branch: &mut String,
) -> (Option<Action>, Option<Cause>) {
    let mut action = None;
    let mut loss = None;
    ui.collapsing("Raízes irregulares · nós e histórico",|ui|{
  let mut enabled=details.is_some();if ui.checkbox(&mut enabled,"Ativar forma e histórico da madeira").changed(){*details=enabled.then(||Details{age_years:age(recipe),..Default::default()});}
  if let Some(d)=details {
   ui.add(egui::DragValue::new(&mut d.seed).prefix("Semente das formas "));
   if recipe.living.is_none(){ui.add(egui::DragValue::new(&mut d.age_years).range(0. ..=growth::MAX_TIME as f64/365.).prefix("Idade de avaliação (anos) "));}else{ui.label(format!("Idade canônica: {:.2} anos",age(recipe)));}
   ui.add(egui::DragValue::new(&mut d.healing_years).range(0.01..=10000.).prefix("Cicatrização autoral (anos) "));
   ui.label("Perfil artístico de crescimento, não calibração biológica nem simulação de fratura.");
   let mut roots=d.roots.is_some();if ui.checkbox(&mut roots,"Variação das raízes arredondadas").changed(){d.roots=roots.then(RootProfile::default);}
   if let Some(r)=&mut d.roots {ui.add(egui::Slider::new(&mut r.bulges,0..=16).text("Engrossamentos por raiz"));
    for (label,v,max) in [("Engrossamento / raio",&mut r.swelling,0.65),("Achatamento variável",&mut r.flattening,0.45),("Desvio do eixo / raio",&mut r.bend,1.),("Relevo / raio",&mut r.grain,0.3)]{ui.add(egui::Slider::new(v,0. ..=max).text(label));}
    ui.add(egui::Slider::new(&mut r.twist,-2. ..=2.).text("Torção do volume (rad/m)"));
   }
   let mut natural=d.natural_loss.is_some();if ui.checkbox(&mut natural,"Perdas de galhos por calendário · árvores vivas").changed(){d.natural_loss=natural.then(NaturalLoss::default);}
   if let Some(n)=&mut d.natural_loss {ui.add(egui::DragValue::new(&mut n.first_year).range(0. ..=100000.).prefix("Primeira perda (anos) "));ui.add(egui::DragValue::new(&mut n.interval_years).range(1. ..=100000.).prefix("Intervalo médio (anos) "));ui.add(egui::Slider::new(&mut n.maximum_events,0..=32).text("Eventos máximos"));ui.label("Somente datas futuras ao avanço; não reconstrói retrospectivamente galhos já desaparecidos.");}
   if ui.button("Adicionar nó histórico editável").clicked() && d.scars.len()<64
    && let Some(parent)=recipe.source.organs.iter().find(|o|o.parent.is_none()) {
     let id=(0..1000).map(|i|format!("authored{i}")).find(|id|!d.scars.iter().any(|s|s.id==*id)).unwrap();
     d.scars.push(Scar{visible:true,removed_organs:vec![],id,parent:parent.id.clone(),branch:None,distance_m:2.,angle:0.,radius_m:0.6,raised_m:0.24,depth_m:0.1,year:(age(recipe)-5.).max(0.),closing_years:d.healing_years,cause:Cause::Historical});
   }
   for s in &mut d.scars {ui.collapsing(format!("Nó {} · {:?}",s.id,s.cause),|ui|{
    ui.checkbox(&mut s.visible,"Mostrar nó · conserva o evento e a perda do galho");
    ui.label(format!("Suporte: {} · galho: {}",s.parent,s.branch.as_deref().unwrap_or("marca autoral")));
    if s.branch.is_none(){ui.text_edit_singleline(&mut s.parent);ui.add(egui::DragValue::new(&mut s.distance_m).range(0. ..=10000.).speed(0.05).prefix("Distância pela madeira (m) "));ui.add(egui::DragValue::new(&mut s.year).range(0. ..=100000.).prefix("Ano do evento "));}
    ui.add(egui::Slider::new(&mut s.angle,-std::f64::consts::PI..=std::f64::consts::PI).text("Ângulo local (rad)"));
    for (label,v) in [("Raio da marca (m) ",&mut s.radius_m),("Borda elevada (m) ",&mut s.raised_m),("Recesso inicial (m) ",&mut s.depth_m),("Cicatrização (anos) ",&mut s.closing_years)]{ui.add(egui::DragValue::new(v).range(0.01..=10000.).speed(0.02).prefix(label));}
   });}
  }
  ui.horizontal(|ui|{if ui.button("Prévia da madeira").clicked(){action=Some(Action::Preview);}
if ui.button("Aplicar madeira e histórico").clicked(){action=Some(Action::Apply);}
if ui.button("Cancelar prévia da madeira").clicked(){action=Some(Action::Cancel);}});
  ui.separator();ui.label("Perda persistente: ID do galho lenhoso");let candidates:Vec<_>=recipe.source.organs.iter().filter(|o|o.parent.as_ref().is_some_and(|id|recipe.source.organs.iter().any(|p|p.id==*id&&p.parent.is_none()))&&matches!(o.shape,Shape::Axis{..})&&roots::system_for(&recipe.source,&o.id).is_none()&&!lost(recipe,&o.id)).collect();
  if branch.is_empty() && let Some(o)=candidates.iter().find(|o|recipe.living.as_ref().is_none_or(|p|p.production.sites.contains_key(&o.id))){*branch=o.id.clone();}
  egui::ComboBox::from_id_salt("wood_loss_branch").selected_text(branch.as_str()).show_ui(ui,|ui|{for o in &candidates{ui.selectable_value(branch,o.id.clone(),o.id.as_str());}});
  ui.text_edit_singleline(branch);
  if ui.button("Registrar galho quebrado").clicked(){loss=Some(Cause::Breakage);}
if ui.button("Podar e registrar cicatriz").clicked(){loss=Some(Cause::Pruning);}
 });
    (action, loss)
}
