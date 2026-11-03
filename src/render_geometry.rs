//! Lossless indexing at immutable geometry upload time. Every vertex attribute
//! participates in identity; seams, flat normals and primitive order survive.
use std::collections::BTreeMap;
use wgpu::util::DeviceExt;

pub(crate) struct Page {
    pub vertices: wgpu::Buffer,
    pub indices: Option<Index>,
    pub count: u32,
    pub vertex_count: usize,
}
pub(crate) struct Index {
    pub buffer: wgpu::Buffer,
    pub format: wgpu::IndexFormat,
}
impl Page {
    pub fn bytes(&self) -> usize {
        self.vertices.size() as usize
            + self
                .indices
                .as_ref()
                .map_or(0, |b| b.buffer.size() as usize)
    }
}

pub(crate) fn upload<const WORDS: usize>(
    device: &wgpu::Device,
    bytes: &[u8],
    indexed: bool,
) -> Vec<Page> {
    let stride = WORDS * 4;
    let source: &[[u32; WORDS]] = bytemuck::cast_slice(bytes);
    let max_bytes = device.limits().max_buffer_size as usize;
    let per_page = (max_bytes / stride).min(u32::MAX as usize) / 3 * 3;
    if !indexed {
        return source.chunks(per_page).map(|v| plain(device, v)).collect();
    }
    // Pages follow device limits. Use u16 when possible, u32 for larger meshes;
    // an index-width choice must not split a normal asset into many draws.
    let max_vertices = (max_bytes / stride).min(u32::MAX as usize);
    let max_indices = (max_bytes / 4).min(u32::MAX as usize) / 3 * 3;
    let mut pages = Vec::new();
    let mut processed = 0;
    let mut pending_plain = None;
    partition(
        source,
        max_vertices,
        max_indices,
        |source, vertices, indices| {
            let indexed_bytes = std::mem::size_of_val(vertices)
                + (indices.len() * index_stride(vertices.len())).next_multiple_of(4);
            if indexed_bytes < std::mem::size_of_val(source) {
                if let Some(start) = pending_plain.take() {
                    let all: &[[u32; WORDS]] = bytemuck::cast_slice(bytes);
                    pages.extend(
                        all[start..processed]
                            .chunks(per_page)
                            .map(|v| plain(device, v)),
                    );
                }
                pages.push(finish(device, source, vertices, indices));
            } else {
                pending_plain.get_or_insert(processed);
            }
            processed += source.len();
        },
    );
    if let Some(start) = pending_plain {
        pages.extend(source[start..].chunks(per_page).map(|v| plain(device, v)));
    }
    pages
}

// IGL1 already contains the index topology. Consume it directly; no runtime
// expansion followed by hashing/deduplication of the same compiled geometry.
pub(crate) fn compiled(
    device: &wgpu::Device,
    vertices: &[u8],
    indices: &[[u16; 3]],
    vertex_count: usize,
) -> Page {
    Page {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("compiled LOD vertices"),
            contents: vertices,
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: Some(Index {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("compiled LOD u16 indices"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            format: wgpu::IndexFormat::Uint16,
        }),
        count: (indices.len() * 3) as u32,
        vertex_count,
    }
}

fn partition<const WORDS: usize>(
    source: &[[u32; WORDS]],
    max_vertices: usize,
    max_indices: usize,
    mut emit: impl FnMut(&[[u32; WORDS]], &[[u32; WORDS]], &[u32]),
) {
    debug_assert!((3..=u32::MAX as usize).contains(&max_vertices));
    debug_assert!(max_indices >= 3 && source.len().is_multiple_of(3));
    let mut unique = BTreeMap::<[u32; WORDS], u32>::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut start = 0;
    for (n, triangle) in source.chunks_exact(3).enumerate() {
        // Reserve for the worst case before admitting an entire triangle.
        if vertices.len() + 3 > max_vertices || indices.len() + 3 > max_indices {
            emit(&source[start..n * 3], &vertices, &indices);
            start = n * 3;
            unique.clear();
            vertices.clear();
            indices.clear();
        }
        for &vertex in triangle {
            let index = *unique.entry(vertex).or_insert_with(|| {
                let index = vertices.len() as u32;
                vertices.push(vertex);
                index
            });
            indices.push(index);
        }
    }
    if !indices.is_empty() {
        emit(&source[start..], &vertices, &indices);
    }
}

fn plain<const WORDS: usize>(device: &wgpu::Device, vertices: &[[u32; WORDS]]) -> Page {
    Page {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shared geometry vertices"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: None,
        count: vertices.len() as u32,
        vertex_count: vertices.len(),
    }
}

fn finish<const WORDS: usize>(
    device: &wgpu::Device,
    source: &[[u32; WORDS]],
    vertices: &[[u32; WORDS]],
    indices: &[u32],
) -> Page {
    let index_bytes = (indices.len() * index_stride(vertices.len())).next_multiple_of(4);
    if std::mem::size_of_val(vertices) + index_bytes >= std::mem::size_of_val(source) {
        return plain(device, source);
    }
    let mut page = plain(device, vertices);
    let short;
    let (bytes, format) = if index_stride(vertices.len()) == 2 {
        short = indices.iter().map(|&i| i as u16).collect::<Vec<_>>();
        (bytemuck::cast_slice(&short), wgpu::IndexFormat::Uint16)
    } else {
        (bytemuck::cast_slice(indices), wgpu::IndexFormat::Uint32)
    };
    page.indices = Some(Index {
        buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shared geometry indices"),
            contents: bytes,
            usage: wgpu::BufferUsages::INDEX,
        }),
        format,
    });
    page.count = indices.len() as u32;
    page
}

fn index_stride(vertices: usize) -> usize {
    if vertices <= u16::MAX as usize { 2 } else { 4 }
}

#[cfg(test)]
mod tests {
    use super::partition;
    // Incorrect page remapping can silently draw arbitrary geometry or erase
    // material seams. Compare every attribute bit and primitive in source order.
    #[test]
    fn page_boundaries_reconstruct_all_attributes_and_primitive_order() {
        let mut source = Vec::new();
        for n in 0..24_000u32 {
            for corner in 0..3u32 {
                let mut vertex = [n * 3 + corner; 16];
                vertex[0] = n / 2; // shared position is NOT a shared full vertex
                vertex[4] = (n % 2) << 31; // includes positive/negative zero bits
                source.push(vertex);
            }
            if n % 5 == 0 {
                source.extend_from_within(source.len() - 3..);
            }
        }
        for (vertices, indices) in [
            (7, 9),
            (u16::MAX as usize, 300_000),
            (u32::MAX as usize, 300_000),
        ] {
            let mut reconstructed = Vec::new();
            let mut pages = 0;
            partition(&source, vertices, indices, |original, unique, remap| {
                assert!(unique.len() <= vertices && remap.len() <= indices);
                let expanded: Vec<_> = remap.iter().map(|&i| unique[i as usize]).collect();
                assert_eq!(expanded, original);
                reconstructed.extend(expanded);
                pages += 1;
            });
            if vertices == u32::MAX as usize {
                assert_eq!(pages, 1);
            } else {
                assert!(pages > 1);
            }
            assert_eq!(reconstructed, source);
        }
    }
}
