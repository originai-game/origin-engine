//! `<Particles>` (CONVENTIONS.md §Particles): an engine particle emitter,
//! the `type particles` asset og.load.emitter loads and og.ParticleSystem
//! runs in the world, drawn in the UI. One simulation, two places it
//! draws: the node steps og-render-data's [`CpuParticles`] on the UI's
//! tick and paints each living particle as a quad of the tree's arena, in
//! painter's order where the node stands and under the clip it is drawn in.
//!
//! The emitter sits at the centre of the node's laid-out box: 1 unit is 1
//! CSS px and +y is up, so `gravity 0 -600 0` falls down the screen. A
//! `box` shape is the node's box (its x and y half extents are the box's;
//! particles are born over all of it); any other shape is born at its
//! centre. It starts when the node first paints (`delay` seconds later
//! when given) and starts again when `restart` changes; its randomness is
//! `seed` and the tick it started, so a replay draws the same.
//!
//! What draws: the emitter's shape, emission, motion and look over life,
//! as a box of its size in the node's ink times its colour (a disc with
//! `round true`), or its `texture` (a `sheet`'s frame) tinted by it,
//! turned by its rotation or along its velocity (`align velocity`). What
//! needs the scene does not: wind and force fields, collision, lighting,
//! distortion, trails, meshes, effect materials, texture arrays, glow and
//! sub-emitters; the arena blends alpha whatever `blend` says
//! ([`Particles::undrawn`] names the first such line for a warning).

use super::Tree;
use og_core::glam::{Mat4, Vec3};
use og_core::math;
use og_draw::paint::{IMAGE_FILL, Paint, PaintKind, PaintQuad};
use og_env::force::Forces;
use og_render_data::{
    CpuParticles, EmitterHandle, EmitterShape, ParticleAlign, ParticleEmitter, ParticleRender,
    ParticleSystem, SimulationSpace, appearance, sheet_frames,
};

/// A `<Particles>` node's props and its running system.
#[derive(Debug)]
pub(crate) struct Particles {
    /// `emitter`: the handle as the game holds it (og.load.emitter's), NaN
    /// none; [`Tree::resolve_emitters`] reads its description.
    handle: f64,
    /// `seed`.
    seed: u32,
    /// `delay`: seconds from the node's first paint (or its restart) to
    /// the emission's start.
    delay: f32,
    /// `restart` as last set: another value starts it again.
    restart: u64,
    /// The emitter's description, the node's own copy (a `box` shape's x
    /// and y are written over with the node's box); `None` until resolved
    /// or for a handle the host does not know.
    emitter: Option<ParticleEmitter>,
    /// The tick emission started; `None`: at the next step.
    start: Option<u64>,
    sim: CpuParticles,
    /// The emitter's centre at the last step, CSS px on screen: where a
    /// `space local` emitter's particles are drawn from.
    centre: [f32; 2],
    /// No field reaches the UI: what the simulation is stepped with.
    forces: Forces,
}

impl Default for Particles {
    fn default() -> Self {
        Particles {
            handle: f64::NAN,
            seed: 0,
            delay: 0.0,
            restart: 0,
            emitter: None,
            start: None,
            sim: CpuParticles::default(),
            centre: [0.0; 2],
            forces: Forces::default(),
        }
    }
}

