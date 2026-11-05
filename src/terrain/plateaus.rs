//! Seeded accumulating strata with broad natural transition regions.
use super::*;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub azimuth_degrees: f64,
    pub width_meters: f64,
    pub meander_radians: f64,
    pub meander_scale_meters: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub seed: u64,
    pub radius_meters: f64,
    pub tread_meters: f64,
    pub minimum_step_meters: f64,
    pub maximum_step_meters: f64,
    pub aspect: f64,
    pub contour_irregularity: f64,
    pub warp_meters: f64,
    pub warp_period_meters: f64,
    pub cliff_fraction: f64,
    pub top_detail_meters: f64,
    #[serde(deserialize_with = "connections")]
    pub connections: Vec<Connection>,
}
fn connections<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Connection>, D::Error> {
    limited::<D, Connection, 16>(d)
}
fn hash(mut x: u64) -> f64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((x ^ (x >> 31)) >> 11) as f64 / 9007199254740991.
}
fn value(x: f64, z: f64, seed: u64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let a = smooth(x - x.floor());
    let b = smooth(z - z.floor());
    let at = |dx: i64, dz: i64| {
        2. * hash(
            seed ^ (ix.wrapping_add(dx) as u64).wrapping_mul(0x517cc1b727220a95)
                ^ (iz.wrapping_add(dz) as u64).rotate_left(31),
        ) - 1.
    };
    mix(mix(at(0, 0), at(1, 0), a), mix(at(0, 1), at(1, 1), a), b)
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        let numbers = [
            self.radius_meters,
            self.tread_meters,
            self.minimum_step_meters,
            self.maximum_step_meters,
            self.aspect,
            self.contour_irregularity,
            self.warp_meters,
            self.warp_period_meters,
            self.cliff_fraction,
            self.top_detail_meters,
        ];
        if numbers.iter().any(|v| !v.is_finite())
            || !radius(self.radius_meters)
            || self.tread_meters < 4.
            || self.minimum_step_meters <= 0.
            || self.maximum_step_meters < self.minimum_step_meters
            || !(0.5..=2.).contains(&self.aspect)
            || !(0. ..=0.3).contains(&self.contour_irregularity)
            || self.warp_period_meters < 1.
            || self.warp_meters < 0.
            || self.warp_meters > 0.08 * self.warp_period_meters
            || !(0.01..=0.5).contains(&self.cliff_fraction)
            || self.top_detail_meters < 0.
            || self.top_detail_meters > 0.1 * (self.maximum_step_meters - self.minimum_step_meters)
            || self.connections.len() > 16
            || !finite_height(self.maximum_height())
        {
            return Err(invalid(
                "Invalid natural terrace shape, finite height envelope or warp contraction",
            ));
        }
        for c in &self.connections {
            if [
                c.azimuth_degrees,
                c.width_meters,
                c.meander_radians,
                c.meander_scale_meters,
            ]
            .iter()
            .any(|v| !v.is_finite())
                || c.width_meters < 2.
                || c.meander_scale_meters < 4.
                || c.meander_radians.abs() > 2.
            {
                return Err(invalid("Invalid natural connection region"));
            }
        }
        Ok(())
    }
    pub fn support_radius(&self) -> f64 {
        self.radius_meters / ((1. - self.contour_irregularity) * self.aspect.min(1.))
            + 3. * self.warp_meters
            + 1.
    }
    pub fn maximum_height(&self) -> f64 {
        self.maximum_step_meters * ((self.radius_meters / self.tread_meters).ceil() + 2.)
            + self.top_detail_meters
    }
    pub fn level_height(&self, k: i64) -> f64 {
        let mid = (self.minimum_step_meters + self.maximum_step_meters) * 0.5;
        let amplitude =
            (self.maximum_step_meters - self.minimum_step_meters) * 0.25 - self.top_detail_meters;
        mid * k as f64 + amplitude * (2. * hash(self.seed ^ (k as u64)) - 2. * hash(self.seed))
    }
    fn warp(&self, p: [f64; 2]) -> [f64; 2] {
        std::array::from_fn(|i| {
            let seed = self.seed ^ (i as u64 + 1).wrapping_mul(0x85ebca6b);
            self.warp_meters
                * (0.8
                    * value(
                        p[0] / self.warp_period_meters,
                        p[1] / self.warp_period_meters,
                        seed,
                    )
                    + 0.2
                        * value(
                            2. * p[0] / self.warp_period_meters,
                            2. * p[1] / self.warp_period_meters,
                            seed ^ 31,
                        ))
        })
    }
    fn outline(&self, a: f64) -> f64 {
        1. + self.contour_irregularity
            * (0.75 * value(2. * a.cos(), 2. * a.sin(), self.seed ^ 203)
                + 0.25 * value(5. * a.cos(), 5. * a.sin(), self.seed ^ 251))
    }
    fn angle(&self, r: f64, c: &Connection) -> f64 {
        c.azimuth_degrees.to_radians()
            + c.meander_radians
                * ((r / c.meander_scale_meters + hash(self.seed) * 6.).sin()
                    + 0.25 * value(r / c.meander_scale_meters, 0., self.seed ^ 17))
    }
    pub fn route_point(&self, r: f64, connection: usize) -> Result<[f64; 2]> {
        self.validate()?;
        let c = self
            .connections
            .get(connection)
            .ok_or_else(|| invalid("Unknown natural connection"))?;
        if !r.is_finite() || r < 0. || r > self.support_radius() {
            return Err(invalid("Connection inspection radius outside source"));
        }
        let a = self.angle(r, c);
        let radius = r / self.outline(a);
        let target = [radius * a.cos(), radius * a.sin() / self.aspect];
        let mut p = target;
        for _ in 0..32 {
            let w = self.warp(p);
            p = [target[0] - w[0], target[1] - w[1]];
        }
        let w = self.warp(p);
        if (p[0] + w[0] - target[0]).hypot(p[1] + w[1] - target[1]) > 0.01 {
            return Err(invalid("Connection warp inversion did not converge"));
        }
        Ok(p)
    }
    pub(super) fn sample(&self, p: [f64; 2]) -> f64 {
        let w = self.warp(p);
        let q = [p[0] + w[0], (p[1] + w[1]) * self.aspect];
        let theta = q[1].atan2(q[0]);
        let r = q[0].hypot(q[1]) * self.outline(theta);
        let u = ((self.radius_meters - r) / self.tread_meters).max(0.);
        if u == 0. {
            return 0.;
        }
        let k = u.floor() as i64;
        let f = u - u.floor();
        let mut mask = 0f64;
        for c in &self.connections {
            let angle = (theta - self.angle(r, c) + std::f64::consts::PI)
                .rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            let rough = value(p[0] / 40., p[1] / 40., self.seed ^ 71);
            let width =
                c.width_meters * (0.85 + 0.15 * value(p[0] / 60., p[1] / 60., self.seed ^ 53));
            let distance = (angle * r + 0.15 * width * rough).abs();
            mask = mask.max(smooth(((1. - distance / width) / 0.6).clamp(0., 1.)));
        }
        let width = mix(self.cliff_fraction, 1., mask);
        let t = smooth(((f - (0.5 - 0.5 * width)) / width).clamp(0., 1.));
        mix(self.level_height(k), self.level_height(k + 1), t)
            + self.top_detail_meters
                * smooth(u.min(1.))
                * value(p[0] / 13., p[1] / 13., self.seed ^ 97)
    }
}
