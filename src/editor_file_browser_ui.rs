//! Finder-like navigation. Only gestures refresh the filesystem cache.
use super::*;
use crate::project_browser::{self as fs, Cursor, Entry, Kind, Query, Sort};
use std::path::{Path, PathBuf};
#[derive(Default, Clone, Copy, PartialEq)]
enum View {
    #[default]
    List,
    Icons,
}
#[derive(Default)]
pub(super) struct Browser {
    pub folder: PathBuf,
    entries: Vec<Entry>,
    places: Vec<(String, PathBuf)>,
    crumbs: Vec<(String, PathBuf)>,
    search: String,
    query: String,
    search_at: Option<f64>,
    hidden: bool,
    sort: Sort,
    descending: bool,
    view: View,
    after: Option<Cursor>,
    previous: Vec<Option<Cursor>>,
    next: Option<Cursor>,
    history: Vec<PathBuf>,
    history_at: usize,
    selected: Option<String>,
    selected_project: Option<String>,
    pub valid: bool,
    pub is_project: bool,
    pub error: String,
    projects: bool,
    address: Option<String>,
    focus_address: bool,
    new_folder: Option<String>,
    focus_new: bool,
    notice: String,
    pub creating: bool,
    list_focus: Option<egui::Id>,
}
fn label(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Computador".into())
}
fn locations() -> Vec<(String, PathBuf)> {
    let mut paths = vec![];
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        paths.push(("Início".into(), home.clone()));
        for (name, folder) in [
            ("Mesa", "Desktop"),
            ("Documentos", "Documents"),
            ("Downloads", "Downloads"),
            ("Projetos no disco", "Projects"),
        ] {
            let path = home.join(folder);
            if path.is_dir() {
                paths.push((name.into(), path));
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        paths.push(("Discos e volumes".into(), PathBuf::from("/Volumes")));
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            for e in entries.flatten().take(16) {
                if e.path().is_dir() {
                    paths.push((label(&e.path()), e.path()));
                }
            }
        }
    }
    #[cfg(unix)]
    paths.push(("Computador".into(), PathBuf::from("/")));
    paths
}
impl Browser {
    pub fn open(&mut self, projects: bool) {
        self.projects = projects;
        self.error.clear();
        self.selected_project = None;
        self.new_folder = None;
        self.address = None;
        if self.folder.as_os_str().is_empty() {
            self.folder = crate::project::default_parent();
        }
        self.places = locations();
        self.navigate(self.folder.clone());
    }
    fn page(&mut self) {
        self.entries.clear();
        self.error.clear();
        self.valid = false;
        self.is_project = false;
        self.next = None;
        let query = Query {
            path: self.folder.to_string_lossy().into_owned(),
            search: self.query.clone(),
            show_hidden: self.hidden,
            sort: self.sort,
            descending: self.descending,
            after: self.after.clone(),
        };
        match fs::browse(&query) {
            Ok(page) => {
                self.folder = page.path.into();
                self.entries = page.entries;
                self.next = page.next;
                self.valid = true;
                self.is_project = page.project.is_some();
                if let Some(error) = page.project_error {
                    self.error = format!("Este projeto precisa de atenção: {error}");
                }
            }
            Err(error) => self.error = error.message,
        }
        self.crumbs = self
            .folder
            .ancestors()
            .map(|p| (label(p), p.to_owned()))
            .collect();
        self.crumbs.reverse();
    }
    fn refresh(&mut self) {
        self.query.clone_from(&self.search);
        self.after = None;
        self.previous.clear();
        self.search_at = None;
        self.selected = None;
        self.list_focus = None;
        self.page();
    }
    fn navigate(&mut self, folder: PathBuf) {
        self.folder = folder;
        self.search.clear();
        self.address = None;
        self.refresh();
        if self.history.get(self.history_at) != Some(&self.folder) {
            self.history
                .truncate((self.history_at + 1).min(self.history.len()));
            self.history.push(self.folder.clone());
            if self.history.len() > 64 {
                self.history.remove(0);
            }
            self.history_at = self.history.len() - 1;
        }
    }
    fn travel(&mut self, forward: bool) {
        let index = if forward {
            self.history_at + 1
        } else {
            self.history_at.saturating_sub(1)
        };
        if let Some(folder) = self.history.get(index).cloned() {
            self.history_at = index;
            self.folder = folder;
            self.search.clear();
            self.address = None;
            self.refresh();
        }
    }
    pub fn destination(&self) -> PathBuf {
        self.entries
            .iter()
            .find(|e| {
                Some(&e.path) == self.selected.as_ref()
                    && matches!(e.kind, Kind::Folder | Kind::Project)
            })
            .map(|e| PathBuf::from(&e.path))
            .unwrap_or_else(|| self.folder.clone())
    }
    pub fn target(&self) -> Option<String> {
        if self.projects {
            return self.selected_project.clone();
        }
        if let Some(entry) = self
            .entries
            .iter()
            .find(|e| Some(&e.path) == self.selected.as_ref())
        {
            match entry.kind {
                Kind::Project => return Some(entry.path.clone()),
                Kind::Manifest => {
                    return Path::new(&entry.path)
                        .parent()
                        .map(|p| p.to_string_lossy().into_owned());
                }
                _ => {}
            }
        }
        self.is_project
            .then(|| self.folder.to_string_lossy().into_owned())
    }
    pub fn select_project(&mut self, path: String) {
        self.projects = true;
        self.selected_project = Some(path);
    }
    pub fn discovered(&mut self, data: &serde_json::Value) {
        self.notice = format!(
            "{} projetos encontrados{}{}",
            data["found"].as_u64().unwrap_or(0),
            if data["truncated"].as_bool() == Some(true) {
                " · busca limitada; escolha uma pasta mais específica"
            } else {
                ""
            },
            if data["errors"].as_array().is_some_and(|a| !a.is_empty()) {
                " · alguns locais não puderam ser lidos"
            } else {
                ""
            }
        );
    }
    pub fn dismiss_transient(&mut self) -> bool {
        self.new_folder.take().is_some() || self.address.take().is_some()
    }
    pub fn created(&mut self, path: &str) {
        self.creating = false;
        self.new_folder = None;
        self.refresh();
        self.selected = Some(path.into());
    }
    pub fn failed(&mut self, error: String) {
        self.creating = false;
        self.error = error;
    }
}
fn age(time: Option<u64>) -> String {
    let Some(time) = time else {
        return "Ainda não aberto".into();
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let seconds = now.saturating_sub(time);
    if seconds < 60 {
        "Agora".into()
    } else if seconds < 3600 {
        format!("há {}min", seconds / 60)
    } else if seconds < 86400 {
        format!("há {}h", seconds / 3600)
    } else {
        format!("há {}dias", seconds / 86400)
    }
}
fn text_field(
    ui: &mut egui::Ui,
    text: &mut String,
    width: f32,
    id: egui::Id,
    focus: &mut bool,
    limit: usize,
) -> egui::Response {
    let mut output = egui::TextEdit::singleline(text)
        .id(id)
        .desired_width(width)
        .char_limit(limit)
        .show(ui);
    if *focus {
        output.response.request_focus();
        output
            .state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(text.chars().count()),
            )));
        output.state.store(ui.ctx(), output.response.id);
        *focus = false;
    }
    output.response
}
fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Project => "Projeto Ignis",
        Kind::Folder => "Pasta",
        Kind::Manifest => "Manifesto Ignis",
        Kind::Scene => "Cena Ignis",
        Kind::File => "Arquivo",
    }
}
fn icon(ui: &egui::Ui, rect: Rect, kind: Kind) {
    let painter = ui.painter();
    let color = if matches!(kind, Kind::Project | Kind::Manifest) {
        EMBER
    } else if kind == Kind::Folder {
        Color32::from_rgb(105, 160, 211)
    } else {
        design_ui::MUTED
    };
    let r = rect.shrink(3.);
    if matches!(kind, Kind::Folder | Kind::Project) {
        painter.rect_filled(
            Rect::from_min_size(r.min, Vec2::new(r.width() * 0.48, r.height() * 0.24)),
            2,
            color.gamma_multiply(0.7),
        );
        painter.rect_filled(
            Rect::from_min_max(r.min + Vec2::new(0., r.height() * 0.17), r.max),
            3,
            color.gamma_multiply(0.22),
        );
        painter.rect_stroke(
            Rect::from_min_max(r.min + Vec2::new(0., r.height() * 0.17), r.max),
            3,
            egui::Stroke::new(1.0_f32, color),
            egui::StrokeKind::Inside,
        );
    } else {
        painter.rect_stroke(
            r,
            2,
            egui::Stroke::new(1.0_f32, color),
            egui::StrokeKind::Inside,
        );
        painter.line_segment(
            [
                r.min + Vec2::new(4., r.height() * 0.5),
                r.max - Vec2::new(4., r.height() * 0.5),
            ],
            egui::Stroke::new(1.0_f32, color),
        );
    }
    if kind == Kind::Project {
        painter.text(
            r.center() + Vec2::new(0., 2.),
            egui::Align2::CENTER_CENTER,
            "I",
            egui::FontId::proportional(13.),
            color,
        );
    }
}
fn row(ui: &mut egui::Ui, entry: &Entry, selected: bool, details: bool) -> egui::Response {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), 34.));
    let response = ui.interact(
        rect,
        ui.id().with(("directory_entry", &entry.path)),
        egui::Sense::click(),
    );
    if selected || response.hovered() || response.has_focus() {
        ui.painter().rect_filled(
            rect,
            4,
            if selected {
                Color32::from_rgb(61, 53, 48)
            } else {
                Color32::from_rgb(43, 45, 51)
            },
        );
    }
    icon(
        ui,
        Rect::from_min_size(rect.min + Vec2::new(5., 4.), Vec2::splat(26.)),
        entry.kind,
    );
    let suffix = if details { 190. } else { 0. };
    let mut name = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((&entry.path, "label"))
            .max_rect(Rect::from_min_max(
                rect.min + Vec2::new(40., 5.),
                rect.max - Vec2::new(suffix, 5.),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    name.add(
        egui::Label::new(egui::RichText::new(&entry.name).color(
            if matches!(entry.kind, Kind::Folder | Kind::Project | Kind::Manifest) {
                design_ui::TEXT
            } else {
                design_ui::MUTED
            },
        ))
        .selectable(false)
        .truncate(),
    );
    if details {
        let painter = ui.painter();
        painter.text(
            Pos2::new(rect.right() - 176., rect.center().y),
            egui::Align2::LEFT_CENTER,
            age(entry.modified),
            egui::FontId::proportional(11.),
            design_ui::MUTED,
        );
        painter.text(
            Pos2::new(rect.right() - 82., rect.center().y),
            egui::Align2::LEFT_CENTER,
            kind_name(entry.kind),
            egui::FontId::proportional(11.),
            design_ui::MUTED,
        );
    }
    response.on_hover_text(format!("{}\n{}", entry.path, kind_name(entry.kind)))
}
pub(super) fn recent_rows(
    ui: &mut egui::Ui,
    service: &Service,
    commands: &mut Vec<Command>,
    limit: usize,
) -> Option<String> {
    let mut open = None;
    for project in service.project_library.data.projects.iter().take(limit) {
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(if project.pinned { "★" } else { "☆" }).frame(false))
                .on_hover_text(if project.pinned {
                    "Desafixar projeto"
                } else {
                    "Fixar projeto"
                })
                .clicked()
            {
                commands.push(Command::PinProject {
                    path: project.path.clone(),
                    pinned: !project.pinned,
                });
            }
            if ui
                .selectable_label(false, &project.name)
                .on_hover_text(&project.path)
                .clicked()
            {
                open = Some(project.path.clone());
            }
        });
    }
    open
}
fn projects(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut Browser,
    commands: &mut Vec<Command>,
    height: f32,
) -> Option<String> {
    let mut opened = None;
    egui::ScrollArea::vertical().id_salt("library").min_scrolled_height(height).max_height(height).show(ui,|ui|{
        let query=state.search.to_lowercase();
        let mut projects:Vec<_>=service.project_library.data.projects.iter().filter(|p|p.name.to_lowercase().contains(&query)||p.path.to_lowercase().contains(&query)).collect();
        projects.sort_by_key(|p|!p.pinned);
        if projects.is_empty(){ui.add_space(40.);ui.heading("Seus jogos começam aqui");ui.label("Os projetos criados ou abertos aparecem nesta lista. Procure uma pasta para adicionar um projeto existente.");if design_ui::action(ui,"Procurar uma pasta…").clicked(){state.projects=false;state.search.clear();state.refresh();}}
        for project in projects{
            let width=ui.available_width();
            let (rect,response)=ui.allocate_exact_size(Vec2::new(width,76.),egui::Sense::click());
            let selected=state.selected_project.as_ref()==Some(&project.path);
            ui.painter().rect_filled(rect,5,if selected{Color32::from_rgb(61,53,48)}else if response.hovered(){Color32::from_rgb(42,44,49)}else{Color32::from_rgb(33,35,40)});
            icon(ui,Rect::from_min_size(rect.min+Vec2::new(10.,15.),Vec2::splat(38.)),Kind::Project);
            let mut text=ui.new_child(egui::UiBuilder::new().id_salt((&project.path,"project_text")).max_rect(Rect::from_min_max(rect.min+Vec2::new(58.,8.),rect.max-Vec2::new(42.,6.))).layout(egui::Layout::top_down(egui::Align::Min)));
            text.style_mut().interaction.selectable_labels = false;
            text.strong(&project.name);text.add(egui::Label::new(egui::RichText::new(&project.path).small().color(design_ui::MUTED)).truncate());text.small(format!("{} cenas · {}",project.scenes,age(project.last_opened)));
            if response.clicked(){state.selected_project=Some(project.path.clone());response.request_focus();state.list_focus=Some(response.id);}
            if response.double_clicked() || (response.has_focus() && ui.input(|i|i.key_pressed(egui::Key::Enter))){opened=Some(project.path.clone());}
            response.context_menu(|ui|{if ui.button(if project.pinned{"Desafixar"}else{"Fixar projeto"}).clicked(){commands.push(Command::PinProject {path:project.path.clone(),pinned:!project.pinned});ui.close();}
            if ui.button("Remover da lista").on_hover_text("Não exclui nenhum arquivo do projeto").clicked(){commands.push(Command::ForgetProject {path:project.path.clone()});ui.close();}});
            let pin=Rect::from_center_size(Pos2::new(rect.right()-20.,rect.top()+20.),Vec2::splat(26.));
            let mut pin_ui=ui.new_child(egui::UiBuilder::new().id_salt((&project.path,"pin")).max_rect(pin));
            if pin_ui.add(egui::Button::new(if project.pinned{"★"}else{"☆"}).frame(false)).on_hover_text("Fixar/desafixar projeto").clicked(){commands.push(Command::PinProject {path:project.path.clone(),pinned:!project.pinned});}
            ui.add_space(2.);
        }
    });
    opened
}

