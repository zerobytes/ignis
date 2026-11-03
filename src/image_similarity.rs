//! Bounded, stateless comparison of aligned encoded RGB8 images.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Cursor};

pub const MAX_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 2048;
const WINDOW: usize = 11;
const TILE: u32 = 32;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 3]>,
}
impl Image {
    fn validate(&self) -> Result<()> {
        if self.width == 0
            || self.height == 0
            || self.width > MAX_DIMENSION
            || self.height > MAX_DIMENSION
            || self.pixels.len() != self.width as usize * self.height as usize
        {
            return Err(invalid(
                "Comparison images require dimensions1..2048 and complete RGB8 pixels",
            ));
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("Comparison PNG exceeds32 MiB"));
        }
        let error = |e| invalid(format!("Invalid comparison PNG: {e}"));
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        decoder.set_limits(png::Limits {
            bytes: 64 * 1024 * 1024,
        });
        decoder.set_transformations(png::Transformations::EXPAND);
        let mut reader = decoder.read_info().map_err(error)?;
        let info = reader.info();
        if info.width == 0
            || info.height == 0
            || info.width > MAX_DIMENSION
            || info.height > MAX_DIMENSION
            || info.bit_depth == png::BitDepth::Sixteen
            || info.animation_control.is_some()
        {
            return Err(invalid(
                "Comparison requires static PNG up to2048x2048, without16-bit samples",
            ));
        }
        let mut buffer = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or_else(|| invalid("PNG output overflow"))?
        ];
        let output = reader.next_frame(&mut buffer).map_err(error)?;
        reader.finish().map_err(error)?;
        let channels = output.color_type.samples();
        let mut pixels = Vec::with_capacity(output.width as usize * output.height as usize);
        for p in buffer[..output.buffer_size()].chunks_exact(channels) {
            let rgb = match output.color_type {
                png::ColorType::Rgb => [p[0], p[1], p[2]],
                png::ColorType::Rgba if p[3] == 255 => [p[0], p[1], p[2]],
                png::ColorType::Grayscale => [p[0]; 3],
                png::ColorType::GrayscaleAlpha if p[1] == 255 => [p[0]; 3],
                _ => {
                    return Err(invalid(
                        "Comparison requires opaque RGB8/grayscale PNG; composite alpha explicitly",
                    ));
                }
            };
            pixels.push(rgb);
        }
        let image = Self {
            width: output.width,
            height: output.height,
            pixels,
        };
        image.validate()?;
        Ok(image)
    }
    pub fn mask(&self, width: u32, height: u32) -> Result<Vec<bool>> {
        self.validate()?;
        if self.width != width || self.height != height {
            return Err(invalid("Comparison mask must match image dimensions"));
        }
        self.pixels
            .iter()
            .map(|p| match *p {
                [0, 0, 0] => Ok(false),
                [255, 255, 255] => Ok(true),
                _ => Err(invalid(
                    "Comparison mask must contain only black0 and white255",
                )),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl Region {
    fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
    fn contains_window(&self, x: u32, y: u32) -> bool {
        self.contains(x, y) && self.contains(x + WINDOW as u32 - 1, y + WINDOW as u32 - 1)
    }
}
pub fn validate_regions(regions: &[Region], width: u32, height: u32) -> Result<()> {
    let mut ids = BTreeSet::new();
    if regions.len() > 16 {
        return Err(invalid("Comparison supports at most16 regions"));
    }
    for r in regions {
        if r.id.is_empty()
            || r.id.len() > 64
            || !r
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            || !ids.insert(&r.id)
            || r.width == 0
            || r.height == 0
            || r.x.checked_add(r.width).is_none_or(|x| x > width)
            || r.y.checked_add(r.height).is_none_or(|y| y > height)
        {
            return Err(invalid(
                "Comparison regions require unique ASCII IDs and nonempty in-bounds rectangles",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub selected_pixels: u32,
    pub coverage_fraction: f64,
    pub changed_pixels: u32,
    pub exact_rgb_match: bool,
    pub rgb_mae: f64,
    pub rgb_rmse: f64,
    pub max_channel_error_u8: u8,
    pub ssim_luma: Option<f64>,
    pub ssim_windows: u32,
}
#[derive(Debug, Clone, Serialize)]
pub struct RegionReport {
    pub region: Region,
    pub metrics: Metrics,
}
#[derive(Debug, Clone, Serialize)]
pub struct TileReport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub selected_pixels: u32,
    pub rgb_mae: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub color_policy: &'static str,
    pub ssim_definition: &'static str,
    pub alignment: &'static str,
    pub mask_applied: bool,
    pub metrics: Metrics,
    pub regions: Vec<RegionReport>,
    pub worst_tiles: Vec<TileReport>,
    pub difference_definition: &'static str,
}
#[derive(Clone, Default)]
struct Accumulator {
    pixels: u32,
    changed: u32,
    abs: u64,
    square: u64,
    max: u8,
    ssim: f64,
    windows: u32,
}
impl Accumulator {
    fn add(&mut self, errors: [u8; 3]) {
        self.pixels += 1;
        self.changed += u32::from(errors != [0; 3]);
        for e in errors {
            self.abs += u64::from(e);
            self.square += u64::from(e) * u64::from(e);
            self.max = self.max.max(e);
        }
    }
    fn finish(&self, area: u32) -> Result<Metrics> {
        if self.pixels == 0 {
            return Err(invalid(
                "Comparison selection/region contains no selected pixels",
            ));
        }
        let channels = f64::from(self.pixels) * 3.;
        Ok(Metrics {
            selected_pixels: self.pixels,
            coverage_fraction: f64::from(self.pixels) / f64::from(area),
            changed_pixels: self.changed,
            exact_rgb_match: self.changed == 0,
            rgb_mae: self.abs as f64 / (channels * 255.),
            rgb_rmse: (self.square as f64 / channels).sqrt() / 255.,
            max_channel_error_u8: self.max,
            ssim_luma: (self.windows > 0)
                .then(|| (self.ssim / f64::from(self.windows)).clamp(-1., 1.)),
            ssim_windows: self.windows,
        })
    }
}
fn moments(a: [u8; 3], b: [u8; 3], selected: bool) -> [f64; 6] {
    let luma = |p: [u8; 3]| {
        (0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2])) / 255.
    };
    let a = luma(a);
    let b = luma(b);
    [a, b, a * a, b * b, a * b, f64::from(selected)]
}
fn ssim(s: [f64; 6]) -> f64 {
    let n = (WINDOW * WINDOW) as f64;
    let a = s[0] / n;
    let b = s[1] / n;
    let va = (s[2] / n - a * a).max(0.);
    let vb = (s[3] / n - b * b).max(0.);
    let cov = s[4] / n - a * b;
    ((2. * a * b + 0.0001) * (2. * cov + 0.0009) / ((a * a + b * b + 0.0001) * (va + vb + 0.0009)))
        .clamp(-1., 1.)
}

pub struct Comparison {
    pub report: Report,
    pub difference_png: Option<Vec<u8>>,
}
pub fn compare(
    a: &Image,
    b: &Image,
    mask: Option<&[bool]>,
    regions: &[Region],
    difference: bool,
) -> Result<Comparison> {
    a.validate()?;
    b.validate()?;
    if a.width != b.width || a.height != b.height {
        return Err(invalid(
            "Comparison images must have identical dimensions; no implicit resize/alignment",
        ));
    }
    validate_regions(regions, a.width, a.height)?;
    if mask.is_some_and(|m| m.len() != a.pixels.len()) {
        return Err(invalid("Comparison mask size mismatch"));
    }
    let (w, h) = (a.width as usize, a.height as usize);
    let columns = w.saturating_sub(WINDOW - 1);
    let mut ring = vec![[0.; 6]; columns * WINDOW];
    let mut vertical = vec![[0.; 6]; columns];
    let mut global = Accumulator::default();
    let mut parts = vec![Accumulator::default(); regions.len()];
    let tile_cols = a.width.div_ceil(TILE);
    let mut tiles = vec![Accumulator::default(); (tile_cols * a.height.div_ceil(TILE)) as usize];
    let mut map = difference.then(|| vec![0; w * h]);
    for y in 0..h {
        let mut horizontal = [0.; 6];
        let mut row = [[0.; 6]; WINDOW];
        for x in 0..w {
            let i = y * w + x;
            let selected = mask.is_none_or(|m| m[i]);
            let values = moments(a.pixels[i], b.pixels[i], selected);
            for (j, total) in horizontal.iter_mut().enumerate() {
                *total += values[j] - row[x % WINDOW][j];
            }
            row[x % WINDOW] = values;
            if selected {
                let errors = std::array::from_fn(|c| a.pixels[i][c].abs_diff(b.pixels[i][c]));
                global.add(errors);
                for (r, part) in regions.iter().zip(&mut parts) {
                    if r.contains(x as u32, y as u32) {
                        part.add(errors);
                    }
                }
                tiles[((y as u32 / TILE) * tile_cols + x as u32 / TILE) as usize].add(errors);
                if let Some(map) = &mut map {
                    map[i] =
                        ((u32::from(errors[0]) + u32::from(errors[1]) + u32::from(errors[2]) + 1)
                            / 3) as u8;
                }
            }
            if x + 1 < WINDOW {
                continue;
            }
            let col = x + 1 - WINDOW;
            let index = (y % WINDOW) * columns + col;
            for (j, total) in vertical[col].iter_mut().enumerate() {
                *total += horizontal[j] - ring[index][j];
            }
            ring[index] = horizontal;
            if y + 1 < WINDOW || vertical[col][5] != (WINDOW * WINDOW) as f64 {
                continue;
            }
            let value = ssim(vertical[col]);
            global.ssim += value;
            global.windows += 1;
            for (r, part) in regions.iter().zip(&mut parts) {
                if r.contains_window(col as u32, (y + 1 - WINDOW) as u32) {
                    part.ssim += value;
                    part.windows += 1;
                }
            }
        }
    }
    let metrics = global.finish(a.width * a.height)?;
    let regions = regions
        .iter()
        .zip(parts)
        .map(|(r, p)| {
            Ok(RegionReport {
                region: r.clone(),
                metrics: p.finish(r.width * r.height)?,
            })
        })
        .collect::<Result<_>>()?;
    let mut worst_tiles: Vec<_> = tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.pixels > 0)
        .map(|(i, t)| TileReport {
            x: (i as u32 % tile_cols) * TILE,
            y: (i as u32 / tile_cols) * TILE,
            width: TILE.min(a.width - (i as u32 % tile_cols) * TILE),
            height: TILE.min(a.height - (i as u32 / tile_cols) * TILE),
            selected_pixels: t.pixels,
            rgb_mae: t.abs as f64 / (f64::from(t.pixels) * 3. * 255.),
        })
        .collect();
    worst_tiles.sort_by(|a, b| {
        b.rgb_mae
            .total_cmp(&a.rgb_mae)
            .then(a.y.cmp(&b.y))
            .then(a.x.cmp(&b.x))
    });
    worst_tiles.truncate(8);
    let difference_png = map
        .map(|map| {
            let mut bytes = Vec::new();
            let mut encoder = png::Encoder::new(&mut bytes, a.width, a.height);
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .map_err(|e| invalid(e.to_string()))?
                .write_image_data(&map)
                .map_err(|e| invalid(e.to_string()))?;
            Ok(bytes)
        })
        .transpose()?;
    Ok(Comparison {
        report: Report {
            version: 1,
            width: a.width,
            height: a.height,
            color_policy: "stored_encoded_rgb8_no_profile_gamma_or_exposure_transform_opaque",
            ssim_definition: "luma_rec709_encoded_0to1_uniform11x11_population_C1_0.0001_C2_0.0009_valid_full_mask_windows",
            alignment: "none_identical_pixel_grid_required",
            mask_applied: mask.is_some(),
            metrics,
            regions,
            worst_tiles,
            difference_definition: "mean_absolute_rgb_u8_rounded_grayscale_0equal_255max_masked_black",
        },
        difference_png,
    })
}
