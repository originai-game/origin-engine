//! Render interpolation between ticks (docs/design.md §2.4: "render
//! interpolates between the last two ticks"; IDL RenderSettings.interpolate).
//!
//! A display frame (`og_engine_frame`) whose accumulator holds part of a
//! tick draws the world between the last two ticks, by alpha = the
//! accumulator over the tick's length (0 the tick before the last, 1 the
//! last): every world matrix extraction asks the frame's hierarchy for
//! (models and skinned models' roots, meshes, lights, sprites, particle
//! emitters, decals, secondary cameras, everything placed on an entity)
//! and og.Camera's position and rotation. The tick before's transforms are
//! the raycast record's hierarchy, which `super::geometry::before_tick`
//! reads as every tick starts; og.Camera's pose is read here then
//! ([`before_tick`]).
//!
//! Presentation only: nothing here writes the world, and the blend is set
//! only while `engine::tick`'s render extracts a display frame
//! ([`InterpState::alpha`]), so the probe's screenshots and captures,
//! `describe` and `pick` extract the tick. A frame on a tick boundary (the
//! accumulator empty before and after it: a host stepping whole ticks, as
//! og run, og render and og test do) has no alpha and draws the tick, so
//! reference frames are the ticks'.

use super::extract::{Blend, Hierarchy, RenderState, f, on, rows};
use glam::{Mat4, Quat, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::World;
use serde_json::Value;

/// RenderSettings.snapDistance's default.
const SNAP_DISTANCE: f32 = 10.0;

/// The renderer's interpolation state (in `RenderState`, never the world).
#[derive(Default)]
pub(crate) struct InterpState {
    /// The display frame being extracted: its alpha, `None` for the tick
    /// (every other extraction, and a frame on a tick boundary).
    pub alpha: Option<f32>,
    /// og.Camera's position and rotation as the last tick started, with
    /// that tick.
    camera: Option<(u64, Vec3, Quat)>,
    /// og.render.snap: entity bits → the tick it was asked at.
    snaps: IdMap<u64, u64>,
    /// og.render.snapCamera's tick.
    camera_snap: Option<u64>,
    /// The blend of the frame being extracted: alpha, the squared snap
    /// distance and the tick the blend starts from.
    active: Option<(f32, f32, u64)>,
    /// The alpha the last display frame was drawn at, `None` at its tick
    /// (dump_render's `interpolation`).
    pub drawn: Option<f32>,
    /// The world was replaced since the last tick started (a restore, a
    /// reload): until the next tick, nothing to blend from.
    stale: bool,
    /// og-anim's output as the last tick started, by entity bits: each
    /// skeleton's joint palette and blendshape weights.
    poses: IdMap<u64, Pose>,
    /// Set while `engine::tick`'s render extracts a display frame (with
    /// or without an alpha): what per-object motion blur moves from.
    pub display: bool,
}

/// A skeleton's pose as a tick left it.
#[derive(Default)]
struct Pose {
    palette: Vec<Mat4>,
    weights: Vec<f32>,
}

/// Collects og-anim's outputs into `new`, reusing `old`'s allocations.
struct Poses<'a> {
    old: &'a mut IdMap<u64, Pose>,
    new: &'a mut IdMap<u64, Pose>,
}

impl og_anim::PaletteSink for Poses<'_> {
    fn joint_palette(&mut self, entity: og_ecs::Entity, _skin: u32, joints: &[Mat4]) {
        let bits = entity.to_bits();
        let mut p = self.old.remove(&bits).unwrap_or_default();
        p.palette.clear();
        p.palette.extend_from_slice(joints);
        p.weights.clear();
        self.new.insert(bits, p);
    }

    fn morph_weights(
        &mut self,
        entity: og_ecs::Entity,
        _mesh: u32,
        weights: &[f32; og_anim::BLENDSHAPES],
    ) {
        if let Some(p) = self.new.get_mut(&entity.to_bits()) {
            p.weights.clear();
            p.weights.extend_from_slice(weights);
        }
    }
}

impl InterpState {
    /// og.render.snap of `entity` (bits) at `tick`.
    pub(crate) fn snap(&mut self, entity: u64, tick: u64) {
        self.snaps.insert(entity, tick);
    }

