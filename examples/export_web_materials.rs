//! Export only the exact native material GPU buffers needed by the web host.
use ignis::materials::{library, preview};
use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("Usage: export_web_materials INPUT.json OUTPUT_DIRECTORY".into());
    }
    let input: serde_json::Value = serde_json::from_slice(&fs::read(&args[0])?)?;
    let output = PathBuf::from(&args[1]);
    fs::create_dir_all(&output)?;
    let mut records = Vec::new();
    for entry in input.as_array().ok_or("Expected input records")? {
        let id = entry["id"].as_str().ok_or("Missing id")?;
        if !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'-')
        {
            return Err("Unsafe output ID".into());
        }
        let material = library::decode(&fs::read(entry["path"].as_str().ok_or("Missing path")?)?)?;
        let packet = preview::encode(&material)?;
        let name = format!("{id}.bin");
        fs::write(output.join(&name), packet)?;
        records.push(serde_json::json!({"id":id,"packet":name}));
    }
    fs::write(output.join("ignis.wgsl"), preview::SHADER)?;
    fs::write(
        output.join("packets.json"),
        serde_json::to_vec_pretty(&records)?,
    )?;
    println!(
        "Exported {} native material packets and the shared Ignis shader",
        records.len()
    );
    Ok(())
}
