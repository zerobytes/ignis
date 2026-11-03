//! Project dialogs and library use shared commands.
use super::*;
#[path = "editor_file_browser_ui.rs"]
mod browser_ui;
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Dialog {
    Create,
    Open,
    Scene,
}

#[derive(Default)]
pub(super) struct State {
    dialog: Option<Dialog>,
    pub(super) pending: Option<Command>,
    name: String,
    browser: browser_ui::Browser,
    opening: bool,
    error: String,
    pub(super) dismissed: bool,
}
impl State {
    pub(super) fn modal(&self) -> bool {
        self.dialog.is_some() || self.pending.is_some()
    }
}
pub(super) fn open(state: &mut State, dialog: Dialog) {
    state.dismissed = true;
    state.dialog = Some(dialog);
    state.name = if dialog == Dialog::Create {
        "Meu jogo"
    } else {
        "Nova cena"
    }
    .into();
    state.error.clear();
    state.opening = false;
    if dialog != Dialog::Scene {
        state.browser.open(dialog == Dialog::Open);
    }
}
pub(super) fn command_result(state: &mut State, data: &serde_json::Value) {
    if data.get("found").is_some() {
        state.browser.discovered(data);
    }
    if data.get("library").is_some() {
        state.error.clear();
    }
    if let Some(path) = data["created_folder"].as_str() {
        state.browser.created(path);
    }
    if state.opening && data.get("project").is_some() && data.get("root").is_some() {
        state.opening = false;
        state.dialog = None;
        state.error.clear();
    }
}
pub(super) fn error(state: &mut State, error: &str) {
    if state.browser.creating {
        state.browser.failed(error.into());
    }
    if state.opening {
        state.opening = false;
        state.error = error.into();
    }
    if state.modal() {
        state.error = error.into();
    }
}
fn begin_open(service: &Service, state: &mut State, path: String, commands: &mut Vec<Command>) {
    if state.dialog.is_none() {
        open(state, Dialog::Open);
        state.browser.select_project(path.clone());
    }
    state.opening = true;
    request(
        service,
        state,
        Command::OpenProject {
            path,
            discard_changes: false,
        },
        commands,
    );
}
pub(super) fn request(
    service: &Service,
    state: &mut State,
    command: Command,
    commands: &mut Vec<Command>,
) {
    if service.scene_dirty() {
        state.pending = Some(command);
    } else {
        commands.push(command);
    }
}
fn discard(command: &mut Command) {
    match command {
        Command::CreateProject {
            discard_changes, ..
        }
        | Command::OpenProject {
            discard_changes, ..
        }
        | Command::NewProjectScene {
            discard_changes, ..
        }
        | Command::OpenProjectScene {
            discard_changes, ..
        } => *discard_changes = true,
        _ => {}
    }
}
pub(super) fn context(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    let caption = service.project.as_ref().map_or("Projeto…".to_owned(), |p| {
        design_ui::short_label(&p.name, 20)
    });
    ui.menu_button(caption, |ui| {
        if let Some(project) = &service.project {
            ui.strong(&project.name);
            ui.weak("Projeto aberto");
            ui.separator();
            if ui.button("Cenas do jogo").clicked() {
                super::open_panel(service, commands, Panel::Project);
                ui.close();
            }
            if ui.button("Nova cena…").clicked() {
                open(state, Dialog::Scene);
                ui.close();
            }
            if let Some(path) = &service.scene_path
                && ui
                    .add_enabled(service.scene_dirty(), egui::Button::new("Salvar cena"))
                    .clicked()
            {
                commands.push(Command::SaveScene { path: path.clone() });
                ui.close();
            }
            ui.separator();
        } else {
            ui.weak("Organize as cenas em um projeto.");
            ui.separator();
        }
        if ui.button("Criar projeto…").clicked() {
            open(state, Dialog::Create);
            ui.close();
        }
        if ui.button("Abrir projeto…").clicked() {
            open(state, Dialog::Open);
            ui.close();
        }
        if !service.project_library.data.projects.is_empty() {
            ui.separator();
            ui.weak("Recentes e fixados");
            if let Some(path) = browser_ui::recent_rows(ui, service, commands, 6) {
                begin_open(service, state, path, commands);
                ui.close();
            }
        }
    });
}

