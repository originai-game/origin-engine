//! The model review's cameras (the command channel's `view` and
//! `turntable`): a camera that frames one entity's drawn bounds from a
//! fixed direction, so a script takes the same front, side, top and back
//! views of a model on every run, whatever og.Camera and its controllers
//! do.
//!
//! The view is rendering state, never simulation: it lives in the engine's
//! `RenderState`, replaces the camera extraction made from og.Camera (the
//! world's Camera resource is never written), and neither a snapshot nor a
//! replay carries it. The bounds are the world box of every instance drawn
//! for the entity and the entities under it through og.Parent (og.Mesh and
//! og.Model parts; a skinned model's posed parts and sprites are not
//! framed), taken once from the last frame's instances before the models
//! pick their levels of detail, so the level drawn is the one the review
//! camera sees, and again from this frame's once every instance is in.

// A publish build registers no review command, so only extraction's half is read.
#![cfg_attr(feature = "publish", allow(dead_code, unused_imports))]

use crate::engine::engine;
use crate::present::extract::{Hierarchy, RenderState};
use glam::Vec3;
use og_core::IdMap;
use og_render::{Camera, Projection};

/// The share of the framed extent left free on each side.
pub(crate) const MARGIN: f32 = 0.1;
/// The orbit's vertical field of view and its height above the horizon.
pub(crate) const ORBIT_FOV: f32 = 30.0;
pub(crate) const ORBIT_ELEVATION: f32 = 20.0;
/// The most frames one `turntable` takes.
pub(crate) const MAX_TURNTABLE: u32 = 72;

/// Where a review camera looks from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Preset {
    /// Orthographic from +Z looking down -Z, y up: a model facing +Z (the
    /// glTF and three.js convention) shows its face, its +X on the right.
    Front,
    /// Orthographic from -Z, y up: its +X on the left.
    Back,
    /// Orthographic from +X looking down -X, y up: its +Z on the left.
    Side,
    /// Orthographic from +Y looking down, -Z up: its front at the bottom.
    Top,
    /// Perspective (ORBIT_FOV) from ORBIT_ELEVATION above the horizon, the
    /// azimuth in degrees from the front towards +X (90 is Side's direction).
    Orbit(f32),
}

impl Preset {
    fn parse(name: &str, degrees: Option<&str>) -> Result<Preset, String> {
        Ok(match name {
            "front" => Preset::Front,
            "back" => Preset::Back,
            "side" => Preset::Side,
            "top" => Preset::Top,
            "orbit" => {
                let d = degrees.unwrap_or("0");
                let d: f32 = d
                    .parse()
                    .ok()
                    .filter(|d: &f32| d.is_finite())
                    .ok_or_else(|| format!("orbit:{d}: the azimuth is degrees, as orbit:45"))?;
                Preset::Orbit(d)
            }
            other => {
                return Err(format!(
                    "'{other}' is no view; write front, back, side, top or orbit:<degrees>"
                ));
            }
        })
    }

    /// The arguments `view` takes after the entity to come back here.
    fn args(self) -> Vec<String> {
        match self {
            Preset::Front => vec!["front".into()],
            Preset::Back => vec!["back".into()],
            Preset::Side => vec!["side".into()],
            Preset::Top => vec!["top".into()],
            Preset::Orbit(d) => vec!["orbit".into(), d.to_string()],
        }
    }
}

/// The review camera in force: the entity framed (bits) and how.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReviewView {
    pub entity: u64,
    /// The id the script named it by (for the turntable's way back).
    pub name: String,
    pub preset: Preset,
}

fn grow_box(lo: &mut Vec3, hi: &mut Vec3, a: Vec3, b: Vec3, m: glam::Mat4) {
    for k in 0..8 {
        let corner = Vec3::new(
            if k & 1 == 0 { a.x } else { b.x },
            if k & 2 == 0 { a.y } else { b.y },
            if k & 4 == 0 { a.z } else { b.z },
        );
        let p = m.transform_point3(corner);
        *lo = lo.min(p);
        *hi = hi.max(p);
    }
}

fn grow_sprite(lo: &mut Vec3, hi: &mut Vec3, sp: &og_render::Sprite) {
    let x0 = -sp.anchor.x * sp.size.x;
    let x1 = (1.0 - sp.anchor.x) * sp.size.x;
    let y0 = -sp.anchor.y * sp.size.y;
    let y1 = (1.0 - sp.anchor.y) * sp.size.y;
    let (sin, cos) = og_core::math::sin_cos(sp.rotation);
    for (lx, ly) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
        let p = Vec3::new(
            sp.position.x + lx * cos - ly * sin,
            sp.position.y + lx * sin + ly * cos,
            sp.position.z,
        );
        *lo = lo.min(p);
        *hi = hi.max(p);
    }
}

