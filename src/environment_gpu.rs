//! Fixed-size resources; pre-integration is lazy and keyed only by radiance content.
use crate::environment::{self, DIFFUSE_SIZE, Environment, LUT_SIZE, MIP_COUNT, SPECULAR_SIZE};

pub(crate) struct Gpu {
    specular: wgpu::Texture,
    diffuse: wgpu::Texture,
    lut: wgpu::Texture,
    pub specular_view: wgpu::TextureView,
    pub diffuse_view: wgpu::TextureView,
    pub lut_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    key: Option<environment::Key>,
    lut_ready: bool,
    bakes: u64,
    lut_bakes: u64,
}
fn texture(device: &wgpu::Device, label: &str, size: u32, layers: u32, mips: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
fn upload(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    level: u32,
    size: u32,
    layers: u32,
    bytes: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: level,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size * 4),
            rows_per_image: Some(size),
        },
        wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        },
    );
}
impl Gpu {
    pub fn new(device: &wgpu::Device) -> Self {
        let specular = texture(
            device,
            "prefiltered procedural environment",
            SPECULAR_SIZE,
            6,
            MIP_COUNT,
        );
        let diffuse = texture(device, "procedural diffuse irradiance", DIFFUSE_SIZE, 6, 1);
        let lut = texture(device, "packed environment DFG", LUT_SIZE, 1, 1);
        let cube_view = |t: &wgpu::Texture| {
            t.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            })
        };
        let specular_view = cube_view(&specular);
        let diffuse_view = cube_view(&diffuse);
        let lut_view = lut.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear environment sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            specular,
            diffuse,
            lut,
            specular_view,
            diffuse_view,
            lut_view,
            sampler,
            key: None,
            lut_ready: false,
            bakes: 0,
            lut_bakes: 0,
        }
    }
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"probe_bakes":self.bakes,"lut_bakes":self.lut_bakes,"logical_texture_bytes":environment::TEXTURE_BYTES})
    }
    pub fn prepare(&mut self, queue: &wgpu::Queue, environment: Option<&Environment>) {
        let Some(e) = environment.filter(|e| e.intensity > 0.) else {
            return;
        };
        if !self.lut_ready {
            upload(queue, &self.lut, 0, LUT_SIZE, 1, &environment::bake_lut());
            self.lut_ready = true;
            self.lut_bakes = self.lut_bakes.saturating_add(1);
        }
        let key = e.key();
        if self.key == Some(key) {
            return;
        }
        let data = environment::bake(key);
        for (level, bytes) in data.specular.iter().enumerate() {
            upload(
                queue,
                &self.specular,
                level as u32,
                SPECULAR_SIZE >> level,
                6,
                bytes,
            );
        }
        upload(queue, &self.diffuse, 0, DIFFUSE_SIZE, 6, &data.diffuse);
        self.key = Some(key);
        self.bakes = self.bakes.saturating_add(1);
    }
}