fn activate(state: &mut Browser, entry: &Entry, opening: bool) -> Option<String> {
    match entry.kind {
        Kind::Project if opening => Some(entry.path.clone()),
        Kind::Folder | Kind::Project => {
            state.navigate(entry.path.clone().into());
            None
        }
        Kind::Manifest if opening => Path::new(&entry.path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned()),
        _ => None,
    }
}
pub(super) fn draw(
    ui: &mut egui::Ui,
    service: &Service,
    state: &mut Browser,
    commands: &mut Vec<Command>,
    opening: bool,
) -> Option<String> {
    let mut next = None;
    let mut opened = None;
    let extra = if state.new_folder.is_some() { 72. } else { 0. }
        + if state.address.is_some() { 32. } else { 0. };
    let body_height = (ui.ctx().content_rect().height() - 360. - extra).clamp(110., 420.);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                !state.projects && state.history_at > 0,
                egui::Button::new("‹"),
            )
            .on_hover_text("Voltar · Option+←")
            .clicked()
        {
            state.travel(false);
        }
        if ui
            .add_enabled(
                !state.projects && state.history_at + 1 < state.history.len(),
                egui::Button::new("›"),
            )
            .on_hover_text("Avançar · Option+→")
            .clicked()
        {
            state.travel(true);
        }
        if ui
            .add_enabled(
                !state.projects && state.folder.parent().is_some(),
                egui::Button::new("↑"),
            )
            .on_hover_text("Pasta acima · ⌘↑")
            .clicked()
        {
            next = state.folder.parent().map(Path::to_owned);
        }
        if ui.button("Atualizar").clicked() {
            if state.projects {
                commands.push(Command::DiscoverProjects {
                    paths: fs::discovery_locations(),
                });
            } else {
                state.refresh();
            }
        }
        if ui
            .add_enabled(
                !state.projects && state.valid,
                egui::Button::new("+ Nova pasta"),
            )
            .on_hover_text("⇧⌘N")
            .clicked()
        {
            state.new_folder = Some("Nova pasta".into());
            state.focus_new = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.menu_button("Opções", |ui| {
                if ui
                    .checkbox(&mut state.hidden, "Mostrar arquivos ocultos")
                    .changed()
                {
                    state.refresh();
                }
                if ui.button("Procurar projetos nesta pasta").clicked() {
                    commands.push(Command::DiscoverProjects {
                        paths: vec![state.folder.to_string_lossy().into_owned()],
                    });
                    state.projects = true;
                    state.search.clear();
                    ui.close();
                }
            });
            if !state.projects {
                ui.selectable_value(&mut state.view, View::Icons, "Ícones");
                ui.selectable_value(&mut state.view, View::List, "Lista");
            }
        });
    });
    ui.add_space(2.);
    if state.projects {
        ui.weak("Projetos · recentes e fixados");
    } else {
        egui::ScrollArea::horizontal()
            .id_salt("path_crumbs")
            .max_height(26.)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (name, path) in &state.crumbs {
                        if ui
                            .small_button(name)
                            .on_hover_text(path.display().to_string())
                            .clicked()
                        {
                            next = Some(path.clone());
                        }
                        ui.weak("›");
                    }
                    if ui.small_button("Editar caminho").clicked() {
                        state.address = Some(state.folder.to_string_lossy().into_owned());
                        state.focus_address = true;
                    }
                });
            });
    }
    let address_id = egui::Id::new("browser_address");
    if let Some(address) = &mut state.address {
        let mut go = false;
        ui.horizontal(|ui| {
            let edit = text_field(
                ui,
                address,
                ui.available_width() - 50.,
                address_id,
                &mut state.focus_address,
                2048,
            );
            go = ui.button("Ir").clicked()
                || (edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
        });
        if go {
            let value = address.clone();
            next = Some(if value == "~" {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| value.into())
            } else if let Some(relative) = value.strip_prefix("~/") {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default()
                    .join(relative)
            } else {
                value.into()
            });
        }
    }
    ui.add_space(4.);
    ui.horizontal_top(|ui| {
        let sidebar_width = 156.;
        let side_rect =
            Rect::from_min_size(ui.cursor().min, Vec2::new(sidebar_width, body_height + 34.));
        ui.painter()
            .rect_filled(side_rect, 4, Color32::from_rgb(26, 28, 32));
        ui.vertical(|ui| {
            ui.set_width(sidebar_width);
            egui::ScrollArea::vertical()
                .id_salt("browser_sidebar")
                .max_height(body_height + 34.)
                .show(ui, |ui| {
                    ui.add_space(6.);
                    if ui
                        .add_sized(
                            [sidebar_width, 30.],
                            egui::Button::new("Projetos Ignis")
                                .selected(state.projects)
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.projects = true;
                        state.search.clear();
                    }
                    ui.add_space(10.);
                    ui.weak("FAVORITOS");
                    for path in &service.project_library.data.favorites {
                        let r = ui
                            .add_sized(
                                [sidebar_width, 28.],
                                egui::Button::new(label(Path::new(path)))
                                    .selected(!state.projects && state.folder == Path::new(path))
                                    .frame(false)
                                    .truncate(),
                            )
                            .on_hover_text(path);
                        if r.clicked() {
                            state.projects = false;
                            next = Some(path.into());
                        }
                        r.context_menu(|ui| {
                            if ui.button("Remover dos favoritos").clicked() {
                                commands.push(Command::SetProjectFavorite {
                                    path: path.clone(),
                                    favorite: false,
                                });
                                ui.close();
                            }
                        });
                    }
                    if service.project_library.data.favorites.is_empty() {
                        ui.small("Fixe pastas que você usa.");
                    }
                    ui.add_space(12.);
                    ui.weak("LOCAIS");
                    for (name, path) in &state.places {
                        if ui
                            .add_sized(
                                [sidebar_width, 28.],
                                egui::Button::new(name)
                                    .selected(!state.projects && state.folder == *path)
                                    .frame(false)
                                    .truncate(),
                            )
                            .on_hover_text(path.display().to_string())
                            .clicked()
                        {
                            state.projects = false;
                            next = Some(path.clone());
                        }
                    }
                });
        });
        ui.add_space(6.);
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            let edit = ui.add(
                egui::TextEdit::singleline(&mut state.search)
                    .hint_text(if state.projects {
                        "Buscar projeto por nome…"
                    } else {
                        "Buscar nesta pasta…"
                    })
                    .desired_width(ui.available_width())
                    .char_limit(256),
            );
            if edit.changed() && !state.projects {
                state.search_at = Some(ui.input(|i| i.time));
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(260));
            }
            if !state.projects
                && ((edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || state
                        .search_at
                        .is_some_and(|at| ui.input(|i| i.time) - at >= 0.25))
            {
                state.refresh();
            }
            if state.projects {
                opened = projects(ui, service, state, commands, body_height);
            } else {
                let details = ui.available_width() > 420.;
                ui.horizontal(|ui| {
                    for (sort, text) in [
                        (Sort::Name, "Nome"),
                        (Sort::Modified, "Modificado"),
                        (Sort::Kind, "Tipo"),
                    ] {
                        let label = if state.sort == sort {
                            format!("{text} {}", if state.descending { "↓" } else { "↑" })
                        } else {
                            text.into()
                        };
                        if ui
                            .add(egui::Button::new(label).frame(false).small())
                            .clicked()
                        {
                            if state.sort == sort {
                                state.descending = !state.descending
                            } else {
                                state.sort = sort;
                                state.descending = false;
                            }
                            state.refresh();
                        }
                    }
                });
                let entries = &state.entries;
                let mut activation = None;
                egui::ScrollArea::vertical()
                    .id_salt("browser_entries")
                    .min_scrolled_height((body_height - 28.).max(80.))
                    .max_height((body_height - 28.).max(80.))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        if entries.is_empty() {
                            ui.add_space(25.);
                            ui.strong(if state.valid {
                                "Nenhum item nesta pasta"
                            } else {
                                "Não foi possível abrir esta pasta"
                            });
                            ui.weak(if state.query.is_empty() {
                                "Crie uma pasta ou escolha outro local na lateral."
                            } else {
                                "Tente outro termo de busca ou ative os arquivos ocultos."
                            });
                        }
                        if state.view == View::List {
                            for entry in entries {
                                let response = row(
                                    ui,
                                    entry,
                                    state.selected.as_ref() == Some(&entry.path),
                                    details,
                                );
                                if response.clicked() {
                                    state.selected = Some(entry.path.clone());
                                    response.request_focus();
                                    state.list_focus = Some(response.id);
                                }
                                if response.double_clicked() {
                                    activation = Some(entry.clone());
                                }
                            }
                        } else {
                            let columns = (ui.available_width() / 112.).floor().max(1.) as usize;
                            egui::Grid::new("file_icons")
                                .spacing(Vec2::new(6., 6.))
                                .show(ui, |ui| {
                                    for (i, entry) in entries.iter().enumerate() {
                                        ui.vertical(|ui| {
                                            ui.set_width(104.);
                                            let (rect, response) = ui.allocate_exact_size(
                                                Vec2::new(104., 76.),
                                                egui::Sense::click(),
                                            );
                                            if state.selected.as_ref() == Some(&entry.path)
                                                || response.hovered()
                                            {
                                                ui.painter().rect_filled(
                                                    rect,
                                                    5,
                                                    Color32::from_rgb(53, 49, 47),
                                                );
                                            }
                                            icon(
                                                ui,
                                                Rect::from_center_size(
                                                    rect.center() - Vec2::new(0., 10.),
                                                    Vec2::splat(40.),
                                                ),
                                                entry.kind,
                                            );
                                            let mut text = ui.new_child(
                                                egui::UiBuilder::new()
                                                    .id_salt((&entry.path, "icon_text"))
                                                    .max_rect(Rect::from_min_max(
                                                        rect.min + Vec2::new(4., 51.),
                                                        rect.max - Vec2::new(4., 1.),
                                                    )),
                                            );
                                            text.add(
                                                egui::Label::new(&entry.name)
                                                    .selectable(false)
                                                    .truncate(),
                                            );
                                            if response.clicked() {
                                                state.selected = Some(entry.path.clone());
                                                response.request_focus();
                                                state.list_focus = Some(response.id);
                                            }
                                            if response.double_clicked() {
                                                activation = Some(entry.clone());
                                            }
                                            response.on_hover_text(&entry.path);
                                        });
                                        if (i + 1) % columns == 0 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        }
                    });
                if let Some(focus) = state.list_focus
                    && ui.memory(|m| m.focused() == Some(focus))
                {
                    let index = entries
                        .iter()
                        .position(|e| Some(&e.path) == state.selected.as_ref())
                        .unwrap_or(0);
                    let direction = ui.input_mut(|i| {
                        if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                            1
                        } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                            -1
                        } else {
                            0
                        }
                    });
                    if direction != 0 && !entries.is_empty() {
                        let index = (index as isize + direction)
                            .clamp(0, entries.len() as isize - 1)
                            as usize;
                        state.selected = Some(entries[index].path.clone());
                    }
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                        activation = entries
                            .iter()
                            .find(|e| Some(&e.path) == state.selected.as_ref())
                            .cloned();
                    }
                }
                if let Some(entry) = activation {
                    opened = activate(state, &entry, opening);
                }
            }
        });
    });
    if !state.projects {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !state.previous.is_empty(),
                    egui::Button::new("‹ Página anterior").small(),
                )
                .clicked()
            {
                state.after = state.previous.pop().flatten();
                state.selected = None;
                state.page();
            }
            ui.weak(format!(
                "{} itens · página {}",
                state.entries.len(),
                state.previous.len() + 1
            ));
            if ui
                .add_enabled(
                    state.next.is_some(),
                    egui::Button::new("Próxima página ›").small(),
                )
                .clicked()
            {
                state.previous.push(state.after.take());
                state.after = state.next.clone();
                state.selected = None;
                state.page();
            }
            let path = state.folder.to_string_lossy().into_owned();
            let favorite = service.project_library.data.favorites.contains(&path);
            if ui
                .add_enabled(
                    state.valid,
                    egui::Button::new(if favorite {
                        "★ Pasta favorita"
                    } else {
                        "☆ Favoritar pasta"
                    })
                    .small(),
                )
                .clicked()
            {
                commands.push(Command::SetProjectFavorite {
                    path,
                    favorite: !favorite,
                });
            }
        });
    }
    if ui.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowLeft)) {
        state.projects = false;
        state.travel(false);
    }
    if ui.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowRight)) {
        state.projects = false;
        state.travel(true);
    }
    if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::ArrowUp)) {
        state.projects = false;
        next = state.folder.parent().map(Path::to_owned);
    }
    if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::L)) {
        state.projects = false;
        state.address = Some(state.folder.to_string_lossy().into_owned());
        state.focus_address = true;
    }
    if !state.projects
        && state.valid
        && ui.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::N,
            )
        })
    {
        state.new_folder = Some("Nova pasta".into());
        state.focus_new = true;
    }
    if let Some(name) = &mut state.new_folder {
        ui.separator();
        ui.strong("Nova pasta");
        let mut create = false;
        let mut cancel = false;
        ui.horizontal(|ui| {
            let edit = text_field(
                ui,
                name,
                (ui.available_width() - 200.).max(160.),
                egui::Id::new("browser_new_folder"),
                &mut state.focus_new,
                80,
            );
            create = ui
                .add_enabled(
                    !state.creating && crate::project::validate_name(name).is_ok(),
                    egui::Button::new("Criar pasta"),
                )
                .clicked()
                || (edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            cancel = ui.button("Cancelar pasta").clicked();
        });
        if create && !state.creating {
            commands.push(Command::CreateProjectFolder {
                parent: state.folder.to_string_lossy().into_owned(),
                name: name.clone(),
            });
            state.creating = true;
        }
        if cancel {
            state.new_folder = None;
        }
    }
    if let Some(folder) = next {
        state.navigate(folder);
    }
    ui.separator();
    let target = if state.projects {
        state
            .selected_project
            .clone()
            .unwrap_or_else(|| "Selecione um projeto para abrir".into())
    } else {
        format!("Pasta escolhida: {}", state.destination().display())
    };
    ui.add(
        egui::Label::new(egui::RichText::new(&target).small().color(design_ui::MUTED)).truncate(),
    )
    .on_hover_text(target);
    if !state.error.is_empty() {
        ui.add(
            egui::Label::new(egui::RichText::new(&state.error).color(Color32::LIGHT_RED))
                .truncate(),
        )
        .on_hover_text(&state.error);
    }
    if let Some(warning) = &service.project_library.warning {
        ui.add(egui::Label::new(egui::RichText::new(warning).color(EMBER)).truncate())
            .on_hover_text(warning);
    }
    if !state.notice.is_empty() {
        ui.add(
            egui::Label::new(
                egui::RichText::new(&state.notice)
                    .small()
                    .color(design_ui::MUTED),
            )
            .truncate(),
        )
        .on_hover_text(&state.notice);
    }
    opened
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Service, Browser, egui::Context) {
        let temp = tempfile::tempdir().unwrap();
        let service = Service::new(
            temp.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let mut state = Browser {
            folder: temp.path().to_owned(),
            ..Default::default()
        };
        state.open(false);
        state.places = vec![("Local do teste".into(), temp.path().to_owned())];
        (temp, service, state, egui::Context::default())
    }
    fn pass(
        ctx: &egui::Context,
        service: &Service,
        state: &mut Browser,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, Vec<Command>) {
        let time = ctx.data_mut(|d| {
            let id = egui::Id::new("test_clock");
            let n = d.get_temp::<f64>(id).unwrap_or(0.) + 0.01;
            d.insert_temp(id, n);
            n
        });
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1100., 800.))),
            events,
            focused: true,
            time: Some(time),
            ..Default::default()
        });
        let mut commands = vec![];
        egui::CentralPanel::default().show(ctx, |ui| {
            draw(ui, service, state, &mut commands, true);
        });
        (ctx.end_pass(), commands)
    }
    fn text_rect(output: &egui::FullOutput, text: &str) -> Rect {
        fn find(shape: &egui::epaint::Shape, text: &str) -> Option<Rect> {
            match shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == text => {
                    Some(Rect::from_min_size(t.pos, t.galley.size()))
                }
                egui::epaint::Shape::Vec(shapes) => shapes.iter().find_map(|s| find(s, text)),
                _ => None,
            }
        }
        output
            .shapes
            .iter()
            .find_map(|s| find(&s.shape, text))
            .unwrap_or_else(|| panic!("Missing text {text}"))
    }
    fn click(
        ctx: &egui::Context,
        service: &Service,
        state: &mut Browser,
        text: &str,
    ) -> Vec<Command> {
        pass(ctx, service, state, vec![]);
        let (out, _) = pass(ctx, service, state, vec![]);
        let pos = text_rect(&out, text).center();
        pass(
            ctx,
            service,
            state,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        );
        pass(
            ctx,
            service,
            state,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        )
        .1
    }
    #[test]
    fn single_click_selects_double_click_enters_and_history_is_real() {
        let (temp, service, mut state, ctx) = fixture();
        let child = temp.path().join("Dados");
        std::fs::create_dir(&child).unwrap();
        state.refresh();
        let root = state.folder.clone();
        click(&ctx, &service, &mut state, "Dados");
        assert_eq!(state.folder, root);
        assert_eq!(state.destination(), child.canonicalize().unwrap());
        click(&ctx, &service, &mut state, "Dados");
        assert_eq!(state.folder, child.canonicalize().unwrap());
        click(&ctx, &service, &mut state, "↑");
        assert_eq!(state.folder, root);
        click(&ctx, &service, &mut state, "‹");
        assert_eq!(state.folder, child.canonicalize().unwrap());
        click(&ctx, &service, &mut state, "›");
        assert_eq!(state.folder, root);
        click(&ctx, &service, &mut state, "Local do teste");
        assert_eq!(state.folder, root);
    }
    #[test]
    fn error_clears_stale_entries_and_project_confirmation() {
        let (temp, service, mut state, ctx) = fixture();
        std::fs::write(temp.path().join(crate::project::FILE), b"{}").unwrap();
        state.refresh();
        assert!(!state.is_project);
        assert!(state.target().is_none());
        std::fs::create_dir(temp.path().join("child")).unwrap();
        state.refresh();
        assert!(!state.entries.is_empty());
        state.navigate(temp.path().join("missing"));
        assert!(!state.valid && !state.is_project);
        assert!(state.entries.is_empty());
        assert!(state.target().is_none());
        assert!(!state.error.is_empty());
        let (_, commands) = pass(&ctx, &service, &mut state, vec![]);
        assert!(commands.is_empty());
    }
    #[test]
    fn new_folder_goes_through_the_dispatcher_and_is_selected_after_reply() {
        let (_temp, mut service, mut state, ctx) = fixture();
        click(&ctx, &service, &mut state, "+ Nova pasta");
        state.new_folder = Some("Arte".into());
        let commands = click(&ctx, &service, &mut state, "Criar pasta");
        assert_eq!(commands.len(), 1);
        assert!(state.creating);
        let before = service.engine.snapshot();
        let result = service
            .execute(commands.into_iter().next().unwrap().into())
            .unwrap()
            .data;
        let path = result["created_folder"].as_str().unwrap();
        state.created(path);
        assert!(Path::new(path).is_dir());
        assert_eq!(state.selected.as_deref(), Some(path));
        assert_eq!(service.engine.snapshot(), before);
        state.new_folder = Some("Arte".into());
        let commands = click(&ctx, &service, &mut state, "Criar pasta");
        let error = service
            .execute(commands.into_iter().next().unwrap().into())
            .err()
            .unwrap();
        state.failed(error.message);
        assert!(!state.creating && !state.error.is_empty());
    }
    #[test]
    fn search_pagination_and_parent_history_do_not_accumulate_unbounded_state() {
        let (temp, service, mut state, ctx) = fixture();
        for i in 0..270 {
            std::fs::create_dir(temp.path().join(format!("Pasta{i:03}"))).unwrap();
        }
        state.refresh();
        assert!(state.next.is_some());
        click(&ctx, &service, &mut state, "Próxima página ›");
        assert_eq!(state.entries.len(), 14);
        click(&ctx, &service, &mut state, "‹ Página anterior");
        assert_eq!(state.entries.len(), 256);
        state.search = "PASTA26".into();
        state.refresh();
        assert_eq!(state.entries.len(), 10);
        assert!(state.previous.is_empty());
        for i in 0..80 {
            let p = temp.path().join(format!("Pasta{i:03}"));
            state.navigate(p);
        }
        assert_eq!(state.history.len(), 64);
    }
}