    /// og.render.snapCamera at `tick`.
    pub(crate) fn snap_camera(&mut self, tick: u64) {
        self.camera_snap = Some(tick);
    }

    /// The world was restored or reloaded: frames draw the tick until the
    /// next tick starts from it.
    pub(crate) fn forget(&mut self) {
        self.stale = true;
    }
}

/// og.Camera's position and unit rotation.
pub(crate) fn camera_pose(c: &Value) -> (Vec3, Quat) {
    let q = &c["quaternion"];
    let rot = Quat::from_xyzw(
        f(q, "x", 0.0),
        f(q, "y", 0.0),
        f(q, "z", 0.0),
        f(q, "w", 1.0),
    )
    .normalize();
    let p = &c["position"];
    (
        Vec3::new(f(p, "x", 0.0), f(p, "y", 0.0), f(p, "z", 0.0)),
        rot,
    )
}

/// The probe's tick start: og.Camera's pose, and the snaps of earlier
/// ticks dropped.
pub(crate) fn before_tick(rs: &mut RenderState, world: &World) {
    let tick = world.time().tick;
    let camera = super::extract::resource(world, rs.camera.as_ref()).map(|c| camera_pose(&c));
    let s = &mut rs.interp;
    s.camera = camera.map(|(p, q)| (tick, p, q));
    s.snaps.retain(|_, t| *t >= tick);
    s.camera_snap = s.camera_snap.filter(|t| *t >= tick);
    s.stale = false;
    let mut old = std::mem::take(&mut s.poses);
    if let Some(anim) = rs.fx.anim.as_ref() {
        anim.write_outputs(&mut Poses {
            old: &mut old,
            new: &mut rs.interp.poses,
        });
    }
}

/// The joint palette and blendshape weights a skinned model is drawn with
/// (og-anim's `palette` and `weights` of the last tick): per joint blended
/// with the tick before's at `alpha` (translation and scale lerped, rotation
/// slerped, weights lerped) when the entity is blended
/// ([`Hierarchy::blend_alpha`]) and its skeleton had the same joints then.
/// Presentation only: og-anim's output, root motion and everything the
/// simulation reads stay at the tick.
pub(crate) fn pose(
    s: &InterpState,
    entity: u64,
    alpha: Option<f32>,
    palette: &[Mat4],
    weights: &mut [f32],
) -> Vec<Mat4> {
    let prev = alpha.zip(s.poses.get(&entity));
    let Some((a, p)) = prev.filter(|(_, p)| p.palette.len() == palette.len()) else {
        return palette.to_vec();
    };
    for (w, &w0) in weights.iter_mut().zip(&p.weights) {
        *w = w0 + (*w - w0) * a;
    }
    p.palette
        .iter()
        .zip(palette)
        .map(|(&m0, &m1)| super::extract::blend(m0, m1, a, f32::INFINITY))
        .collect()
}

/// Starts the frame's blend on `h` (the tick's hierarchy, just read) when
/// the frame is a display frame between ticks, RenderSettings.interpolate
/// is on and the last tick started from the transforms the raycast record
/// read: anything else (no tick yet, a restore or a reload since,
/// [`InterpState::forget`]) draws the tick.
pub(crate) fn begin(
    world: &World,
    rs: &mut RenderState,
    settings: Option<&Value>,
    h: &mut Hierarchy,
) {
    rs.interp.drawn = None;
    rs.interp.active = None;
    let Some(alpha) = rs.interp.alpha.filter(|_| !rs.interp.stale) else {
        return;
    };
    if !settings.is_none_or(|s| on(s, "interpolate", true)) {
        return;
    }
    let Some(started) = world.time().tick.checked_sub(1) else {
        return;
    };
    if rs.geo.record_tick() != Some(started) {
        return;
    }
    let snap = settings.map_or(SNAP_DISTANCE, |s| f(s, "snapDistance", SNAP_DISTANCE));
    let snap_sq = if snap > 0.0 {
        snap * snap
    } else {
        f32::INFINITY
    };
    let mut held = IdMap::new();
    for (&e, &t) in rs.interp.snaps.iter() {
        if t >= started {
            held.insert(e, ());
        }
    }
    if let Some(layout) = &rs.draw.interpolation {
        for (e, v) in rows(world, layout) {
            if on(&v, "off", false) {
                held.insert(e, ());
            }
        }
    }
    h.set_blend(Blend {
        prev: rs.geo.take_hierarchy(),
        alpha,
        snap_sq,
        held,
    });
    rs.interp.active = Some((alpha, snap_sq, started));
    rs.interp.drawn = Some(alpha);
}

