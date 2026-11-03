//! Session-only component selection and geometric visibility shared by UI and MCP.
use crate::{
    engine::Engine,
    mesh_topology::Components,
    model::{Camera, Result, invalid},
};
use glam::{Mat4, Vec2, Vec3};
use rapier3d::{
    na::{Point3, Vector3},
    parry::{
        query::{Ray, RayCast},
        shape::TriMesh,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Vertices,
    Edges,
    Faces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionStatus {
    InViewport,
    OutsideViewport,
    BeforeNear,
    AfterFar,
    BehindCamera,
    NonFinite,
}

pub struct Projection {
    matrix: Mat4,
    width: f32,
    height: f32,
    eye: Vec3,
    forward: Vec3,
}
impl Projection {
    pub fn new(camera: &Camera, width: u32, height: u32) -> Result<Self> {
        if !(1..=16384).contains(&width) || !(1..=16384).contains(&height) {
            return Err(invalid("Selection viewport requires1..16384 pixels"));
        }
        camera.validate()?;
        Ok(Self {
            matrix: crate::picking::view_projection(camera, width, height),
            width: width as f32,
            height: height as f32,
            eye: camera.position.into(),
            forward: (Vec3::from(camera.target) - Vec3::from(camera.position)).normalize(),
        })
    }
    /// Pixel coordinates local to the viewport; near/far/behind-camera clipping matches picking.
    pub fn project(&self, world: Vec3) -> Option<Vec2> {
        self.project_clipped(world).ok()
    }
    fn project_clipped(&self, world: Vec3) -> std::result::Result<Vec2, ProjectionStatus> {
        use ProjectionStatus as Status;
        let p = self.matrix * world.extend(1.);
        if !p.is_finite() {
            return Err(Status::NonFinite);
        }
        if p.w <= 0. {
            return Err(Status::BehindCamera);
        }
        if p.z < 0. {
            return Err(Status::BeforeNear);
        }
        if p.z > p.w {
            return Err(Status::AfterFar);
        }
        let p = p.truncate() / p.w;
        Ok(Vec2::new(
            (p.x + 1.) * 0.5 * self.width,
            (1. - p.y) * 0.5 * self.height,
        ))
    }
    /// Same clip-space rules as overlays; XY outside the viewport remains projectable.
    pub fn project_classified(&self, world: Vec3) -> (ProjectionStatus, Option<Vec2>) {
        use ProjectionStatus as Status;
        let pixel = match self.project_clipped(world) {
            Ok(pixel) => pixel,
            Err(status) => return (status, None),
        };
        let status =
            if pixel.x >= 0. && pixel.x < self.width && pixel.y >= 0. && pixel.y < self.height {
                Status::InViewport
            } else {
                Status::OutsideViewport
            };
        (status, Some(pixel))
    }
}

/// One bounded acceleration structure. Camera/layout/selection changes do not rebuild world geometry.
#[derive(Default)]
pub struct Visibility {
    key: Option<(u64, u64)>,
    surface: Option<TriMesh>,
}
impl Visibility {
    pub fn prepare(&mut self, engine: &Engine) -> Result<()> {
        let key = (engine.revision, engine.simulation.tick);
        if self.key == Some(key) {
            return Ok(());
        }
        let frame = engine.render_frame();
        self.prepare_frame(&frame)?;
        self.key = Some(key);
        Ok(())
    }
    pub(crate) fn prepare_frame(&mut self, frame: &crate::render::RenderFrame) -> Result<()> {
        let total = frame
            .meshes
            .triangle_count()
            .checked_mul(3)
            .and_then(|n| n.checked_add(frame.boxes.len() * 36));
        if total.is_none_or(|n| n > u32::MAX as usize) {
            return Err(invalid(
                "Component visibility exceeds the u32 collision index format",
            ));
        }
        let mut vertices = Vec::with_capacity(frame.meshes.vertices.len() + frame.boxes.len() * 36);
        let mut triangles =
            Vec::with_capacity(frame.meshes.vertices.len() / 3 + frame.boxes.len() * 12);
        let mut append = |p: [Vec3; 3]| {
            let [a, b, c] = p;
            if (b - a).cross(c - a).length_squared() == 0. {
                return;
            }
            let start = vertices.len() as u32;
            vertices.extend(p.map(|v| Point3::new(v.x, v.y, v.z)));
            triangles.push([start, start + 1, start + 2]);
        };
        for t in frame.meshes.vertices.chunks_exact(3) {
            append([0, 1, 2].map(|i| Vec3::from(t[i].position)));
        }
        for draw in frame.meshes.draws.iter().filter(|d| d.shared) {
            let mut positions = frame.meshes.draw_positions(draw);
            while let (Some(a), Some(b), Some(c)) =
                (positions.next(), positions.next(), positions.next())
            {
                append([a, b, c].map(Vec3::from));
            }
        }
        let mut box_vertices: [crate::render::Vertex; 36] = bytemuck::Zeroable::zeroed();
        for item in &frame.boxes {
            crate::render::box_vertices(item, &mut box_vertices);
            for t in box_vertices.chunks_exact(3) {
                append([0, 1, 2].map(|i| Vec3::from(t[i].position)));
            }
        }
        let surface = if triangles.is_empty() {
            None
        } else {
            Some(
                TriMesh::new(vertices, triangles)
                    .map_err(|e| invalid(format!("Cannot prepare mesh visibility: {e}")))?,
            )
        };
        self.surface = surface;
        self.key = None;
        Ok(())
    }
    pub fn visible(&self, projection: &Projection, point: Vec3) -> bool {
        let delta = point - projection.eye;
        let distance = delta.length();
        if !distance.is_finite() || distance <= 0. {
            return false;
        }
        let direction = delta / distance;
        let cosine = direction.dot(projection.forward);
        if cosine <= 0. {
            return false;
        }
        let near = crate::picking::NEAR / cosine;
        // A few f32 ULPs prevent a surface's own triangles from hiding its vertices.
        let bias = 0.00002 + point.abs().max_element() * f32::EPSILON * 4.;
        let length = distance - near - bias;
        if length <= 0. {
            return false;
        }
        let origin = projection.eye + direction * near;
        let ray = Ray::new(
            Point3::new(origin.x, origin.y, origin.z),
            Vector3::new(direction.x, direction.y, direction.z),
        );
        self.surface
            .as_ref()
            .is_none_or(|s| s.cast_local_ray(&ray, length, false).is_none())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn region(
    engine: &Engine,
    instance: &str,
    kind: Kind,
    min: [f32; 2],
    max: [f32; 2],
    width: u32,
    height: u32,
    xray: bool,
    camera: &Camera,
) -> Result<Option<Components>> {
    let projection = Projection::new(camera, width, height)?;
    if min.iter().chain(max.iter()).any(|x| !x.is_finite())
        || min[0] < 0.
        || min[1] < 0.
        || max[0] > width as f32
        || max[1] > height as f32
        || min[0] > max[0]
        || min[1] > max[1]
    {
        return Err(invalid(
            "Selection rectangle must be finite, ordered and inside the viewport",
        ));
    }
    if engine
        .scene
        .plants
        .connections
        .iter()
        .any(|c| c.child_instance == instance)
    {
        return Err(invalid(
            "Select the visible assembled mesh instead of a consumed plant instance",
        ));
    }
    let item = engine
        .scene
        .meshes
        .instances
        .iter()
        .find(|i| i.id == instance)
        .ok_or_else(|| invalid("Mesh instance not found"))?;
    let asset = engine.scene.meshes.asset(&item.asset)?;
    let pose = engine.scene.meshes.pose(instance, engine.simulation.tick)?;
    let mut visibility = Visibility::default();
    if !xray {
        visibility.prepare(engine)?;
    }
    let inside = |point: Vec3| {
        projection.project(point).is_some_and(|p| {
            p.x >= min[0]
                && p.x <= max[0]
                && p.y >= min[1]
                && p.y <= max[1]
                && (xray || visibility.visible(&projection, point))
        })
    };
    let components = match kind {
        Kind::Vertices => Components::Vertices {
            indices: pose
                .positions
                .iter()
                .enumerate()
                .filter_map(|(i, p)| inside((*p).into()).then_some(i as u32))
                .collect(),
        },
        Kind::Edges => Components::Edges {
            vertices: crate::mesh_topology::Topology::inspect(&asset.geometry)?
                .edges
                .into_iter()
                .filter_map(|e| {
                    let [a, b] = e.vertices.map(|i| Vec3::from(pose.positions[i as usize]));
                    inside((a + b) * 0.5).then_some(e.vertices)
                })
                .collect(),
        },
        Kind::Faces => Components::Faces {
            indices: asset
                .geometry
                .triangles
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    let [a, b, c] = t.map(|i| Vec3::from(pose.positions[i as usize]));
                    inside((a + b + c) / 3.).then_some(i as u32)
                })
                .collect(),
        },
    };
    Ok((count(&components) > 0).then_some(components))
}
pub fn count(c: &Components) -> usize {
    match c {
        Components::Vertices { indices } | Components::Faces { indices } => indices.len(),
        Components::Edges { vertices } => vertices.len(),
    }
}
pub fn combine(
    previous: Option<Components>,
    next: Option<Components>,
    additive: bool,
) -> Option<Components> {
    if !additive {
        return next;
    }
    match (previous, next) {
        (
            Some(Components::Vertices { indices: mut a }),
            Some(Components::Vertices { indices: b }),
        ) => {
            a.extend(b);
            a.sort_unstable();
            a.dedup();
            Some(Components::Vertices { indices: a })
        }
        (Some(Components::Faces { indices: mut a }), Some(Components::Faces { indices: b })) => {
            a.extend(b);
            a.sort_unstable();
            a.dedup();
            Some(Components::Faces { indices: a })
        }
        (Some(Components::Edges { vertices: mut a }), Some(Components::Edges { vertices: b })) => {
            a.extend(b);
            a.sort_unstable();
            a.dedup();
            Some(Components::Edges { vertices: a })
        }
        (old, None) => old,
        (_, next) => next,
    }
}
