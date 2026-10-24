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

use super::engine::engine;
use super::extract::{Hierarchy, RenderState};
#[cfg(test)]
use glam::Mat4;
use glam::Vec3;
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

/// The world box of what `rs.scene.instances` draws for the view's entity
/// and its descendants; `of` is each instance's entity (bits).
pub(crate) fn bounds(rs: &mut RenderState, h: &Hierarchy, of: &[u64]) -> Option<(Vec3, Vec3)> {
    let target = rs.review.as_ref()?.entity;
    let picked: Vec<_> = rs
        .scene
        .instances
        .iter()
        .zip(of)
        .filter(|(_, e)| h.descends(**e, target))
        .map(|(i, _)| (i.mesh, i.transform))
        .collect();
    let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    for (mesh, m) in picked {
        let Some((a, b)) = super::geometry::mesh_bounds(rs, mesh) else {
            continue;
        };
        for k in 0..8 {
            let corner = Vec3::new(
                if k & 1 == 0 { a.x } else { b.x },
                if k & 2 == 0 { a.y } else { b.y },
                if k & 4 == 0 { a.z } else { b.z },
            );
            let p = m.transform_point3(corner);
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    (lo.cmple(hi).all() && lo.is_finite() && hi.is_finite()).then_some((lo, hi))
}

/// The camera that frames the box `lo`..`hi` from `preset` at `aspect`
/// (width over height), MARGIN free on each side of the binding extent.
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

/// The camera's view-projection matrix (tests and the turntable's notes).
#[cfg(test)]
fn view_projection(c: &Camera, aspect: f32) -> Mat4 {
    let view = glam::camera::rh::view::look_at_mat4(c.position, c.target, c.up);
    let proj = match c.projection {
        Projection::Orthographic { height } => {
            let h = height * 0.5;
            glam::camera::rh::proj::directx::orthographic(
                -h * aspect,
                h * aspect,
                -h,
                h,
                c.near,
                c.far,
            )
        }
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            aspect,
            c.near,
            c.far,
        ),
    };
    proj * view
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Where the box's corners land in normalised device coordinates.
    fn ndc(c: &Camera, aspect: f32, lo: Vec3, hi: Vec3) -> Vec<Vec3> {
        let m = view_projection(c, aspect);
        (0..8)
            .map(|k| {
                let p = Vec3::new(
                    if k & 1 == 0 { lo.x } else { hi.x },
                    if k & 2 == 0 { lo.y } else { hi.y },
                    if k & 4 == 0 { lo.z } else { hi.z },
                );
                m.project_point3(p)
            })
            .collect()
    }

    #[test]
    fn the_orbit_fits_the_bounding_sphere_inside_the_frame() {
        let (lo, hi) = (Vec3::new(-1.0, 0.0, -0.25), Vec3::new(1.0, 1.0, 0.25));
        for aspect in [0.5f32, 1.0, 16.0 / 9.0] {
            for degrees in [0.0f32, 37.0, 90.0, 200.0] {
                let c = camera(Preset::Orbit(degrees), lo, hi, aspect);
                for p in ndc(&c, aspect, lo, hi) {
                    assert!(
                        p.x.abs() < 1.0 && p.y.abs() < 1.0,
                        "{degrees}° at {aspect}: {p}"
                    );
                    assert!((0.0..1.0).contains(&p.z), "inside near and far: {p}");
                }
            }
        }
        // 90° looks from +X: the eye's x is the largest of the orbit's.
        let side = camera(Preset::Orbit(90.0), lo, hi, 1.0);
        assert!(side.position.x > 1.0 && side.position.z.abs() < 1e-3);
        assert!(side.position.y > 0.5, "above the horizon");
    }
}