/// Ends [`begin`]: the raycast record's hierarchy back.
pub(crate) fn end(rs: &mut RenderState, h: &mut Hierarchy) {
    if let Some(prev) = h.take_blend() {
        rs.geo.put_hierarchy(prev);
    }
    rs.interp.active = None;
}

/// og.Camera's drawn pose, whose tick pose is `position` and `rot`.
pub(crate) fn camera(s: &InterpState, position: Vec3, rot: Quat) -> (Vec3, Quat) {
    let Some((alpha, snap_sq, started)) = s.active else {
        return (position, rot);
    };
    let Some((tick, p0, r0)) = s.camera else {
        return (position, rot);
    };
    if tick != started
        || s.camera_snap.is_some_and(|t| t >= started)
        || p0.distance_squared(position) > snap_sq
    {
        return (position, rot);
    }
    let r = if r0 == rot {
        rot
    } else {
        og_core::math::quat_slerp(r0, rot, alpha).normalize()
    };
    (p0.lerp(position, alpha), r)
}

/// The alpha of a display frame: `None` (the tick) on a tick boundary,
/// the accumulator empty before (`before`) and after (`after`) it, and
/// when the probe took fewer ticks than were due (a lockstep gate or a
/// screenshot held them: going on blending would draw the same two ticks
/// again from 0); otherwise `after` over `tick_ms`.
pub(crate) fn frame_alpha(before: f64, after: f64, tick_ms: f64, held: bool) -> Option<f32> {
    if held || (before == 0.0 && after == 0.0) || tick_ms <= 0.0 {
        return None;
    }
    Some((after / tick_ms).clamp(0.0, 1.0) as f32)
}

/// How far each anchored UI element (og.ui.openWorldAnchor) moves on a
/// frame between ticks, CSS px: its point projected with the drawn camera
/// at its entity's drawn place, less where the tick's camera put it at the
/// tick, so a nameplate stays on the entity it names
/// (`og_ui::UiFrame::emit_shifted`). Empty on a frame at the tick; an
/// element clamped to the screen's edge stays.
pub(crate) fn ui_shifts(
    rs: &RenderState,
    h: &mut Hierarchy,
    camera: Option<&Value>,
) -> IdMap<u64, Vec2> {
    let mut out = IdMap::new();
    if rs.interp.active.is_none() {
        return out;
    }
    let Some(ui) = rs.ui.clone() else {
        return out;
    };
    let ui = ui.lock();
    let points = ui.world_anchor_points();
    let Some(tick_view) = ui.world_view().filter(|_| !points.is_empty()) else {
        return out;
    };
    let vp = ui.viewport();
    let (w, ht) = (vp.width.max(1.0), vp.height.max(1.0));
    let drawn_view = match camera {
        Some(c) => {
            let (p, q) = camera_pose(c);
            let (p, q) = self::camera(&rs.interp, p, q);
            let mut c = c.clone();
            c["position"] = serde_json::json!({ "x": p.x, "y": p.y, "z": p.z });
            c["quaternion"] = serde_json::json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w });
            super::api::camera_view(&c, w, ht)
        }
        None => tick_view,
    };
    for a in points.iter().filter(|a| !a.clamped) {
        let (at_tick, drawn) = match a.target {
            og_ui::WorldTarget::Point(p) => (p, p),
            og_ui::WorldTarget::Entity(e) => (
                h.tick_world_of(e).w_axis.truncate(),
                h.world_of(e).w_axis.truncate(),
            ),
        };
        let (Some(t), Some(d)) = (
            tick_view.project(at_tick + a.offset, w, ht),
            drawn_view.project(drawn + a.offset, w, ht),
        ) else {
            continue;
        };
        let shift = Vec2::new(d.0 - t.0, d.1 - t.1);
        if shift != Vec2::ZERO && shift.is_finite() {
            out.insert(a.key, shift);
        }
    }
    out
}
