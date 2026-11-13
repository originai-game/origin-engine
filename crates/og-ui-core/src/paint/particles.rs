//! UI particles (`docs/ui.md` §3.3): emitters attached to an element and
//! clipped by its layer (confetti, sparks, coin streams, rarity bursts),
//! drawn as og-render's particle mass elements, each particle a few values
//! (where it starts, where it flies, its spin, colour and start tick) whose
//! motion the shader evaluates: a document's og-emitter and burst events.
//!
//! Particles spawned by an emitter live in the world state (they replay),
//! at fixed screen positions from where the element was drawn when they
//! started: they keep flying when the element moves or leaves.

use crate::layout::Rect;
use crate::style::Srgba;
use crate::world::{Reader, WorldCmd};
use og_core::glam::Vec2;
use og_draw::{
    UiMassBatch, UiMassInstance, UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate,
    UiParticleShape,
};
use std::sync::Arc;

/// Where on its element an emitter's particles start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmitFrom {
    /// Its centre.
    #[default]
    Center,
    /// Anywhere inside its box.
    Area,
    /// On its box's edges.
    Edges,
}

/// An emitter on an element (og-emitter, a burst).
#[derive(Clone, Debug, PartialEq)]
pub struct ParticlesDesc {
    /// Particles a second while it is built (0: none).
    pub rate: f32,
    /// Particles at once on a tick `fire` is set (a burst).
    pub burst: u32,
    pub fire: bool,
    /// How long `rate` emits after the emitter's start, ms (0: while it is
    /// built).
    pub duration_ms: f32,
    /// Milliseconds a particle lives (0: 900).
    pub lifetime_ms: f32,
    pub shape: UiParticleShape,
    /// Its size, CSS px (a long thin rectangle is a streak).
    pub size: Vec2,
    /// Picked by each particle's seed (empty: the accent).
    pub colors: Vec<Srgba>,
    /// How far a particle travels over its life, CSS px, smallest to largest.
    pub travel: (f32, f32),
    /// Its heading, radians clockwise from +x (-π/2: up), and the spread
    /// about it (2π: every way).
    pub direction: f32,
    pub spread: f32,
    /// Added downward over its life, CSS px at its end.
    pub gravity: f32,
    /// Its travel slows to rest (an ease-out) instead of keeping its speed.
    pub drag: bool,
    /// Turns over its life, up to this many either way.
    pub spin: f32,
    /// Its rotation follows its heading (a streak flying outward).
    pub align: bool,
    pub from: EmitFrom,
    /// The scale at the end of its life (1: unchanged).
    pub end_scale: f32,
    /// Fades out toward the end of its life.
    pub fade: bool,
    /// Clipped by the element's clip (a scroll view, a masked panel).
    pub clip: bool,
}

impl Default for ParticlesDesc {
    fn default() -> Self {
        ParticlesDesc {
            rate: 0.0,
            burst: 0,
            fire: false,
            duration_ms: 0.0,
            lifetime_ms: 0.0,
            shape: UiParticleShape::Square,
            size: Vec2::new(8.0, 8.0),
            colors: Vec::new(),
            travel: (40.0, 120.0),
            direction: -std::f32::consts::FRAC_PI_2,
            spread: std::f32::consts::TAU,
            gravity: 0.0,
            drag: true,
            spin: 0.0,
            align: false,
            from: EmitFrom::Center,
            end_scale: 1.0,
            fade: true,
            clip: true,
        }
    }
}

impl ParticlesDesc {
    /// A burst ([`crate::BurstDesc`]) as particles: `count` rays (0: 8) and as many sparks
    /// thrown out to `radius` (0: 1.4 × the element's size) from its centre.
    pub fn burst(desc: &crate::BurstDesc, color: Srgba, size: f32, ms: f32) -> ParticlesDesc {
        let count = if desc.count == 0 { 8 } else { desc.count };
        let radius = if desc.radius > 0.0 {
            desc.radius
        } else {
            1.4 * size
        };
        ParticlesDesc {
            burst: count * 2,
            fire: true,
            lifetime_ms: ms,
            size: Vec2::new((size * 0.45).max(4.0), (size * 0.08).max(2.0)),
            colors: vec![color],
            travel: (radius * 0.7, radius),
            spread: std::f32::consts::TAU,
            align: true,
            end_scale: 0.15,
            ..ParticlesDesc::default()
        }
    }
}

