//! Eye authoring is a view over the same bounded Configuration used by MCP.
use crate::meshes::character::{
    Configuration, Source,
    eyes::{CONTROLS, FIT_CONTROLS},
};

pub(super) fn controls(
    ui: &mut egui::Ui,
    source: &Source,
    config: &mut Configuration,
    linked: &mut bool,
    selected: &mut usize,
) -> bool {
    if source.eyes.is_empty() {
        return false;
    }
    *selected = (*selected).min(source.eyes.len() - 1);
    let eye = &source.eyes[*selected];
    let key = |name: &str| format!("{}_{}", eye.prefix, name);
    let before = config.parameters.clone();
    if eye.fit.is_some() {
        egui::CollapsingHeader::new("Olhos · globo e encaixe").default_open(true).show(ui, |ui| {
            ui.checkbox(linked, "Editar os dois olhos juntos");
            ui.horizontal(|ui| {for (i, _) in source.eyes.iter().enumerate() {
                ui.selectable_value(selected, i, if i == 0 { "Esquerdo" } else { "Direito" });
            }});
            ui.small("A profundidade move o globo e acompanha a borda das pálpebras. Zero mantém o encaixe natural. Valores positivos recuam o olho.");
            if eye.fit.as_ref().is_some_and(|f|f.spherical.is_some()) {ui.small("O globo é esférico. O encaixe ajusta as pálpebras ao redor dele.");}
            for &(name, label, _, _, _) in FIT_CONTROLS {
                let p=source.parameters.iter().find(|p|p.name==key(name)).unwrap();
                let (min,max,default)=(p.minimum,p.maximum,p.default);
                let value = config.parameters.entry(key(name)).or_insert(default);
                ui.label(label);
                ui.horizontal(|ui| {
                    if name == "globe_depth" {
                        let mut mm = *value * 1000.;
                        if ui.add(egui::Slider::new(&mut mm, min*1000. ..=max*1000.).suffix(" mm").max_decimals(2)).changed() { *value = mm / 1000.; }
                    } else { ui.add(egui::Slider::new(value, min..=max)); }
                    if ui.small_button("↺").clicked() { *value = default; }
                });
            }
        });
    }
    egui::CollapsingHeader::new("Olhos · íris").default_open(true).show(ui, |ui| {
        ui.checkbox(linked,"Editar os dois olhos juntos");
        ui.horizontal(|ui| {
            for (i,_) in source.eyes.iter().enumerate() {ui.selectable_value(selected,i,if i==0 {"Esquerdo"} else {"Direito"});}
        });
        ui.small("Cor, pigmento e relevo são gerados ao editar. A pupila é manual; não reage à luz automaticamente.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Cores:");
            for (label,outer,inner) in [
                ("Castanho",[0.16,0.085,0.025],[0.24,0.12,0.025]),
                ("Mel",[0.3,0.20,0.055],[0.37,0.16,0.025]),
                ("Verde",[0.11,0.19,0.095],[0.24,0.15,0.035]),
                ("Azul",[0.065,0.19,0.32],[0.075,0.12,0.13]),
                ("Cinza",[0.16,0.185,0.20],[0.12,0.13,0.12]),
            ] {
                if ui.small_button(label).clicked() {
                    for (names,values) in [(["red","green","blue"],outer),(["inner_red","inner_green","inner_blue"],inner)] {
                        for (name,v) in names.into_iter().zip(values) {config.parameters.insert(key(name),v);}
                    }
                }
            }
        });
        for (label,names) in [("Pigmento externo",["red","green","blue"]),("Pigmento central",["inner_red","inner_green","inner_blue"])] {
            let values=names.map(|n|config.parameters[&key(n)]);
            let c=egui::Color32::from(egui::Rgba::from_rgb(values[0],values[1],values[2]));let mut srgb=[c.r(),c.g(),c.b()];
            ui.horizontal(|ui| {ui.label(label);if ui.color_edit_button_srgb(&mut srgb).changed() {
                let linear=egui::Rgba::from(egui::Color32::from_rgb(srgb[0],srgb[1],srgb[2])).to_array();
                for (name,v) in names.into_iter().zip(linear) {config.parameters.insert(key(name),v);}
            }});
        }
        for &(name,label,_,_,_) in &CONTROLS[6..] {
            let id=key(name);
            let parameter=source.parameters.iter().find(|p|p.name==id).unwrap();
            let (min,max,default)=(parameter.minimum,parameter.maximum,parameter.default);
            let value=config.parameters.entry(id).or_insert(default);
            ui.label(label);
            ui.horizontal(|ui| {
                if name=="quality" {
                    egui::ComboBox::from_id_salt("iris_quality").selected_text(["64", "128", "256"][value.round() as usize]).show_ui(ui,|ui| {
                        for (i,label) in ["64 · leve", "128 · padrão", "256 · close"].iter().enumerate() {ui.selectable_value(value,i as f32,*label);}
                    });
                } else if ["radius","depth","relief"].contains(&name) {
                    let mut mm=*value*1000.;if ui.add(egui::DragValue::new(&mut mm).range(min*1000. ..=max*1000.).speed(0.005).suffix(" mm").max_decimals(3)).changed() {*value=mm/1000.;}
                } else {ui.add(egui::Slider::new(value,min..=max).max_decimals(if name=="seed" {0} else {3}));}
                if ui.small_button("↺").clicked() {*value=default;}
            });
        }
        ui.small("128: cerca de 1 MiB por desenho; 256: cerca de 4 MiB. A córnea adiciona uma superfície transparente.");
    });
    let changed = config.parameters != before;
    if changed && *linked {
        for other in &source.eyes {
            if other.prefix == eye.prefix {
                continue;
            }
            for &(name, _, _, _, _) in
                CONTROLS
                    .iter()
                    .chain(if eye.fit.is_some() && other.fit.is_some() {
                        FIT_CONTROLS
                    } else {
                        &[]
                    })
            {
                let own = key(name);
                if config.parameters.get(&own) != before.get(&own) {
                    config.parameters.insert(
                        format!("{}_{}", other.prefix, name),
                        config.parameters[&own],
                    );
                }
            }
        }
    }
    changed
}
