//! Native Land of Survivors gameplay component. No rendering, animation, Unreal code or Python tick.
#[path = "schema.rs"]
mod schema;
use glam::Vec3;
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    geometry::Volume,
    model::{Result, Scene, invalid},
};
pub use schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
const MAX_ACTIVE: usize = 4096;
const MAX_EVENTS: usize = 512;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Fighter {
    pub class_id: String,
    pub specialization: String,
    pub focus: Focus,
    pub form: Form,
    pub team: String,
    pub health: f64,
    pub max_health: f64,
    pub mana: f64,
    pub max_mana: f64,
    pub armor: f64,
    pub mounted: bool,
    pub ai: bool,
    pub last_combat: Option<u64>,
    pub life: u64,
    pub cooldowns: BTreeMap<String, u64>,
    pub bars: BTreeMap<String, Vec<String>>,
    pub statuses: Vec<Status>,
    pub casting: Option<Cast>,
    pub channel_until: u64,
    pub input: [f32; 2],
    pub facing: [f32; 3],
    pub spawn: [f32; 3],
    pub threat: BTreeMap<String, f64>,
    pub attack_at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub kind: EffectKind,
    pub source: String,
    pub ability: String,
    pub amount: f64,
    pub expires: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Aim {
    pub target: Option<String>,
    pub point: Option<[f32; 3]>,
    pub direction: Option<[f32; 3]>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Cast {
    pub ability: String,
    pub completes: u64,
    pub aim: Aim,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Shot {
    pub source: String,
    pub ability: String,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub remaining: f32,
    pub life: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pulse {
    pub source: String,
    pub ability: String,
    pub aim: Aim,
    pub center: [f32; 3],
    pub next: u64,
    pub remaining: u64,
    pub follow: bool,
    pub burn: bool,
    pub channel: bool,
    pub life: u64,
    pub target_life: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub tick: u64,
    pub source: String,
    pub ability: String,
    pub target: Option<String>,
    pub kind: String,
    pub amount: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub tick: u64,
    pub fighters: BTreeMap<String, Fighter>,
    pub shots: Vec<Shot>,
    pub pulses: Vec<Pulse>,
    pub events: VecDeque<Event>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    version: u32,
    catalog: Catalog,
    state: State,
    physics: Vec<u8>,
}
pub struct Game {
    pub engine: Engine,
    pub catalog: Catalog,
    pub state: State,
}
impl Fighter {
    pub fn new(
        class_id: &str,
        specialization: &str,
        focus: Focus,
        team: &str,
        spawn: [f32; 3],
    ) -> Self {
        Self {
            class_id: class_id.into(),
            specialization: specialization.into(),
            focus,
            form: Form::Humanoid,
            team: team.into(),
            health: 1000.,
            max_health: 1000.,
            mana: 600.,
            max_mana: 600.,
            armor: 20.,
            mounted: false,
            ai: false,
            last_combat: None,
            life: 0,
            cooldowns: BTreeMap::new(),
            bars: BTreeMap::new(),
            statuses: Vec::new(),
            casting: None,
            channel_until: 0,
            input: [0.; 2],
            facing: [0., 0., -1.],
            spawn,
            threat: BTreeMap::new(),
            attack_at: 0,
        }
    }
    fn bar_key(&self) -> String {
        format!("{}:{:?}:{:?}", self.specialization, self.focus, self.form)
    }
    pub fn speed_factor(&self, now: u64) -> f32 {
        if self.health <= 0.
            || self
                .statuses
                .iter()
                .any(|s| s.kind == EffectKind::Root && s.expires > now)
        {
            return 0.;
        }
        let slow = self
            .statuses
            .iter()
            .filter(|s| s.kind == EffectKind::Slow && s.expires > now)
            .map(|s| s.amount)
            .fold(0., f64::max);
        ((1. - slow) * if self.form == Form::Feline { 1.3 } else { 1. }) as f32
    }
    pub fn shield(&self) -> f64 {
        self.statuses
            .iter()
            .filter(|s| s.kind == EffectKind::Shield)
            .map(|s| s.amount)
            .sum()
    }
}
impl Game {
    pub fn new(
        scene: Scene,
        catalog: Catalog,
        mut fighters: BTreeMap<String, Fighter>,
    ) -> Result<Self> {
        catalog.validate()?;
        scene.validate()?;
        for (id, f) in &mut fighters {
            let e = scene
                .entities
                .iter()
                .find(|e| &e.id == id)
                .ok_or_else(|| invalid("Game fighter has no physical actor"))?;
            if e.character_controller.is_none() || e.vitality.is_none() || e.abilities.is_some() {
                return Err(invalid(
                    "Game fighters require ground controller/vitality, without a second legacy ability loadout",
                ));
            }
            Self::validate_fighter(f)?;
        }
        if fighters.is_empty() || fighters.len() > 128 {
            return Err(invalid("Game supports1..128 physical fighters"));
        }
        let mut engine = Engine::new(scene)?;
        engine.execute(
            Command::SetExecutionMode {
                mode: ExecutionMode::Continuous,
            }
            .into(),
        )?;
        let mut out = Self {
            engine,
            catalog,
            state: State {
                fighters,
                ..Default::default()
            },
        };
        for id in out.state.fighters.keys().cloned().collect::<Vec<_>>() {
            if out.available(&id).len() < 10 {
                return Err(invalid(
                    "Fighter class/specialization/focus has no complete kit",
                ));
            }
            out.ensure_bar(&id);
        }
        out.sync_health();
        Ok(out)
    }
    fn validate_fighter(f: &Fighter) -> Result<()> {
        if ![f.health, f.max_health, f.mana, f.max_mana, f.armor]
            .iter()
            .all(|v| v.is_finite() && *v >= 0. && *v <= 1e9)
            || f.max_health <= 0.
            || f.health > f.max_health
            || f.mana > f.max_mana
            || f.spawn.iter().any(|v| !v.is_finite())
            || f.input.iter().any(|v| !v.is_finite() || v.abs() > 1.)
            || !Vec3::from(f.facing).is_finite()
            || Vec3::from(f.facing).length_squared() < 0.001
            || f.statuses.len() > 256
            || f.cooldowns.len() > 1024
            || f.bars.len() > 32
            || f.threat.len() > 128
        {
            return Err(invalid("Invalid fighter state"));
        }
        Ok(())
    }
    fn emit(&mut self, source: &str, ability: &str, target: Option<&str>, kind: &str, amount: f64) {
        if self.state.events.len() == MAX_EVENTS {
            self.state.events.pop_front();
        }
        self.state.events.push_back(Event {
            tick: self.state.tick,
            source: source.into(),
            ability: ability.into(),
            target: target.map(str::to_owned),
            kind: kind.into(),
            amount,
        });
    }
    pub fn position(&self, id: &str) -> Vec3 {
        Vec3::from(
            self.engine
                .simulation
                .position(id)
                .expect("validated fighter instance"),
        )
    }
    pub fn available(&self, id: &str) -> Vec<String> {
        let Some(f) = self.state.fighters.get(id) else {
            return vec![];
        };
        self.catalog
            .abilities
            .iter()
            .filter(|a| {
                a.class_id == f.class_id
                    && a.specialization == f.specialization
                    && a.form == f.form
                    && a.focuses.contains(&f.focus)
            })
            .map(|a| a.id.clone())
            .collect()
    }
    fn ensure_bar(&mut self, id: &str) {
        let available = self.available(id);
        let f = &mut self.state.fighters.get_mut(id).unwrap();
        let key = f.bar_key();
        f.bars
            .entry(key)
            .or_insert_with(|| available.into_iter().take(10).collect());
    }
    pub fn bar(&self, id: &str) -> Vec<String> {
        self.state
            .fighters
            .get(id)
            .and_then(|f| f.bars.get(&f.bar_key()))
            .cloned()
            .unwrap_or_default()
    }
    pub fn equip(&mut self, id: &str, slots: Vec<String>) -> Result<()> {
        self.can_rebuild(id)?;
        let allowed = self.available(id);
        if slots.len() != 10
            || slots.iter().collect::<BTreeSet<_>>().len() != 10
            || slots.iter().any(|s| !allowed.contains(s))
        {
            return Err(invalid("Equip ten distinct available abilities"));
        }
        let f = self.state.fighters.get_mut(id).unwrap();
        f.bars.insert(f.bar_key(), slots);
        Ok(())
    }
    fn can_rebuild(&self, id: &str) -> Result<()> {
        let f = self
            .state
            .fighters
            .get(id)
            .ok_or_else(|| invalid("Unknown fighter"))?;
        if f.health <= 0.
            || f.mounted
            || f.form != Form::Humanoid
            || f.casting.is_some()
            || f.channel_until > self.state.tick
            || f.last_combat.is_some_and(|t| self.state.tick < t + 480)
            || self.state.shots.iter().any(|s| s.source == id)
            || self.state.pulses.iter().any(|p| p.source == id)
        {
            return Err(invalid(
                "Build changes require alive, unmounted humanoid, no active ability and eight seconds out of combat",
            ));
        }
        Ok(())
    }
    pub fn build(&mut self, id: &str, specialization: &str, focus: Focus) -> Result<()> {
        self.can_rebuild(id)?;
        if !self.catalog.abilities.iter().any(|a| {
            a.class_id == self.state.fighters[id].class_id
                && a.specialization == specialization
                && a.focuses.contains(&focus)
        }) {
            return Err(invalid("Specialization does not belong to this class"));
        }
        let f = self.state.fighters.get_mut(id).unwrap();
        f.specialization = specialization.into();
        f.focus = focus;
        self.ensure_bar(id);
        Ok(())
    }
    pub fn set_mounted(&mut self, id: &str, mounted: bool) -> Result<()> {
        let f = self
            .state
            .fighters
            .get_mut(id)
            .ok_or_else(|| invalid("Unknown fighter"))?;
        if mounted && (f.form != Form::Humanoid || f.casting.is_some() || f.health <= 0.) {
            return Err(invalid("Cannot mount transformed, casting or dead"));
        }
        f.mounted = mounted;
        Ok(())
    }
    pub fn movement(&mut self, id: &str, input: [f32; 2]) -> Result<()> {
        if input.iter().any(|v| !v.is_finite() || v.abs() > 1.) {
            return Err(invalid("Movement axes must be finite in -1..1"));
        }
        self.state
            .fighters
            .get_mut(id)
            .ok_or_else(|| invalid("Unknown fighter"))?
            .input = input;
        Ok(())
    }
    fn ally(&self, a: &str, b: &str) -> bool {
        self.state.fighters[a].team == self.state.fighters[b].team
    }
    fn matches(&self, source: &str, target: &str, r: Relation) -> bool {
        match r {
            Relation::SelfActor => source == target,
            Relation::Ally => self.ally(source, target),
            Relation::Enemy => !self.ally(source, target),
        }
    }
    fn visible(&self, from: Vec3, to: Vec3) -> bool {
        !self
            .engine
            .scene
            .entities
            .iter()
            .filter(|e| matches!(e.body, Some(ignis::model::BodyKind::Fixed)))
            .any(|e| {
                let center = self
                    .engine
                    .simulation
                    .position(&e.id)
                    .unwrap_or(e.transform.position);
                Volume::entity(e, center.into())
                    .cast(from, to - from, Vec3::ZERO, true, 1e-7)
                    .is_some()
            })
    }
    fn ground(&self, p: [f32; 3]) -> bool {
        p.iter().all(|v| v.is_finite() && v.abs() <= 10000.)
            && self
                .engine
                .scene
                .entities
                .iter()
                .filter(|e| e.body == Some(ignis::model::BodyKind::Fixed))
                .any(|e| {
                    Volume::entity(e, Vec3::from(e.transform.position))
                        .top(p[0], p[2])
                        .is_some_and(|h| (h - p[1]).abs() <= 0.1)
                })
    }
    fn validate_cast(&self, id: &str, a: &Ability, aim: &Aim, starting: bool) -> Result<()> {
        let f = self
            .state
            .fighters
            .get(id)
            .ok_or_else(|| invalid("Unknown fighter"))?;
        if f.health <= 0. || f.mounted || !self.available(id).iter().any(|s| s == &a.id) {
            return Err(invalid(
                "Ability unavailable for this life, class, focus or form",
            ));
        }
        if starting && (f.casting.is_some() || f.channel_until > self.state.tick) {
            return Err(invalid("Already casting/channeling"));
        }
        if f.cooldowns.get(&a.id).is_some_and(|t| *t > self.state.tick) {
            return Err(invalid("Ability on cooldown"));
        }
        if f.mana < a.cost {
            return Err(invalid("Insufficient mana"));
        }
        if aim
            .direction
            .is_some_and(|d| !Vec3::from(d).is_finite() || Vec3::from(d).length_squared() < 1e-8)
            || aim
                .point
                .is_some_and(|p| p.iter().any(|v| !v.is_finite() || v.abs() > 10000.))
            || aim
                .target
                .as_ref()
                .is_some_and(|t| !self.state.fighters.contains_key(t))
        {
            return Err(invalid("Invalid aim direction, point or actor"));
        }
        if matches!(a.delivery, Delivery::Target)
            || a.delivery == Delivery::Projectile && aim.target.is_some()
        {
            let target = aim
                .target
                .as_deref()
                .ok_or_else(|| invalid("Explicit actor target required"))?;
            let t = self
                .state
                .fighters
                .get(target)
                .ok_or_else(|| invalid("Unknown target"))?;
            if t.health <= 0.
                || !a
                    .effects
                    .iter()
                    .any(|e| self.matches(id, target, e.relation))
            {
                return Err(invalid("Target dead or relation denied"));
            }
            if self.position(id).distance(self.position(target)) > a.range {
                return Err(invalid("Target outside range"));
            }
            if !self.visible(self.position(id), self.position(target)) {
                return Err(invalid("Target occluded"));
            }
        }
        if matches!(a.delivery, Delivery::Ground | Delivery::Zone) {
            let p = aim
                .point
                .ok_or_else(|| invalid("Ground selection must be explicitly confirmed"))?;
            if !self.ground(p)
                || (self.position(id) - Vec3::from(p)).length() > a.range
                || !self.visible(self.position(id), Vec3::from(p) + Vec3::Y * 0.1)
            {
                return Err(invalid("Invalid ground surface, range or visibility"));
            }
        }
        if a.delivery == Delivery::Dash && f.speed_factor(self.state.tick) == 0. {
            return Err(invalid("Cannot dash while rooted"));
        }
        Ok(())
    }
    pub fn cast_slot(&mut self, id: &str, slot: usize, aim: Aim) -> Result<()> {
        let bar = self.bar(id);
        let a = bar
            .get(
                slot.checked_sub(1)
                    .ok_or_else(|| invalid("Slots are1..10"))?,
            )
            .ok_or_else(|| invalid("Slots are1..10"))?
            .clone();
        self.cast(id, &a, aim)
    }
    /// Authoring/test API can invoke any unlocked ability; the player bar uses cast_slot.
    pub fn cast(&mut self, id: &str, ability: &str, aim: Aim) -> Result<()> {
        let a = self.catalog.get(ability)?.clone();
        self.validate_cast(id, &a, &aim, true)?;
        let completes = self.state.tick + ticks(a.cast);
        self.state.fighters.get_mut(id).unwrap().casting = Some(Cast {
            ability: ability.into(),
            completes,
            aim,
        });
        self.emit(id, ability, None, "cast_started", 0.);
        if completes == self.state.tick {
            self.commit(id)?;
        }
        Ok(())
    }
    pub fn cancel(&mut self, id: &str) -> Result<()> {
        let f = self
            .state
            .fighters
            .get_mut(id)
            .ok_or_else(|| invalid("Unknown fighter"))?;
        let name = f.casting.take().map(|c| c.ability).unwrap_or_default();
        f.channel_until = 0;
        self.state.pulses.retain(|p| !(p.source == id && p.channel));
        self.emit(id, &name, None, "cancelled", 0.);
        Ok(())
    }
    fn direction(&self, id: &str, aim: &Aim) -> Vec3 {
        let direction = aim
            .direction
            .map(Vec3::from)
            .or_else(|| {
                aim.target
                    .as_ref()
                    .map(|t| self.position(t) - self.position(id))
            })
            .or_else(|| aim.point.map(|p| Vec3::from(p) - self.position(id)))
            .unwrap_or_else(|| Vec3::from(self.state.fighters[id].facing))
            .normalize_or_zero();
        if direction.length_squared() > 0.5 {
            direction
        } else {
            Vec3::from(self.state.fighters[id].facing).normalize_or_zero()
        }
    }
    fn commit(&mut self, id: &str) -> Result<()> {
        let cast = self.state.fighters[id].casting.as_ref().unwrap().clone();
        let a = self.catalog.get(&cast.ability)?.clone();
        if let Err(e) = self.validate_cast(id, &a, &cast.aim, false) {
            self.state.fighters.get_mut(id).unwrap().casting = None;
            self.emit(id, &a.id, None, "cast_failed", 0.);
            return Err(e);
        }
        if self.state.shots.len() + a.shots as usize > MAX_ACTIVE
            || self.state.pulses.len() + 128 > MAX_ACTIVE
        {
            self.state.fighters.get_mut(id).unwrap().casting = None;
            return Err(invalid("Game delivery budget exhausted"));
        }
        let direction = self.direction(id, &cast.aim);
        let now = self.state.tick;
        let f = self.state.fighters.get_mut(id).unwrap();
        f.mana -= a.cost;
        f.casting = None;
        f.cooldowns.insert(a.id.clone(), now + ticks(a.cooldown));
        f.last_combat = Some(now);
        f.facing = direction.to_array();
        let life = f.life;
        self.emit(id, &a.id, None, "committed", a.cost);
        if a.delivery == Delivery::Projectile {
            for shot in 0..a.shots {
                let spread = (shot as f32 - (a.shots - 1) as f32 / 2.) * 0.045;
                let d = glam::Quat::from_rotation_y(spread) * direction;
                self.state.shots.push(Shot {
                    source: id.into(),
                    ability: a.id.clone(),
                    position: self.position(id).to_array(),
                    direction: d.to_array(),
                    remaining: a.range,
                    life,
                });
            }
        } else if a.delivery == Delivery::Dash {
            let moved = self
                .engine
                .simulation
                .displace_character(id, (direction * a.range).to_array())?;
            self.emit(
                id,
                &a.id,
                Some(id),
                "move",
                Vec3::from(moved).length() as f64,
            );
            self.apply_area(id, &a, &cast.aim, self.position(id), false);
        } else if a.delivery == Delivery::Ground {
            self.apply_area(
                id,
                &a,
                &cast.aim,
                Vec3::from(cast.aim.point.unwrap()),
                false,
            );
        } else if a.delivery == Delivery::Zone {
            let center = Vec3::from(cast.aim.point.unwrap());
            self.apply_area(id, &a, &cast.aim, center, false);
            self.schedule(id, &a, &cast.aim, center, false, false);
        } else if a.delivery == Delivery::Cone && a.periodic {
            self.apply_area(id, &a, &cast.aim, self.position(id), false);
            self.state.fighters.get_mut(id).unwrap().channel_until = now + ticks(a.duration);
            self.schedule(id, &a, &cast.aim, self.position(id), false, true);
        } else if a.delivery == Delivery::Target {
            self.apply_target_chain(id, &a, &cast.aim, a.periodic);
        } else {
            self.apply_area(id, &a, &cast.aim, self.position(id), false);
            if a.periodic {
                self.schedule(id, &a, &cast.aim, self.position(id), false, false);
            }
        }
        self.sync_health();
        Ok(())
    }
    fn schedule(
        &mut self,
        source: &str,
        a: &Ability,
        aim: &Aim,
        center: Vec3,
        burn: bool,
        channel: bool,
    ) {
        let count = (ticks(a.duration) / ticks(a.interval).max(1)).max(1);
        let remaining = if burn { count } else { count.saturating_sub(1) };
        if remaining == 0 {
            return;
        }
        let mut pulse_aim = aim.clone();
        if a.delivery != Delivery::Target {
            pulse_aim.direction = Some(self.direction(source, aim).to_array());
            pulse_aim.target = None;
        }
        self.state.pulses.push(Pulse {
            source: source.into(),
            ability: a.id.clone(),
            aim: pulse_aim,
            center: center.to_array(),
            next: self.state.tick + ticks(a.interval).max(1),
            remaining,
            follow: a.delivery == Delivery::Target || channel,
            burn,
            channel,
            life: self.state.fighters[source].life,
            target_life: if a.delivery == Delivery::Target {
                aim.target.as_ref().map(|t| self.state.fighters[t].life)
            } else {
                None
            },
        });
    }
    fn apply_target_chain(&mut self, source: &str, a: &Ability, aim: &Aim, periodic: bool) {
        let Some(first) = aim.target.as_ref() else {
            return;
        };
        let mut target = first.clone();
        let mut visited = BTreeSet::new();
        for _ in 0..a.chain {
            visited.insert(target.clone());
            self.apply_effects(source, &target, a, false);
            if periodic {
                self.schedule(
                    source,
                    a,
                    &Aim {
                        target: Some(target.clone()),
                        ..aim.clone()
                    },
                    self.position(&target),
                    false,
                    false,
                );
            }
            let center = self.position(&target);
            let next = self
                .state
                .fighters
                .iter()
                .filter(|(id, f)| {
                    f.health > 0.
                        && !visited.contains(*id)
                        && a.effects
                            .iter()
                            .any(|e| self.matches(source, id, e.relation))
                })
                .map(|(id, _)| (center.distance_squared(self.position(id)), id.clone()))
                .filter(|(d, id)| *d <= 36. && self.visible(center, self.position(id)))
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            if let Some((_, id)) = next {
                target = id;
            } else {
                break;
            }
        }
    }
    fn apply_area(&mut self, source: &str, a: &Ability, aim: &Aim, center: Vec3, burn: bool) {
        let radius = if burn { a.radius } else { a.radius.max(0.) };
        let origin = self.position(source);
        let direction = self.direction(source, aim);
        let targets = self
            .state
            .fighters
            .iter()
            .filter_map(|(id, f)| {
                if f.health <= 0. {
                    return None;
                }
                let p = self.position(id);
                let delta = p - center;
                let inside = if a.delivery == Delivery::Cone && !burn {
                    let d = p - origin;
                    d.length() <= a.range
                        && (d.length_squared() < 0.001
                            || direction.dot(d.normalize()) >= 0.70710677)
                } else if radius <= 0. {
                    id == source
                } else {
                    delta.x * delta.x + delta.z * delta.z <= radius * radius && delta.y.abs() < 3.
                };
                let sight_from = if matches!(
                    a.delivery,
                    Delivery::Ground | Delivery::Zone | Delivery::Projectile
                ) || burn
                {
                    center + Vec3::Y * 0.1
                } else {
                    origin
                };
                (inside && self.visible(sight_from, p)).then(|| id.clone())
            })
            .collect::<Vec<_>>();
        for target in targets {
            self.apply_effects(source, &target, a, burn);
        }
    }
    fn apply_effects(&mut self, source: &str, target: &str, a: &Ability, burn: bool) {
        let effects = if burn {
            vec![Effect {
                kind: EffectKind::Damage,
                relation: Relation::Enemy,
                amount: a.burn,
                seconds: 0.,
                form: Form::Humanoid,
            }]
        } else {
            a.effects.clone()
        };
        for e in effects {
            if !self.matches(source, target, e.relation) || self.state.fighters[target].health <= 0.
            {
                continue;
            }
            let now = self.state.tick;
            let mut applied = e.amount;
            match e.kind {
                EffectKind::Damage => {
                    applied = self.damage(source, target, &a.id, e.amount, a.execute);
                }
                EffectKind::Heal => {
                    let f = self.state.fighters.get_mut(target).unwrap();
                    applied = e.amount.min(f.max_health - f.health);
                    f.health += applied;
                }
                EffectKind::Mana => {
                    let f = self.state.fighters.get_mut(target).unwrap();
                    applied = e.amount.min(f.max_mana - f.mana);
                    f.mana += applied;
                }
                EffectKind::Cleanse => {
                    let f = self.state.fighters.get_mut(target).unwrap();
                    let old = f.statuses.len();
                    f.statuses
                        .retain(|s| !matches!(s.kind, EffectKind::Slow | EffectKind::Root));
                    applied = (old - f.statuses.len()) as f64;
                }
                EffectKind::Form => {
                    let f = self.state.fighters.get_mut(target).unwrap();
                    f.form = e.form;
                    self.ensure_bar(target);
                }
                EffectKind::Move => {
                    continue;
                }
                EffectKind::Push => {
                    let dir = (self.position(target) - self.position(source))
                        .with_y(0.)
                        .normalize_or_zero();
                    let moved = self.engine.simulation.displace_character(
                        target,
                        (dir * e.amount as f32 * e.seconds as f32).to_array(),
                    );
                    applied = moved.map_or(0., |d| Vec3::from(d).length() as f64);
                }
                _ => {
                    let f = self.state.fighters.get_mut(target).unwrap();
                    if let Some(s) = f
                        .statuses
                        .iter_mut()
                        .find(|s| s.source == source && s.ability == a.id && s.kind == e.kind)
                    {
                        s.expires = now + ticks(e.seconds).max(1);
                        s.amount = if e.kind == EffectKind::Shield {
                            (s.amount + e.amount).min(f.max_health)
                        } else {
                            e.amount
                        };
                    } else if f.statuses.len() < 256 {
                        f.statuses.push(Status {
                            kind: e.kind,
                            source: source.into(),
                            ability: a.id.clone(),
                            amount: e.amount,
                            expires: now + ticks(e.seconds).max(1),
                        });
                    } else {
                        self.emit(source, &a.id, Some(target), "effect_budget_exhausted", 0.);
                        continue;
                    }
                    if matches!(
                        e.kind,
                        EffectKind::Root | EffectKind::Slow | EffectKind::Taunt
                    ) {
                        f.last_combat = Some(now);
                    }
                }
            }
            self.emit(
                source,
                &a.id,
                Some(target),
                &format!("{:?}", e.kind).to_lowercase(),
                applied,
            );
        }
    }
    pub fn damage(
        &mut self,
        source: &str,
        target: &str,
        ability: &str,
        amount: f64,
        execute: bool,
    ) -> f64 {
        let now = self.state.tick;
        let f = self.state.fighters.get_mut(target).unwrap();
        let armor = f.armor * if f.form == Form::Bear { 1.8 } else { 1. };
        let mitigation = f
            .statuses
            .iter()
            .filter(|s| s.kind == EffectKind::Mitigation && s.expires > now)
            .map(|s| s.amount)
            .fold(0., f64::max);
        let mut damage = amount * 100. / (100. + armor) * (1. - mitigation);
        if execute && f.health / f.max_health <= 0.25 {
            damage *= 2.;
        }
        for shield in f
            .statuses
            .iter_mut()
            .filter(|s| s.kind == EffectKind::Shield && s.expires > now)
        {
            let absorbed = damage.min(shield.amount);
            damage -= absorbed;
            shield.amount -= absorbed;
        }
        let actual = damage.min(f.health);
        f.health -= actual;
        f.last_combat = Some(now);
        *f.threat.entry(source.into()).or_default() += actual;
        if f.health <= 0. {
            f.form = Form::Humanoid;
            f.casting = None;
            f.channel_until = 0;
            f.input = [0.; 2];
            f.statuses.clear();
        }
        if f.health <= 0. {
            self.emit(source, ability, Some(target), "death", 0.);
        }
        actual
    }
    fn sync_health(&mut self) {
        for (slot, &index) in self.engine.simulation.actors.actors.iter().enumerate() {
            let id = &self.engine.scene.entities[index as usize].id;
            if let Some(f) = self.state.fighters.get(id)
                && let Some(h) = self.engine.simulation.vitality.state.actors[slot].as_mut()
            {
                h.units = (f.health * 1000.).round() as i64;
                h.maximum = (f.max_health * 1000.).round() as i64;
                h.scale = 1000;
            }
        }
    }
    fn update_projectiles(&mut self) {
        let shots = std::mem::take(&mut self.state.shots);
        for mut shot in shots {
            let a = self.catalog.get(&shot.ability).unwrap().clone();
            let from = Vec3::from(shot.position);
            let delta = Vec3::from(shot.direction) * (a.speed / 60.).min(shot.remaining);
            let hit = self
                .engine
                .scene
                .entities
                .iter()
                .filter(|e| e.id != shot.source && e.body.is_some())
                .filter_map(|e| {
                    let center = self
                        .engine
                        .simulation
                        .position(&e.id)
                        .unwrap_or(e.transform.position);
                    Volume::entity(e, center.into())
                        .cast(from, delta, Vec3::splat(a.diameter / 2.), false, 1e-8)
                        .map(|t| (t, e.id.clone()))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            let fraction = hit.as_ref().map_or(1., |h| h.0);
            shot.position = (from + delta * fraction).to_array();
            shot.remaining -= delta.length() * fraction;
            if let Some((_, target)) = hit {
                if a.radius > 0. {
                    self.apply_area(
                        &shot.source,
                        &a,
                        &Aim::default(),
                        Vec3::from(shot.position),
                        false,
                    );
                } else if self.state.fighters.contains_key(&target) {
                    self.apply_effects(&shot.source, &target, &a, false);
                }
                self.emit(&shot.source, &a.id, Some(&target), "impact", 0.);
                if a.burn > 0. {
                    let aim = Aim::default();
                    self.schedule(
                        &shot.source,
                        &a,
                        &aim,
                        Vec3::from(shot.position),
                        true,
                        false,
                    );
                }
            } else if shot.remaining > 0. {
                self.state.shots.push(shot);
            } else {
                self.emit(&shot.source, &a.id, None, "miss", 0.);
            }
        }
    }
    fn update_pulses(&mut self) {
        let pulses = std::mem::take(&mut self.state.pulses);
        for mut p in pulses {
            if self.state.fighters[&p.source].life != p.life
                || p.channel
                    && (self.state.fighters[&p.source].health <= 0.
                        || self.state.fighters[&p.source].channel_until <= self.state.tick)
            {
                continue;
            }
            if let Some(target) = p.aim.target.as_ref()
                && (self.state.fighters[target].health <= 0.
                    || p.target_life != Some(self.state.fighters[target].life))
            {
                continue;
            }
            if p.next <= self.state.tick {
                let a = self.catalog.get(&p.ability).unwrap().clone();
                if p.follow && !p.channel {
                    self.apply_effects(&p.source, p.aim.target.as_deref().unwrap(), &a, false);
                } else {
                    let center = if p.channel {
                        self.position(&p.source)
                    } else {
                        Vec3::from(p.center)
                    };
                    self.apply_area(&p.source, &a, &p.aim, center, p.burn);
                }
                p.remaining -= 1;
                p.next += ticks(a.interval).max(1);
            }
            if p.remaining > 0 {
                self.state.pulses.push(p);
            }
        }
    }
    pub fn step(&mut self, count: u32) -> Result<()> {
        if !(1..=3600).contains(&count)
            || self
                .state
                .tick
                .checked_add(u64::from(count))
                .is_none_or(|t| t > ignis::engine::MAX_RUNTIME_TICK)
        {
            return Err(invalid("Step1..3600 ticks"));
        }
        for _ in 0..count {
            self.state.tick += 1;
            for f in self.state.fighters.values_mut() {
                f.statuses.retain(|s| {
                    s.expires > self.state.tick && (s.kind != EffectKind::Shield || s.amount > 0.)
                });
            }
            let due = self
                .state
                .fighters
                .iter()
                .filter(|(_, f)| {
                    f.casting
                        .as_ref()
                        .is_some_and(|c| c.completes <= self.state.tick)
                })
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            for id in due {
                let _ = self.commit(&id);
            }
            self.update_projectiles();
            self.update_pulses();
            self.sync_health();
            self.engine.advance(1)?;
            let ids = self.state.fighters.keys().cloned().collect::<Vec<_>>();
            for id in ids {
                if self.state.fighters[&id].health <= 0. {
                    continue;
                }
                let f = &self.state.fighters[&id];
                let mut input = Vec3::new(f.input[0], 0., f.input[1]);
                if f.ai {
                    let taunt = f
                        .statuses
                        .iter()
                        .filter(|s| {
                            s.kind == EffectKind::Taunt
                                && self
                                    .state
                                    .fighters
                                    .get(&s.source)
                                    .is_some_and(|s| s.health > 0.)
                        })
                        .max_by_key(|s| s.expires)
                        .map(|s| s.source.clone());
                    let target = taunt.or_else(|| {
                        f.threat
                            .iter()
                            .filter(|(t, _)| {
                                self.state.fighters.get(*t).is_some_and(|s| s.health > 0.)
                            })
                            .max_by(|a, b| a.1.total_cmp(b.1))
                            .map(|(t, _)| t.clone())
                    });
                    if let Some(target) = target {
                        let d = self.position(&target) - self.position(&id);
                        if d.length() > 2. {
                            input = d.with_y(0.).normalize_or_zero();
                        } else if f.attack_at <= self.state.tick {
                            self.damage(&id, &target, "basic_attack", 8., false);
                            self.state.fighters.get_mut(&id).unwrap().attack_at =
                                self.state.tick + 60;
                        }
                    }
                }
                let factor = self.state.fighters[&id].speed_factor(self.state.tick);
                let delta = input / input.length().max(1.) * (6. / 60.) * factor;
                if delta.length_squared() > 0. {
                    self.engine
                        .simulation
                        .displace_character(&id, delta.to_array())?;
                    self.state.fighters.get_mut(&id).unwrap().facing = input.normalize().to_array();
                }
            }
            self.sync_health();
        }
        Ok(())
    }
    pub fn save(&self) -> Result<Vec<u8>> {
        let physical = ignis::portable_save::PortableSave::capture(&self.engine)?.encode()?;
        let save = Save {
            version: 1,
            catalog: self.catalog.clone(),
            state: self.state.clone(),
            physics: physical,
        };
        serde_json::to_vec(&save).map_err(|e| invalid(e.to_string()))
    }
    pub fn load(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 32 * 1024 * 1024 {
            return Err(invalid("Game save exceeds32MiB"));
        }
        let save: Save = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
        save.catalog.validate()?;
        if save.version != 1
            || save.state.fighters.is_empty()
            || save.state.fighters.len() > 128
            || save.state.shots.len() > MAX_ACTIVE
            || save.state.pulses.len() > MAX_ACTIVE
            || save.state.events.len() > MAX_EVENTS
        {
            return Err(invalid("Invalid game save version/budget"));
        }
        let engine = Engine::from_portable_save(&save.physics, Some(ExecutionMode::Continuous))?;
        if engine.simulation.tick != save.state.tick {
            return Err(invalid("Game and physics clocks disagree"));
        }
        let out = Self {
            engine,
            catalog: save.catalog,
            state: save.state,
        };
        out.validate_saved()?;
        Ok(out)
    }
    fn validate_saved(&self) -> Result<()> {
        let valid_aim = |aim: &Aim| {
            aim.target
                .as_ref()
                .is_none_or(|t| self.state.fighters.contains_key(t))
                && aim
                    .point
                    .is_none_or(|p| p.iter().all(|v| v.is_finite() && v.abs() <= 10000.))
                && aim.direction.is_none_or(|d| {
                    Vec3::from(d).is_finite() && Vec3::from(d).length_squared() > 1e-8
                })
        };
        for (id, f) in &self.state.fighters {
            Self::validate_fighter(f)?;
            if self.engine.simulation.position(id).is_none() || self.available(id).len() < 10 {
                return Err(invalid("Saved fighter missing physical actor or kit"));
            }
            for (key, bar) in &f.bars {
                if key.is_empty()
                    || bar.len() != 10
                    || bar.iter().collect::<BTreeSet<_>>().len() != 10
                    || bar.iter().any(|id| self.catalog.get(id).is_err())
                {
                    return Err(invalid("Invalid saved bar"));
                }
            }
            if self.bar(id).len() != 10
                || self.bar(id).iter().any(|a| !self.available(id).contains(a))
            {
                return Err(invalid(
                    "Current saved action bar is not available to this build",
                ));
            }
            for id in f.cooldowns.keys() {
                self.catalog.get(id)?;
            }
            if let Some(c) = &f.casting {
                self.catalog.get(&c.ability)?;
                if c.completes < self.state.tick || !valid_aim(&c.aim) {
                    return Err(invalid("Expired saved cast"));
                }
            }
            for s in &f.statuses {
                if !self.state.fighters.contains_key(&s.source)
                    || self.catalog.get(&s.ability).is_err()
                    || !s.amount.is_finite()
                    || s.amount < 0.
                    || !matches!(
                        s.kind,
                        EffectKind::Shield
                            | EffectKind::Mitigation
                            | EffectKind::Slow
                            | EffectKind::Root
                            | EffectKind::Taunt
                    )
                    || matches!(s.kind, EffectKind::Mitigation | EffectKind::Slow) && s.amount > 1.
                    || s.expires <= self.state.tick
                {
                    return Err(invalid("Invalid saved status"));
                }
            }
        }
        for s in &self.state.shots {
            self.catalog.get(&s.ability)?;
            if !self.state.fighters.contains_key(&s.source)
                || !Vec3::from(s.position).is_finite()
                || !Vec3::from(s.direction).is_finite()
                || (Vec3::from(s.direction).length() - 1.).abs() > 0.001
                || !s.remaining.is_finite()
                || s.remaining < 0.
            {
                return Err(invalid("Invalid saved projectile"));
            }
        }
        for p in &self.state.pulses {
            let a = self.catalog.get(&p.ability)?;
            if !self.state.fighters.contains_key(&p.source)
                || p.remaining == 0
                || p.remaining > 216000
                || p.next < self.state.tick
                || !Vec3::from(p.center).is_finite()
                || !valid_aim(&p.aim)
                || p.follow != (a.delivery == Delivery::Target || p.channel)
                || p.follow
                    && !p.channel
                    && (p.aim.target.is_none()
                        || p.target_life.is_none()
                        || a.delivery != Delivery::Target)
                || p.channel && (a.delivery != Delivery::Cone || !a.periodic)
                || p.burn && a.burn <= 0.
                || p.aim
                    .target
                    .as_ref()
                    .is_some_and(|t| !self.state.fighters.contains_key(t))
            {
                return Err(invalid("Invalid saved pulse"));
            }
        }
        Ok(())
    }
}
