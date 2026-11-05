//! Local junction resolution, shared by authored edits and procedural recipes.
use super::*;
/// Incident overrides compose by maximum; absent overrides preserve each legacy path.
pub(super) fn level(recipe: &Recipe, organs: impl IntoIterator<Item = usize>, legacy: u8) -> u8 {
    organs
        .into_iter()
        .filter_map(|i| {
            recipe
                .edits
                .get(&recipe.source.organs[i].id)
                .and_then(|e| e.junction_refinement)
        })
        .max()
        .unwrap_or(legacy)
}
pub fn editor(ui: &mut egui::Ui, edit: &mut OrganEdit) -> Option<wood_surface::ui::Action> {
    use wood_surface::ui::Action;
    let mut action = None;
    ui.collapsing("Resolução local das junções",|ui|{
        let mut enabled=edit.junction_refinement.is_some();
        if ui.checkbox(&mut enabled,"Sobrescrever resolução nas junções deste órgão").changed(){edit.junction_refinement=enabled.then_some(2);}
        if let Some(level)=&mut edit.junction_refinement {ui.add(egui::Slider::new(level,0..=3).text("Subdivisões interiores"));}
        ui.label("Vizinhos com ajustes usam o maior nível. Cada nível subdivide faces interiores; a copa fora das junções conserva sua resolução.");
        ui.label("Isso não reduz colares nem corrige tangência ou continuidade de veios. O orçamento final continua obrigatório.");
        ui.horizontal(|ui|{
            if ui.button("Prévia das junções").clicked(){action=Some(Action::Preview);}
            if ui.button("Aplicar junções").clicked(){action=Some(Action::Apply);}
            if ui.button("Cancelar prévia das junções").clicked(){action=Some(Action::Cancel);}
        });
    });
    action
}
