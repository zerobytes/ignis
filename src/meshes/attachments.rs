//! Authored hierarchy sized from the document; no physics or sensory parenting.
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Hierarchy {
    parents: Vec<Option<(usize, Option<usize>)>>,
    order: Vec<usize>,
    pub(super) assets: Vec<usize>,
}
impl Hierarchy {
    pub(super) fn new(document: &Document) -> Result<Self> {
        let count = document.instances.len();
        let instances: BTreeMap<_, _> = document
            .instances
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id.as_str(), i))
            .collect();
        let assets: BTreeMap<_, _> = document
            .assets
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id.as_str(), i))
            .collect();
        let mut out = Self {
            parents: vec![None; count],
            order: Vec::with_capacity(count),
            assets: Vec::with_capacity(count),
        };
        for (index, instance) in document.instances.iter().enumerate() {
            out.assets.push(
                *assets
                    .get(instance.asset.as_str())
                    .ok_or_else(|| invalid("Mesh asset not found"))?,
            );
            let Some(attachment) = &instance.attachment else {
                continue;
            };
            id(&attachment.parent)?;
            let parent = *instances
                .get(attachment.parent.as_str())
                .ok_or_else(|| invalid("Attachment parent instance not found"))?;
            let bone = attachment
                .bone
                .as_ref()
                .map(|name| {
                    id(name)?;
                    document.assets[assets[document.instances[parent].asset.as_str()]]
                        .rig
                        .as_ref()
                        .and_then(|r| r.bones.iter().position(|b| &b.name == name))
                        .ok_or_else(|| invalid("Attachment requires an existing named parent bone"))
                })
                .transpose()?;
            out.parents[index] = Some((parent, bone));
        }
        // Iterative traversal: an authored deep chain must not overflow the host stack.
        let mut states = vec![0u8; count];
        let mut path = Vec::new();
        for index in 0..count {
            if states[index] == 2 {
                continue;
            }
            path.clear();
            let mut cursor = Some(index);
            while let Some(i) = cursor {
                if states[i] == 1 {
                    return Err(invalid("Mesh attachment cycle"));
                }
                if states[i] == 2 {
                    break;
                }
                states[i] = 1;
                path.push(i);
                cursor = out.parents[i].map(|(parent, _)| parent);
            }
            for i in path.drain(..).rev() {
                states[i] = 2;
                out.order.push(i);
            }
        }
        let mut scales = vec![1.; count];
        for &index in &out.order {
            let inherited = out.parents[index].map_or(1., |(parent, _)| scales[parent]);
            scales[index] = inherited * f64::from(document.instances[index].scale);
            if !scales[index].is_finite() || !(f64::from(0.001f32)..=100.).contains(&scales[index])
            {
                return Err(invalid("Effective attachment scale must be0.001..100"));
            }
        }
        Ok(out)
    }
}

static EMPTY: Skeleton = Skeleton {
    clip_tick: None,
    posed: Vec::new(),
    skin: Vec::new(),
};
pub(super) struct Prepared {
    pub(super) worlds: Vec<Mat4>,
    skeletons: Vec<Option<Box<Skeleton>>>,
}
impl Prepared {
    pub(super) fn skeleton(&self, index: usize) -> &Skeleton {
        self.skeletons[index].as_deref().unwrap_or(&EMPTY)
    }
    pub(super) fn new(document: &Document, hierarchy: &Hierarchy, tick: u64) -> Self {
        let mut out = Self {
            worlds: vec![Mat4::IDENTITY; document.instances.len()],
            skeletons: (0..document.instances.len()).map(|_| None).collect(),
        };
        for &index in &hierarchy.order {
            let i = &document.instances[index];
            let asset = &document.assets[hierarchy.assets[index]];
            if asset.rig.is_some() {
                out.skeletons[index] = Some(Box::new(skeleton(asset, i, tick)));
            }
            let local = transform(i.position, i.rotation) * Mat4::from_scale(Vec3::splat(i.scale));
            out.worlds[index] = hierarchy.parents[index].map_or(local, |(parent, bone)| {
                let anchor = bone.map_or(Mat4::IDENTITY, |bone| out.skeleton(parent).posed[bone]);
                out.worlds[parent] * anchor * local
            });
        }
        out
    }
}
