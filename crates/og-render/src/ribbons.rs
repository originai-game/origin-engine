//! Trails, ribbons and beams: camera-facing strips built on the CPU each
//! frame and drawn blended in the 3D pass (one draw per texture and blend).
//!
//! - A [`Trail`] is a per-entity history: the renderer remembers the
//!   positions it was given tick by tick (by `id`) for its style's
//!   lifetime, so the game only moves the head. Width and colour follow the
//!   style's curves over the trail's length (0 at the head, 1 at the tail).
//! - A [`Ribbon`] is an explicit polyline with a width and colour per point.
//! - A [`Beam`] joins two points (lasers, lightning, tethers): its texture
//!   scrolls along it with the scene's clock and it can jitter.

use crate::particles::{ColorCurve, Curve, ParticleBlend, hash_seed, noise1};
use crate::scene::TextureHandle;
use glam::{Vec3, Vec4};
use std::collections::{HashMap, VecDeque};

/// How a strip's texture runs along it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RibbonUv {
    /// Once over the whole length.
    Stretch,
    /// Repeating every `length` world units.
    Tile { length: f32 },
}

/// The look of a trail: an asset ([`crate::RenderScene::add_trail_style`]).
#[derive(Clone, Debug, PartialEq)]
pub struct TrailStyle {
    /// How long a remembered position lasts, in seconds.
    pub lifetime: f32,
    /// A new point is kept when the head has moved this far since the last.
    pub min_distance: f32,
    /// Width in world units over the length (head 0, tail 1).
    pub width: Curve,
    /// Linear RGBA over the length; above 1 blooms.
    pub color: ColorCurve,
    pub texture: Option<TextureHandle>,
    pub uv: RibbonUv,
    pub blend: ParticleBlend,
}

impl Default for TrailStyle {
    fn default() -> Self {
        TrailStyle {
            lifetime: 0.4,
            min_distance: 0.05,
            width: Curve::linear(0.15, 0.0),
            color: ColorCurve::new(&[(0.0, [1.0; 4]), (1.0, [1.0, 1.0, 1.0, 0.0])]),
            texture: None,
            uv: RibbonUv::Stretch,
            blend: ParticleBlend::Additive,
        }
    }
}

/// A handle to a trail style of a [`crate::RenderScene`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrailStyleHandle(pub(crate) u32);

impl TrailStyleHandle {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// One trailing entity this frame: its head and its clock. The renderer
/// keeps the history by `id` (an entity absent from a frame loses it; a tick
/// that goes backwards restarts it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trail {
    pub id: u64,
    pub style: TrailStyleHandle,
    pub position: glam::Vec3,
    pub tick: u64,
}

/// One point of a [`Ribbon`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RibbonPoint {
    pub position: Vec3,
    pub width: f32,
    /// Linear RGBA.
    pub color: [f32; 4],
}

/// An explicit camera-facing strip through `points`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ribbon {
    pub points: Vec<RibbonPoint>,
    pub texture: Option<TextureHandle>,
    pub uv: RibbonUv,
    pub blend: ParticleBlend,
}

/// A strip between two points whose texture scrolls along it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    /// Linear RGBA (above 1 blooms).
    pub color: [f32; 4],
    pub texture: Option<TextureHandle>,
    /// Texture repeats per world unit along the beam.
    pub tile: f32,
    /// Texture scroll in repeats per second (by the scene's tick).
    pub scroll: f32,
    /// Segments (1 straight; more to jitter).
    pub segments: u32,
    /// Sideways jitter in world units (lightning), changing every tick.
    pub jitter: f32,
    pub seed: u64,
    pub blend: ParticleBlend,
}

impl Default for Beam {
    fn default() -> Self {
        Beam {
            start: Vec3::ZERO,
            end: Vec3::Y,
            width: 0.1,
            color: [1.0; 4],
            texture: None,
            tile: 1.0,
            scroll: 0.0,
            segments: 1,
            jitter: 0.0,
            seed: 0,
            blend: ParticleBlend::Additive,
        }
    }
}

/// `struct RibbonIn` in ribbon.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RibbonVertex {
    position_v: [f32; 4],
    color: [f32; 4],
    /// x: u, y: 1 when textured.
    extra: [f32; 4],
}

impl RibbonVertex {
    /// A vertex at `position` with strip coordinates (`u`, `v`); textured
    /// vertices multiply the run's texture (white without one), others get
    /// the soft edge across v.
    pub(crate) fn new(
        position: glam::Vec3,
        u: f32,
        v: f32,
        color: [f32; 4],
        textured: bool,
    ) -> Self {
        RibbonVertex {
            position_v: position.extend(v).to_array(),
            color,
            extra: [u, f32::from(textured), 0.0, 0.0],
        }
    }
}