/// An emitter on an element this frame.
#[derive(Clone, Debug)]
pub struct EmitterEntry {
    pub node: usize,
    pub key: u64,
    pub desc: ParticlesDesc,
    /// Fire on this tick (a burst's start); `None`: on `desc.fire`.
    pub at: Option<u64>,
}

/// An emitter's particles and its rate's remainder (world state).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmitterState {
    pub template: Option<Arc<UiMassTemplate>>,
    pub instances: Vec<UiMassInstance>,
    pub carry: f32,
    /// Ticks spawned so far (a seed for the next).
    pub spawned: u32,
}

impl EmitterState {
    pub fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.instances.len() as u64).to_le_bytes());
        out.extend_from_slice(bytemuck::cast_slice(&self.instances));
        out.extend_from_slice(&self.carry.to_le_bytes());
        out.extend_from_slice(&self.spawned.to_le_bytes());
        let t = self.template.as_deref();
        let f = |v: f32| v.to_le_bytes();
        // The template's values a restore draws it again with.
        let (shape, size, life, gravity, drag, end, fade) = match t {
            Some(t) => (
                match t.kind {
                    UiMassKind::Particle {
                        shape: UiParticleShape::Circle,
                    } => 1u8,
                    UiMassKind::Particle {
                        shape: UiParticleShape::Diamond,
                    } => 2,
                    _ => 0,
                },
                t.size,
                t.lifetime,
                t.gravity,
                t.drag,
                t.keys.last().map_or(1.0, |k| k.scale),
                t.keys.last().is_some_and(|k| k.alpha < 0.5),
            ),
            None => (255, Vec2::ZERO, 0, 0.0, false, 1.0, false),
        };
        out.push(shape);
        out.extend_from_slice(&f(size.x));
        out.extend_from_slice(&f(size.y));
        out.extend_from_slice(&life.to_le_bytes());
        out.extend_from_slice(&f(gravity));
        out.push(u8::from(drag));
        out.extend_from_slice(&f(end));
        out.push(u8::from(fade));
    }

    pub fn load(r: &mut Reader) -> Option<EmitterState> {
        let n = r.u64()? as usize;
        let size = std::mem::size_of::<UiMassInstance>();
        let raw = r.take(n.checked_mul(size)?)?;
        let instances = raw
            .chunks_exact(size)
            .map(bytemuck::pod_read_unaligned)
            .collect();
        let carry = r.f32()?;
        let spawned = r.u32()?;
        let shape = r.take(1)?[0];
        let (w, h) = (r.f32()?, r.f32()?);
        let life = r.u32()?;
        let gravity = r.f32()?;
        let drag = r.take(1)?[0] != 0;
        let end = r.f32()?;
        let fade = r.take(1)?[0] != 0;
        let shape = match shape {
            255 => None,
            1 => Some(UiParticleShape::Circle),
            2 => Some(UiParticleShape::Diamond),
            _ => Some(UiParticleShape::Square),
        };
        Some(EmitterState {
            template: shape.map(|s| template(s, Vec2::new(w, h), life, gravity, drag, end, fade)),
            instances,
            carry,
            spawned,
        })
    }
}

/// A particle template.
fn template(
    shape: UiParticleShape,
    size: Vec2,
    lifetime: u32,
    gravity: f32,
    drag: bool,
    end_scale: f32,
    fade: bool,
) -> Arc<UiMassTemplate> {
    Arc::new(UiMassTemplate {
        kind: UiMassKind::Particle { shape },
        size,
        pivot: Vec2::new(0.5, 0.5),
        lifetime: lifetime.max(1),
        gravity,
        drag,
        keys: vec![
            UiMassKey {
                t: 0.0,
                ..Default::default()
            },
            UiMassKey {
                t: if fade { 0.6 } else { 1.0 },
                scale: 1.0 + (end_scale - 1.0) * 0.6,
                ..Default::default()
            },
            UiMassKey {
                t: 1.0,
                scale: end_scale,
                alpha: if fade { 0.0 } else { 1.0 },
                ease: og_tween::Ease::QuadIn,
                ..Default::default()
            },
        ],
        ..Default::default()
    })
}

