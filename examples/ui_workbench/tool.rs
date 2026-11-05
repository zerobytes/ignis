use anyhow::Result;
use egui::{Color32, RichText};
use ignis::{
    editor_ui::Graphics,
    engine::Command,
    game_ui::{Capture, Edit, Event, Frame, Kind, Node, Operation, Runtime, Text, Theme, Value},
    model::{BodyKind, RoomRecipe},
    service::Service,
};
use std::{collections::BTreeMap, path::Path, time::Instant};

pub struct Tool {
    pub service: Service,
    pub runtime: Runtime,
    pub frame: Frame,
    pub capture: Capture,
    pub hits: BTreeMap<String, egui::Response>,
    pub error: String,
    pub modal: bool,
    pub updates: u32,
    pub layout_ms: f64,
    pub draw_ms: f64,
    selected: String,
    draft: Node,
    theme: Theme,
    draft_revision: u64,
    path: String,
    portuguese: bool,
}
impl Tool {
    pub fn new(root: &Path) -> Result<Self> {
        let mut scene = RoomRecipe::default().generate()?;
        scene.entities.retain(|e| e.body == Some(BodyKind::Fixed));
        let mut service = Service::new(root, scene)?;
        service.execute(
            Command::Ui {
                operation: Box::new(Operation::Replace {
                    document: Box::new(serde_json::from_str(include_str!(
                        "../ui-workbench.ignis-ui"
                    ))?),
                }),
                expected_ui_revision: Some(0),
            }
            .into(),
        )?;
        let mut runtime = Runtime::new("workbench_preview")?;
        runtime.set_value("name", Value::Text(String::new()))?;
        runtime.set_value("progress", Value::Number(0.25))?;
        runtime.set_value("status", Value::Text("01 / 04  ·  Ready to explore".into()))?;
        let d = service.game_ui.document();
        let draft = d.root.find("title").unwrap().clone();
        let theme = d.theme.clone();
        let draft_revision = service.game_ui.revision();
        Ok(Self {
            service,
            runtime,
            frame: Frame::default(),
            capture: Capture::default(),
            hits: BTreeMap::new(),
            error: String::new(),
            modal: false,
            updates: 0,
            layout_ms: 0.,
            draw_ms: 0.,
            selected: "title".into(),
            draft,
            theme,
            draft_revision,
            path: "artifacts/ui-workbench/current.ignis-ui".into(),
            portuguese: false,
        })
    }
    pub fn command(&mut self, op: Operation, expected: Option<u64>) {
        match self.service.execute(
            Command::Ui {
                operation: Box::new(op),
                expected_ui_revision: expected,
            }
            .into(),
        ) {
            Ok(_) => {
                self.error.clear();
                self.refresh();
            }
            Err(e) => self.error = e.to_string(),
        }
    }
    fn refresh(&mut self) {
        let d = self.service.game_ui.document();
        if d.root.find(&self.selected).is_none() {
            self.selected = d.root.id.clone();
        }
        self.draft = d.root.find(&self.selected).unwrap().clone();
        self.theme = d.theme.clone();
        self.draft_revision = self.service.game_ui.revision();
    }
    pub fn language(&mut self, portuguese: bool) -> Result<()> {
        self.portuguese = portuguese;
        self.runtime
            .set_locale(if portuguese { "pt-BR" } else { "en" })?;
        self.status()
    }
    fn status(&mut self) -> Result<()> {
        let text = if self.portuguese {
            "Atualizações recebidas do consumidor"
        } else {
            "Updates received from the consumer"
        };
        self.runtime.set_value(
            "status",
            Value::Text(format!("{}  ·  {text}", self.updates)),
        )?;
        Ok(())
    }
    fn header(&mut self, ctx: &egui::Context, commands: &mut Vec<(Operation, Option<u64>)>) {
        egui::TopBottomPanel::top("ui_tool_header")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(22, 27, 35))
                    .inner_margin(14.),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("IGNIS")
                            .size(21.)
                            .strong()
                            .color(Color32::from_rgb(243, 147, 89)),
                    );
                    ui.label(RichText::new("UI WORKBENCH").size(15.));
                    ui.separator();
                    let buttons = [
                        ("undo", "Desfazer", Operation::Undo),
                        ("redo", "Refazer", Operation::Redo),
                        (
                            "save",
                            "Salvar",
                            Operation::Save {
                                path: self.path.clone(),
                            },
                        ),
                        (
                            "load",
                            "Carregar",
                            Operation::Load {
                                path: self.path.clone(),
                            },
                        ),
                    ];
                    for (id, label, operation) in buttons {
                        let response = ui.button(label);
                        if response.clicked() {
                            commands.push((operation, Some(self.service.game_ui.revision())));
                        }
                        self.hits.insert(id.into(), response);
                    }
                    let label = if self.portuguese {
                        "PT-BR / EN"
                    } else {
                        "EN / PT-BR"
                    };
                    if ui.button(label).clicked() {
                        self.language(!self.portuguese).unwrap();
                    }
                });
            });
    }
    fn inspector(&mut self, ctx: &egui::Context, commands: &mut Vec<(Operation, Option<u64>)>) {
        egui::SidePanel::left("ui_inspector")
            .default_width(280.)
            .width_range(220. ..=400.)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().id_salt("inspector_scroll").show(ui, |ui| {
                    ui.add_space(10.);
                    ui.heading("Hierarquia");
                    let document = self.service.game_ui.snapshot();
                    tree(ui, &document.root, 0, &mut self.selected, &mut self.hits);
                    if self.draft.id != self.selected { self.refresh(); }
                    ui.add_space(12.);
                    ui.separator();
                    ui.heading("Propriedades");
                    ui.monospace(&self.draft.id);
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.draft.visible, "Visível");
                        ui.checkbox(&mut self.draft.enabled, "Ativo");
                    });
                    let mut mode = match self.draft.text {
                        Text::Literal(_) => 0, Text::Key(_) => 1, Text::Binding(_) => 2,
                    };
                    egui::ComboBox::from_id_salt("text_mode")
                        .selected_text(["Texto", "Tradução", "Dado"][mode])
                        .show_ui(ui, |ui| {
                            for (index, label) in ["Texto", "Tradução", "Dado"].iter().enumerate() {
                                ui.selectable_value(&mut mode, index, *label);
                            }
                        });
                    let mut text = match &self.draft.text {
                        Text::Literal(t) | Text::Key(t) | Text::Binding(t) => t.clone(),
                    };
                    self.hits.insert("property_text".into(), ui.text_edit_singleline(&mut text));
                    self.draft.text = match mode {
                        0 => Text::Literal(text), 1 => Text::Key(text), _ => Text::Binding(text),
                    };
                    if let Some(action) = &mut self.draft.action {
                        ui.label("Evento"); ui.text_edit_singleline(action);
                    }
                    if let Some(binding) = &mut self.draft.binding {
                        ui.label("Chave do dado"); ui.text_edit_singleline(binding);
                    }
                    ui.horizontal(|ui| {
                        ui.label("Padding");
                        ui.add(egui::DragValue::new(&mut self.draft.layout.padding).range(0. ..=64.));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Altura mínima");
                        ui.add(egui::DragValue::new(&mut self.draft.layout.min_height).range(0. ..=1024.));
                    });
                    let mut constrained = self.draft.layout.max_width.is_some();
                    ui.checkbox(&mut constrained, "Limitar largura");
                    if constrained {
                        ui.add(egui::DragValue::new(self.draft.layout.max_width.get_or_insert(300.)).range(24. ..=4096.));
                    } else { self.draft.layout.max_width = None; }
                    ui.collapsing("Tema do documento", |ui| { edit_theme(ui, &mut self.theme); });
                    let apply = ui.button("Aplicar propriedades");
                    if apply.clicked() {
                        commands.push((Operation::Edit { edits: vec![
                            Edit::ReplaceNode { id: self.selected.clone(), node: Box::new(self.draft.clone()) },
                            Edit::Theme { theme: self.theme.clone() },
                        ] }, Some(self.draft_revision)));
                    }
                    self.hits.insert("apply".into(), apply);
                    if ui.small_button("Restaurar rascunho").clicked() { self.refresh(); }
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("+ Texto").clicked() {
                            let mut node = Node::new(format!("label_{}", self.service.game_ui.revision()), Kind::Label);
                            node.text = Text::Literal("Novo texto".into());
                            let parent = if self.draft.kind.container() { self.draft.id.clone() } else { document.root.id.clone() };
                            let index = document.root.find(&parent).unwrap().children.len();
                            commands.push((Operation::Edit { edits: vec![Edit::Insert { parent, index, node: Box::new(node) }] }, Some(self.service.game_ui.revision())));
                        }
                        if ui.button("Remover").clicked() {
                            commands.push((Operation::Edit { edits: vec![Edit::Remove { id: self.selected.clone() }] }, Some(self.service.game_ui.revision())));
                        }
                    });
                    ui.collapsing("Organizar elemento", |ui| {
                        let mut targets = vec![];
                        containers(&document.root, &mut targets);
                        for parent in targets {
                            if parent.id == self.selected { continue; }
                            ui.horizontal(|ui| {
                                ui.label(&parent.id);
                                if ui.small_button("Mover ao início").clicked() {
                                    commands.push((Operation::Edit { edits: vec![Edit::Move { id: self.selected.clone(), parent: parent.id.clone(), index: 0 }] }, Some(self.service.game_ui.revision())));
                                }
                            });
                        }
                    });
                    ui.add_space(10.);
                    ui.label("Arquivo de UI"); ui.text_edit_singleline(&mut self.path);
                    ui.small(format!("Revisão UI {} · rascunho {} · cena {}", self.service.game_ui.revision(), self.draft_revision, self.service.engine.revision));
                    ui.small("Aplicar publica o rascunho. Selecionar outro elemento descarta mudanças locais. Tab navega; Enter ativa o foco.");
                });
            });
    }
    pub fn frame(&mut self, g: &mut Graphics, raw: egui::RawInput) -> egui::FullOutput {
        let start = Instant::now();
        let mut commands = vec![];
        self.hits.clear();
        let output = g.build_custom(raw, |ctx, texture| {
            self.header(ctx, &mut commands);
            self.inspector(ctx, &mut commands);
            egui::TopBottomPanel::bottom("ui_status").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.small(format!("UI CPU {:.2} ms · pintura + 3D {:.2} ms", self.layout_ms, self.draw_ms));
                    if self.error.is_empty() { ui.small("Prévia do mesmo runtime · dados fora do documento"); }
                    else { ui.colored_label(Color32::from_rgb(255,135,125), &self.error); }
                });
            });
            let c = self.service.game_ui.document().theme.background;
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(Color32::from_rgba_unmultiplied(c[0],c[1],c[2],c[3])))
                .show(ctx, |ui| { self.frame = self.runtime.show(ui, &self.service.game_ui, Some(texture)); });
            let was_modal = self.modal;
            if self.modal {
                let response = egui::Modal::new("workbench_modal".into()).show(ctx, |ui| {
                    ui.set_max_width(320.);
                    ui.heading("Pausa na interação");
                    ui.label("O consumidor escolhe o que fazer com este evento. A interface de fundo não deve receber cliques ou teclas.");
                    let close = ui.button("Fechar");
                    let clicked = close.clicked();
                    self.hits.insert("close_modal".into(), close);
                    clicked
                });
                if response.inner || response.should_close() { self.modal = false; }
            }
            self.capture = Capture::read(ctx, self.modal);
            if !commands.is_empty() || !self.frame.events.is_empty() || was_modal != self.modal {
                ctx.request_repaint();
            }
            self.frame.viewport
        });
        self.layout_ms = start.elapsed().as_secs_f64() * 1000.;
        for (operation, revision) in commands {
            self.command(operation, revision);
        }
        for event in self.frame.events.clone() {
            if let Event::Activated { action, .. } = event {
                match action.as_str() {
                    "increment" => {
                        self.updates += 1;
                        self.runtime
                            .set_value(
                                "progress",
                                Value::Number(((self.updates % 5) + 1) as f32 / 5.),
                            )
                            .unwrap();
                        self.status().unwrap();
                    }
                    "modal" => self.modal = true,
                    _ => {}
                }
            }
        }
        if self.modal {
            self.capture = Capture {
                pointer: true,
                keyboard: true,
            };
        }
        g.update_custom_scene(&self.service);
        output
    }
}
fn tree(
    ui: &mut egui::Ui,
    node: &Node,
    depth: usize,
    selected: &mut String,
    hits: &mut BTreeMap<String, egui::Response>,
) {
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 12.);
        let response = ui.selectable_label(
            *selected == node.id,
            format!("{}  ·  {:?}", node.id, node.kind),
        );
        if response.clicked() {
            *selected = node.id.clone();
        }
        hits.insert(format!("select_{}", node.id), response);
    });
    for child in &node.children {
        tree(ui, child, depth + 1, selected, hits);
    }
}
fn containers<'a>(node: &'a Node, result: &mut Vec<&'a Node>) {
    if node.kind.container() {
        result.push(node);
    }
    for child in &node.children {
        containers(child, result);
    }
}
fn edit_theme(ui: &mut egui::Ui, theme: &mut Theme) {
    let colors = [
        ("Fundo", &mut theme.background),
        ("Superfície", &mut theme.surface),
        ("Texto", &mut theme.text),
        ("Normal", &mut theme.normal),
        ("Hover", &mut theme.hover),
        ("Pressionado", &mut theme.pressed),
        ("Desabilitado", &mut theme.disabled),
        ("Foco", &mut theme.focus),
        ("Destaque", &mut theme.accent),
    ];
    for (label, color) in colors {
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba_unmultiplied(color);
            ui.label(label);
        });
    }
    ui.add(egui::Slider::new(&mut theme.font_size, 8. ..=48.).text("Fonte"));
    ui.add(egui::Slider::new(&mut theme.spacing, 0. ..=64.).text("Espaço"));
    ui.add(egui::Slider::new(&mut theme.rounding, 0..=64).text("Cantos"));
}
