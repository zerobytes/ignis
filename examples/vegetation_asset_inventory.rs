//! Streaming metadata inventory; does not allocate the source vertex/index arrays.
use serde::{
    Deserialize, Deserializer,
    de::{IgnoredAny, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, fs::File, io::BufReader};
fn count<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    struct Counter;
    impl<'de> Visitor<'de> for Counter {
        type Value = usize;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("triangle array")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<usize, A::Error> {
            let mut n = 0;
            while a.next_element::<IgnoredAny>()?.is_some() {
                n += 1;
            }
            Ok(n)
        }
    }
    d.deserialize_seq(Counter)
}
#[derive(Deserialize)]
struct Geometry {
    #[serde(deserialize_with = "count")]
    triangles: usize,
    material_coordinates: Option<IgnoredAny>,
}
#[derive(Deserialize)]
struct Asset {
    id: String,
    #[serde(default)]
    shading: String,
    geometry: Geometry,
    material: serde_json::Value,
    rig: Option<IgnoredAny>,
}
#[derive(Deserialize)]
struct Instance {
    asset: String,
}
#[derive(Deserialize)]
struct Meshes {
    assets: Vec<Asset>,
    instances: Vec<Instance>,
}
#[derive(Deserialize)]
struct Scene {
    meshes: Meshes,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("scene required")?;
    let s: Scene = serde_json::from_reader(BufReader::new(File::open(path)?))?;
    let mut counts = BTreeMap::new();
    for i in s.meshes.instances {
        *counts.entry(i.asset).or_insert(0u64) += 1;
    }
    let mut rows=s.meshes.assets.into_iter().map(|a|serde_json::json!({"asset":a.id,"triangles":a.geometry.triangles,"instances":counts.get(&a.id).copied().unwrap_or(0),"logical_triangles":a.geometry.triangles as u64*counts.get(&a.id).copied().unwrap_or(0),"shading":a.shading,"coordinates":a.geometry.material_coordinates.is_some(),"rig":a.rig.is_some(),"material":a.material})).collect::<Vec<_>>();
    rows.sort_by_key(|r| std::cmp::Reverse(r["logical_triangles"].as_u64().unwrap()));
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
