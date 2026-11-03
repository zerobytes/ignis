use super::*;
#[path = "editor_particle_fire_ui.rs"]
mod fire_ui;
use crate::particles::{Document, Operation as Op, Preset};
#[derive(Default)]
pub(super) struct State {
    fire: fire_ui::State,
    revision: Option<u64>,
    draft: String,
    error: String,
    path: String,
    source: String,
    tick: u64,
}
fn send(commands: &mut Vec<Command>, op: Op) {
    commands.push(Command::Particles {
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
  ui.heading("Partículas");ui.label("Fontes reutilizáveis · sprites · materiais compartilhados");
  if state.revision!=Some(service.engine.revision){state.draft=serde_json::to_string_pretty(&service.engine.scene.particles).unwrap_or_default();state.revision=Some(service.engine.revision);state.error.clear();}
  ui.horizontal_wrapped(|ui|{for(name,preset,id)in[("Faíscas",Preset::Sparks,"sparks"),("Fumaça",Preset::Smoke,"smoke")] {
   let mut unique=id.to_owned();let mut n=1;while service.engine.scene.particles.sources.iter().any(|s|s.id==unique)||service.engine.scene.particles.instances.iter().any(|s|s.id==unique){unique=format!("{id}_{n}");n+=1;}
   if ui.button(name).clicked(){send(commands,Op::Preset{preset,id:unique});}
  }});
  fire_ui::controls(ui,service,&mut state.fire,commands);
  ui.separator();ui.label(format!("{} fontes · {} instâncias",service.engine.scene.particles.sources.len(),service.engine.scene.particles.instances.len()));
  ui.collapsing("Tempo do preview",|ui|{
   ui.horizontal(|ui|{ui.label("Tick (60 Hz)");ui.add(egui::DragValue::new(&mut state.tick).range(0..=u64::MAX));});
   ui.horizontal_wrapped(|ui|{if ui.button("Ver neste instante").clicked(){send(commands,Op::Preview{tick:Some(state.tick)});}
if ui.button("Seguir mundo").clicked(){send(commands,Op::Preview{tick:None});}
if ui.button("Reiniciar preview").clicked(){state.tick=0;send(commands,Op::Preview{tick:Some(0)});}});
   ui.label(service.engine.particle_preview.map_or_else(||format!("Mundo · tick {}",service.engine.simulation.tick),|t|format!("Preview fixo · tick {t}")));
  });
  ui.collapsing("Fonte .ignis-vfx",|ui|{
   ui.horizontal(|ui|{ui.label("ID");ui.text_edit_singleline(&mut state.source);});
   if state.path.is_empty(){state.path="assets/particles/effect.ignis-vfx".into();}
   ui.text_edit_singleline(&mut state.path);
   ui.horizontal(|ui|{if ui.button("Salvar fonte").clicked(){send(commands,Op::SaveSource{id:state.source.clone(),path:state.path.clone()});}
if ui.button("Carregar fonte").clicked(){send(commands,Op::LoadSource{path:state.path.clone()});}});
  });
  ui.separator();ui.label("Documento editável");ui.small("Emissores: taxa, burst, seed, forma, cone, vida, módulos e material. Instâncias: posição global, rotação, escala, cor e início.");
  ui.add(egui::TextEdit::multiline(&mut state.draft).code_editor().desired_rows(24).desired_width(f32::INFINITY));
  ui.horizontal(|ui|{
   if ui.add_enabled(!service.engine.playing,egui::Button::new("Aplicar")).clicked(){match serde_json::from_str::<Document>(&state.draft).map_err(|e|e.to_string()).and_then(|d|d.validate().map(|()|d).map_err(|e|e.to_string())){Ok(document)=>{state.error.clear();send(commands,Op::Replace{document:std::sync::Arc::new(document)});},Err(e)=>state.error=e}}
   if ui.button("Descartar rascunho").clicked(){state.revision=None;}
  });
  if !state.error.is_empty(){ui.colored_label(egui::Color32::LIGHT_RED,&state.error);}
  ui.small("Efeitos visuais analíticos. Colisão, fluidos e espelhos ficam para etapas futuras.");
 });
}
