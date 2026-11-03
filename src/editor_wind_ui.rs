use super::*;
use crate::wind::{Document, Operation as Op};
#[derive(Default)]
pub(super) struct State {
    revision: Option<u64>,
    draft: String,
    error: String,
    path: String,
}
fn send(commands: &mut Vec<Command>, operation: Op) {
    commands.push(Command::Wind {
        operation: Box::new(operation),
    });
}
pub(super) fn properties(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    egui::ScrollArea::vertical().show(ui,|ui| {
        ui.heading("Vento");
        ui.label("Correntes, rajadas, turbulência e vórtices · metros e segundos");
        if state.revision!=Some(service.engine.revision) {
            state.revision=Some(service.engine.revision);
            state.draft=serde_json::to_string_pretty(&service.engine.scene.wind).unwrap_or_default();
            state.error.clear();
        }
        let mut d=(*service.engine.scene.wind).clone();
        let mut changed=false;
        ui.collapsing("Corrente média (m/s)",|ui| {
            for (i,name) in ["X","Y","Z"].iter().enumerate() {
                ui.horizontal(|ui| { ui.label(*name); changed|=ui.add(egui::DragValue::new(&mut d.field.mean[i]).speed(0.1).range(-200. ..=200.)).changed(); });
            }
        });
        if changed && !service.engine.playing {send(commands,Op::Replace {document:std::sync::Arc::new(d)});}
        ui.label(format!("{} corpos · {} avaliações ponderadas/tick",service.engine.scene.wind.bodies.len(),service.engine.scene.wind.work()));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Reiniciar").clicked() {commands.push(Command::Reset);}
            if ui.button(if service.engine.playing {"Pausar"} else {"Executar"}).clicked() {
                commands.push(if service.engine.playing {Command::Pause} else {Command::Play});
            }
        });
        ui.separator();
        ui.label("Documento de vento e superfícies aerodinâmicas");
        ui.add(egui::TextEdit::multiline(&mut state.draft).code_editor().desired_rows(22).desired_width(f32::INFINITY));
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!service.engine.playing,egui::Button::new("Aplicar")).clicked() {
                match serde_json::from_str::<Document>(&state.draft).map_err(|e|e.to_string()).and_then(|d|d.validate().map(|()|d).map_err(|e|e.message)) {
                    Ok(document)=>send(commands,Op::Replace{document:std::sync::Arc::new(document)}),
                    Err(e)=>state.error=e,
                }
            }
            if ui.button("Descartar rascunho").clicked() {state.revision=None;}
        });
        if state.path.is_empty() {state.path="assets/wind/environment.ignis-wind".into();}
        ui.text_edit_singleline(&mut state.path);
        ui.horizontal(|ui| {
            if ui.button("Salvar fonte").clicked() {send(commands,Op::Save{path:state.path.clone()});}
            if ui.button("Carregar fonte").clicked() {send(commands,Op::Load{path:state.path.clone()});}
        });
        if !state.error.is_empty() {ui.colored_label(egui::Color32::LIGHT_RED,&state.error);}
        ui.small("Superfícies rígidas com força e torque. Colisão por envoltória convexa; sem simulação de esteiras do ar.");
    });
}
