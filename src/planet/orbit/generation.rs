//! Bounded batches behind an unchanged, closed current atlas. A candidate is
//! published only after every resource (including conforming seams) is ready.
use super::*;
pub(super) struct Pending {
    pub resources: BTreeMap<ResourceKey, Arc<Resource>>,
    missing: Vec<ResourceKey>,
    registry: topology::Registry,
    pub anchor: Anchor,
    pub eye: DVec3,
    view: View,
    pub settings: Settings,
    pub maximum: f64,
    pub born: u64,
    pub sample_hits: u64,
    pub age: u64,
    count: usize,
}
impl Pending {
    pub fn new(
        leaves: BTreeSet<Tile>,
        atlas: &Atlas,
        anchor: Anchor,
        view: &View,
        maximum: f64,
    ) -> Self {
        let registry = topology::Registry::new(&leaves, atlas.max_level, atlas.source.grid);
        let mut resources = BTreeMap::new();
        let mut missing = vec![];
        let mut count = 0;
        for tile in leaves {
            let key = ResourceKey {
                tile,
                grid: atlas.source.grid,
                signature: registry.signature(tile),
            };
            if let Some(r) = atlas.resources.get(&key) {
                count += r.asset.geometry.positions.len();
                resources.insert(key, r.clone());
            } else {
                missing.push(key);
            }
        }
        // Pop in the same deterministic order as the synchronous atlas builder.
        missing.reverse();
        Self {
            resources,
            missing,
            registry,
            anchor,
            eye: observer(anchor, view),
            view: view.clone(),
            settings: atlas.settings.clone(),
            maximum,
            born: 0,
            sample_hits: 0,
            age: 0,
            count,
        }
    }
    pub fn remaining(&self) -> usize {
        self.missing.len()
    }
    pub fn outdated(&self, settings: &Settings, anchor: Anchor, view: &View) -> bool {
        self.settings != *settings
            || self.eye != observer(anchor, view)
            || self.anchor.basis != anchor.basis
    }
    pub fn reuse(&mut self, prepared: &BTreeMap<ResourceKey, Arc<Resource>>) -> Result<()> {
        self.missing.retain(|key| {
            if let Some(resource) = prepared.get(key) {
                self.count += resource.asset.geometry.positions.len();
                self.born += 1;
                self.sample_hits += resource.sample_hits as u64;
                self.resources.insert(key.clone(), resource.clone());
                false
            } else {
                true
            }
        });
        if self.count > 2_000_000 {
            return Err(invalid(
                "Planet visual atlas exceeds2M vertices; previous coverage is retained",
            ));
        }
        Ok(())
    }
    pub fn step(
        &mut self,
        doc: &Document,
        config: &super::super::runtime::Config,
        atlas: &Atlas,
        paced: bool,
        cache: &mut atmospheric_geometry::Cache,
    ) -> Result<usize> {
        self.age += 1;
        let started = std::time::Instant::now();
        let previous: BTreeMap<_, _> = atlas.resources.iter().map(|(k, r)| (k.tile, r)).collect();
        let mut built = 0;
        while let Some(key) = self.missing.last() {
            if paced
                && built > 0
                && (built >= atlas.settings.generation_tiles_per_update
                    || started.elapsed().as_secs_f64() * 1000. >= atlas.settings.generation_time_ms)
            {
                break;
            }
            let resource = topology::resource_prepared(
                doc,
                config,
                key,
                &self.registry,
                atlas.max_level,
                self.settings
                    .reuse_vertex_samples
                    .then(|| previous.get(&key.tile).map(|r| r.as_ref()))
                    .flatten(),
                atlas.source.sampler(doc),
            )?;
            self.count += resource.asset.geometry.positions.len();
            if self.count > 2_000_000 {
                return Err(invalid(
                    "Planet visual atlas exceeds2M vertices; previous coverage is retained",
                ));
            }
            self.sample_hits += resource.sample_hits as u64;
            self.born += 1;
            built += 1;
            let key = self.missing.pop().unwrap();
            let resource = Arc::new(resource);
            if self.settings.atmosphere_geometry && self.settings.atmosphere_optimization {
                cache.prepare(
                    (&key, &resource),
                    config,
                    &self.settings,
                    &atlas.light,
                    atlas.source.upper,
                    (self.anchor, &self.view),
                );
            }
            self.resources.insert(key, resource);
        }
        Ok(built)
    }
}
