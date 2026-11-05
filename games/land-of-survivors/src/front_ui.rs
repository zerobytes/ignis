use crate::{
    characters::*,
    front_end::{Action, Front, Page},
};
use egui::{Color32, RichText, Vec2};
const GOLD: Color32 = Color32::from_rgb(224, 192, 124);
pub struct Ui {
    pub logo: egui::TextureHandle,
    pub hits: std::collections::BTreeMap<String, egui::Rect>,
}
impl Ui {
    pub fn new(ctx: &egui::Context) -> anyhow::Result<Self> {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.spacing.item_spacing = Vec2::new(10., 10.);
        style.spacing.button_padding = Vec2::new(16., 11.);
        style.visuals.panel_fill = Color32::from_rgb(12, 24, 24);
        style.visuals.override_text_color = Some(Color32::from_rgb(235, 232, 220));
        style.visuals.selection.bg_fill = Color32::from_rgb(53, 95, 77);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(60, 99, 77);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(47, 76, 63);
        style.visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0_f32, Color32::from_rgb(87, 86, 62));
        ctx.set_style(style);
        let mut decoder = png::Decoder::new(std::io::Cursor::new(include_bytes!(
            "../assets/ui/logo.png"
        )));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info()?;
        let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut bytes)?;
        bytes.truncate(info.buffer_size());
        anyhow::ensure!(
            info.color_type == png::ColorType::Rgba,
            "Logo must have RGBA transparency"
        );
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [info.width as usize, info.height as usize],
            &bytes,
        );
        Ok(Self {
            logo: ctx.load_texture(
                "land-of-survivors-logo",
                image,
                egui::TextureOptions::LINEAR,
            ),
            hits: Default::default(),
        })
    }
    fn button(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        text: &str,
        enabled: bool,
        primary: bool,
    ) -> bool {
        let mut b = egui::Button::new(RichText::new(text).color(if primary {
            GOLD
        } else {
            Color32::from_rgb(229, 227, 216)
        }))
        .min_size(Vec2::new(ui.available_width(), 42.));
        if primary {
            b = b
                .fill(Color32::from_rgb(33, 58, 47))
                .stroke(egui::Stroke::new(1.0_f32, GOLD));
        }
        let r = ui.add_enabled(enabled, b);
        self.hits.insert(id.into(), r.rect);
        r.clicked()
    }
    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        texture: egui::TextureId,
        f: &mut Front,
    ) -> (Option<egui::Rect>, Vec<Action>) {
        self.hits.clear();
        let mut actions = vec![];
        let pt = f.portuguese;
        let tr = |en: &str, br: &str| if pt { br.to_string() } else { en.to_string() };
        let viewport = egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                ui.painter().image(
                    texture,
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                    Color32::WHITE,
                );
                ui.painter().rect_filled(
                    rect,
                    0.,
                    Color32::from_black_alpha(if f.page == Page::Title { 140 } else { 36 }),
                );
                rect
            })
            .inner;
        egui::Area::new("language".into())
            .anchor(egui::Align2::RIGHT_TOP, [-24., 20.])
            .show(ctx, |ui| {
                if ui
                    .button(if pt { "PT-BR  /  EN" } else { "EN  /  PT-BR" })
                    .clicked()
                {
                    actions.push(Action::Language(!pt));
                }
            });
        if f.page == Page::Title {
            egui::Area::new("title".into())
                .anchor(egui::Align2::CENTER_CENTER, [0., -22.])
                .show(ctx, |ui| {
                    ui.set_width((viewport.width() * 0.58).clamp(350., 720.));
                    ui.add(egui::Image::new(&self.logo).fit_to_exact_size(Vec2::new(
                        ui.available_width(),
                        ui.available_width() / 2.52,
                    )));
                    ui.add_space(24.);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(tr("A WORLD. YOUR STORY.", "UM MUNDO. A SUA HISTÓRIA."))
                                .color(GOLD)
                                .size(14.),
                        );
                        ui.add_space(20.);
                        ui.allocate_ui_with_layout(
                            Vec2::new(280., 110.),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                if self.button(
                                    ui,
                                    "begin",
                                    &tr("CHARACTERS", "PERSONAGENS"),
                                    f.store.is_some(),
                                    true,
                                ) {
                                    actions.push(Action::Roster);
                                }
                                ui.label(tr(
                                    "Create your identity. Choose your path.",
                                    "Crie sua identidade. Escolha seu caminho.",
                                ));
                            },
                        );
                    });
                });
        } else if f.page != Page::Session {
            egui::Area::new("brand".into())
                .anchor(egui::Align2::CENTER_TOP, [80., 18.])
                .show(ctx, |ui| {
                    ui.add(egui::Image::new(&self.logo).fit_to_exact_size(Vec2::new(330., 131.)));
                });
            let panel_width = (viewport.width() * 0.28).clamp(290., 380.);
            egui::Window::new("character_panel").id(egui::Id::new("character_panel_window"))
                .title_bar(false).resizable(false).movable(false).collapsible(false)
                .fixed_rect(egui::Rect::from_min_size(egui::pos2(24.,24.),Vec2::new(panel_width+44.,viewport.height()-48.)))
                .frame(egui::Frame::new().fill(Color32::from_rgba_unmultiplied(10,23,23,242)).inner_margin(22.).corner_radius(8.).stroke(egui::Stroke::new(1.0_f32,Color32::from_rgb(99,91,60))))
                .show(ctx,|ui|{
     egui::ScrollArea::vertical().max_height((viewport.height()-98.).max(300.)).auto_shrink([false,false]).show(ui,|ui|{
      ui.heading(RichText::new(match f.page{Page::Create=>tr("CREATE CHARACTER","CRIAR PERSONAGEM"),Page::Cosmetics=>tr("APPEARANCE","APARÊNCIA"),_=>tr("YOUR CHARACTERS","SEUS PERSONAGENS")}).color(GOLD));ui.add_space(8.);
      if f.page==Page::Roster {
       ui.small(tr("Saved on this device","Salvos neste dispositivo"));
       if let Some(store)=&f.store {
        for c in &store.roster.characters {
         let selected=store.roster.selected.as_ref()==Some(&c.id);
         let label=format!("{}\n{} · {} · {} 1",c.identity.name,people(c.identity.people,pt),class(c.identity.class,pt),tr("Level","Nível"));
         let r=ui.add_sized([ui.available_width(),66.],egui::Button::selectable(selected,label));self.hits.insert(format!("character_{}",c.id),r.rect);if r.clicked(){actions.push(Action::Select(c.id.clone()));}
        }
        if store.roster.characters.is_empty(){ui.label(tr("Your story begins with a character.","Sua história começa com um personagem."));ui.add_space(15.);}
       }
       if self.button(ui,"new",&tr("+ New character","+ Novo personagem"),f.store.is_some(),true){actions.push(Action::New);}
       if f.selected().is_some(){
        if self.button(ui,"appearance",&tr("Edit appearance","Editar aparência"),true,false){actions.push(Action::Cosmetics);}
        ui.separator();
        if self.button(ui,"offline",&tr("Play offline","Jogar offline"),true,true){actions.push(Action::Offline);}
        ui.label(tr("Multiplayer session","Sessão multiplayer"));
        ui.add(egui::TextEdit::singleline(&mut f.bind).hint_text("127.0.0.1:43000").desired_width(f32::INFINITY));
        ui.small(tr("Use your LAN IP to invite another device.","Use seu IP da rede para convidar outro dispositivo."));
        if self.button(ui,"host",&tr("Host session · up to 8 players","Hospedar sessão · até 8 jogadores"),true,false){actions.push(Action::Host);}
        ui.add(egui::TextEdit::multiline(&mut f.invitation).hint_text(tr("Paste invitation","Cole o convite")).desired_rows(2).char_limit(32768).desired_width(f32::INFINITY));
        if self.button(ui,"join",&tr("Join session","Entrar na sessão"),!f.invitation.trim().is_empty(),false){actions.push(Action::Join);}
        ui.small(tr("Network participants are temporary. Server-side characters and progress are not yet saved.","Participantes de rede são temporários. Personagens e progresso ainda não são salvos no servidor."));
       }
      } else {
       let old=f.draft.clone();
       if f.page==Page::Create {
        ui.label(tr("Name","Nome"));let r=ui.add(egui::TextEdit::singleline(&mut f.draft.name).char_limit(24).desired_width(f32::INFINITY));self.hits.insert("name".into(),r.rect);
        ui.label(tr("People","Povo"));ui.horizontal(|ui|{ui.selectable_value(&mut f.draft.people,People::HighElf,tr("High Elves","Altos Elfos"));ui.selectable_value(&mut f.draft.people,People::Orc,tr("Orcs","Orcs"));});
        ui.label(tr("Body","Corpo"));ui.horizontal(|ui|{ui.selectable_value(&mut f.draft.body,Body::Feminine,tr("Feminine","Feminino"));ui.selectable_value(&mut f.draft.body,Body::Masculine,tr("Masculine","Masculino"));});
        ui.label(tr("Class","Classe"));ui.horizontal_wrapped(|ui|{for c in [Class::Mage,Class::Warrior,Class::Archer,Class::Druid]{ui.selectable_value(&mut f.draft.class,c,class(c,pt));}});
        ui.label(tr("Allegiance","Facção"));ui.horizontal(|ui|{ui.selectable_value(&mut f.draft.faction,Faction::Elves,tr("Elves","Elfos"));ui.selectable_value(&mut f.draft.faction,Faction::Orcs,tr("Orcs","Orcs"));});
        ui.separator();
       }
       for (title,value,count) in [(tr("Face","Rosto"),&mut f.draft.appearance.face,3u8),(tr("Skin","Pele"),&mut f.draft.appearance.skin,4),(tr("Eyes","Olhos"),&mut f.draft.appearance.eyes,4),(tr("Ears","Orelhas"),&mut f.draft.appearance.ears,3)] {
        ui.label(title);ui.horizontal(|ui|{for i in 0..count{ui.selectable_value(value,i,format!("{}",i+1));}});
       }
       ui.small(tr("Hair: none · more styles coming later","Cabelo: sem cabelo · mais estilos posteriormente"));
       if old.people!=f.draft.people || old.body!=f.draft.body || old.appearance!=f.draft.appearance{actions.push(Action::Preview);}
       ui.add_space(10.);
       if f.page==Page::Create {
        let valid=f.draft.validate().is_ok() && f.preview_error.is_empty();
        if self.button(ui,"confirm",&tr("Create character","Criar personagem"),valid,true){actions.push(Action::Create);}
        if !f.draft.name.trim().is_empty() && let Err(e)=f.draft.validate(){ui.small(e.to_string());}
       } else if self.button(ui,"save_cosmetics",&tr("Save appearance","Salvar aparência"),true,true){actions.push(Action::SaveCosmetics);}
       if self.button(ui,"cancel",&tr("Cancel","Cancelar"),true,false){actions.push(Action::Cancel);}
      }
     });
   });
            egui::Area::new("preview_note".into())
                .anchor(egui::Align2::RIGHT_BOTTOM, [-24., -24.])
                .show(ctx, |ui| {
                    ui.label(tr(
                        "Appearance preview · character art in development",
                        "Prévia de aparência · arte dos personagens em desenvolvimento",
                    ));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Rotate left", "Girar à esquerda")).clicked() {
                            f.yaw -= 0.35;
                            actions.push(Action::Preview);
                        }
                        if ui.button(tr("Rotate right", "Girar à direita")).clicked() {
                            f.yaw += 0.35;
                            actions.push(Action::Preview);
                        }
                    });
                    if !f.preview_error.is_empty() {
                        ui.colored_label(Color32::LIGHT_RED, &f.preview_error);
                    }
                });
        } else {
            egui::Area::new("session_hud".into())
                .fixed_pos([24., 24.])
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_black_alpha(220))
                        .inner_margin(16.)
                        .show(ui, |ui| {
                            ui.set_width(340.);
                            ui.label(RichText::new("LAND OF SURVIVORS").color(GOLD));
                            if let Some(s) = &f.session {
                                ui.label(if s.offline {
                                    tr(
                                        "Offline · progress saved when leaving",
                                        "Offline · progresso salvo ao sair",
                                    )
                                } else {
                                    format!(
                                        "{}: {}",
                                        tr("Connection", "Conexão"),
                                        s.status["state"].as_str().unwrap_or("connecting")
                                    )
                                });
                                if !s.invitation.is_empty()
                                    && ui.button(tr("Copy invitation", "Copiar convite")).clicked()
                                {
                                    ctx.copy_text(s.invitation.clone());
                                }
                                if let Some(e) = s.status.get("error").and_then(|e| e.as_str()) {
                                    ui.colored_label(Color32::LIGHT_RED, e);
                                }
                            }
                            ui.label(tr(
                                "WASD to move · Shift to run",
                                "WASD para mover · Shift para correr",
                            ));
                            ui.small(tr(
                                "Character in rest pose · art in development",
                                "Personagem em repouso · arte em desenvolvimento",
                            ));
                            if self.button(
                                ui,
                                "leave",
                                &tr("Return to characters", "Voltar aos personagens"),
                                true,
                                false,
                            ) {
                                actions.push(Action::Leave);
                            }
                        });
                });
        }
        if !f.error.is_empty() {
            egui::Area::new("error".into())
                .anchor(egui::Align2::CENTER_BOTTOM, [0., -18.])
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(63, 24, 24))
                        .inner_margin(12.)
                        .show(ui, |ui| {
                            ui.set_max_width(760.);
                            ui.colored_label(Color32::LIGHT_RED, &f.error);
                        });
                });
        }
        (Some(viewport), actions)
    }
}
pub fn people(p: People, pt: bool) -> &'static str {
    match (p, pt) {
        (People::HighElf, false) => "High Elf",
        (People::HighElf, true) => "Alto Elfo",
        (People::Orc, _) => "Orc",
    }
}
pub fn class(c: Class, pt: bool) -> &'static str {
    match (c, pt) {
        (Class::Mage, false) => "Mage",
        (Class::Mage, true) => "Mago",
        (Class::Warrior, false) => "Warrior",
        (Class::Warrior, true) => "Guerreiro",
        (Class::Archer, false) => "Archer",
        (Class::Archer, true) => "Arqueiro",
        (Class::Druid, false) => "Druid",
        (Class::Druid, true) => "Druida",
    }
}
