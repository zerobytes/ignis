//! Viewport light preferences, using the shared validated Light contract.
use super::*;
use crate::model::DirectionalShadow;
use glam::Vec3;

const PANEL_OPEN: &str = "ignis-lighting-panel-open";
const PANEL_WINDOW: &str = "ignis-lighting-panel-window";

pub(super) fn controls(ui: &mut egui::Ui, service: &Service, commands: &mut Vec<Command>) {
    let button = ui
        .button("Iluminação")
        .on_hover_text("Configurar luz da cena e da viewport");
    let ctx = ui.ctx().clone();
    let state = egui::Id::new(PANEL_OPEN);
    let mut open = ctx.data(|d| d.get_temp::<bool>(state)).unwrap_or(false);
    if button.clicked() {
        open = !open;
    }
    // The color picker owns egui's single memory-popup slot. The enclosing
    // settings must not compete for that same slot or both disappear.
    egui::Window::new("Iluminação")
        .id(egui::Id::new(PANEL_WINDOW))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_pos(button.rect.left_bottom()+egui::vec2(0.,8.))
        .default_width(360.)
        .show(&ctx, |ui| {
            ui.set_width(360.);
            let height = (ui.ctx().content_rect().height() - 140.).max(200.);
            egui::ScrollArea::vertical()
                .max_height(height)
                .show(ui, |ui| {
                    ui.heading("Luz da cena");
                    ui.small("Iluminação salva com a cena e usada no jogo. Cores em RGB linear.");
                    ui.small("Ao ajustar a luz da cena, a viewport passa a usá-la.");
                    ui.label(if service.editor.work_light.is_some(){"Prévia: luz de trabalho"}else{"Prévia: luz da cena"});
                    let mut scene_light = service.engine.scene.light.clone();
                    let source_changed = source_controls(ui, &mut scene_light);
                    ui.separator();
                    ui.heading("Atmosfera da cena");
                    ui.small("Esmaecimento por distância, salvo com a cena e usado no jogo.");
                    let mut atmosphere_enabled = scene_light.atmosphere.is_some();
                    let mut atmosphere = scene_light.atmosphere.unwrap_or_default();
                    let mut atmosphere_changed = ui.checkbox(&mut atmosphere_enabled, "Ativar atmosfera").changed();
                    ui.add_enabled_ui(atmosphere_enabled, |ui| {
                        atmosphere_changed |= design_ui::field(ui, "Densidade / m", |ui| {
                            ui.add(egui::Slider::new(&mut atmosphere.density, 0.0..=1.0)
                                .logarithmic(true).smallest_positive(0.00001))
                        }).changed();
                        if atmosphere.density > 0.0 {
                            ui.small(format!("Metade do contraste a {:.1} m", std::f32::consts::LN_2 / atmosphere.density));
                        } else {
                            ui.small("Densidade zero: sem clareamento por distância.");
                        }
                        atmosphere_changed |= design_ui::field(ui, "Cor do ar", |ui| {
                            design_ui::color(ui, &mut atmosphere.color)
                        }).changed();
                        atmosphere_changed |= design_ui::field(ui, "Cor solar", |ui| {
                            design_ui::color(ui, &mut atmosphere.sun_color)
                        }).changed();
                        atmosphere_changed |= design_ui::field(ui, "Influência solar", |ui| {
                            ui.add(egui::Slider::new(&mut atmosphere.sun_strength, 0.0..=1.0))
                        }).changed();
                        ui.small("A direção e a intensidade da luz controlam a influência solar. Aproximação homogênea, sem nuvens.");
                        let mut layered = atmosphere.layers.is_some();
                        let mut layers = atmosphere.layers.unwrap_or_default();
                        let mut layer_changed = ui.checkbox(&mut layered, "Separar céu e silhuetas distantes").changed();
                        ui.add_enabled_ui(layered, |ui| {
                            layer_changed |= design_ui::field(ui,"Cor do céu",|ui|design_ui::color(ui,&mut layers.sky_color)).changed();
                            layer_changed |= design_ui::field(ui,"Cor base das montanhas",|ui|design_ui::color(ui,&mut layers.mountain_color)).changed();
                            layer_changed |= design_ui::field(ui,"Influência sobre detalhes",|ui|ui.add(egui::Slider::new(&mut layers.influence,0.0..=1.0))).changed();
                            layer_changed |= design_ui::field(ui,"Início · m",|ui|ui.add(egui::Slider::new(&mut layers.detail_start,0.0..=layers.detail_end.next_down().max(0.0)))).changed();
                            layer_changed |= design_ui::field(ui,"Final · m",|ui|ui.add(egui::Slider::new(&mut layers.detail_end,layers.detail_start.next_up()..=100_000.0).logarithmic(true))).changed();
                            ui.small("Os detalhes internos somem antes do contorno. A densidade controla o clareamento com a distância. Umidade/clima podem dirigir estes valores pelo mesmo comando.");
                        });
                        if layer_changed { atmosphere.layers = layered.then_some(layers); atmosphere_changed=true; }

                    });
                    ui.add_enabled_ui(atmosphere_enabled, |ui| {
                        let mut enabled=atmosphere.edge_softness.is_some();
                        let mut edges=atmosphere.edge_softness.unwrap_or_default();
                        let mut changed=ui.checkbox(&mut enabled,"Suavizar bordas distantes").changed();
                        ui.add_enabled_ui(enabled,|ui| {
                            changed |= design_ui::field(ui,"Intensidade",|ui|ui.add(egui::Slider::new(&mut edges.strength,0.0..=1.0))).changed();
                            changed |= design_ui::field(ui,"Raio · px físicos",|ui|ui.add(egui::Slider::new(&mut edges.radius_px,0..=12))).changed();
                            changed |= design_ui::field(ui,"Início da suavização · m",|ui|ui.add(egui::Slider::new(&mut edges.start,0.0..=edges.end.next_down().max(0.0)))).changed();
                            changed |= design_ui::field(ui,"Força máxima · m",|ui|ui.add(egui::Slider::new(&mut edges.end,edges.start.next_up()..=100000.0).logarithmic(true))).changed();
                            ui.small("Mistura as bordas com o fundo. Intensidade zero preserva a imagem nítida. Quando ativo, acrescenta dois passes e memória de imagem; não reduz triângulos.");
                        });
                        if changed {atmosphere.edge_softness=enabled.then_some(edges);atmosphere_changed=true;}
                    });
                    ui.add_enabled_ui(atmosphere_enabled, |ui| {
                        atmosphere_changed |= ui.checkbox(&mut atmosphere.reduce_details,"Usar representações compiladas").changed();
                        atmosphere_changed |= design_ui::field(ui,"Erro de contorno · px",|ui| ui.add(egui::Slider::new(&mut atmosphere.silhouette_error_px,0.0..=2.0))).changed();
                        atmosphere_changed |= design_ui::field(ui,"Erro de cor linear",|ui| ui.add(egui::Slider::new(&mut atmosphere.color_error,0.0..=0.05).fixed_decimals(3))).changed();
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Compilar malhas distantes").clicked() {
                            commands.push(Command::EditScene {edits:vec![crate::engine::SceneEdit::CompileDistantMeshes]});
                        }
                        if ui.button("Remover compilados").clicked() {
                            commands.push(Command::EditScene {edits:vec![crate::engine::SceneEdit::ClearDistantMeshes]});
                        }
                    });
                    ui.small(format!("{} malhas compiladas. Rigs, sombras e IBL mantêm a fonte neste recorte.",service.engine.scene.distant_meshes.len()));
                    if atmosphere_changed || source_changed {
                        scene_light.atmosphere = atmosphere_enabled.then_some(atmosphere);
                        commands.push(Command::SetLight { light: scene_light });
                        ui.ctx().request_repaint();
                    }
                    ui.separator();
                    ui.heading("Iluminação da viewport");
                    ui.small("Ajuste o relevo sem alterar a luz do jogo.");
                    ui.add_space(12.);
                    let preview_scene = source_changed && service.editor.work_light.is_some();
                    let mut enabled = service.editor.work_light.is_some() && !preview_scene;
                    let mut light = service
                        .editor
                        .work_light
                        .clone()
                        .unwrap_or_else(|| service.engine.scene.light.clone());
                    let mut changed = ui.checkbox(&mut enabled, "Usar luz de trabalho").changed();
                    changed |= preview_scene;
                    ui.add_space(8.);
                    let before_controls = changed;
                    ui.add_enabled_ui(enabled, |ui| {
                        let source = -Vec3::from(light.direction).normalize_or_zero();
                        let mut azimuth = source.x.atan2(source.z).to_degrees();
                        let mut elevation = source.y.clamp(-1., 1.).asin().to_degrees();
                        let a = design_ui::field(ui, "Direção", |ui| {
                            ui.add(egui::Slider::new(&mut azimuth, -180.0..=180.).suffix("°"))
                        })
                        .changed();
                        let b = design_ui::field(ui, "Altura", |ui| {
                            ui.add(egui::Slider::new(&mut elevation, -85.0..=85.).suffix("°"))
                        })
                        .changed();
                        if a || b {
                            let (az, el) = (azimuth.to_radians(), elevation.to_radians());
                            light.direction =
                                [-az.sin() * el.cos(), -el.sin(), -az.cos() * el.cos()];
                            changed = true;
                        }
                        changed |= design_ui::field(ui, "Ambiente", |ui| {
                            ui.add(egui::Slider::new(&mut light.ambient, 0.0..=1.))
                        })
                        .changed();
                        ui.small("Menos ambiente acentua as deformações.");
                        ui.add_space(12.);
                        let mut shadows = light.shadow.is_some();
                        if ui.checkbox(&mut shadows, "Sombras").changed() {
                            light.shadow = shadows.then(|| DirectionalShadow {
                                center: service
                                    .editor
                                    .current()
                                    .camera
                                    .as_ref()
                                    .unwrap_or(&service.engine.scene.camera)
                                    .target,
                                extent: 16.,
                                resolution: 1024,
                                bias: 0.002, receiver_plane_bias: false,
                            });
                            changed = true;
                        }
                        if let Some(shadow) = &mut light.shadow {
                            changed |= design_ui::field(ui, "Alcance", |ui| {
                                ui.add(
                                    egui::Slider::new(&mut shadow.extent, 0.25..=256.)
                                        .logarithmic(true)
                                        .suffix(" m"),
                                )
                            })
                            .changed();
                            design_ui::field(ui, "Resolução", |ui| {
                                egui::ComboBox::from_id_salt("shadow_resolution")
                                    .selected_text(format!("{} px", shadow.resolution))
                                    .show_ui(ui, |ui| {
                                        for res in [512, 1024, 2048] {
                                            changed |= ui
                                                .selectable_value(
                                                    &mut shadow.resolution,
                                                    res,
                                                    format!("{res} px"),
                                                )
                                                .changed();
                                        }
                                    });
                            });
                            ui.add_space(8.);
                            egui::CollapsingHeader::new("Ajustes avançados").show(ui, |ui| {
                                ui.label("Centro da sombra · m");
                                changed |= design_ui::vector(ui, &mut shadow.center, 0.1);
                                if design_ui::action(ui, "Centralizar na vista").clicked() {
                                    shadow.center = service
                                        .editor
                                        .current()
                                        .camera
                                        .as_ref()
                                        .unwrap_or(&service.engine.scene.camera)
                                        .target;
                                    changed = true;
                                }
                                changed |= design_ui::field(ui, "Bias", |ui| {
                                    ui.add(
                                        design_ui::Scrub::new(&mut shadow.bias)
                                            .range(0.0..=0.01)
                                            .speed(0.00005)
                                            .fixed_decimals(4),
                                    )
                                })
                                .changed();
                                changed |= ui.checkbox(&mut shadow.receiver_plane_bias, "Corrigir inclinação do receptor").changed();
                            });
                        }
                    });
                    // Disabled sliders may clamp a local draft; that is not an edit.
                    if !enabled { changed = before_controls; }
                    ui.add_space(12.);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Restaurar contraste").clicked() {
                            light = crate::editor::studio_light().unwrap();
                            light.shadow.as_mut().unwrap().center = service
                                .editor
                                .current()
                                .camera
                                .as_ref()
                                .unwrap_or(&service.engine.scene.camera)
                                .target;
                            enabled = true;
                            changed = true;
                        }
                        if ui.button("Luz da cena").clicked() {
                            enabled = false;
                            changed = true;
                        }
                    });
                    if changed {
                        ui.ctx().request_repaint();
                        edit(
                            commands,
                            Action::Lighting {
                                light: enabled.then_some(light),
                            },
                        );
                    }
                });
        });
    ctx.data_mut(|d| d.insert_temp(state, open));
}

