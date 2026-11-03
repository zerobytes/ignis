use ignis::{
    engine::{Command, Request},
    image_similarity::{self as sim, Image, Region},
    model::RoomRecipe,
    service::Service,
};

fn image(w: u32, h: u32, color: [u8; 3]) -> Image {
    Image {
        width: w,
        height: h,
        pixels: vec![color; (w * h) as usize],
    }
}
fn region(id: &str, x: u32, y: u32, width: u32, height: u32) -> Region {
    Region {
        id: id.into(),
        x,
        y,
        width,
        height,
    }
}
fn png(w: u32, h: u32, color: png::ColorType, depth: png::BitDepth, data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, w, h);
    encoder.set_color(color);
    encoder.set_depth(depth);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(data)
        .unwrap();
    bytes
}
fn save(path: &std::path::Path, image: &Image) {
    let data: Vec<_> = image.pixels.iter().flatten().copied().collect();
    std::fs::write(
        path,
        png(
            image.width,
            image.height,
            png::ColorType::Rgb,
            png::BitDepth::Eight,
            &data,
        ),
    )
    .unwrap();
}
#[test]
fn identity_and_constant_error_have_analytic_results() {
    let a = image(32, 24, [0; 3]);
    let b = image(32, 24, [255; 3]);
    let same = sim::compare(&a, &a, None, &[], true).unwrap();
    assert!(same.report.metrics.exact_rgb_match);
    assert_eq!(same.report.metrics.rgb_mae, 0.);
    assert_eq!(same.report.metrics.ssim_windows, 22 * 14);
    assert!((same.report.metrics.ssim_luma.unwrap() - 1.).abs() < 1e-12);
    assert!(
        Image::decode(same.difference_png.as_ref().unwrap())
            .unwrap()
            .pixels
            .iter()
            .all(|p| *p == [0; 3])
    );
    let report = sim::compare(&a, &b, None, &[], true).unwrap();
    let m = report.report.metrics;
    assert_eq!(m.rgb_mae, 1.);
    assert_eq!(m.rgb_rmse, 1.);
    assert_eq!(m.max_channel_error_u8, 255);
    assert_eq!(m.changed_pixels, 32 * 24);
    assert!((m.ssim_luma.unwrap() - 0.0001 / 1.0001).abs() < 1e-12);
    assert!(
        Image::decode(report.difference_png.as_ref().unwrap())
            .unwrap()
            .pixels
            .iter()
            .all(|p| *p == [255; 3])
    );
}