/// FNV-1a of a prop's text: `restart` compares by it.
fn key(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

impl Particles {
    /// A prop of the node (`emitter`, `seed`, `delay`, `restart`): whether
    /// `name` is one. `emitter` is the handle's number (empty or 0: none);
    /// a new one, like a new `restart`, starts it again from nothing.
    pub(crate) fn set(&mut self, name: &str, value: &str) -> bool {
        match name {
            "emitter" => {
                let handle = match value.trim().parse::<f64>() {
                    Ok(h) if h != 0.0 => h,
                    _ => f64::NAN,
                };
                if handle.to_bits() != self.handle.to_bits() {
                    self.handle = handle;
                    self.emitter = None;
                    self.again();
                }
            }
            "seed" => self.seed = value.trim().parse::<f64>().unwrap_or(0.0) as u32,
            "delay" => self.delay = value.trim().parse::<f32>().unwrap_or(0.0).max(0.0),
            "restart" => {
                let k = key(value);
                if k != self.restart {
                    self.restart = k;
                    self.again();
                }
            }
            _ => return false,
        }
        true
    }

    /// Starts again at the next step, from no particles.
    fn again(&mut self) {
        self.start = None;
        self.sim.last_tick = None;
    }

    /// The handle whose description it still needs, if any.
    fn wants(&self) -> Option<f64> {
        (self.emitter.is_none() && !self.handle.is_nan()).then_some(self.handle)
    }

    /// The first line of `e` the UI does not draw, for the host's warning;
    /// `None` when all of it draws.
    pub(crate) fn undrawn(e: &ParticleEmitter) -> Option<&'static str> {
        let checks: [(bool, &'static str); 13] = [
            (!matches!(e.render, ParticleRender::Billboard), "render"),
            (
                matches!(
                    e.shape,
                    EmitterShape::Mesh { .. } | EmitterShape::Spline { .. }
                ),
                "shape mesh or spline",
            ),
            (e.wind != 0.0, "wind"),
            (e.force_fields != 0.0, "forceFields"),
            (e.collision.is_some(), "collision"),
            (e.lit, "lit"),
            (e.distortion != 0.0, "distortion"),
            (e.trail.is_some(), "trail"),
            (!e.sub_emitters.is_empty(), "subEmitter"),
            (!e.events.is_empty(), "events"),
            (e.effect.is_some(), "effect"),
            (e.array.is_some(), "array"),
            (
                matches!(
                    e.align,
                    ParticleAlign::Upright { .. } | ParticleAlign::Facing { .. }
                ),
                "align upright or facing",
            ),
        ];
        checks.iter().find(|c| c.0).map(|c| c.1)
    }

    /// Steps the system to `tick` (of `tps` a second) with its node's box
    /// centred at `centre` on screen and `size` across: whether anything
    /// of it is still to draw (a particle living, or emission to come).
    /// Stepped again at the same tick it does nothing; after ticks it was
    /// not stepped on (a hidden node) it catches up as the engine's
    /// systems do.
    fn step(&mut self, tick: u64, tps: u32, centre: [f32; 2], size: [f32; 2]) -> bool {
        let Some(e) = self.emitter.as_mut() else {
            return false;
        };
        if let EmitterShape::Box { half_extents } = &mut e.shape {
            (half_extents.x, half_extents.y) = (size[0] / 2.0, size[1] / 2.0);
        }
        let rate = tps.max(1) as f32;
        let start = *self
            .start
            .get_or_insert_with(|| tick + (self.delay * rate).round() as u64);
        self.centre = centre;
        if tick < start {
            return true;
        }
        let sys = ParticleSystem {
            start_tick: start,
            ..ParticleSystem::new(
                0,
                EmitterHandle(0),
                Mat4::from_translation(Vec3::new(centre[0], -centre[1], 0.0)),
                // The start is part of the seed: each play differs, and a
                // replay's are the same.
                u64::from(self.seed) | start << 32,
                tick,
            )
        };
        self.sim
            .advance(e, &sys, rate, &[], &[], &[], None, None, &self.forces);
        let elapsed = (tick + 1 - start) as f32 / rate;
        e.looping || elapsed <= e.duration || self.sim.slots.iter().any(|p| p.alive())
    }

    /// Paints the living particles into `p`: `ink` the node's ink (already
    /// faded by its opacity), `clip` what it is drawn in, `xf` its
    /// transform of `p.transforms`.
    fn paint(&self, p: &mut Paint, ink: [f32; 4], clip: [f32; 4], xf: u32) {
        let Some(e) = &self.emitter else {
            return;
        };
        let frames = sheet_frames(&e.sheet);
        let texture = e.texture.map(|t| t.to_bits());
        let local = e.space == SimulationSpace::Local;
        for s in self.sim.slots.iter().filter(|s| s.alive()) {
            let ([w, h], color, rotation, frame) = appearance(e, s, frames);
            let alpha = ink[3] * color[3].clamp(0.0, 1.0);
            let (mut h, mut turn) = (h, -rotation);
            if let ParticleAlign::Velocity { stretch } = e.align {
                // Its height along the way it flies on screen (y down).
                let (vx, vy) = (s.vel_life[0], -s.vel_life[1]);
                h += (vx * vx + vy * vy).sqrt() * stretch.abs();
                turn = math::atan2(-vx, vy);
            }
            if w <= 0.0 || h <= 0.0 || alpha <= 0.0 {
                continue;
            }
            // Where it is on screen: the world of a system here is the
            // screen with +y up.
            let (mut x, mut y) = (s.pos_age[0], -s.pos_age[1]);
            if local {
                (x, y) = (x + self.centre[0], y + self.centre[1]);
            }
            let color = [
                ink[0] * color[0].clamp(0.0, 1.0),
                ink[1] * color[1].clamp(0.0, 1.0),
                ink[2] * color[2].clamp(0.0, 1.0),
                alpha,
            ];
            let mut quad = PaintQuad {
                rect: [x - w / 2.0, y - h / 2.0, w, h],
                clip,
                color,
                radius: [0.0; 4],
                p: [0.0; 4],
                kind: PaintKind::Solid as u32,
                a: 0,
                b: 0,
                xf: if turn == 0.0 {
                    xf
                } else {
                    turned(p, xf, [x, y], turn)
                },
                uv: [0.0; 4],
                wipe: [0.0; 4],
                mask: 0,
                filter: 0,
            };
            match texture {
                Some(bits) => {
                    quad.kind = PaintKind::Image as u32;
                    (quad.a, quad.b) = (bits, IMAGE_FILL);
                    quad.uv = frame_uv(e.sheet.columns, e.sheet.rows, frame);
                }
                None if e.round => quad.radius = [w.min(h) / 2.0; 4],
                None => {}
            }
            p.quads.push(quad);
        }
    }
}

/// Frame `frame` of a sheet `columns` by `rows`, left to right and top to
/// bottom, as an image quad's region (u0, v0, u1, v1).
fn frame_uv(columns: u32, rows: u32, frame: u32) -> [f32; 4] {
    let (c, r) = (columns.max(1), rows.max(1));
    let (x, y) = (frame % c, (frame / c).min(r - 1));
    let (w, h) = (1.0 / c as f32, 1.0 / r as f32);
    [
        x as f32 * w,
        y as f32 * h,
        (x + 1) as f32 * w,
        (y + 1) as f32 * h,
    ]
}

/// The transform `parent` of `p.transforms` with a turn of `angle`
/// (radians, clockwise on screen) about `at` under it: its index.
fn turned(p: &mut Paint, parent: u32, at: [f32; 2], angle: f32) -> u32 {
    if p.transforms.is_empty() {
        p.transforms.push(og_draw::paint::IDENTITY);
    }
    let (sin, cos) = math::sin_cos(angle);
    let (a, b, c, d) = (cos, sin, -sin, cos);
    let e = at[0] - (a * at[0] + c * at[1]);
    let f = at[1] - (b * at[0] + d * at[1]);
    let [pa, pb, pc, pd, pe, pf, _, _] = p.transforms[parent as usize];
    p.transforms.push([
        pa * a + pc * b,
        pb * a + pd * b,
        pa * c + pc * d,
        pb * c + pd * d,
        pa * e + pc * f + pe,
        pb * e + pd * f + pf,
        0.0,
        0.0,
    ]);
    p.transforms.len() as u32 - 1
}

/// The tree's side of its `<Particles>` nodes: the UI tick they step to
/// and which of them still wait for their emitter's description.
#[derive(Debug, Default)]
pub(crate) struct Clock {
    tick: u64,
    tps: u32,
    /// The player's reduced-motion setting ([`Tree::set_time`]).
    pub(crate) reduced: bool,
    /// The elements whose emitter is not resolved yet.
    pub(crate) unresolved: Vec<u32>,
    /// Something of a system painted last is still to draw: the next
    /// frame paints again.
    pub(crate) live: bool,
}

impl Tree {
    /// The UI tick the next [`Tree::present`] steps its `<Particles>` to,
    /// and the ticks a second: the tick's number, never a wall clock.
    pub fn set_tick(&mut self, tick: u64, tps: u32) {
        self.particles.tick = tick;
        self.particles.tps = tps;
    }

    /// Whether a `<Particles>` node waits for its emitter's description
    /// ([`Tree::resolve_emitters`]).
    pub fn wants_emitters(&self) -> bool {
        !self.particles.unresolved.is_empty()
    }

    /// Gives each `<Particles>` node that waits for it its emitter's
    /// description: `emitters` answers a handle as the game holds it
    /// (og.load.emitter's), `None` for one it does not know, which draws
    /// nothing. Returns each line a resolved emitter has that the UI does
    /// not draw ([`Particles::undrawn`]), once per node, for a warning.
    pub fn resolve_emitters<'a>(
        &mut self,
        emitters: &dyn Fn(f64) -> Option<&'a ParticleEmitter>,
    ) -> Vec<&'static str> {
        let mut undrawn = Vec::new();
        for e in std::mem::take(&mut self.particles.unresolved) {
            let Some(p) = self
                .els
                .get_mut(e as usize)
                .filter(|el| !el.dead)
                .and_then(|el| el.particles.as_deref_mut())
            else {
                continue;
            };
            if let Some(found) = p.wants().and_then(emitters) {
                undrawn.extend(Particles::undrawn(found));
                p.emitter = Some(found.clone());
                self.paint = true;
            }
        }
        undrawn
    }

    /// The emitters [`Tree::resolve_emitters`] answered from changed (an
    /// asset loaded again): every `<Particles>` node reads its own again
    /// and starts from nothing.
    pub fn reload_emitters(&mut self) {
        self.particles.unresolved.clear();
        for (e, el) in self.els.iter_mut().enumerate() {
            if let Some(p) = el.particles.as_deref_mut().filter(|_| !el.dead) {
                p.emitter = None;
                p.again();
                if p.wants().is_some() {
                    self.particles.unresolved.push(e as u32);
                }
            }
        }
        self.paint = true;
    }

    /// A prop of `<Particles>` node `e` (ops' setProp): whether `name` is
    /// one of its own.
    pub(crate) fn set_particles(&mut self, e: u32, name: &str, value: &str) -> bool {
        let Some(p) = self.els[e as usize].particles.as_deref_mut() else {
            return false;
        };
        if !p.set(name, value) {
            return false;
        }
        if p.wants().is_some() && !self.particles.unresolved.contains(&e) {
            self.particles.unresolved.push(e);
        }
        self.paint = true;
        true
    }

    /// Steps element `e`'s system to the tree's tick and paints it, as the
    /// repaint reaches it: `rect` its box on screen, `ink` its ink faded
    /// by its opacity, `clip` and `xf` what it is drawn in. Under reduced
    /// motion it draws nothing, and what would have started then never
    /// plays late.
    pub(crate) fn paint_particles(
        &mut self,
        e: usize,
        rect: [f32; 4],
        ink: [f32; 4],
        clip: [f32; 4],
        xf: u32,
    ) {
        let Some(p) = self.els[e].particles.as_deref_mut() else {
            return;
        };
        let clock = &mut self.particles;
        if clock.reduced {
            p.start.get_or_insert(clock.tick);
            return;
        }
        let centre = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
        clock.live |= p.step(clock.tick, clock.tps, centre, [rect[2], rect[3]]);
        p.paint(&mut self.drawn, ink, clip, xf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_render_data::{Burst, ColorCurve};

    /// A one-shot burst of 8 at once, flying up at 60 px/s for a second,
    /// white throughout.
    fn burst() -> ParticleEmitter {
        ParticleEmitter {
            max_particles: 16,
            shape: EmitterShape::Point,
            rate: 0.0,
            bursts: vec![Burst {
                time: 0.0,
                count: 8,
                cycles: 1,
                interval: 0.0,
            }],
            duration: 0.5,
            looping: false,
            lifetime: [1.0, 1.0],
            speed: [60.0, 60.0],
            size: [4.0, 4.0],
            gravity: Vec3::ZERO,
            color_over_life: ColorCurve::constant([1.0; 4]),
            ..ParticleEmitter::default()
        }
    }

    fn node(e: ParticleEmitter) -> Particles {
        Particles {
            handle: 1.0,
            emitter: Some(e),
            ..Particles::default()
        }
    }

    fn painted(p: &Particles) -> Paint {
        let mut out = Paint::default();
        out.clear();
        p.paint(&mut out, [1.0, 0.5, 0.25, 1.0], [0.0, 0.0, 800.0, 600.0], 0);
        out
    }

    /// The emitter's +y is up the screen, 1 unit a CSS px: a burst thrown
    /// up at 60 px/s from a node centred at (400, 300) is born there on
    /// its first tick and stands 30 px above it 30 ticks of a 60th later,
    /// each particle a 4 px box in the node's ink.
    #[test]
    fn a_burst_flies_up_the_screen_in_the_nodes_ink() {
        let mut p = node(burst());
        for tick in 0..=30 {
            assert!(p.step(tick, 60, [400.0, 300.0], [0.0, 0.0]));
        }
        let out = painted(&p);
        assert_eq!(out.quads.len(), 8);
        for q in &out.quads {
            assert_eq!(q.kind, PaintKind::Solid as u32);
            assert_eq!(q.color, [1.0, 0.5, 0.25, 1.0]);
            let (cx, cy) = (q.rect[0] + q.rect[2] / 2.0, q.rect[1] + q.rect[3] / 2.0);
            assert!((cx - 400.0).abs() < 0.01, "x {cx}");
            assert!((cy - 270.0).abs() < 0.01, "y {cy}");
            assert_eq!([q.rect[2], q.rect[3]], [4.0, 4.0]);
        }
    }

    /// A one-shot is over once its emission and its last particle are: the
    /// node then asks for no more frames and paints nothing.
    #[test]
    fn a_one_shot_ends() {
        let mut p = node(burst());
        let mut live = true;
        for tick in 0..70 {
            live = p.step(tick, 60, [0.0, 0.0], [0.0, 0.0]);
        }
        assert!(!live);
        assert!(painted(&p).quads.is_empty());
    }

    /// `delay` holds the start back; a new `restart` plays it again from
    /// nothing, and the same value does not.
    #[test]
    fn delay_and_restart() {
        let mut p = node(burst());
        assert!(p.set("delay", "0.5"));
        for tick in 0..30 {
            assert!(p.step(tick, 60, [0.0, 0.0], [0.0, 0.0]));
            assert!(painted(&p).quads.is_empty(), "nothing before its start");
        }
        p.step(30, 60, [0.0, 0.0], [0.0, 0.0]);
        assert_eq!(painted(&p).quads.len(), 8);
        for tick in 31..200 {
            p.step(tick, 60, [0.0, 0.0], [0.0, 0.0]);
        }
        assert!(painted(&p).quads.is_empty());
        assert!(p.set("delay", "0"));
        assert!(p.set("restart", "2"));
        p.step(200, 60, [0.0, 0.0], [0.0, 0.0]);
        assert_eq!(painted(&p).quads.len(), 8, "a new restart plays again");
        let before = p.sim.slots.clone();
        assert!(p.set("restart", "2"));
        p.step(201, 60, [0.0, 0.0], [0.0, 0.0]);
        assert_ne!(p.sim.slots, before, "the same restart lets it run on");
        assert_eq!(painted(&p).quads.len(), 8);
    }

    /// A `box` shape is the node's box: particles are born over all of it,
    /// whatever half extents the file gave.
    #[test]
    fn a_box_shape_is_the_nodes_box() {
        let mut p = node(ParticleEmitter {
            shape: EmitterShape::Box {
                half_extents: Vec3::new(1.0, 1.0, 0.0),
            },
            speed: [0.0, 0.0],
            bursts: vec![Burst {
                time: 0.0,
                count: 16,
                cycles: 1,
                interval: 0.0,
            }],
            ..burst()
        });
        p.step(0, 60, [400.0, 300.0], [200.0, 20.0]);
        let out = painted(&p);
        assert_eq!(out.quads.len(), 16);
        let xs = out.quads.iter().map(|q| q.rect[0] + 2.0);
        let (lo, hi) = xs.fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
        assert!(lo >= 300.0 && hi <= 500.0, "inside the box: {lo}..{hi}");
        assert!(hi - lo > 100.0, "spread over it: {lo}..{hi}");
        for q in &out.quads {
            let cy = q.rect[1] + 2.0;
            assert!((290.0..=310.0).contains(&cy), "y {cy}");
        }
    }

    /// The same seed and start tick draw the same; another seed or another
    /// start does not.
    #[test]
    fn seed_and_start_decide_the_draw() {
        let spread = || ParticleEmitter {
            spread_degrees: 180.0,
            ..burst()
        };
        let run = |seed: &str, from: u64| {
            let mut p = node(spread());
            p.set("seed", seed);
            for tick in from..from + 10 {
                p.step(tick, 60, [0.0, 0.0], [0.0, 0.0]);
            }
            painted(&p)
                .quads
                .iter()
                .map(|q| [q.rect[0], q.rect[1]])
                .collect::<Vec<_>>()
        };
        assert_eq!(run("3", 100), run("3", 100));
        assert_ne!(run("3", 100), run("4", 100));
        assert_ne!(run("3", 100), run("3", 101));
    }

    /// A step at a tick already stepped does nothing, so a frame drawn
    /// between ticks repaints the same particles; a step every tick
    /// allocates nothing once its slots are made.
    #[test]
    fn a_tick_steps_once() {
        let mut p = node(burst());
        p.step(0, 60, [0.0, 0.0], [0.0, 0.0]);
        p.step(1, 60, [0.0, 0.0], [0.0, 0.0]);
        let at = p.sim.slots.clone();
        p.step(1, 60, [0.0, 0.0], [0.0, 0.0]);
        assert_eq!(p.sim.slots, at);
    }

    /// A turning particle gets a transform about its own centre under the
    /// node's; `align velocity` lays its height along its flight.
    #[test]
    fn rotation_and_velocity_alignment() {
        let mut p = node(ParticleEmitter {
            rotation_degrees: [90.0, 90.0],
            ..burst()
        });
        p.step(0, 60, [100.0, 100.0], [0.0, 0.0]);
        let out = painted(&p);
        let q = out.quads[0];
        assert_ne!(q.xf, 0);
        let [a, b, c, d, e, f, _, _] = out.transforms[q.xf as usize];
        // A quarter turn about the particle's centre leaves it in place.
        let (cx, cy) = (q.rect[0] + 2.0, q.rect[1] + 2.0);
        assert!((a * cx + c * cy + e - cx).abs() < 1e-3);
        assert!((b * cx + d * cy + f - cy).abs() < 1e-3);
        assert!(a.abs() < 1e-5 && (b.abs() - 1.0).abs() < 1e-5);

        let mut p = node(ParticleEmitter {
            align: ParticleAlign::Velocity { stretch: 0.1 },
            ..burst()
        });
        p.step(0, 60, [100.0, 100.0], [0.0, 0.0]);
        let out = painted(&p);
        let q = out.quads[0];
        // Flying straight up at 60 px/s: 4 + 60 × 0.1 tall, not turned
        // off the vertical.
        assert_eq!(q.rect[2], 4.0);
        assert!((q.rect[3] - 10.0).abs() < 1e-3, "height {}", q.rect[3]);
        let m = out.transforms[q.xf as usize];
        assert!(m[1].abs() < 1e-4 && m[2].abs() < 1e-4, "upright: {m:?}");
    }

    /// Through the tree's ops and frames: a `<Particles>` paints its
    /// emitter's burst, and none of it under reduced motion.
    #[test]
    fn reduced_motion_draws_nothing() {
        let quads = |reduced: bool| {
            let mut t = Tree::new();
            t.resize(400.0, 300.0);
            let text = |b: &mut Vec<u8>, s: &str| {
                b.extend_from_slice(&(s.len() as u32).to_le_bytes());
                b.extend_from_slice(s.as_bytes());
            };
            // Create element 1 as tag 7, set its emitter, append it to
            // the root.
            let mut b = 3u32.to_le_bytes().to_vec();
            b.push(1);
            b.extend_from_slice(&1u32.to_le_bytes());
            b.push(7);
            b.push(6);
            b.extend_from_slice(&1u32.to_le_bytes());
            text(&mut b, "emitter");
            text(&mut b, "1");
            b.push(8);
            b.extend_from_slice(&0u32.to_le_bytes());
            b.extend_from_slice(&1u32.to_le_bytes());
            b.extend_from_slice(&u32::MAX.to_le_bytes());
            t.apply(&b).unwrap();
            let emitter = burst();
            t.resolve_emitters(&|_| Some(&emitter));
            let (fonts, icons) = (
                crate::fonts::Fonts::default(),
                crate::icons::IconAtlas::default(),
            );
            for tick in 0..=10u64 {
                t.set_time(tick as f64 / 60.0, reduced);
                t.set_tick(tick, 60);
                t.frame(&fonts, &icons);
            }
            t.drawn.quads.len()
        };
        assert_eq!(quads(false), 8);
        assert_eq!(quads(true), 0);
    }

    #[test]
    fn undrawn_names_what_the_ui_leaves_out() {
        assert_eq!(Particles::undrawn(&burst()), None);
        let lit = ParticleEmitter {
            lit: true,
            ..burst()
        };
        assert_eq!(Particles::undrawn(&lit), Some("lit"));
    }
}