/// The world box of what `rs.scene` draws for the view's entity and its
/// descendants (static instances, skinned/morphed instances, and sprites);
/// `of` is each static instance's entity (bits).
pub(crate) fn bounds(rs: &mut RenderState, h: &Hierarchy, of: &[u64]) -> Option<(Vec3, Vec3)> {
    let target = rs.review.as_ref()?.entity;
    let picked: Vec<_> = rs
        .scene
        .objects
        .instances
        .iter()
        .zip(of)
        .filter(|(_, e)| h.descends(**e, target))
        .map(|(i, _)| (i.mesh, i.transform))
        .collect();
    let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for (mesh, m) in picked {
        let Some((a, b)) = crate::present::geometry::mesh_bounds(rs, mesh) else {
            continue;
        };
        grow_box(&mut lo, &mut hi, a, b, m);
    }
    if !rs.scene.objects.skinned_instances.is_empty() && !rs.atmo.palettes.is_empty() {
        let by_palette: IdMap<u32, u64> =
            IdMap::from_iter(rs.atmo.palettes.iter().map(|(&e, &p)| (p, e)));
        let mut fallback_meshes = Vec::new();
        for inst in &rs.scene.objects.skinned_instances {
            let Some(&owner) = by_palette.get(&inst.palette) else {
                continue;
            };
            if !h.descends(owner, target) {
                continue;
            }
            if let Some(di) = inst.deformed
                && let Some(def) = rs.scene.objects.deformed_vertices.get(di as usize)
                && !def.positions.is_empty()
            {
                for &p in &def.positions {
                    let wp = inst.transform.transform_point3(p);
                    lo = lo.min(wp);
                    hi = hi.max(wp);
                }
                continue;
            }
            if let (Some(mesh), Some(skin), Some(palette)) = (
                rs.scene.mesh(inst.mesh),
                rs.scene.skin(inst.mesh),
                rs.scene.objects.joint_palettes.get(inst.palette as usize),
            ) && skin.joints.len() == mesh.vertices.len()
                && skin.weights.len() == mesh.vertices.len()
                && !palette.matrices.is_empty()
            {
                for ((v, joints), weights) in
                    mesh.vertices.iter().zip(&skin.joints).zip(&skin.weights)
                {
                    let p0 = Vec3::from_array(v.position);
                    let mut posed = Vec3::ZERO;
                    let mut sum = 0.0f32;
                    for k in 0..4 {
                        let w = weights[k];
                        if w > 0.0 {
                            let m = palette
                                .matrices
                                .get(joints[k] as usize)
                                .copied()
                                .unwrap_or(glam::Mat4::IDENTITY);
                            posed += m.transform_point3(p0) * w;
                            sum += w;
                        }
                    }
                    let local = if sum > 0.0 { posed / sum } else { p0 };
                    let wp = inst.transform.transform_point3(local);
                    lo = lo.min(wp);
                    hi = hi.max(wp);
                }
                continue;
            }
            fallback_meshes.push((inst.mesh, inst.transform));
        }
        for (mesh, m) in fallback_meshes {
            if let Some((a, b)) = crate::present::geometry::mesh_bounds(rs, mesh) {
                grow_box(&mut lo, &mut hi, a, b, m);
            }
        }
    }
    for &(idx, owner) in &rs.ids.sprites {
        if !h.descends(owner, target) {
            continue;
        }
        if let Some(sp) = rs.scene.sprite.sprites.get(idx as usize) {
            grow_sprite(&mut lo, &mut hi, sp);
        }
    }
    (lo.cmple(hi).all() && lo.is_finite() && hi.is_finite()).then_some((lo, hi))
}

/// The camera that frames the box `lo`..`hi` from `preset` at `aspect`
/// (width over height), MARGIN free on each side of the binding extent.
// Tooling only: the review orbit camera.
#[allow(clippy::disallowed_methods)]
pub(crate) fn camera(preset: Preset, lo: Vec3, hi: Vec3, aspect: f32) -> Camera {
    let aspect = if aspect.is_finite() && aspect > 0.0 {
        aspect
    } else {
        1.0
    };
    let centre = (lo + hi) * 0.5;
    let half = ((hi - lo) * 0.5).max(Vec3::splat(1e-4));
    let radius = half.length();
    // Direction from the centre to the eye, the up vector, and the half
    // extents seen across and up.
    let (back, up, across, upward) = match preset {
        Preset::Front => (Vec3::Z, Vec3::Y, half.x, half.y),
        Preset::Back => (Vec3::NEG_Z, Vec3::Y, half.x, half.y),
        Preset::Side => (Vec3::X, Vec3::Y, half.z, half.y),
        Preset::Top => (Vec3::Y, Vec3::NEG_Z, half.x, half.z),
        Preset::Orbit(degrees) => {
            let (az, el) = (degrees.to_radians(), ORBIT_ELEVATION.to_radians());
            let back = Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos());
            // The bounding sphere fits the narrower field of view.
            let half_v = (ORBIT_FOV.to_radians() * 0.5).tan();
            let half_fov = half_v.min(half_v * aspect).atan();
            let distance = radius * (1.0 + 2.0 * MARGIN) / half_fov.sin();
            return Camera {
                position: centre + back * distance,
                target: centre,
                up: Vec3::Y,
                projection: Projection::Perspective {
                    fov_y_degrees: ORBIT_FOV,
                },
                near: (distance - radius * 1.5).max(distance * 0.01),
                far: distance + radius * 1.5,
                ..Camera::default()
            };
        }
    };
    let height = (2.0 * upward).max(2.0 * across / aspect) * (1.0 + 2.0 * MARGIN);
    let distance = radius * 2.0 + 1.0;
    Camera {
        position: centre + back * distance,
        target: centre,
        up,
        projection: Projection::Orthographic { height },
        near: (distance - radius * 1.5).max(0.01),
        far: distance + radius * 1.5,
        ..Camera::default()
    }
}