// Independent reference: direct11x11 windows and deviations from the mean,
// rather than the production sliding sums / E[x²]-E[x]² implementation.
fn brute_ssim(a: &Image, b: &Image, r: &Region, mask: Option<&[bool]>) -> Option<f64> {
    if r.width < 11 || r.height < 11 {
        return None;
    }
    let luma = |p: [u8; 3]| {
        (f64::from(p[0]) * 0.2126 + f64::from(p[1]) * 0.7152 + f64::from(p[2]) * 0.0722) / 255.
    };
    let mut sum = 0.;
    let mut count = 0;
    for y in r.y..=r.y + r.height - 11 {
        for x in r.x..=r.x + r.width - 11 {
            let mut pairs = Vec::new();
            for dy in 0..11 {
                for dx in 0..11 {
                    let i = ((y + dy) * a.width + x + dx) as usize;
                    if mask.is_none_or(|m| m[i]) {
                        pairs.push((luma(a.pixels[i]), luma(b.pixels[i])));
                    }
                }
            }
            if pairs.len() != 121 {
                continue;
            }
            let ma = pairs.iter().map(|p| p.0).sum::<f64>() / 121.;
            let mb = pairs.iter().map(|p| p.1).sum::<f64>() / 121.;
            let va = pairs.iter().map(|p| (p.0 - ma).powi(2)).sum::<f64>() / 121.;
            let vb = pairs.iter().map(|p| (p.1 - mb).powi(2)).sum::<f64>() / 121.;
            let cov = pairs.iter().map(|p| (p.0 - ma) * (p.1 - mb)).sum::<f64>() / 121.;
            sum += (2. * ma * mb + 0.0001) * (2. * cov + 0.0009)
                / ((ma * ma + mb * mb + 0.0001) * (va + vb + 0.0009));
            count += 1;
        }
    }
    (count > 0).then(|| sum / f64::from(count))
}
#[test]
fn sliding_statistics_match_independent_reference_with_mask_and_region_borders() {
    let mut a = image(37, 31, [0; 3]);
    let mut b = a.clone();
    for (i, (a, b)) in a.pixels.iter_mut().zip(&mut b.pixels).enumerate() {
        *a = [
            (i * 71 % 256) as u8,
            (i * 23 % 256) as u8,
            (i * 11 % 256) as u8,
        ];
        *b = [
            (i * 73 % 256) as u8,
            (i * 19 % 256) as u8,
            (i * 7 % 256) as u8,
        ];
    }
    let r = region("middle", 3, 5, 25, 22);
    let mut mask = vec![true; a.pixels.len()];
    mask[0] = false;
    mask[36] = false;
    mask[20 * 37 + 25] = false;
    for mask in [None, Some(mask.as_slice())] {
        let report = sim::compare(&a, &b, mask, std::slice::from_ref(&r), false)
            .unwrap()
            .report;
        let reference = brute_ssim(&a, &b, &region("all", 0, 0, 37, 31), mask).unwrap();
        assert!((report.metrics.ssim_luma.unwrap() - reference).abs() < 1e-10);
        assert!(
            (report.regions[0].metrics.ssim_luma.unwrap() - brute_ssim(&a, &b, &r, mask).unwrap())
                .abs()
                < 1e-10
        );
    }
}
#[test]
fn regions_expose_errors_hidden_by_background_and_masks_have_exact_support() {
    let a = image(256, 256, [255; 3]);
    let mut b = a.clone();
    for y in 100..116 {
        for x in 100..116 {
            b.pixels[y * 256 + x] = [0; 3];
        }
    }
    let r = region("door", 100, 100, 16, 16);
    let report = sim::compare(&a, &b, None, std::slice::from_ref(&r), false)
        .unwrap()
        .report;
    assert!(report.metrics.rgb_mae < 0.004);
    assert_eq!(report.regions[0].metrics.rgb_mae, 1.);
    assert_eq!(report.regions[0].metrics.ssim_windows, 36);
    assert_eq!(report.worst_tiles[0].x, 96);
    assert_eq!(report.worst_tiles[0].y, 96);
    let mut mask = vec![true; a.pixels.len()];
    for y in 100..116 {
        for x in 100..116 {
            mask[y * 256 + x] = false;
        }
    }
    let report = sim::compare(&a, &b, Some(&mask), &[], true).unwrap();
    assert!(report.report.metrics.exact_rgb_match);
    assert_eq!(report.report.metrics.selected_pixels, 256 * 256 - 16 * 16);
    assert!(
        Image::decode(report.difference_png.as_ref().unwrap())
            .unwrap()
            .pixels
            .iter()
            .all(|p| *p == [0; 3])
    );
    assert!(sim::compare(&a, &b, Some(&mask), &[r], false).is_err());
}
#[test]
fn translated_pattern_changes_structure_and_ssim_can_be_negative() {
    let mut a = image(32, 32, [0; 3]);
    let mut b = a.clone();
    for y in 0..32 {
        for x in 0..32 {
            a.pixels[y * 32 + x] = [if (x + y) % 2 == 0 { 0 } else { 255 }; 3];
            b.pixels[y * 32 + x] = [255 - a.pixels[y * 32 + x][0]; 3];
        }
    }
    let report = sim::compare(&a, &b, None, &[], false).unwrap().report;
    assert!(report.metrics.ssim_luma.unwrap() < 0.);
    assert_eq!(report.metrics.rgb_mae, 1.);
    let repeated = sim::compare(&a, &b, None, &[], false).unwrap().report;
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        serde_json::to_value(repeated).unwrap()
    );
}
#[test]
fn small_or_sparse_support_does_not_fabricate_ssim_and_empty_support_rejects() {
    let a = image(12, 12, [123; 3]);
    let mut mask = vec![false; 144];
    mask[25] = true;
    let report = sim::compare(&a, &a, Some(&mask), &[region("point", 1, 2, 1, 1)], false)
        .unwrap()
        .report;
    assert_eq!(report.metrics.ssim_luma, None);
    assert_eq!(report.regions[0].metrics.ssim_luma, None);
    assert_eq!(report.metrics.selected_pixels, 1);
    assert!(sim::compare(&a, &a, Some(&[false; 144]), &[], false).is_err());
    let small = image(1, 1, [0; 3]);
    assert_eq!(
        sim::compare(&small, &small, None, &[], false)
            .unwrap()
            .report
            .metrics
            .ssim_luma,
        None
    );
}
#[test]
fn png_decoding_expands_gray_palette_and_rejects_alpha_depth_and_invalid_inputs() {
    let gray = png(
        2,
        1,
        png::ColorType::Grayscale,
        png::BitDepth::Eight,
        &[0, 255],
    );
    assert_eq!(
        Image::decode(&gray).unwrap().mask(2, 1).unwrap(),
        [false, true]
    );
    let opaque = png(
        1,
        1,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[1, 2, 3, 255],
    );
    assert_eq!(Image::decode(&opaque).unwrap().pixels, [[1, 2, 3]]);
    let alpha = png(
        1,
        1,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[1, 2, 3, 254],
    );
    assert!(Image::decode(&alpha).is_err());
    let wide = png(
        1,
        1,
        png::ColorType::Grayscale,
        png::BitDepth::Sixteen,
        &[0, 0],
    );
    assert!(Image::decode(&wide).is_err());
    assert!(Image::decode(b"not png").is_err());
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 2, 1);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::One);
    encoder.set_palette(&[0, 0, 0, 255, 255, 255][..]);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&[0b01000000])
        .unwrap();
    assert_eq!(Image::decode(&bytes).unwrap().pixels, [[0; 3], [255; 3]]);
    assert!(image(1, 1, [12, 12, 12]).mask(1, 1).is_err());
    assert!(image(1, 1, [255; 3]).mask(2, 1).is_err());
}
#[test]
fn invalid_regions_dimensions_and_masks_reject() {
    let a = image(12, 12, [0; 3]);
    for r in [
        region("", 0, 0, 1, 1),
        region("bad", 0, 0, 0, 1),
        region("bad", 12, 0, 1, 1),
        region("bad", u32::MAX, 0, 2, 1),
    ] {
        assert!(sim::compare(&a, &a, None, &[r], false).is_err());
    }
    let r = region("same", 0, 0, 1, 1);
    assert!(sim::compare(&a, &a, None, &[r.clone(), r], false).is_err());
    assert!(sim::compare(&a, &a, Some(&[true]), &[], false).is_err());
    assert!(sim::compare(&a, &image(13, 12, [0; 3]), None, &[], false).is_err());
    let malformed = Image {
        width: 2049,
        height: 1,
        pixels: vec![],
    };
    assert!(sim::compare(&malformed, &malformed, None, &[], false).is_err());
}
fn command(reference: &str, candidate: &str, difference: Option<&str>) -> Command {
    Command::CompareImages {
        reference_path: reference.into(),
        candidate_path: candidate.into(),
        mask_path: None,
        regions: vec![],
        difference_path: difference.map(str::to_owned),
    }
}
#[test]
fn service_keeps_state_and_inputs_on_success_failure_alias_and_revision_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let a = image(16, 16, [12; 3]);
    save(&dir.path().join("a.png"), &a);
    save(&dir.path().join("b.png"), &a);
    let mut s = Service::new(dir.path(), RoomRecipe::default().generate().unwrap()).unwrap();
    let before = s.engine.snapshot();
    let reply = s
        .execute(command("a.png", "b.png", Some("results/diff.png")).into())
        .unwrap();
    assert_eq!(reply.data["report"]["metrics"]["exact_rgb_match"], true);
    assert_eq!(s.engine.snapshot(), before);
    let original = std::fs::read(dir.path().join("a.png")).unwrap();
    for c in [
        command("a.png", "b.png", Some("a.png")),
        command("../a.png", "b.png", None),
        command("a.png", "missing.png", Some("new/out.png")),
    ] {
        assert!(s.execute(c.into()).is_err());
        assert_eq!(s.engine.snapshot(), before);
    }
    assert!(!dir.path().join("new").exists());
    assert!(
        s.execute(Request {
            expected_revision: Some(999),
            command: command("a.png", "b.png", None)
        })
        .is_err()
    );
    assert_eq!(std::fs::read(dir.path().join("a.png")).unwrap(), original);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("a.png", dir.path().join("alias.png")).unwrap();
        assert!(
            s.execute(command("alias.png", "b.png", Some("a.png")).into())
                .is_err()
        );
        assert!(
            s.execute(command("alias.png", "b.png", Some("alias.png")).into())
                .is_err()
        );
        assert_eq!(std::fs::read(dir.path().join("a.png")).unwrap(), original);
    }
}
