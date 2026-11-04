//! Full static meshes for the browser's native vs_shared_position_coordinates path.
use super::{Document, Shading};
use crate::model::{Result, invalid};
use glam::Vec3;
use serde_json::{Value, json};

pub const MAX_BYTES: usize = 192 * 1024 * 1024;

pub struct Packet {
    pub metadata: Value,
    pub data: Vec<u8>,
}

fn append<T: bytemuck::Pod>(out: &mut Vec<u8>, data: &[T]) -> usize {
    let offset = out.len();
    for word in bytemuck::cast_slice::<_, u8>(data).chunks_exact(4) {
        out.extend(u32::from_ne_bytes(word.try_into().unwrap()).to_le_bytes());
    }
    offset
}

pub fn prepare(document: &Document) -> Result<Packet> {
    document.validate()?;
    if document.instances.is_empty() || document.instances.len() > 128 {
        return Err(invalid("Mesh preview requires1..128 static instances"));
    }
    let mut sources = Vec::new();
    let mut estimated = 0usize;
    for instance in &document.instances {
        let asset = document.asset(&instance.asset)?;
        if asset.rig.is_some()
            || instance.playback.is_some()
            || instance.attachment.is_some()
            || asset.shading != Shading::Smooth
            || asset.geometry.material_coordinates.is_some()
            || asset.material.transparent()
        {
            return Err(invalid(
                "Mesh preview1 requires smooth static position-coordinate meshes",
            ));
        }
        estimated += asset.geometry.positions.len() * 24 + asset.geometry.triangles.len() * 12 + 80;
        sources.push(asset.material.clone());
    }
    if estimated > MAX_BYTES {
        return Err(invalid("Mesh preview geometry exceeds192MiB"));
    }
    let (materials, program) = crate::materials::recipe::pack(&sources);
    let mut data = Vec::with_capacity(estimated + materials.len() * 1392 + program.len() * 16);
    append(&mut data, &materials);
    let program_offset = append(&mut data, &program);
    let mut parts = Vec::new();
    let (mut low, mut high) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for (index, instance) in document.instances.iter().enumerate() {
        let asset = document.asset(&instance.asset)?;
        let geo = &asset.geometry;
        let pose = document.pose(&instance.id, 0)?;
        for p in &pose.positions {
            let p = Vec3::from(*p);
            low = low.min(p);
            high = high.max(p);
        }
        let mut normals = vec![Vec3::ZERO; geo.positions.len()];
        crate::mesh_normals::accumulate(&geo.triangles, &geo.positions, &mut normals);
        if normals.iter().any(|n| *n == Vec3::ZERO || !n.is_finite()) {
            return Err(invalid(
                "Indexed preview requires unambiguous native smooth normals",
            ));
        }
        let vertex_offset = data.len();
        for (p, n) in geo.positions.iter().zip(normals) {
            append(&mut data, &[*p, n.to_array()]);
        }
        let index_offset = append(&mut data, &geo.triangles);
        let instance_offset = append(&mut data, &pose.world_transform);
        append(&mut data, &instance.color);
        data.extend((index as u32).to_le_bytes());
        let mut portable_color = instance.color;
        if let Some(procedural) = &asset.material.procedural {
            let mut mean = [0.; 3];
            for y in 0..16 {
                for x in 0..16 {
                    let sample =
                        procedural.sample([(x as f32 + 0.5) / 16., (y as f32 + 0.5) / 16.], 0.);
                    for (value, sample) in mean.iter_mut().zip(sample.color_roughness) {
                        *value += sample / 256.;
                    }
                }
            }
            for i in 0..3 {
                portable_color[i] *= mean[i];
            }
        }
        parts.push(json!({"id":instance.id,"vertices":geo.positions.len(),"indices":geo.triangles.len()*3,
            "vertex_offset":vertex_offset,"index_offset":index_offset,"instance_offset":instance_offset,
            "matrix":pose.world_transform,"color":instance.color,"portable_color":portable_color,"material":index}));
    }
    if data.len() > MAX_BYTES {
        return Err(invalid("Mesh preview packet exceeds192MiB"));
    }
    Ok(Packet {
        metadata: json!({"version":1,"vertex_stride":24,"material_stride":1392,
        "material_bytes":program_offset,"program_offset":program_offset,"program_bytes":program.len()*16,
        "bounds":{"min":low.to_array(),"max":high.to_array()},"parts":parts,"quality":"full"}),
        data,
    })
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let mut metadata = serde_json::to_vec(&self.metadata).unwrap();
        while !metadata.len().is_multiple_of(4) {
            metadata.push(b' ');
        }
        let mut out = Vec::with_capacity(16 + metadata.len() + self.data.len());
        out.extend(b"IGMP");
        for value in [1, metadata.len() as u32, self.data.len() as u32] {
            out.extend(value.to_le_bytes());
        }
        out.extend(metadata);
        out.extend(&self.data);
        out
    }

    /// Portable full-geometry glTF2 export. Native procedural fields stay in the companion sources.
    pub fn glb(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        let (mut views, mut accessors, mut meshes, mut nodes, mut materials) =
            (vec![], vec![], vec![], vec![], vec![]);
        for part in self.metadata["parts"].as_array().unwrap() {
            let vertices = part["vertices"].as_u64().unwrap() as usize;
            let indices = part["indices"].as_u64().unwrap() as usize;
            let vo = part["vertex_offset"].as_u64().unwrap() as usize;
            let io = part["index_offset"].as_u64().unwrap() as usize;
            let base = binary.len();
            binary.extend_from_slice(&self.data[vo..vo + vertices * 24]);
            let index_base = binary.len();
            binary.extend_from_slice(&self.data[io..io + indices * 4]);
            let view = views.len();
            views.push(json!({"buffer":0,"byteOffset":base,"byteLength":vertices*24,"byteStride":24,"target":34962}));
            views.push(
                json!({"buffer":0,"byteOffset":index_base,"byteLength":indices*4,"target":34963}),
            );
            let (mut min, mut max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
            for vertex in self.data[vo..vo + vertices * 24].chunks_exact(24) {
                for i in 0..3 {
                    let v = f32::from_le_bytes(vertex[i * 4..i * 4 + 4].try_into().unwrap());
                    min[i] = min[i].min(v);
                    max[i] = max[i].max(v);
                }
            }
            let accessor = accessors.len();
            accessors.push(json!({"bufferView":view,"byteOffset":0,"componentType":5126,"count":vertices,"type":"VEC3","min":min,"max":max}));
            accessors.push(json!({"bufferView":view,"byteOffset":12,"componentType":5126,"count":vertices,"type":"VEC3"}));
            accessors.push(
                json!({"bufferView":view+1,"componentType":5125,"count":indices,"type":"SCALAR"}),
            );
            let i = meshes.len();
            let c = part["portable_color"].as_array().unwrap();
            materials.push(json!({"name":part["id"],"doubleSided":true,"pbrMetallicRoughness":{"baseColorFactor":[c[0],c[1],c[2],1.0],"metallicFactor":0,"roughnessFactor":0.7}}));
            meshes.push(json!({"name":part["id"],"primitives":[{"attributes":{"POSITION":accessor,"NORMAL":accessor+1},"indices":accessor+2,"material":i}]}));
            let matrix: Vec<Value> = part["matrix"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|c| c.as_array().unwrap().iter().cloned())
                .collect();
            nodes.push(json!({"name":part["id"],"mesh":i,"matrix":matrix}));
        }
        let mut description=serde_json::to_vec(&json!({"asset":{"version":"2.0","generator":"Ignis native mesh exporter","copyright":"Guilherme Oliveira / Zerobytes, CC BY 4.0"},
            "scene":0,"scenes":[{"nodes":(0..nodes.len()).collect::<Vec<_>>()}],"nodes":nodes,"meshes":meshes,"materials":materials,
            "buffers":[{"byteLength":binary.len()}],"bufferViews":views,"accessors":accessors})).unwrap();
        while !description.len().is_multiple_of(4) {
            description.push(b' ');
        }
        let mut out = Vec::new();
        out.extend(b"glTF");
        out.extend(2u32.to_le_bytes());
        out.extend(((28 + description.len() + binary.len()) as u32).to_le_bytes());
        out.extend((description.len() as u32).to_le_bytes());
        out.extend(b"JSON");
        out.extend(description);
        out.extend((binary.len() as u32).to_le_bytes());
        out.extend(b"BIN\0");
        out.extend(binary);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_packet_and_glb_preserve_native_geometry() {
        let document: Document = serde_json::from_value(json!({
            "assets":[{"id":"triangle","shading":"smooth","geometry":{
                "positions":[[0,0,0],[1,0,0],[0,1,0]],"triangles":[[0,1,2]]}}],
            "instances":[{"id":"test","asset":"triangle","position":[2,3,4],
                "rotation":[0,0,0,1],"scale":2,"color":[0.2,0.4,0.6]}]
        }))
        .unwrap();
        let packet = prepare(&document).unwrap();
        assert_eq!(packet.metadata["bounds"]["min"], json!([2., 3., 4.]));
        assert_eq!(packet.metadata["bounds"]["max"], json!([4., 5., 4.]));
        let offset = packet.metadata["parts"][0]["vertex_offset"]
            .as_u64()
            .unwrap() as usize;
        let index = packet.metadata["parts"][0]["index_offset"]
            .as_u64()
            .unwrap() as usize;
        assert_eq!(
            &packet.data[offset + 12..offset + 24],
            bytemuck::cast_slice::<_, u8>(&[0f32, 0., 1.])
        );
        assert_eq!(
            &packet.data[index..index + 12],
            bytemuck::cast_slice::<_, u8>(&[0u32, 1, 2])
        );
        let encoded = packet.encode();
        assert_eq!(&encoded[..4], b"IGMP");
        let glb = packet.glb();
        assert_eq!(&glb[..4], b"glTF");
        assert_eq!(
            u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize,
            glb.len()
        );
        let length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let description: Value = serde_json::from_slice(&glb[20..20 + length]).unwrap();
        assert_eq!(description["accessors"][0]["count"], 3);
        assert_eq!(
            &glb[28 + length..28 + length + 72],
            &packet.data[offset..offset + 72]
        );
        let mut invalid = document;
        invalid.assets[0].shading = Shading::Flat;
        assert!(prepare(&invalid).is_err());
        assert!(prepare(&Document::default()).is_err());
    }
}