pub(super) fn panel(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    let Some(project) = &service.project else {
        ui.label("Nenhum projeto aberto.");
        if ui
            .add_sized(
                [ui.available_width(), 28.],
                design_ui::primary("Criar projeto…"),
            )
            .clicked()
        {
            open(state, Dialog::Create);
        }
        if design_ui::action(ui, "Abrir projeto…").clicked() {
            open(state, Dialog::Open);
        }
        ui.add_space(12.);
        ui.strong("Seus projetos");
        if let Some(path) = browser_ui::recent_rows(ui, service, commands, 8) {
            begin_open(service, state, path, commands);
        }
        return;
    };
    ui.strong(&project.name);
    ui.small("Cenas do jogo");
    ui.add_space(8.);
    if design_ui::action(ui, "+ Nova cena").clicked() {
        open(state, Dialog::Scene);
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        for path in &project.scenes {
            let selected = service.scene_path.as_ref() == Some(path);
            let label = path
                .strip_prefix("scenes/")
                .unwrap_or(path)
                .trim_end_matches(".scene.json");
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        selected,
                        format!(
                            "{}{}",
                            if project.entry_scene == *path {
                                "▶ "
                            } else {
                                ""
                            },
                            label
                        ),
                    )
                    .clicked()
                    && !selected
                {
                    request(
                        service,
                        state,
                        Command::OpenProjectScene {
                            path: path.clone(),
                            discard_changes: false,
                        },
                        commands,
                    );
                }
            });
        }
    });
    if let Some(path) = &service.scene_path
        && project.entry_scene != *path
        && ui.button("Usar como cena inicial do jogo").clicked()
    {
        commands.push(Command::SetProjectEntry { path: path.clone() });
    }
    ui.separator();
    ui.weak("Área de trabalho organiza os painéis. Projeto contém o jogo.");
}
pub(super) fn draw(
    ctx: &egui::Context,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    if !state.dismissed
        && service.project.is_none()
        && service.scene_path.is_none()
        && !service.scene_dirty()
    {
        egui::Window::new("Comece seu jogo")
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Ignis");
                ui.label("Crie um projeto para organizar suas cenas e começar a construir.");
                ui.add_space(10.);
                if ui
                    .add_sized(
                        [ui.available_width(), 28.],
                        design_ui::primary("Criar projeto…"),
                    )
                    .clicked()
                {
                    open(state, Dialog::Create);
                }
                if design_ui::action(ui, "Abrir projeto…").clicked() {
                    open(state, Dialog::Open);
                }
                if !service.project_library.data.projects.is_empty() {
                    ui.separator();
                    ui.strong("Projetos recentes");
                    if let Some(path) = browser_ui::recent_rows(ui, service, commands, 5) {
                        begin_open(service, state, path, commands);
                    }
                }
                ui.separator();
                ui.weak("Áreas de trabalho só organizam os painéis.");
                if ui.small_button("Explorar cena temporária").clicked() {
                    state.dismissed = true;
                }
            });
    }
    if let Some(dialog) = state.dialog
        && state.pending.is_none()
    {
        egui::Modal::new(egui::Id::new("project_file_dialog")).show(ctx, |ui| {
            let width = if dialog == Dialog::Scene {
                400.
            } else {
                (ctx.content_rect().width() - 52.).clamp(540., 1000.)
            };
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.heading(match dialog {
                    Dialog::Create => "Novo projeto",
                    Dialog::Open => "Abrir projeto",
                    Dialog::Scene => "Nova cena",
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("×").on_hover_text("Fechar").clicked() {
                        state.dialog = None;
                    }
                });
            });
            if dialog != Dialog::Open {
                design_ui::field(
                    ui,
                    if dialog == Dialog::Create {
                        "Nome do jogo"
                    } else {
                        "Nome da cena"
                    },
                    |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.name)
                                .char_limit(80)
                                .desired_width(ui.available_width()),
                        );
                    },
                );
            }
            ui.add_space(4.);
            if dialog != Dialog::Scene {
                if let Some(path) = browser_ui::draw(
                    ui,
                    service,
                    &mut state.browser,
                    commands,
                    dialog == Dialog::Open,
                ) {
                    begin_open(service, state, path, commands);
                }
                if dialog == Dialog::Create {
                    ui.add(
                        egui::Label::new(format!(
                            "Novo projeto: {}",
                            state.browser.destination().join(&state.name).display()
                        ))
                        .truncate(),
                    );
                }
            }
            if !state.error.is_empty() {
                ui.add(
                    egui::Label::new(egui::RichText::new(&state.error).color(Color32::LIGHT_RED))
                        .truncate(),
                )
                .on_hover_text(&state.error);
            }
            ui.add_space(6.);
            ui.horizontal(|ui| {
                let enabled = !state.opening
                    && match dialog {
                        Dialog::Open => state.browser.target().is_some(),
                        Dialog::Create => {
                            state.browser.valid
                                && crate::project::validate_name(&state.name).is_ok()
                        }
                        Dialog::Scene => crate::project::validate_name(&state.name).is_ok(),
                    };
                if ui
                    .add_enabled(
                        enabled,
                        design_ui::primary(match dialog {
                            Dialog::Open => "Abrir projeto",
                            Dialog::Create => "Criar projeto",
                            Dialog::Scene => "Criar cena",
                        }),
                    )
                    .clicked()
                {
                    state.opening = true;
                    let command = match dialog {
                        Dialog::Create => Command::CreateProject {
                            path: state
                                .browser
                                .destination()
                                .join(&state.name)
                                .to_string_lossy()
                                .into_owned(),
                            name: state.name.clone(),
                            discard_changes: false,
                        },
                        Dialog::Open => Command::OpenProject {
                            path: state.browser.target().unwrap(),
                            discard_changes: false,
                        },
                        Dialog::Scene => Command::NewProjectScene {
                            name: state.name.clone(),
                            discard_changes: false,
                        },
                    };
                    request(service, state, command, commands);
                }
                if ui.button("Cancelar").clicked() {
                    state.dialog = None;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(if dialog == Dialog::Open {
                        "Duplo clique para abrir · Enter para entrar"
                    } else {
                        "O destino é mostrado antes de criar"
                    });
                });
            });
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
                && !state.browser.dismiss_transient()
            {
                state.dialog = None;
            }
        });
    }
    if state.pending.is_some() {
        egui::Modal::new(egui::Id::new("project_unsaved_dialog")).show(ctx, |ui| {
            ui.heading("Alterações não salvas");
            ui.label("A cena atual tem alterações. Como deseja continuar?");
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        service.scene_path.is_some(),
                        egui::Button::new("Salvar e continuar"),
                    )
                    .clicked()
                {
                    commands.push(Command::SaveScene {
                        path: service.scene_path.clone().unwrap(),
                    });
                    commands.push(state.pending.take().unwrap());
                }
                if ui.button("Descartar e continuar").clicked() {
                    let mut command = state.pending.take().unwrap();
                    discard(&mut command);
                    commands.push(command);
                }
                if ui.button("Cancelar").clicked() {
                    state.pending = None;
                    state.opening = false;
                }
            });
            if service.scene_path.is_none() {
                ui.weak("Salve a cena temporária pelo menu Arquivo para mantê-la.");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_dialog_actions_remain_on_screen_in_wide_and_compact_layouts() {
        let temp = tempfile::tempdir().unwrap();
        let service = Service::new(
            temp.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        for (w, h) in [(1440., 900.), (1000., 800.), (900., 600.), (800., 600.)] {
            let ctx = egui::Context::default();
            let mut state = super::super::UiState::default();
            state.project.browser.folder = temp.path().to_owned();
            open(&mut state.project, Dialog::Create);
            let mut output = None;
            for _ in 0..3 {
                ctx.begin_pass(egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
                    focused: true,
                    ..Default::default()
                });
                super::super::draw_editor(
                    &ctx,
                    &service,
                    &mut state,
                    &mut vec![],
                    egui::TextureId::User(0),
                    &mut None,
                );
                output = Some(ctx.end_pass());
            }
            fn texts(shape: &egui::epaint::Shape, out: &mut Vec<(String, Rect)>) {
                match shape {
                    egui::epaint::Shape::Text(t) => out.push((
                        t.galley.text().into(),
                        Rect::from_min_size(t.pos, t.galley.size()),
                    )),
                    egui::epaint::Shape::Vec(shapes) => shapes.iter().for_each(|s| texts(s, out)),
                    _ => {}
                }
            }
            let mut labels = vec![];
            for shape in &output.unwrap().shapes {
                texts(&shape.shape, &mut labels);
            }
            for name in ["Criar projeto", "Cancelar", "Novo projeto"] {
                let (_, rect) = labels
                    .iter()
                    .find(|(t, _)| t == name)
                    .unwrap_or_else(|| panic!("Missing {name} at {w}x{h}"));
                assert!(
                    rect.top() >= 0. && rect.bottom() < h && rect.left() >= 0. && rect.right() < w,
                    "{name} out of screen at {w}x{h}: {rect:?}"
                );
            }
            assert!(
                labels
                    .iter()
                    .any(|(t, _)| t.starts_with("Pasta escolhida:"))
            );
        }
    }
    #[test]
    fn open_failure_keeps_the_browser_and_unsaved_cancel_keeps_the_document() {
        let temp = tempfile::tempdir().unwrap();
        let mut service = Service::new(
            temp.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let mut state = State::default();
        state.browser.folder = temp.path().to_owned();
        open(&mut state, Dialog::Open);
        state.opening = true;
        error(&mut state, "Projeto não encontrado");
        assert!(state.dialog == Some(Dialog::Open) && !state.opening && !state.error.is_empty());
        service
            .execute(
                Command::CreateSceneObject {
                    kind: crate::project::ObjectKind::Visual,
                }
                .into(),
            )
            .unwrap();
        let before = service.engine.snapshot();
        let mut commands = vec![];
        request(
            &service,
            &mut state,
            Command::OpenProject {
                path: temp.path().to_string_lossy().into_owned(),
                discard_changes: false,
            },
            &mut commands,
        );
        assert!(commands.is_empty() && state.pending.is_some());
        state.pending = None;
        state.opening = false;
        assert_eq!(service.engine.snapshot(), before);
    }
}
