//! `og.render.light2dAt`: the 2D light reaching a point, for game rules.
//!
//! The light map is presentation (the GPU's polar shadow maps,
//! og-render's `light2d.rs`); a system that asks "is the player lit" gets
//! the same lights and occluders evaluated here on the CPU, with hard
//! shadows through the same outlines. It may run inside the tick and its
//! answer may be stored, so every number comes from + - * /, sqrt and
//! og-core's deterministic transcendentals (`og_core::math::pow`, `cos`),
//! with poses composed by `og_core::math::world_pose`: never std's or
//! libm's trig, never the shader (`light2d.wgsl` uses the GPU's atan and
//! pow, which differ by vendor).

use crate::engine::{engine, running};
use crate::host::wire::{FromJson, record};
use glam::{Quat, Vec2, Vec3};
use og_ecs::{Entity, QueryDesc, SchemaId, World};
use serde_json::Value;

/// A light as the evaluation sees it: world position and axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Light {
    pub position: Vec2,
    /// A spot's axis, unit length.
    pub axis: Vec2,
    /// Colour times intensity.
    pub color: Vec3,
    pub radius: f32,
    pub falloff: f32,
    pub cast_shadows: bool,
    /// 0 point, 1 spot, 2 global.
    pub kind: u32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub layers: u32,
}

/// An occluder's outline in world space, in order around it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Occluder {
    pub outline: Vec<Vec2>,
    pub layers: u32,
}

