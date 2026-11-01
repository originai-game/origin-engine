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
use glam::{Quat, Vec3};
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