/// A seeded value in 0..1 (splitmix64): the same key, tick and index give
/// the same particle on every machine.
fn rand01(key: u64, tick: u64, i: u32, salt: u64) -> f32 {
    let mut z = key
        ^ tick.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (u64::from(i) << 20)
        ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

impl crate::ui::Paint {
    /// A burst (the preset [`ParticlesDesc::burst`]) on node `i` from tick
    /// `since`, after `delay` ticks.
    pub(crate) fn burst_particles(
        &mut self,
        i: usize,
        desc: &crate::BurstDesc,
        since: u64,
        world: &mut crate::world::WorldUi,
        env: &crate::ui::Env,
        tree: &crate::ui::Tree,
    ) {
        let t = env.theme;
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else {
            t.durations_ms.slow + t.durations_ms.base
        };
        let size = match tree.nodes[i].layout.used.width {
            crate::layout::Size::Fixed(w) => w,
            _ => 32.0,
        };
        let color = desc.color.unwrap_or(t.palette.primary);
        let delay = (desc.delay_ms * env.ticks_per_second as f32 / 1000.0).round() as u64;
        let key = crate::id::child_key(tree.nodes[i].key, "burst");
        world.declared.emitters.push(EmitterEntry {
            node: i,
            key,
            desc: ParticlesDesc::burst(desc, color, size, ms),
            at: Some(since + delay),
        });
    }

    /// Spawns this tick's particles and makes each emitter's batch.
    pub(crate) fn world_particles(
        &mut self,
        tick: u32,
        ui_world: &mut crate::world::WorldUi,
        env: &crate::ui::Env,
        interaction: &crate::ui::Interaction,
        tree: &crate::ui::Tree,
    ) {
        let now = interaction.state.tick;
        let reduced = env.theme.reduced_motion;
        let accent = env.theme.palette.primary;
        let tps = env.ticks_per_second as f32;
        let emitters = std::mem::take(&mut ui_world.declared.emitters);
        let mut live = Vec::new();
        for e in &emitters {
            live.push(e.key);
            let n = &tree.nodes[e.node];
            let (rect, world, clip) = (n.placed.rect, n.placed.world, n.placed.clip);
            let d = &e.desc;
            let life = ((if d.lifetime_ms > 0.0 {
                d.lifetime_ms
            } else {
                900.0
            }) * tps
                / 1000.0)
                .round()
                .max(1.0) as u32;
            let st = ui_world.state.particles.entry(e.key).or_default();
            st.template = Some(template(
                d.shape,
                d.size,
                life,
                d.gravity,
                d.drag,
                d.end_scale,
                d.fade,
            ));
            // How many start now.
            let fire = match e.at {
                Some(t) => t == now,
                None => d.fire,
            };
            let mut count = if fire { d.burst } else { 0 };
            let emitting = match (e.at, d.duration_ms > 0.0) {
                (Some(start), true) => {
                    now.saturating_sub(start) as f32 * 1000.0 < d.duration_ms * tps
                }
                _ => true,
            };
            st.carry += if emitting { d.rate / tps } else { 0.0 };
            let whole = st.carry.floor();
            st.carry -= whole;
            count += whole as u32;
            if reduced {
                count = 0;
            }
            for i in 0..count {
                let r = |salt| rand01(e.key, now, i + st.spawned, salt);
                let local = match d.from {
                    EmitFrom::Center => {
                        Vec2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
                    }
                    EmitFrom::Area => {
                        Vec2::new(rect.x + r(1) * rect.width, rect.y + r(2) * rect.height)
                    }
                    EmitFrom::Edges => {
                        let k = r(1) * 2.0 * (rect.width + rect.height);
                        let (w, h) = (rect.width, rect.height);
                        if k < w {
                            Vec2::new(rect.x + k, rect.y)
                        } else if k < w + h {
                            Vec2::new(rect.right(), rect.y + k - w)
                        } else if k < 2.0 * w + h {
                            Vec2::new(rect.right() - (k - w - h), rect.bottom())
                        } else {
                            Vec2::new(rect.x, rect.bottom() - (k - 2.0 * w - h))
                        }
                    }
                };
                let p = crate::world::mapped(&world, local);
                let heading = d.direction + (r(3) - 0.5) * d.spread;
                let (s, c) = og_core::math::sin_cos(heading);
                let dist = d.travel.0 + (d.travel.1 - d.travel.0) * r(4);
                let color = if d.colors.is_empty() {
                    accent
                } else {
                    d.colors[((r(5) * d.colors.len() as f32) as usize).min(d.colors.len() - 1)]
                };
                let turns = (r(6) * 2.0 - 1.0) * d.spin;
                let rot = if d.align {
                    heading
                } else {
                    r(7) * std::f32::consts::TAU
                };
                st.instances.push(UiMassInstance {
                    position: [p.x, p.y, 0.0],
                    value: 0.0,
                    color: color.linear(),
                    start: tick,
                    seed: i,
                    scale: 0.75 + 0.5 * r(8),
                    velocity: [c * dist, s * dist, turns, rot],
                    ..Default::default()
                });
            }
            st.spawned = st.spawned.wrapping_add(count);
            let lifetime = st.template.as_ref().map_or(1, |t| t.lifetime);
            st.instances
                .retain(|i| tick.wrapping_sub(i.start) < lifetime);
            if st.instances.is_empty() {
                continue;
            }
            let Some(template) = st.template.clone() else {
                continue;
            };
            let bounds = {
                let reach = d.travel.1 + d.gravity.max(0.0) + d.size.x.max(d.size.y);
                let mut b: Option<Rect> = None;
                for i in &st.instances {
                    let r = Rect::new(
                        i.position[0] - reach,
                        i.position[1] - reach,
                        2.0 * reach,
                        2.0 * reach,
                    );
                    b = Some(b.map_or(r, |a| union(&a, &r)));
                }
                b.unwrap_or_default()
            };
            let batch = UiMassBatch {
                template,
                space: UiMassSpace::Screen,
                instances: st.instances.clone(),
                layer: 0,
                clip: None,
            };
            let vp = env.viewport;
            let clip = if d.clip {
                clip
            } else {
                Rect::new(0.0, 0.0, vp.width, vp.height)
            };
            let key = tree.nodes[e.node].key;
            ui_world.placement.marks.entry(key).or_default().leave.push(
                crate::draw::DrawCmd::World(Box::new(WorldCmd::Mass {
                    batch: Box::new(batch),
                    bounds,
                    clip,
                })),
            );
        }
        // Emitters not built this tick keep their particles flying (over
        // the screen, unclipped) until they end.
        let vp = env.viewport;
        let screen = Rect::new(0.0, 0.0, vp.width, vp.height);
        let mut orphans = Vec::new();
        ui_world.state.particles.retain(|k, st| {
            if live.contains(k) {
                return true;
            }
            let lifetime = st.template.as_ref().map_or(1, |t| t.lifetime);
            st.instances
                .retain(|i| tick.wrapping_sub(i.start) < lifetime);
            if st.instances.is_empty() {
                return false;
            }
            if let Some(t) = &st.template {
                orphans.push(UiMassBatch {
                    template: t.clone(),
                    space: UiMassSpace::Screen,
                    instances: st.instances.clone(),
                    layer: 0,
                    clip: None,
                });
            }
            true
        });
        if !orphans.is_empty() {
            let cmds = orphans
                .into_iter()
                .map(|b| {
                    crate::draw::DrawCmd::World(Box::new(WorldCmd::Mass {
                        batch: Box::new(b),
                        bounds: screen,
                        clip: screen,
                    }))
                })
                .collect::<Vec<_>>();
            // On the root, after everything.
            let root = tree.nodes[0].key;
            ui_world
                .placement
                .marks
                .entry(root)
                .or_default()
                .leave
                .extend(cmds);
        }
        ui_world.declared.emitters = emitters;
    }
}

fn union(a: &Rect, b: &Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect::new(
        x,
        y,
        a.right().max(b.right()) - x,
        a.bottom().max(b.bottom()) - y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_are_seeded_by_key_tick_and_index() {
        let a = rand01(7, 100, 3, 1);
        assert_eq!(a, rand01(7, 100, 3, 1));
        assert_ne!(a, rand01(7, 101, 3, 1));
        assert_ne!(a, rand01(8, 100, 3, 1));
        assert!((0.0..1.0).contains(&a));
        // Spread over the range, not stuck at one end.
        let mean = (0..1000).map(|i| rand01(1, 2, i, 3)).sum::<f32>() / 1000.0;
        assert!((mean - 0.5).abs() < 0.05, "{mean}");
    }
}