#[cfg(test)]
#[path = "editor_lighting_ui_tests.rs"]
mod tests;

/// Edits the existing MCP Light value; no parallel editor-only lighting model.
fn source_controls(ui: &mut egui::Ui, light: &mut crate::model::Light) -> bool {
    let mut changed = false;
    ui.push_id("scene_light", |ui| {
        egui::CollapsingHeader::new("Direção, intensidade e ambiente").show(ui, |ui| {
            // Keep the authored vector until a direction control is edited.
            let source = -Vec3::from(light.direction).normalize_or_zero();
            let mut azimuth = source.x.atan2(source.z).to_degrees();
            let mut elevation = source.y.clamp(-1., 1.).asin().to_degrees();
            let a = design_ui::field(ui, "Direção", |ui| ui.add(egui::Slider::new(&mut azimuth, -180.0..=180.).suffix("°"))).changed();
            let b = design_ui::field(ui, "Altura", |ui| ui.add(egui::Slider::new(&mut elevation, -90.0..=90.).suffix("°"))).changed();
            if a || b {
                let (az, el) = (azimuth.to_radians(), elevation.to_radians());
                light.direction = [-az.sin()*el.cos(), -el.sin(), -az.cos()*el.cos()];
                changed = true;
            }
            changed |= design_ui::field(ui, "Intensidade direta", |ui| ui.add(egui::Slider::new(&mut light.direct_intensity, 0.0..=4.))).changed();
            changed |= design_ui::field(ui, "Cor direta (RGB linear)", |ui| design_ui::color(ui, &mut light.direct_color)).changed();
            for (channel, label) in ["R linear", "G linear", "B linear"].into_iter().enumerate() {
                changed |= design_ui::field(ui, label, |ui| ui.add(egui::DragValue::new(&mut light.direct_color[channel]).range(0.0..=1.).speed(0.005))).changed();
            }
            changed |= design_ui::field(ui, "Ambiente uniforme", |ui| ui.add(egui::Slider::new(&mut light.ambient, 0.0..=1.))).changed();
            ui.small("O ambiente uniforme ilumina também as cavidades; não representa luz indireta local.");
            let mut enabled = light.environment.is_some();
            let mut probe = light.environment.unwrap_or(crate::environment::Environment {
                sky: [0.55, 0.65, 0.78], horizon: [0.58, 0.62, 0.57],
                ground: [0.13, 0.16, 0.075], axis: [0., 1., 0.], intensity: 0.5,
            });
            let mut probe_changed = ui.checkbox(&mut enabled, "Ambiente IBL").changed();
            ui.add_enabled_ui(enabled, |ui| {
                probe_changed |= design_ui::field(ui, "Céu", |ui| design_ui::color(ui, &mut probe.sky)).changed();
                probe_changed |= design_ui::field(ui, "Horizonte", |ui| design_ui::color(ui, &mut probe.horizon)).changed();
                probe_changed |= design_ui::field(ui, "Solo", |ui| design_ui::color(ui, &mut probe.ground)).changed();
                probe_changed |= design_ui::field(ui, "Intensidade IBL", |ui| ui.add(egui::Slider::new(&mut probe.intensity, 0.0..=4.))).changed();
                let pole = Vec3::from(probe.axis).normalize_or_zero();
                let mut az = pole.x.atan2(pole.z).to_degrees();
                let mut el = pole.y.clamp(-1.,1.).asin().to_degrees();
                let a = design_ui::field(ui, "Direção do céu", |ui| ui.add(egui::Slider::new(&mut az, -180.0..=180.).suffix("°"))).changed();
                let b = design_ui::field(ui, "Altura do céu", |ui| ui.add(egui::Slider::new(&mut el, -90.0..=90.).suffix("°"))).changed();
                if a || b {
                    let (az,el) = (az.to_radians(),el.to_radians());
                    probe.axis = [az.sin()*el.cos(),el.sin(),az.cos()*el.cos()];
                    probe_changed = true;
                }
                ui.small("Probe distante: reflexão e preenchimento, sem oclusão da geometria local.");
            });
            if probe_changed { light.environment = enabled.then_some(probe); changed = true; }
        });
        egui::CollapsingHeader::new("Sombras da cena").show(ui, |ui| {
            let mut enabled = light.shadow.is_some();
            let mut shadow = light.shadow.clone().unwrap_or(DirectionalShadow {
                center: [0.; 3], extent: 16., resolution: 1024, bias: 0.002, receiver_plane_bias: false,
            });
            let mut shadow_changed = ui.checkbox(&mut enabled, "Sombras direcionais").changed();
            ui.add_enabled_ui(enabled, |ui| {
                ui.label("Centro · m");
                shadow_changed |= design_ui::vector(ui, &mut shadow.center, 0.1);
                shadow_changed |= design_ui::field(ui, "Meia largura · m", |ui| ui.add(egui::Slider::new(&mut shadow.extent, 0.25..=256.).logarithmic(true))).changed();
                design_ui::field(ui, "Resolução", |ui| {
                    egui::ComboBox::from_id_salt("resolution").selected_text(format!("{} px", shadow.resolution)).show_ui(ui, |ui| {
                        for res in [512,1024,2048] {
                            shadow_changed |= ui.selectable_value(&mut shadow.resolution, res, format!("{res} px")).changed();
                        }
                    });
                });
                shadow_changed |= design_ui::field(ui, "Bias de profundidade", |ui| ui.add(design_ui::Scrub::new(&mut shadow.bias).range(0.0..=0.01).speed(0.00001).fixed_decimals(6))).changed();
                shadow_changed |= ui.checkbox(&mut shadow.receiver_plane_bias, "Corrigir inclinação do receptor").changed();
                ui.small(format!("Texel: {:.2} mm · bias na direção da luz: {:.2} mm", 2000.*shadow.extent/shadow.resolution as f32, 1000.*shadow.bias*(4.*shadow.extent-0.01)));
                ui.small("Cobertura menor melhora o detalhe, mas objetos fora do volume deixam de receber sombra. O filtro atual é PCF 3×3.");
            });
            if shadow_changed { light.shadow = enabled.then_some(shadow); changed = true; }
        });
    });
    changed
}