pub(crate) const RIBBON_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<RibbonVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
};

/// A run of strip vertices drawn with one texture and blend.
pub(crate) struct RibbonRun {
    pub texture: Option<TextureHandle>,
    pub blend: ParticleBlend,
    /// Drawn in the sprite order at this layer (a 2D trail), not with the 3D scene.
    pub layer: Option<i32>,
    pub first: u32,
    pub count: u32,
}

#[derive(Default)]
pub(crate) struct RibbonFrame {
    pub vertices: Vec<RibbonVertex>,
    pub runs: Vec<RibbonRun>,
    pub strips: u32,
    pub segments: u32,
}

struct TrailHistory {
    style: usize,
    points: VecDeque<(Vec3, u64)>,
    last_tick: u64,
    seen: u64,
}

/// The renderer's trail histories, by id.
#[derive(Default)]
pub(crate) struct Trails {
    histories: HashMap<u64, TrailHistory>,
    scene_id: u64,
    frame: u64,
}

struct Strip {
    layer: Option<i32>,
    texture: Option<TextureHandle>,
    blend: ParticleBlend,
    uv: RibbonUv,
    points: Vec<RibbonPoint>,
    /// Added to u (beam scroll).
    u_offset: f32,
}

impl Trails {
    /// Updates the histories and builds every strip of the frame. `eye` and
    /// `forward` orient the strips (an orthographic camera faces along
    /// `forward` everywhere).
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        scene_id: u64,
        styles: &[TrailStyle],
        trails: &[Trail],
        trail_layers: &[(u64, i32)],
        ribbons: &[Ribbon],
        beams: &[Beam],
        tick: u64,
        tick_rate: f32,
        eye: Vec3,
        forward: Vec3,
        orthographic: bool,
    ) -> Result<RibbonFrame, String> {
        self.frame += 1;
        if self.scene_id != scene_id {
            self.scene_id = scene_id;
            self.histories.clear();
        }
        let tick_rate = tick_rate.max(1e-3);
        let mut strips: Vec<Strip> = Vec::new();
        for t in trails {
            let style = styles
                .get(t.style.index())
                .ok_or_else(|| format!("trail {}: style {}", t.id, t.style.index()))?;
            let life_ticks = (style.lifetime.max(0.0) * tick_rate).ceil() as u64;
            let h = self.histories.entry(t.id).or_insert_with(|| TrailHistory {
                style: t.style.index(),
                points: VecDeque::new(),
                last_tick: t.tick,
                seen: 0,
            });
            if h.style != t.style.index() || t.tick < h.last_tick {
                h.points.clear();
                h.style = t.style.index();
            }
            h.seen = self.frame;
            if h.points.is_empty() {
                h.points.push_back((t.position, t.tick));
            } else if t.tick > h.last_tick {
                // One sample per elapsed tick along the path since the last frame.
                let (from, _) = *h.points.back().expect("non-empty");
                let steps = (t.tick - h.last_tick).min(life_ticks.max(1) + 1);
                for s in 1..=steps {
                    let f = s as f32 / steps as f32;
                    let p = from.lerp(t.position, f);
                    let at = t.tick - (steps - s);
                    let last = h.points.back().expect("non-empty").0;
                    if p.distance(last) >= style.min_distance {
                        h.points.push_back((p, at));
                    }
                }
            }
            h.last_tick = t.tick;
            while h
                .points
                .front()
                .is_some_and(|&(_, at)| t.tick.saturating_sub(at) > life_ticks)
            {
                h.points.pop_front();
            }
            // Head first: the live position, then the history newest to oldest.
            let mut path: Vec<Vec3> = vec![t.position];
            for &(p, _) in h.points.iter().rev() {
                if p.distance(*path.last().expect("head")) > 1e-5 {
                    path.push(p);
                }
            }
            if path.len() < 2 {
                continue;
            }
            let lengths = cumulative(&path);
            let total = lengths.last().copied().unwrap_or(0.0).max(1e-6);
            let points = path
                .iter()
                .zip(&lengths)
                .map(|(&p, &s)| {
                    let f = s / total;
                    RibbonPoint {
                        position: p,
                        width: style.width.eval(f),
                        color: style.color.eval(f),
                    }
                })
                .collect();
            strips.push(Strip {
                layer: trail_layers
                    .iter()
                    .find(|(id, _)| *id == t.id)
                    .map(|(_, l)| *l),
                texture: style.texture,
                blend: style.blend,
                uv: style.uv,
                points,
                u_offset: 0.0,
            });
        }
        let frame = self.frame;
        self.histories.retain(|_, h| h.seen == frame);
        for r in ribbons.iter().filter(|r| r.points.len() >= 2) {
            strips.push(Strip {
                layer: None,
                texture: r.texture,
                blend: r.blend,
                uv: r.uv,
                points: r.points.clone(),
                u_offset: 0.0,
            });
        }
        let time = tick as f32 / tick_rate;
        for b in beams {
            let segments = b.segments.clamp(1, 256);
            let axis = b.end - b.start;
            let dir = axis.normalize_or(Vec3::Y);
            let helper = if dir.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
            let side = dir.cross(helper).normalize();
            let up = dir.cross(side);
            let seed = hash_seed(b.seed) ^ (tick as u32).wrapping_mul(0x85eb_ca6b);
            let points = (0..=segments)
                .map(|i| {
                    let f = i as f32 / segments as f32;
                    let mut p = b.start + axis * f;
                    if b.jitter > 0.0 && i > 0 && i < segments {
                        // Zero at the ends, a sine envelope between.
                        let env = (f * std::f32::consts::PI).sin();
                        p += (side * noise1(f * 5.0, seed) + up * noise1(f * 5.0 + 17.0, seed))
                            * b.jitter
                            * env;
                    }
                    RibbonPoint {
                        position: p,
                        width: b.width,
                        color: b.color,
                    }
                })
                .collect();
            strips.push(Strip {
                layer: None,
                texture: b.texture,
                blend: b.blend,
                uv: RibbonUv::Tile {
                    length: 1.0 / b.tile.max(1e-6),
                },
                points,
                u_offset: -time * b.scroll,
            });
        }
        // Group by (texture, blend) for one draw each.
        // (Layered strips after the 3D ones, by layer: each is its own run.)
        strips.sort_by_key(|s| (s.layer, s.texture, s.blend as u8));
        let mut out = RibbonFrame::default();
        for s in &strips {
            let first = out.vertices.len() as u32;
            emit_strip(s, eye, forward, orthographic, &mut out.vertices);
            let count = out.vertices.len() as u32 - first;
            if count == 0 {
                continue;
            }
            out.strips += 1;
            out.segments += count / 6;
            match out.runs.last_mut() {
                Some(r) if r.texture == s.texture && r.blend == s.blend && r.layer == s.layer => {
                    r.count += count
                }
                _ => out.runs.push(RibbonRun {
                    texture: s.texture,
                    blend: s.blend,
                    layer: s.layer,
                    first,
                    count,
                }),
            }
        }
        Ok(out)
    }
}