/// An occluder's outline in its own frame: the polygon of `points` (at
/// least three x, y pairs, at most sixteen), else the width by height
/// rectangle about the origin. The renderer's `occluder_2d` reads the same.
pub(crate) fn local_outline(points: &[f32], width: f32, height: f32) -> Vec<Vec2> {
    let n = (points.len() / 2).min(16);
    if n >= 3 {
        return (0..n)
            .map(|i| Vec2::new(points[2 * i], points[2 * i + 1]))
            .collect();
    }
    let (w, h) = (width * 0.5, height * 0.5);
    vec![
        Vec2::new(-w, -h),
        Vec2::new(w, -h),
        Vec2::new(w, h),
        Vec2::new(-w, h),
    ]
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Whether `p` is inside the closed polygon `poly` (even-odd).
pub(crate) fn contains(poly: &[Vec2], p: Vec2) -> bool {
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// Whether the segments p-q and a-b cross (touching counts).
fn crosses(p: Vec2, q: Vec2, a: Vec2, b: Vec2) -> bool {
    let r = q - p;
    let s = b - a;
    let den = cross(r, s);
    if den == 0.0 {
        return false;
    }
    let t = cross(a - p, s) / den;
    let u = cross(a - p, r) / den;
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The light reaching `point` on the light layers `mask`.
pub(crate) fn light_at(point: Vec2, mask: u32, lights: &[Light], occluders: &[Occluder]) -> Vec3 {
    let mut sum = Vec3::ZERO;
    for l in lights.iter().filter(|l| l.layers & mask != 0) {
        if l.kind == 2 {
            sum += l.color;
            continue;
        }
        let to = point - l.position;
        let dist = to.length();
        let r = l.radius.max(1e-3);
        if dist >= r {
            continue;
        }
        // Faded to zero slope over the last tenth (light2d.wgsl does the same).
        let mut atten = og_core::math::pow(1.0 - dist / r, l.falloff.max(0.01))
            * (1.0 - smoothstep(0.9, 1.0, dist / r));
        if l.kind == 1 {
            let outer = l
                .outer_angle
                .max(l.inner_angle)
                .clamp(0.0, std::f32::consts::PI);
            let inner = l.inner_angle.clamp(0.0, outer);
            let (co, ci) = (og_core::math::cos(outer), og_core::math::cos(inner));
            let c = if dist > 1e-4 {
                (to.x * l.axis.x + to.y * l.axis.y) / dist
            } else {
                1.0
            };
            atten *= smoothstep(co, ci.max(co + 1e-4), c);
        }
        if atten <= 0.0 {
            continue;
        }
        if l.cast_shadows && dist > 1e-4 {
            let blocked = occluders.iter().any(|o| {
                o.layers & l.layers != 0
                    && o.outline.len() >= 3
                    && !contains(&o.outline, point)
                    && !contains(&o.outline, l.position)
                    && (0..o.outline.len()).any(|i| {
                        let (a, b) = (o.outline[i], o.outline[(i + 1) % o.outline.len()]);
                        crosses(l.position, point, a, b)
                    })
            });
            if blocked {
                continue;
            }
        }
        sum += l.color * atten;
    }
    sum
}

// ── reading the world ────────────────────────────────────────────────────

/// What the call reads: the world between ticks, or a running system's
/// context (which refuses a record the system did not declare).
trait Reader {
    fn id(&self, name: &str) -> Option<SchemaId>;
    fn entities(&self, name: &str) -> Result<Vec<Entity>, String>;
    fn bytes(&self, e: Entity, id: SchemaId) -> Result<Vec<u8>, String>;
}

fn query(name: &str) -> QueryDesc {
    QueryDesc {
        read: vec![name.to_string()],
        ..QueryDesc::default()
    }
}

impl Reader for &World {
    fn id(&self, name: &str) -> Option<SchemaId> {
        self.id_by_name(name).ok()
    }
    fn entities(&self, name: &str) -> Result<Vec<Entity>, String> {
        let q = self.query(&query(name)).map_err(|e| e.to_string())?;
        Ok(q.flat_map(|a| a.entities().to_vec()).collect())
    }
    fn bytes(&self, e: Entity, id: SchemaId) -> Result<Vec<u8>, String> {
        self.get_bytes(e, id).map_err(|e| e.to_string())
    }
}

impl Reader for &og_ecs::SystemContext<'_> {
    fn id(&self, name: &str) -> Option<SchemaId> {
        self.id_by_name(name).ok()
    }
    fn entities(&self, name: &str) -> Result<Vec<Entity>, String> {
        let q = self.query_with(&query(name)).map_err(|e| e.to_string())?;
        Ok(q.flat_map(|a| a.entities().to_vec()).collect())
    }
    fn bytes(&self, e: Entity, id: SchemaId) -> Result<Vec<u8>, String> {
        self.get_bytes(e, id).map_err(|e| e.to_string())
    }
}

type Pose = (Vec3, Quat, Vec3);

/// Lights and occluders at their world poses, in entity order.
fn gather(
    r: &impl Reader,
    layouts: &std::collections::BTreeMap<String, crate::host::codec::Layout>,
) -> Result<(Vec<Light>, Vec<Occluder>), String> {
    let rows = |name: &str| -> Result<Vec<(Entity, Value)>, String> {
        let (Some(id), Some(layout)) = (r.id(name), layouts.get(name)) else {
            return Ok(Vec::new());
        };
        let mut es = r.entities(name)?;
        og_core::sort::sort_by_key(&mut es, |e| e.index());
        es.into_iter()
            .map(|e| Ok((e, layout.decode(&r.bytes(e, id)?))))
            .collect()
    };
    let lights = rows("og.Light2D")?;
    let occluders = rows("og.Occluder2D")?;
    if lights.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let transform = r.id("og.Transform").zip(layouts.get("og.Transform"));
    let parent = r.id("og.Parent");
    let local = |e: Entity| -> Result<Option<Pose>, String> {
        let Some((id, layout)) = transform else {
            return Ok(None);
        };
        let Ok(b) = r.bytes(e, id) else {
            return Ok(None);
        };
        let t: record::Transform = layout.read(&b)?;
        Ok(Some((t.position, t.quaternion, t.scale)))
    };
    let pose = |e: Entity| -> Pose {
        let own = local(e)
            .ok()
            .flatten()
            .unwrap_or((Vec3::ZERO, Quat::IDENTITY, Vec3::ONE));
        og_core::math::world_pose(
            e,
            own,
            |c| {
                let b = r.bytes(c, parent?).ok()?;
                let bits = u64::from_le_bytes(b.get(0..8)?.try_into().ok()?);
                (bits != 0).then(|| Entity::from_bits(bits))
            },
            |p| local(p).ok().flatten(),
        )
    };
    // A system that did not declare og.Transform or og.Parent would read
    // every pose as the identity: refuse instead.
    if let Some((e, _)) = lights.first() {
        if let Some((id, _)) = transform
            && let Err(err) = r.bytes(*e, id)
            && err.contains("declare")
        {
            return Err(err);
        }
        if let Some(id) = parent
            && let Err(err) = r.bytes(*e, id)
            && err.contains("declare")
        {
            return Err(err);
        }
    }
    let lights = lights
        .into_iter()
        .map(|(e, v)| {
            let v = record::Light2D::from_json(&v)?;
            let (p, q, _) = pose(e);
            let x = og_core::facing::forward_2d(q);
            let axis = Vec2::new(x.x, x.y);
            let len = axis.length();
            Ok(Light {
                position: Vec2::new(p.x, p.y),
                axis: if len > 0.0 { axis / len } else { Vec2::X },
                color: Vec3::new(v.color.r, v.color.g, v.color.b) * v.intensity,
                radius: v.distance,
                falloff: v.falloff,
                cast_shadows: v.cast_shadows,
                kind: v.kind as u32,
                inner_angle: v.inner_angle,
                outer_angle: v.outer_angle,
                layers: v.layers,
            })
        })
        .collect::<Result<_, String>>()?;
    let occluders = occluders
        .into_iter()
        .map(|(e, v)| {
            let v = record::Occluder2D::from_json(&v)?;
            let (p, q, s) = pose(e);
            let outline = local_outline(&v.points, v.width, v.height)
                .into_iter()
                .map(|l| {
                    let w = p + q * (s * Vec3::new(l.x, l.y, 0.0));
                    Vec2::new(w.x, w.y)
                })
                .collect();
            Ok(Occluder {
                outline,
                layers: v.layers,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok((lights, occluders))
}

pub fn og_render_light2d_at(
    world: u32,
    point: String,
    layers: Option<u32>,
) -> Result<crate::host::wire::Color, String> {
    (|| -> Result<crate::host::wire::Color, String> {
        let eng = engine(world)?;
        let at: Vec2 = crate::host::wire::arg(Some(&point), "point")?;
        if !at.is_finite() {
            return Err("light2dAt: the point must be finite { x, y }".into());
        }
        let state = eng.state.borrow();
        let (lights, occluders) = match running(world) {
            // SAFETY: the frame is published while its system runs.
            Some(fr) => gather(&&*unsafe { fr.ctx() }, &state.layouts),
            None => gather(&&*eng.world()?, &state.layouts),
        }
        .map_err(|e| format!("light2dAt: {e}"))?;
        let c = light_at(at, layers.unwrap_or(1), &lights, &occluders);
        Ok(crate::host::wire::Color {
            r: c.x,
            g: c.y,
            b: c.z,
        })
    })()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(position: Vec2) -> Light {
        Light {
            position,
            axis: Vec2::X,
            color: Vec3::new(1.0, 0.5, 0.25),
            radius: 6.0,
            falloff: 1.7,
            cast_shadows: true,
            kind: 0,
            inner_angle: 0.35,
            outer_angle: 0.6,
            layers: u32::MAX,
        }
    }

    fn wall(center: Vec2, w: f32, h: f32) -> Occluder {
        Occluder {
            outline: local_outline(&[], w, h)
                .into_iter()
                .map(|p| p + center)
                .collect(),
            layers: u32::MAX,
        }
    }

    #[test]
    fn falloff_shadow_inside_and_layers() {
        let lights = [point(Vec2::ZERO)];
        let occ = [wall(Vec2::new(2.0, 0.0), 0.4, 2.0)];
        // Behind the wall: dark; beside it: lit by (1 - 3/6)^1.7.
        assert_eq!(light_at(Vec2::new(3.0, 0.0), 1, &lights, &occ), Vec3::ZERO);
        let open = light_at(Vec2::new(0.0, 3.0), 1, &lights, &occ);
        // 0.5^1.7, worked out apart from og-core.
        assert!((open.x - 0.307_786_1).abs() < 1e-6, "{open}");
        // Inside the wall itself: not shadowed by it.
        assert!(light_at(Vec2::new(2.1, 0.0), 1, &lights, &occ).x > 0.0);
        // An occluder on other layers than the light's blocks nothing.
        let other = [Occluder {
            layers: 0,
            ..occ[0].clone()
        }];
        assert!(light_at(Vec2::new(3.0, 0.0), 1, &lights, &other).x > 0.0);
        // A light on no layer of the point's does not reach it; past the radius nothing does.
        let red = [Light {
            layers: 2,
            ..lights[0]
        }];
        assert_eq!(light_at(Vec2::new(0.0, 3.0), 1, &red, &[]), Vec3::ZERO);
        assert_eq!(light_at(Vec2::new(0.0, 6.5), 1, &lights, &[]), Vec3::ZERO);
    }

    #[test]
    fn a_concave_polygon_shadows_only_behind_its_body() {
        // A U open to +y: a point in its notch sees a light above it.
        let u = Occluder {
            outline: local_outline(
                &[
                    -1.0, 0.0, 1.0, 0.0, 1.0, 2.0, 0.5, 2.0, 0.5, 0.5, -0.5, 0.5, -0.5, 2.0, -1.0,
                    2.0,
                ],
                0.0,
                0.0,
            ),
            layers: u32::MAX,
        };
        let lights = [point(Vec2::new(0.0, 4.0))];
        assert!(light_at(Vec2::new(0.0, 1.0), 1, &lights, std::slice::from_ref(&u)).x > 0.0);
        assert_eq!(light_at(Vec2::new(0.0, -0.5), 1, &lights, &[u]), Vec3::ZERO);
    }

    #[test]
    fn a_light_ends_at_its_radius_without_a_rim() {
        // falloff 0.5 alone is still 0.07 a hair inside the radius (sqrt of
        // 0.005); faded over the last tenth it is near 0 there, and whole
        // short of the fade.
        let soft = [Light {
            falloff: 0.5,
            color: Vec3::ONE,
            ..point(Vec2::ZERO)
        }];
        assert!(light_at(Vec2::new(5.97, 0.0), 1, &soft, &[]).x < 0.001);
        let inside = light_at(Vec2::new(3.0, 0.0), 1, &soft, &[]).x;
        assert!((inside - 0.5f32.sqrt()).abs() < 1e-6, "{inside}");
    }

    #[test]
    fn a_spot_lights_its_cone_only() {
        let spot = [Light {
            kind: 1,
            axis: Vec2::Y,
            ..point(Vec2::ZERO)
        }];
        assert!(light_at(Vec2::new(0.0, 2.0), 1, &spot, &[]).x > 0.0);
        assert_eq!(light_at(Vec2::new(2.0, 0.0), 1, &spot, &[]), Vec3::ZERO);
    }

    /// The bits are pinned: a game may store the answer, so a change here
    /// moves every world that did. Inputs are asymmetric (a spot off axis,
    /// a fractional falloff) so a swapped operand changes them.
    #[test]
    fn the_bits_are_pinned() {
        let lights = [
            point(Vec2::new(0.25, -0.5)),
            Light {
                kind: 1,
                axis: Vec2::new(0.6, 0.8),
                color: Vec3::new(0.3, 0.9, 1.1),
                falloff: 2.3,
                radius: 7.5,
                ..point(Vec2::new(-1.0, -1.0))
            },
        ];
        let occ = [wall(Vec2::new(-3.0, 0.0), 0.5, 3.0)];
        let c = light_at(Vec2::new(0.7, 1.9), 1, &lights, &occ);
        let bits = [c.x.to_bits(), c.y.to_bits(), c.z.to_bits()];
        // By hand: r = (1 - 2.4419/6)^1.7 + 0.3 (1 - 3.3615/7.5)^2.3 = 0.41136 + 0.07642.
        assert_eq!(bits, [1056554820, 1054781690, 1053040382], "{c}");
    }
}
