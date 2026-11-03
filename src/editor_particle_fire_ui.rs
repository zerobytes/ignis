use super::*;
use crate::particles::fire::{Kind, Settings};

#[derive(Default)]
pub(super) struct State {
    id: String,
    settings: Settings,
    key: Option<(String, u64)>,
}
fn apply(service: &Service, commands: &mut Vec<Command>, id: String, settings: Settings) {
    if service.engine.playing {
        commands.push(Command::Pause);
    }
    send(commands, Op::Fire { id, settings });
    send(commands, Op::Preview { tick: None });
    commands.push(Command::Play);
}
pub(super) fn controls(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    egui::CollapsingHeader::new("Fogo · tipos e intensidade")
        .default_open(true)
        .show(ui, |ui| {
            if state.id.is_empty() {
                state.id = service
                    .engine
                    .scene
                    .particles
                    .sources
                    .iter()
                    .find(|s| s.fire.is_some())
                    .map_or_else(|| "fire".into(), |s| s.id.clone());
            }
            let recipe = service
                .engine
                .scene
                .particles
                .sources
                .iter()
                .find(|s| s.id == state.id)
                .and_then(|s| s.fire.as_ref().map(|r| r.settings));
            let key = (state.id.clone(), service.engine.revision);
            if state.key.as_ref() != Some(&key) {
                if let Some(settings) = recipe {
                    state.settings = settings;
                }
                state.key = Some(key);
            }
            ui.horizontal_wrapped(|ui| {
                for kind in Kind::ALL {
                    if ui.button(kind.label()).clicked() {
                        state.settings = Settings::for_kind(kind);
                        let base = format!("fire_{}", kind.id());
                        let mut id = base.clone();
                        let mut n = 1;
                        while service
                            .engine
                            .scene
                            .particles
                            .sources
                            .iter()
                            .any(|s| s.id == id)
                            || service
                                .engine
                                .scene
                                .particles
                                .instances
                                .iter()
                                .any(|p| p.id == id)
                        {
                            id = format!("{base}_{n}");
                            n += 1;
                        }
                        state.id = id.clone();
                        apply(service, commands, id, state.settings);
                    }
                }
            });
            egui::ComboBox::from_id_salt("fire_source")
                .selected_text("Selecionar fogo da cena")
                .show_ui(ui, |ui| {
                    for source in &service.engine.scene.particles.sources {
                        if let Some(recipe) = &source.fire
                            && ui
                                .selectable_label(state.id == source.id, &source.id)
                                .clicked()
                        {
                            state.id = source.id.clone();
                            state.settings = recipe.settings;
                            state.key = None;
                        }
                    }
                });
            ui.horizontal(|ui| {
                ui.label("Fonte");
                ui.text_edit_singleline(&mut state.id);
            });
            egui::ComboBox::from_id_salt("fire_kind")
                .selected_text(state.settings.kind.label())
                .show_ui(ui, |ui| {
                    for kind in Kind::ALL {
                        ui.selectable_value(&mut state.settings.kind, kind, kind.label());
                    }
                });
            ui.add(egui::Slider::new(&mut state.settings.intensity, 0.0..=3.0).text("Intensidade"));
            ui.add(egui::Slider::new(&mut state.settings.height, 0.1..=8.0).text("Altura (m)"));
            ui.add(egui::Slider::new(&mut state.settings.width, 0.03..=5.0).text("Largura (m)"));
            ui.add(
                egui::Slider::new(&mut state.settings.turbulence, 0.0..=2.0).text("Turbulência"),
            );
            ui.horizontal(|ui| {
                ui.label("Variação");
                ui.add(egui::DragValue::new(&mut state.settings.seed));
            });
            if ui
                .button(if recipe.is_some() {
                    "Aplicar e executar"
                } else {
                    "Criar e executar"
                })
                .clicked()
            {
                apply(service, commands, state.id.clone(), state.settings);
            }
            if recipe.is_some()
                && ui
                    .add_enabled(
                        !service.engine.playing,
                        egui::Button::new("Editar emissores manualmente"),
                    )
                    .clicked()
            {
                send(
                    commands,
                    Op::DetachFire {
                        id: state.id.clone(),
                    },
                );
            }
            ui.small("Zero apaga o efeito. Os parâmetros são salvos junto da fonte.");
        });
}