/// Points `cam` at `view`'s framing of `bounds` on a target of `size`,
/// keeping its exposure; nothing without a view or bounds.
pub(crate) fn apply(
    cam: &mut Camera,
    view: Option<&ReviewView>,
    size: (u32, u32),
    bounds: Option<(Vec3, Vec3)>,
) {
    let (Some(view), Some((lo, hi))) = (view, bounds) else {
        return;
    };
    let framed = camera(
        view.preset,
        lo,
        hi,
        size.0.max(1) as f32 / size.1.max(1) as f32,
    );
    *cam = Camera {
        exposure: cam.exposure,
        ..framed
    };
}

/// The command channel's `view` and `turntable` (tools: not in a publish build).
#[cfg(not(feature = "publish"))]
pub(crate) fn register(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "view",
        og_probe::CommandDesc {
            usage: "view:<entity>:front|back|side|top|orbit[:<degrees>] | view:off".into(),
            doc: "frame an entity's drawn bounds (with its og.Parent children) from a fixed direction until view:off, in place of og.Camera (never written): front, back, side and top are orthographic (front from +Z, side from +X, top from +Y with the front at the bottom), orbit a 30° perspective 20° above the horizon at an azimuth from the front towards +X; 10% of the frame free on each side".into(),
        },
        move |world, args| {
            let first = args.first().map(String::as_str).unwrap_or("");
            let eng = engine(engine_id)?;
            if first == "off" {
                eng.render.borrow_mut().review = None;
                return Ok(());
            }
            let usage = "write view:<entity>:front|back|side|top|orbit[:<degrees>] or view:off";
            if first.is_empty() {
                return Err(usage.into());
            }
            let e = og_probe::Names
                .resolve(world, first)
                .ok_or_else(|| format!("no live entity '{first}'"))?;
            let name = args.get(1).ok_or(usage)?;
            let preset = Preset::parse(name, args.get(2).map(String::as_str))?;
            eng.render.borrow_mut().review = Some(ReviewView {
                entity: e.to_bits(),
                name: first.to_owned(),
                preset,
            });
            Ok(())
        },
    );
    let inbox = probe.inbox();
    let _ = probe.register_command(
        "turntable",
        og_probe::CommandDesc {
            usage: "turntable:<entity>:<n>[:<prefix>]".into(),
            doc: "take n frames (at most 72) of an entity orbited at view's orbit, azimuths 360/n apart from the front, at the start of the next tick and all of the same tick; written as <prefix>-00.png, <prefix>-01.png, ... (default prefix turntable-<entity>); the view in force before comes back after".into(),
        },
        move |world, args| {
            let usage = "write turntable:<entity>:<n>[:<prefix>]";
            let id = args.first().filter(|a| !a.is_empty()).ok_or(usage)?;
            og_probe::Names
                .resolve(world, id)
                .ok_or_else(|| format!("no live entity '{id}'"))?;
            let n: u32 = args
                .get(1)
                .and_then(|n| n.parse().ok())
                .filter(|n| (1..=MAX_TURNTABLE).contains(n))
                .ok_or_else(|| format!("the frame count is 1 to {MAX_TURNTABLE}; {usage}"))?;
            let prefix = args
                .get(2)
                .filter(|p| !p.is_empty())
                .cloned()
                .unwrap_or_else(|| format!("turntable-{id}"));
            let before = engine(engine_id)?.render.borrow().review.clone();
            for k in 0..n {
                let degrees = 360.0 * k as f32 / n as f32;
                inbox.push(
                    "view",
                    vec![id.clone(), "orbit".into(), degrees.to_string()],
                );
                inbox.push("screenshot", vec![format!("{prefix}-{k:02}.png")]);
            }
            match before {
                Some(v) => {
                    let mut back = vec![v.name];
                    back.extend(v.preset.args());
                    inbox.push("view", back);
                }
                None => inbox.push("view", vec!["off".into()]),
            }
            Ok(())
        },
    );
}
