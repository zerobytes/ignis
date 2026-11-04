//! Native controls edit the same persistent Surface accepted through Botany commands.
use super::*;
pub enum Action {
    Preview,
    Apply,
    Cancel,
}
fn number(ui: &mut egui::Ui, label: &str, value: &mut f64, lo: f64, hi: f64) {
    ui.add(
        egui::DragValue::new(value)
            .speed(0.01)
            .range(lo..=hi)
            .prefix(label),
    );
}
fn fresh<'a>(prefix: &str, ids: impl Iterator<Item = &'a str>) -> String {
    let used: BTreeSet<_> = ids.collect();
    (0..1000)
        .map(|i| format!("{prefix}{i}"))
        .find(|s| !used.contains(s.as_str()))
        .unwrap()
}
pub fn editor(ui: &mut egui::Ui, edit: &mut OrganEdit, radius: f64, tip: f64) -> Option<Action> {
    let mut action = None;
    egui::CollapsingHeader::new("Volume de madeira · seções / cordões / vales").default_open(true).show(ui, |ui| {
        ui.label(
            "Dimensões em metros da fonte; escala da planta e raio do órgão multiplicam a forma.",
        );
        let mut enabled = edit.wood_surface.is_some();
        if ui
            .checkbox(&mut enabled, "Forma volumétrica por órgão")
            .changed()
        {
            edit.wood_surface = enabled.then(|| {
                let mut s = Surface::default();
                s.sections[0].radii.fill(radius);
                s.sections[1].radii.fill(radius * tip);
                s
            });
        }
        if let Some(s) = &mut edit.wood_surface {
            ui.add(egui::Slider::new(&mut s.sides, 12..=32).text("Amostras ao redor deste órgão"));
            ui.add(
                egui::Slider::new(&mut s.samples, 8..=128).text("Amostras ao longo deste órgão"),
            );
            ui.label(format!(
                "Tubo: até {} vértices antes de junções; orçamento global mantido",
                (s.samples as usize + 1) * s.sides as usize + 2
            ));
            let mut remove = None;
            let can_remove = s.sections.len() > 2;
            for (i, section) in s.sections.iter_mut().enumerate() {
                ui.push_id(section.id.clone(), |ui| {
                    ui.collapsing(format!("Seção {}", section.id), |ui| {
                        let minimum=section.radii.iter().copied().fold(f64::INFINITY,f64::min);
                        let maximum=section.radii.iter().copied().fold(0.,f64::max);
                        ui.label(format!("Diâmetros radiais da seção: {:.2} a {:.2} m (antes de relevo/escala)",2.*minimum,2.*maximum));
                        number(ui, "Altura normalizada ", &mut section.u, 0., 1.);
                        number(ui, "Orientação (rad) ", &mut section.angle, -100., 100.);
                        number(ui, "Centro N (m) ", &mut section.offset[0], -20., 20.);
                        number(ui, "Centro B (m) ", &mut section.offset[1], -20., 20.);
                        // Spatial radial handles and numeric fields edit identical contour samples.
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(205., 205.), egui::Sense::hover());
                        let center = rect.center();
                        let range = section.radii.iter().copied().fold(0.1, f64::max) * 1.3;
                        let range_id = ui.id().with("contour_range");
                        let range = ui.ctx().data_mut(|data| {
                            if !ui.input(|i| i.pointer.any_down())
                                || data.get_temp::<f64>(range_id).is_none()
                            {
                                data.insert_temp(range_id, range);
                            }
                            data.get_temp::<f64>(range_id).unwrap()
                        });
                        let scale = 90. / range as f32;
                        let n = section.radii.len();
                        let points: Vec<_> = section
                            .radii
                            .iter()
                            .enumerate()
                            .map(|(k, r)| {
                                let a = section.angle + std::f64::consts::TAU * k as f64 / n as f64;
                                center
                                    + egui::vec2(a.cos() as f32, -a.sin() as f32)
                                        * (*r as f32 * scale)
                            })
                            .collect();
                        ui.painter().add(egui::Shape::closed_line(
                            points.clone(),
                            egui::Stroke::new(1.0_f32, egui::Color32::LIGHT_GREEN),
                        ));
                        for (k, p) in points.iter().enumerate() {
                            let response = ui.interact(
                                egui::Rect::from_center_size(*p, egui::vec2(12., 12.)),
                                ui.id().with(k),
                                egui::Sense::drag(),
                            );
                            ui.painter()
                                .circle_filled(*p, 4., egui::Color32::LIGHT_BLUE);
                            if response.dragged()
                                && let Some(pointer) = response.interact_pointer_pos()
                            {
                                // Frozen visual scale during a frame; numerical field remains exact.
                                section.radii[k] =
                                    (pointer.distance(center) / scale).clamp(0.02, 50.) as f64;
                            }
                        }
                        for (k, r) in section.radii.iter_mut().enumerate() {
                            number(ui, &format!("Raio {k} (m) "), r, 0.02, 50.);
                        }
                        if can_remove && i > 0 && ui.button("Remover seção").clicked() {
                            remove = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = remove {
                s.sections.remove(i);
            }
            if s.sections.len() < 32 && ui.button("Inserir seção entre as duas últimas").clicked()
            {
                let n = s.sections.len();
                let mut section = s.sections[n - 2].clone();
                section.id = fresh("section", s.sections.iter().map(|s| s.id.as_str()));
                section.u = (section.u + s.sections[n - 1].u) * 0.5;
                s.sections.insert(n - 1, section);
            }
            if ui.button("Ordenar seções por altura").clicked() {
                s.sections.sort_by(|a, b| a.u.total_cmp(&b.u));
            }
            let mut remove = None;
            let mut swap = None;
            for (i, r) in s.relief.iter_mut().enumerate() {
                ui.push_id(r.id.clone(), |ui| {
                    ui.collapsing(format!("Relevo {}", r.id), |ui| {
                        ui.checkbox(&mut r.enabled, "Ativo");
                        number(ui, "Largura (m) ", &mut r.width, 0.02, 20.);
                        number(
                            ui,
                            "Altura + / profundidade − (m) ",
                            &mut r.height,
                            -10.,
                            10.,
                        );
                        number(ui, "Transição nas pontas (u) ", &mut r.end_fade, 0., 0.5);
                        for p in &mut r.path {
                            ui.horizontal(|ui| {
                                number(ui, "u ", &mut p.u, 0., 1.);
                                number(ui, "Ângulo (rad) ", &mut p.angle, -100., 100.);
                            });
                        }
                        if r.path.len() < 32
                            && ui.button("Inserir controle intermediário").clicked()
                        {
                            let n = r.path.len();
                            r.path.insert(
                                n - 1,
                                Control {
                                    u: (r.path[n - 2].u + r.path[n - 1].u) * 0.5,
                                    angle: (r.path[n - 2].angle + r.path[n - 1].angle) * 0.5,
                                },
                            );
                        }
                        if i > 0 && ui.button("Mover para cima").clicked() {
                            swap = Some(i);
                        }
                        if ui.button("Remover relevo").clicked() {
                            remove = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = remove {
                s.relief.remove(i);
            } else if let Some(i) = swap {
                s.relief.swap(i, i - 1);
            }
            if s.relief.len() < 32 && ui.button("Novo cordão / vale").clicked() {
                s.relief.push(Relief {
                    id: fresh("relief", s.relief.iter().map(|r| r.id.as_str())),
                    enabled: true,
                    path: vec![Control { u: 0., angle: 0. }, Control { u: 1., angle: 0.5 }],
                    width: 0.4,
                    height: 0.2,
                    end_fade: 0.1,
                });
            }
            if let Err(error) = s.validate() {
                ui.colored_label(egui::Color32::LIGHT_RED, error.to_string());
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Prévia do volume").clicked() {
                action = Some(Action::Preview);
            }
            if ui.button("Aplicar volume").clicked() {
                action = Some(Action::Apply);
            }
            if ui.button("Cancelar prévia do volume").clicked() {
                action = Some(Action::Cancel);
            }
        });
    });
    action
}
