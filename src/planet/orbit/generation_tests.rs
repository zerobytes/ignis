use super::*;
fn split(a: &Atlas) -> BTreeSet<Tile> {
    let mut leaves: BTreeSet<_> = a.resources.keys().map(|k| k.tile).collect();
    let tile = *leaves.iter().find(|t| t.level < a.max_level).unwrap();
    leaves.remove(&tile);
    leaves.extend(tile.children());
    leaves
}
#[test]
fn planet_orbit_generation_batches_preserve_installed_atlas_and_match_eager_payload() {
    let (c, mut a) = super::atmospheric_geometry_tests::fixture(25_484_000.);
    a.settings.generation_tiles_per_update = 1;
    a.settings.generation_time_ms = 100.;
    let leaves = split(&a);
    let mut eager = generation::Pending::new(leaves.clone(), &a, a.anchor, &a.view, 0.);
    let mut cache = atmospheric_geometry::Cache::default();
    eager.step(&c.document, &c, &a, false, &mut cache).unwrap();
    let installed = a.resources.clone();
    let plan = generation::Pending::new(leaves, &a, a.anchor, &a.view, 0.);
    assert!(plan.remaining() > 1);
    a.pending = Some(plan);
    let mut updates = 0;
    loop {
        a.update_paced(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
        assert!(
            !a.forced_synchronous,
            "fixture must exercise the admitted queue: {}",
            a.inspect()
        );
        assert!(a.last_generated_tiles <= 1);
        updates += 1;
        if a.pending.is_none() {
            break;
        }
        assert_eq!(a.resources.len(), installed.len());
        assert!(
            a.resources
                .iter()
                .all(|(k, r)| Arc::ptr_eq(r, &installed[k])),
            "partial atlas must never be installed"
        );
        assert!(updates < 100);
    }
    assert!(updates > 1);
    assert_eq!(
        a.resources.keys().collect::<Vec<_>>(),
        eager.resources.keys().collect::<Vec<_>>()
    );
    for (key, r) in &a.resources {
        let expected = &eager.resources[key];
        assert_eq!(r.keys, expected.keys);
        assert_eq!(
            r.asset.geometry.positions,
            expected.asset.geometry.positions
        );
        assert_eq!(
            r.asset.geometry.triangles,
            expected.asset.geometry.triangles
        );
        assert_eq!(r.normals, expected.normals);
        assert_eq!(r.coordinates, expected.coordinates);
    }
}
#[test]
fn planet_orbit_generation_never_defers_expired_detail_and_explicit_set_cancels_queue() {
    let (c, mut a) = super::atmospheric_geometry_tests::fixture(25_484_000.);
    a.settings.generation_tiles_per_update = 1;
    a.pending = Some(generation::Pending::new(
        split(&a),
        &a,
        a.anchor,
        &a.view,
        0.,
    ));
    a.update_paced(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
    assert!(a.pending.is_some());
    // The explicit source/settings operation must replace the pending target,
    // rather than publish an obsolete plan after a zoom.
    a.settings.distance_meters = 4.;
    a.update(&c.document, &c, a.anchor, 1.62, 1.).unwrap();
    assert!(a.pending.is_none());
    let (_, mut old) = super::atmospheric_geometry_tests::fixture(25_484_000.);
    old.settings.distance_meters = 4.;
    old.update_paced(&c.document, &c, old.anchor, 1.62, 1.)
        .unwrap();
    assert!(
        old.forced_synchronous,
        "nearby detail expired, so deferring would degrade quality"
    );
    assert!(old.pending.is_none());
    assert_eq!(
        a.resources.keys().collect::<Vec<_>>(),
        old.resources.keys().collect::<Vec<_>>()
    );
    let (_, mut stale) = super::atmospheric_geometry_tests::fixture(25_484_000.);
    stale.settings.generation_tiles_per_update = 1;
    stale.pending = Some(generation::Pending::new(
        split(&stale),
        &stale,
        stale.anchor,
        &stale.view,
        0.,
    ));
    stale
        .update_paced(&c.document, &c, stale.anchor, 1.62, 1.)
        .unwrap();
    assert!(stale.pending.is_some());
    stale.settings.distance_meters = 4.;
    stale
        .update_paced(&c.document, &c, stale.anchor, 1.62, 1.)
        .unwrap();
    assert!(stale.forced_synchronous && stale.pending.is_none());
    assert_eq!(
        stale.resources.keys().collect::<Vec<_>>(),
        a.resources.keys().collect::<Vec<_>>(),
        "urgent fallback must finish the CURRENT view, never an obsolete queued view"
    );
}
