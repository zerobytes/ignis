//! Forms from the shared typed schema: the catalog cannot silently lag the MCP API.
use super::*;
use serde_json::{Value, json};
use std::sync::OnceLock;
fn command_schema() -> &'static Value {
    static S: OnceLock<Value> = OnceLock::new();
    S.get_or_init(|| serde_json::to_value(schemars::schema_for!(Command)).unwrap())
}
fn entity_schema() -> &'static Value {
    static S: OnceLock<Value> = OnceLock::new();
    S.get_or_init(|| serde_json::to_value(schemars::schema_for!(Entity)).unwrap())
}
fn component_schema() -> &'static Value {
    static S: OnceLock<Value> = OnceLock::new();
    S.get_or_init(|| {
        let mut schema = entity_schema().clone();
        if let Some(p) = schema["properties"].as_object_mut() {
            p.retain(|k, _| {
                !matches!(
                    k.as_str(),
                    "id" | "name" | "transform" | "color" | "visible" | "body"
                )
            });
        }
        schema
    })
}
#[derive(Default)]
pub(super) struct State {
    pub open: bool,
    search: String,
    selected: usize,
    value: Value,
    pub error: String,
    category: usize,
    pub awaiting: bool,
    pub result: Option<Value>,
    component_key: Option<(String, u64)>,
    component_value: Value,
}
fn resolve<'a>(s: &'a Value, root: &'a Value) -> &'a Value {
    s.get("$ref")
        .and_then(Value::as_str)
        .and_then(|p| root.pointer(p.strip_prefix('#')?))
        .unwrap_or(s)
}
fn alternatives(s: &Value) -> Option<&Vec<Value>> {
    s.get("oneOf")
        .or_else(|| s.get("anyOf"))
        .and_then(Value::as_array)
}
fn tag(s: &Value, root: &Value) -> String {
    let s = resolve(s, root);
    s.get("properties")
        .and_then(Value::as_object)
        .and_then(|p| {
            p.values()
                .find_map(|v| v.get("const").and_then(Value::as_str))
        })
        .unwrap_or("Opção")
        .into()
}
fn label(s: &str) -> String {
    match s {
        "model_editor_mesh" => "Modelar faces e arestas com prévia".into(),
        "sculpt_stroke" => "Traço de escultura com limites automáticos".into(),
        "sculpt_detailed" => "Volume com refinamento local".into(),
        "model_selection" => "Extrusão, inset e bevel de componentes".into(),
        "set_editor_transform" => "Configurar gizmos de transformação".into(),
        "inspect_editor_transform" => "Inspecionar alvo de transformação".into(),
        "preview_editor_transform" => "Prévia de transformação".into(),
        "transform_editor" => "Transformar objeto ou componentes".into(),
        "set_editor_mesh_view" => "Visualização da malha".into(),
        "select_editor_mesh_region" => "Selecionar componentes por retângulo".into(),
        "xray" => "Raio X · incluir componentes ocultos".into(),
        "create_editor_mesh" => "Criar primitiva editável".into(),
        "select_editor_mesh" => "Selecionar malha".into(),
        "pick_editor_mesh" => "Consultar superfície da malha".into(),
        "inspect_project" => "Inspecionar projeto".into(),
        "browse_project_directory" => "Navegar por arquivos e pastas".into(),
        "create_project_folder" => "Criar pasta".into(),
        "inspect_project_library" => "Projetos recentes e favoritos".into(),
        "use_project_library" => "Arquivo da biblioteca de projetos".into(),
        "register_project" => "Adicionar projeto à biblioteca".into(),
        "pin_project" => "Fixar projeto".into(),
        "forget_project" => "Remover projeto da lista".into(),
        "set_project_favorite" => "Pasta favorita".into(),
        "discover_projects" => "Procurar projetos em pastas".into(),
        "create_project" => "Criar projeto".into(),
        "open_project" => "Abrir projeto".into(),
        "new_project_scene" => "Criar cena no projeto".into(),
        "open_project_scene" => "Abrir cena do projeto".into(),
        "set_project_entry" => "Definir cena inicial do jogo".into(),
        "create_scene_object" => "Adicionar objeto ou NPC".into(),
        "duplicate_scene_object" => "Duplicar objeto".into(),
        "inspect_editor" => "Inspecionar editor".into(),
        "select_editor_at" => "Selecionar na viewport".into(),
        "edit_editor" => "Configurar áreas de trabalho".into(),
        "edit_mesh" => "Operações de modelagem".into(),
        "preview_mesh_edit" => "Prévia de operação de modelagem".into(),
        "set_rig" => "Definir esqueleto e pesos".into(),
        "put_clip" => "Criar ou atualizar clipe".into(),
        "transform_vertices" => "Transformar vértices selecionados".into(),
        "transform_islands" => "Transformar peças conectadas".into(),
        "set_npc_templates" => "Configurar modelos de NPC".into(),
        "set_npc_generators" => "Configurar geradores de NPC".into(),
        "set_ability_catalog" => "Configurar habilidades do jogo".into(),
        "view" => "Visualização".into(),
        "edges" => "Arestas".into(),
        "body" => "Tipo físico".into(),
        "mode" => "Modo de percepção".into(),
        "perceptive" => "Percepção · visão, memória e som".into(),
        "omniscient" => "Onisciente · diagnóstico".into(),
        "range" => "Alcance (m)".into(),
        "cooldown_ticks" => "Recarga (ticks)".into(),
        "cast_ticks" => "Tempo de preparo (ticks)".into(),
        "costs" => "Custos de execução".into(),
        "effects" => "Efeitos".into(),
        "recipient" => "Destinatário".into(),
        "source" => "Origem".into(),
        "initial_facing_degrees" => "Direção inicial (°)".into(),
        "search_turn_degrees_per_second" => "Giro durante busca (°/s)".into(),
        "npc" => "Comportamento do NPC".into(),
        "awareness" => "Percepção e memória".into(),
        "target" => "Alvo".into(),
        "surface" => "Piso de navegação".into(),
        "speed" => "Velocidade (m/s)".into(),
        "stop_distance" => "Distância de parada (m)".into(),
        "enabled" => "Ativo".into(),
        "patrol" => "Patrulha".into(),
        "objectives" => "Objetivos".into(),
        "recognition" => "Reconhecimento visual".into(),
        "territory" => "Território e retorno".into(),
        "communication" => "Comunicação".into(),
        "targeting" => "Escolha de alvos".into(),
        "navigation_recovery" => "Recuperação de navegação".into(),
        "sound_attention" => "Atenção sonora".into(),
        "mobility" => "Locomoção espacial".into(),
        "spatial_vision" => "Visão vertical".into(),
        "vitality" => "Vida".into(),
        "abilities" => "Habilidades e recursos".into(),
        "character_controller" => "Motor terrestre · rampas e degraus".into(),
        "hearing" => "Audição".into(),
        "vision_range" => "Alcance de visão (m)".into(),
        "fov_degrees" => "Campo de visão (°)".into(),
        "memory_ticks" => "Memória (ticks)".into(),
        "search_ticks" => "Busca (ticks)".into(),
        _ => {
            let s = s
                .split('_')
                .map(|word| match word {
                    "inspect" => "inspecionar",
                    "select" => "selecionar",
                    "pick" => "consultar",
                    "open" => "abrir",
                    "new" => "novo",
                    "object" => "objeto",
                    "objects" => "objetos",
                    "duplicate" => "duplicar",
                    "at" => "no ponto",
                    "entry" => "inicial",
                    "view" => "visualização",
                    "actor" => "personagem",
                    "query" => "consultar",
                    "edit" => "editar",
                    "create" => "criar",
                    "set" => "configurar",
                    "preview" => "prévia",
                    "plan" => "planejar",
                    "save" => "salvar",
                    "load" => "abrir",
                    "remove" => "remover",
                    "put" => "definir",
                    "mesh" => "malha",
                    "meshes" => "malhas",
                    "scene" => "cena",
                    "scenes" => "cenas",
                    "project" => "projeto",
                    "rig" => "esqueleto",
                    "clip" => "clipe",
                    "vertices" => "vértices",
                    "faces" => "faces",
                    "bones" => "ossos",
                    "material" => "material",
                    "materials" => "materiais",
                    "finish" => "acabamento",
                    "light" => "luz",
                    "geometry" => "geometria",
                    "components" => "componentes",
                    "selection" => "seleção",
                    "normals" => "normais",
                    "topology" => "topologia",
                    "asset" => "asset",
                    "attributes" => "atributos",
                    "abilities" => "habilidades",
                    "controls" => "controles",
                    "population" => "população",
                    "spawn" => "nascimento",
                    "generators" => "geradores",
                    "play" => "executar",
                    "pause" => "pausar",
                    "step" => "avançar",
                    "reset" => "reiniciar",
                    "capture" => "capturar",
                    "plants" => "plantas",
                    _ => word,
                })
                .collect::<Vec<_>>()
                .join(" ");
            let mut chars = s.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        }
    }
}
fn kind(s: &Value) -> &str {
    s["type"]
        .as_str()
        .or_else(|| {
            s["type"]
                .as_array()
                .and_then(|a| a.iter().filter_map(Value::as_str).find(|x| *x != "null"))
        })
        .unwrap_or("")
}
fn choice(value: &Value) -> String {
    match value.as_str().unwrap_or("") {
        "npc" => "NPC".into(),
        "player" => "Jogador".into(),
        "fixed" => "Estático · colisão".into(),
        "dynamic" => "Físico".into(),
        s => label(s),
    }
}
fn nullable(s: &Value, root: &Value) -> bool {
    let s = resolve(s, root);
    s["type"] == "null"
        || s["type"]
            .as_array()
            .is_some_and(|a| a.contains(&json!("null")))
        || alternatives(s).is_some_and(|a| a.iter().any(|x| nullable(x, root)))
}
fn context_defaults(value: &mut Value, service: &Service) {
    match value {
        Value::Object(fields) => {
            for (key, v) in fields {
                if key == "reference" && v.get("scope").is_some() && v.get("revision").is_some() {
                    *v = serde_json::to_value(service.engine.mesh_reference()).unwrap();
                } else if key == "asset" && v.as_str() == Some("") {
                    if let Some(i) = super::mesh_ui::selected(service) {
                        *v = json!(i.asset);
                    }
                } else if key == "instance" && v.as_str() == Some("") {
                    if let Some(i) = &service.selected_mesh {
                        *v = json!(i);
                    }
                } else {
                    context_defaults(v, service);
                }
            }
        }
        Value::Array(values) => {
            for v in values {
                context_defaults(v, service)
            }
        }
        _ => {}
    }
}
fn category(name: &str) -> usize {
    if name.contains("mesh") || name.contains("plant") {
        1
    } else if [
        "npc",
        "generator",
        "spawn",
        "control",
        "ability",
        "vitality",
        "faction",
    ]
    .iter()
    .any(|k| name.contains(k))
    {
        2
    } else if ["project", "scene", "editor"]
        .iter()
        .any(|k| name.contains(k))
    {
        3
    } else if ["replay", "inspect", "query", "capture"]
        .iter()
        .any(|k| name.contains(k))
    {
        4
    } else {
        5
    }
}
fn seed(s: &Value, root: &Value, depth: u8) -> Value {
    if depth > 10 {
        return Value::Null;
    }
    let s = resolve(s, root);
    if let Some(v) = s.get("const").or_else(|| s.get("default")) {
        return v.clone();
    }
    if let Some(a) = alternatives(s) {
        return seed(
            a.iter()
                .find(|x| x.get("type") != Some(&json!("null")))
                .unwrap_or(&a[0]),
            root,
            depth + 1,
        );
    }
    if let Some(a) = s.get("enum").and_then(Value::as_array) {
        return a[0].clone();
    }
    if s.get("properties").is_some() {
        let mut out = serde_json::Map::new();
        for (k, v) in s["properties"].as_object().unwrap() {
            if s["required"]
                .as_array()
                .is_some_and(|a| a.contains(&json!(k)))
            {
                out.insert(k.clone(), seed(v, root, depth + 1));
            }
        }
        return Value::Object(out);
    }
    match kind(s) {
        "boolean" => json!(true),
        "string" => json!(""),
        "integer" => json!(s["minimum"].as_i64().unwrap_or(0)),
        "number" => json!(s["minimum"].as_f64().unwrap_or(0.)),
        "array" => {
            if let Some(items) = s["prefixItems"].as_array() {
                json!(
                    items
                        .iter()
                        .map(|x| seed(x, root, depth + 1))
                        .collect::<Vec<_>>()
                )
            } else {
                json!([])
            }
        }
        _ => Value::Null,
    }
}
fn form(
    ui: &mut egui::Ui,
    value: &mut Value,
    schema: &Value,
    root: &Value,
    service: &Service,
    depth: u8,
) {
    if depth > 12 {
        ui.weak("Subestrutura extensa: abra uma operação menor.");
        return;
    }
    let schema = resolve(schema, root);
    if let Some(variants) = alternatives(schema) {
        let options: Vec<_> = variants
            .iter()
            .filter(|s| s.get("type") != Some(&json!("null")))
            .collect();
        if options.len() > 1 {
            let current = options
                .iter()
                .position(|s| {
                    let properties = resolve(s, root)["properties"].as_object();
                    properties.is_some_and(|p| {
                        p.iter()
                            .any(|(k, v)| v.get("const").is_some_and(|c| value.get(k) == Some(c)))
                    })
                })
                .unwrap_or(0);
            let mut selected = current;
            egui::ComboBox::from_id_salt("variant")
                .selected_text(label(&tag(options[current], root)))
                .show_ui(ui, |ui| {
                    for (i, s) in options.iter().enumerate() {
                        ui.selectable_value(&mut selected, i, label(&tag(s, root)));
                    }
                });
            if selected != current {
                *value = seed(options[selected], root, 0);
                context_defaults(value, service);
            }
            form(ui, value, options[selected], root, service, depth + 1);
            return;
        } else if let Some(s) = options.first() {
            form(ui, value, s, root, service, depth + 1);
            return;
        }
    }
    if let Some(choices) = schema.get("enum").and_then(Value::as_array) {
        egui::ComboBox::from_id_salt("enum")
            .selected_text(choice(value))
            .show_ui(ui, |ui| {
                for choice in choices {
                    ui.selectable_value(value, choice.clone(), self::choice(choice));
                }
            });
        return;
    }
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        if !value.is_object() {
            *value = seed(schema, root, 0);
        }
        let mut fields: Vec<_> = properties.iter().collect();
        fields.sort_by_key(|(key, _)| match key.as_str() {
            "body" | "enabled" => 0,
            "npc" | "target" => 1,
            "speed" => 2,
            "stop_distance" => 3,
            "surface" => 4,
            "awareness" => 5,
            "patrol" => 6,
            _ => 20,
        });
        for (key, child) in fields {
            if child.get("const").is_some() {
                continue;
            }
            ui.push_id(key, |ui| {
                if key=="reference" && value.get(key).is_some_and(|v|v.get("scope").is_some() && v.get("revision").is_some()) {
                    ui.weak(format!("Seleção da revisão {}",value[key]["revision"]));
                    if ui.small_button("Usar seleção atual").clicked(){value[key]=serde_json::to_value(service.engine.mesh_reference()).unwrap();}
                    return;
                }
                let required = !nullable(child,root) && schema["required"]
                    .as_array()
                    .is_some_and(|a| a.contains(&json!(key)));
                let present = value.get(key).is_some_and(|v| !v.is_null());
                if !present && !required {
                    if ui.small_button(format!("+ {}", label(key))).clicked() {
                        let mut next = seed(child, root, 0);
                        if key=="awareness" {next=json!({"mode":"perceptive","vision_range":12.,"fov_degrees":100.,"initial_facing_degrees":0.,"memory_ticks":180,"search_ticks":240,"search_turn_degrees_per_second":90.});}
                        if key=="vitality" {next=json!({"initial":100,"maximum":100,"scale":1});}
                        if key=="character_controller" {next=serde_json::to_value(crate::character_controller::Config::default()).unwrap();}
                        if key=="patrol" {next=json!({"route":{"kind":"points","points":[[-1.,-1.],[1.,-1.]]},"traversal":"ping_pong","wait_ticks":60,"travel_ticks":600,"retry_ticks":60,"arrival_distance":0.3});}

                        if next.is_string() {
                            if key == "surface" {
                                next = json!(
                                    service
                                        .engine
                                        .scene
                                        .entities
                                        .iter()
                                        .find(|e| e.body == Some(crate::model::BodyKind::Fixed))
                                        .map(|e| e.id.as_str())
                                        .unwrap_or("floor")
                                );
                            } else if key == "target" {
                                next = json!("player");
                            }
                        }
                        value[key] = next;
                    }
                    return;
                }
                if !present {
                    value[key] = seed(child, root, 0);
                }
                let composite = value[key].is_object() || value[key].is_array();
                if composite {
                    egui::CollapsingHeader::new(label(key))
                        .default_open(depth < 1)
                        .show(ui, |ui| {
                            if let Some(description) =
                                child.get("description").and_then(Value::as_str)
                            {
                                ui.small(description);
                            }
                            form(ui, &mut value[key], child, root, service, depth + 1);
                            if !required && ui.small_button("Remover configuração").clicked() {
                                value.as_object_mut().unwrap().remove(key);
                            }
                        });
                } else {
                    ui.label(label(key));
                    if key == "surface"
                        || key == "target"
                        || key == "actor"
                        || key == "asset"
                        || key == "instance"
                    {
                        let names:Vec<String>=match key.as_str() {
                            "asset"=>service.engine.scene.meshes.assets.iter().map(|a|a.id.clone()).collect(),
                            "instance"=>service.engine.scene.meshes.instances.iter().map(|a|a.id.clone()).collect(),
                            "surface"=>service.engine.scene.entities.iter().filter(|e|e.body==Some(crate::model::BodyKind::Fixed) && e.geometry.is_box()).map(|e|e.id.clone()).collect(),
                            _=>service.engine.simulation.definitions().entities.iter().filter(|e|matches!(e.body,Some(crate::model::BodyKind::Player|crate::model::BodyKind::Npc)) && service.engine.simulation.position(&e.id).is_some()).map(|e|e.id.clone()).collect()
                        };
                        egui::ComboBox::from_id_salt("reference")
                            .selected_text(value[key].as_str().unwrap_or("Escolher"))
                            .show_ui(ui, |ui| {
                                for name in names {
                                    ui.selectable_value(&mut value[key], json!(name), name);
                                }
                            });
                    } else {
                        form(ui, &mut value[key], child, root, service, depth + 1);
                    }
                    if !required && ui.small_button("Remover").clicked() {
                        value.as_object_mut().unwrap().remove(key);
                    }
                }
            });
        }
    } else if kind(schema) == "object" {
        if !value.is_object() {
            *value = json!({});
        }
        let keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
        for key in keys {
            ui.push_id(&key, |ui| {
                ui.label(&key);
                form(
                    ui,
                    &mut value[&key],
                    &schema["additionalProperties"],
                    root,
                    service,
                    depth + 1,
                );
                if ui.small_button("Remover entrada").clicked() {
                    value.as_object_mut().unwrap().remove(&key);
                }
            });
        }
        let id = ui.id().with("new_map_key");
        let mut key = ui
            .ctx()
            .data_mut(|d| d.get_temp::<String>(id).unwrap_or_default());
        ui.add(egui::TextEdit::singleline(&mut key).hint_text("Nome da nova entrada"));
        if ui
            .add_enabled(
                !key.trim().is_empty(),
                egui::Button::new("Adicionar entrada"),
            )
            .clicked()
        {
            value[&key] = seed(&schema["additionalProperties"], root, 0);
            key.clear();
        }
        ui.ctx().data_mut(|d| d.insert_temp(id, key));
    } else if let Some(array) = value.as_array_mut() {
        let start_id = ui.id().with("array_page");
        let mut page = ui
            .ctx()
            .data_mut(|d| d.get_temp::<usize>(start_id).unwrap_or(0));
        let pages = array.len().div_ceil(16).max(1);
        page = page.min(pages - 1);
        ui.horizontal(|ui| {
            ui.weak(format!(
                "{} itens · página {}/{}",
                array.len(),
                page + 1,
                pages
            ));
            if page > 0 && ui.small_button("‹").clicked() {
                page -= 1;
            }
            if page + 1 < pages && ui.small_button("›").clicked() {
                page += 1;
            }
        });
        ui.ctx().data_mut(|d| d.insert_temp(start_id, page));
        let fixed = schema.get("prefixItems").and_then(Value::as_array);
        let mut remove = None;
        for (i, item) in array.iter_mut().enumerate().skip(page * 16).take(16) {
            ui.push_id(i, |ui| {
                ui.label(format!("{}", i + 1));
                let child = fixed.and_then(|f| f.get(i)).unwrap_or(&schema["items"]);
                form(ui, item, child, root, service, depth + 1);
                if fixed.is_none() && ui.small_button("− Remover item").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            array.remove(i);
        }
        if fixed.is_none()
            && array.len() < schema["maxItems"].as_u64().unwrap_or(32768).min(32768) as usize
            && ui.small_button("+ Adicionar item").clicked()
        {
            array.push(seed(&schema["items"], root, 0));
        }
    } else if let Value::Bool(v) = value {
        ui.checkbox(v, "Habilitado");
    } else if let Some(v) = value.as_str() {
        let mut text = v.to_owned();
        ui.add(egui::TextEdit::singleline(&mut text).desired_width(ui.available_width()));
        *value = json!(text);
    } else if value.is_number() {
        if kind(schema) == "integer" {
            let mut n = value.as_i64().unwrap_or(0);
            if ui.add(egui::DragValue::new(&mut n).speed(1.)).changed() {
                *value = json!(n);
            }
        } else {
            let mut n = value.as_f64().unwrap_or(0.) as f32;
            if ui
                .add_sized(
                    [ui.available_width(), 26.],
                    design_ui::Scrub::new(&mut n).speed(0.05),
                )
                .changed()
            {
                *value = json!(n);
            }
        }
    } else {
        ui.weak("Escolha uma configuração.");
    }
}
pub(super) fn components(
    ui: &mut egui::Ui,
    draft: &mut Entity,
    service: &Service,
    state: &mut State,
) {
    let key = (draft.id.clone(), service.engine.revision);
    if state.component_key.as_ref() != Some(&key) {
        state.component_value = serde_json::to_value(&*draft).unwrap();
        state.component_key = Some(key);
    }
    let value = &mut state.component_value;
    let root = entity_schema();
    let schema = component_schema();
    egui::CollapsingHeader::new("Componentes e comportamento")
        .default_open(draft.npc.is_some())
        .show(ui, |ui| form(ui, value, schema, root, service, 0));
    let mut merged = serde_json::to_value(&*draft).unwrap();
    for key in schema["properties"].as_object().unwrap().keys() {
        if let Some(v) = value.get(key) {
            merged[key] = v.clone();
        } else {
            merged.as_object_mut().unwrap().remove(key);
        }
    }
    match serde_json::from_value(merged) {
        Ok(next) => *draft = next,
        Err(e) => {
            ui.colored_label(EMBER, format!("Complete a configuração: {e}"));
        }
    }
}
fn result(ui: &mut egui::Ui, v: &Value, depth: usize) {
    if depth > 8 {
        ui.weak("Mais níveis disponíveis pela inspeção específica.");
        return;
    }
    match v {
        Value::Object(values) => {
            for (k, v) in values {
                ui.collapsing(label(k), |ui| result(ui, v, depth + 1));
            }
        }
        Value::Array(values) => {
            ui.weak(format!("{} itens", values.len()));
            for (i, v) in values.iter().enumerate().take(64) {
                ui.collapsing(format!("{}", i + 1), |ui| result(ui, v, depth + 1));
            }
        }
        _ => {
            ui.label(
                v.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string()),
            );
        }
    }
}
pub(super) fn show(
    ctx: &egui::Context,
    service: &Service,
    state: &mut State,
    commands: &mut Vec<Command>,
) {
    if !state.open {
        return;
    }
    let root = command_schema();
    let variants = root["oneOf"].as_array().unwrap();
    let mut open = true;
    egui::Window::new("Ferramentas da Ignis").open(&mut open).default_size([780.,600.]).show(ctx,|ui|{
        ui.label("Encontre uma ferramenta por tarefa ou categoria. Selecione uma malha na cena para preencher seu contexto.");
        ui.add(egui::TextEdit::singleline(&mut state.search).hint_text("Buscar tarefa, NPC, mesh, replay…"));
        ui.horizontal_wrapped(|ui|{for (i,name) in ["Todas","Geometria e rig","NPCs e jogo","Projetos e editor","Inspeção e replay","Outras"].iter().enumerate(){ui.selectable_value(&mut state.category,i,*name);}});
        ui.columns(2,|cols|{
            egui::ScrollArea::vertical().id_salt("tools").max_height(480.).show(&mut cols[0],|ui|{
                let query=state.search.to_lowercase();
                for (i,s) in variants.iter().enumerate(){let name=tag(s,root);let title=label(&name);if state.category!=0 && category(&name)!=state.category {continue}
                    if !query.is_empty() && !format!("{name} {title}").to_lowercase().contains(&query){continue}
                    if ui.selectable_label(i==state.selected,title).clicked(){state.selected=i;state.value=seed(s,root,0);context_defaults(&mut state.value,service);state.error.clear();state.result=None;}
                }
            });
            egui::ScrollArea::vertical().id_salt("parameters").max_height(480.).show(&mut cols[1],|ui|{
                let s=&variants[state.selected.min(variants.len()-1)];if state.value.is_null(){state.value=seed(s,root,0);}
                ui.heading(label(&tag(s,root)));if let Some(d)=s.get("description").and_then(Value::as_str){ui.small(d);}
                form(ui,&mut state.value,s,root,service,0);
                ui.separator();
                if ui.add(design_ui::primary("Executar operação")).clicked(){match serde_json::from_value::<Command>(state.value.clone()){
                    Ok(command)=>{commands.push(command);state.awaiting=true;state.error.clear();},Err(e)=>state.error=format!("Complete os campos: {e}")
                }}
                if !state.error.is_empty(){ui.colored_label(EMBER,&state.error);}
                if let Some(v)=&state.result{ui.separator();ui.strong("Resultado");result(ui,v,0);}
            });
        });
    });
    state.open = open;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_uses_every_shared_command_and_nullables_preserve_types() {
        let schema = command_schema();
        let variants = schema["oneOf"].as_array().unwrap();
        assert!(variants.len() > 100);
        let mut names = std::collections::BTreeSet::new();
        for s in variants {
            let name = tag(s, schema);
            assert_ne!(name, "Opção");
            assert!(names.insert(name.clone()));
            assert_eq!(seed(s, schema, 0)["op"], name);
        }
        assert_eq!(
            seed(&json!({"type":["integer","null"]}), schema, 0),
            json!(0)
        );
        assert_eq!(
            seed(&json!({"type":["boolean","null"]}), schema, 0),
            json!(true)
        );
        assert!(nullable(&json!({"type":["integer","null"]}), schema));
    }
    #[test]
    fn idle_forms_do_not_add_components_or_change_visual_bodies() {
        let tmp = tempfile::tempdir().unwrap();
        let service = Service::new(
            tmp.path(),
            crate::model::RoomRecipe::default().generate().unwrap(),
        )
        .unwrap();
        let mut entity = service.engine.scene.entities[0].clone();
        entity.body = None;
        entity.npc = None;
        entity.footsteps = None;
        let original = entity.clone();
        let ctx = egui::Context::default();
        let mut value = serde_json::to_value(&entity).unwrap();
        for _ in 0..2 {
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    form(
                        ui,
                        &mut value,
                        entity_schema(),
                        entity_schema(),
                        &service,
                        0,
                    )
                });
            });
        }
        entity = serde_json::from_value(value).unwrap();
        assert_eq!(entity, original);
    }
}