fn cumulative(points: &[Vec3]) -> Vec<f32> {
    let mut s = 0.0;
    let mut out = Vec::with_capacity(points.len());
    out.push(0.0);
    for w in points.windows(2) {
        s += w[0].distance(w[1]);
        out.push(s);
    }
    out
}

/// Two triangles per segment, each point's edge across the view.
fn emit_strip(
    s: &Strip,
    eye: Vec3,
    forward: Vec3,
    orthographic: bool,
    out: &mut Vec<RibbonVertex>,
) {
    let n = s.points.len();
    let lengths = cumulative(&s.points.iter().map(|p| p.position).collect::<Vec<_>>());
    let total = lengths.last().copied().unwrap_or(0.0).max(1e-6);
    let textured = s.texture.is_some() as u32 as f32;
    let edge = |i: usize| -> (Vec3, Vec3, f32) {
        let p = s.points[i].position;
        let prev = s.points[i.saturating_sub(1)].position;
        let next = s.points[(i + 1).min(n - 1)].position;
        let tangent = (next - prev).normalize_or(Vec3::X);
        let to_eye = if orthographic {
            -forward
        } else {
            (eye - p).normalize_or(-forward)
        };
        let side = tangent.cross(to_eye).normalize_or(Vec3::Y) * (s.points[i].width * 0.5);
        let u = match s.uv {
            RibbonUv::Stretch => lengths[i] / total,
            RibbonUv::Tile { length } => lengths[i] / length.max(1e-6),
        } + s.u_offset;
        (p - side, p + side, u)
    };
    let vertex = |p: Vec3, v: f32, u: f32, c: [f32; 4]| RibbonVertex {
        position_v: p.extend(v).to_array(),
        color: c,
        extra: Vec4::new(u, textured, 0.0, 0.0).to_array(),
    };
    let mut previous = edge(0);
    for i in 1..n {
        let current = edge(i);
        let (c0, c1) = (s.points[i - 1].color, s.points[i].color);
        let (a0, b0, u0) = previous;
        let (a1, b1, u1) = current;
        out.extend_from_slice(&[
            vertex(a0, 0.0, u0, c0),
            vertex(b0, 1.0, u0, c0),
            vertex(b1, 1.0, u1, c1),
            vertex(a0, 0.0, u0, c0),
            vertex(b1, 1.0, u1, c1),
            vertex(a1, 0.0, u1, c1),
        ]);
        previous = current;
    }
}
