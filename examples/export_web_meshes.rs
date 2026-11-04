//! Export full native botanical meshes, an editable scene, portable GLB and native capture.
use ignis::{
    botany, engine::Engine, materials::library, meshes, model::RoomRecipe, render::Renderer,
};
use std::{
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("Usage: export_web_meshes INPUT.json OUTPUT_DIRECTORY".into());
    }
    let inputs: serde_json::Value =
        serde_json::from_reader(BufReader::new(fs::File::open(&args[0])?))?;
    let output = PathBuf::from(&args[1]);
    fs::create_dir_all(&output)?;
    let gpu_instance = wgpu::Instance::default();
    let mut renderer = pollster::block_on(Renderer::new(&gpu_instance, None))?;
    for input in inputs.as_array().ok_or("Expected records")? {
        let id = input["id"].as_str().ok_or("Missing id")?;
        if !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'-')
        {
            return Err("Invalid ID".into());
        }
        println!("Compiling {id}");
        let recipe: botany::Recipe = serde_json::from_reader(BufReader::new(fs::File::open(
            input["path"].as_str().ok_or("Missing source")?,
        )?))?;
        recipe.validate()?;
        let document = compile(&recipe, input)?;
        let packet = meshes::preview::prepare(&document)?;
        let directory = output.join(id);
        fs::create_dir_all(&directory)?;

        fs::write(directory.join("mesh.glb"), packet.glb())?;
        let metadata = packet.metadata.clone();
        drop(packet);
        let mut scene = RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()?;
        for e in &mut scene.entities {
            e.visible = Some(false);
        }
        scene.meshes = document;
        scene.botany = vec![recipe.clone()];
        let min: Vec<f32> = serde_json::from_value(metadata["bounds"]["min"].clone())?;
        let max: Vec<f32> = serde_json::from_value(metadata["bounds"]["max"].clone())?;
        let center = glam::Vec3::from_array(std::array::from_fn(|i| (min[i] + max[i]) * 0.5));
        let radius =
            glam::Vec3::from_array(std::array::from_fn(|i| (max[i] - min[i]) * 0.5)).length();
        scene.camera.target = center.to_array();
        scene.camera.position = (center + glam::Vec3::new(0.9, 0.3, 2.55) * radius).to_array();
        scene.camera.fov_degrees = 45.;
        scene.light.direction = [-0.57357645, -0.65, -0.81915206];
        scene.light.ambient = 0.3;
        scene.light.shadow = None;
        scene.validate()?;
        serde_json::to_writer(
            BufWriter::new(fs::File::create(directory.join("tree.scene.json"))?),
            &scene,
        )?;
        let mut frame = Engine::new(scene)?.render_frame();
        frame.suppress_diagnostic_grid = true;
        fs::write(
            directory.join("thumbnail.png"),
            renderer.capture(&frame, 640, 640)?,
        )?;
        fs::write(
            directory.join("mesh.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        let mut preview_recipe = recipe;
        preview_recipe.version = preview_recipe.version.max(4);
        preview_recipe.settings.axis_samples = 8;
        preview_recipe.settings.sides = 6;
        preview_recipe.settings.blade_samples = 4;
        let preview_document = compile(&preview_recipe, input)?;
        let mut preview = meshes::preview::prepare(&preview_document)?;
        preview.metadata["quality"] = serde_json::json!("reduced-tessellation");
        preview.metadata["tessellation"] =
            serde_json::json!({"axis_samples":8,"sides":6,"blade_samples":4});
        fs::write(directory.join("preview.igmp"), preview.encode())?;
        fs::write(
            directory.join("preview.json"),
            serde_json::to_vec_pretty(&preview.metadata)?,
        )?;
        println!(
            "Exported {id}: {} parts",
            metadata["parts"].as_array().unwrap().len()
        );
    }
    Ok(())
}

fn compile(
    recipe: &botany::Recipe,
    input: &serde_json::Value,
) -> Result<meshes::Document, Box<dyn std::error::Error>> {
    let mut document = botany::bake::compile(recipe)?.document;
    if let Some(overrides) = input["materials"].as_object() {
        for (asset, path) in overrides {
            let material =
                library::decode(&fs::read(path.as_str().ok_or("Invalid material path")?)?)?;
            document
                .assets
                .iter_mut()
                .find(|a| a.id == *asset)
                .ok_or("Material asset missing")?
                .material = material;
        }
    }
    Ok(document)
}
