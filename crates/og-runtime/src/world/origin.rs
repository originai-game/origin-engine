//! og.WorldOrigin (R12, `docs/design.md` §2.4): the world's zero follows
//! its focus, so every place in it stays near zero however far the world
//! goes.
//!
//! As each tick starts, before any system runs (the probe's before-tick
//! hook, ahead of world cells, the raycast record and interpolation), the
//! focus is read from world state (og.WorldOrigin.focus's Transform, else
//! the first og.StreamAnchor, else og.Camera.position, refused in an og.net
//! session). Past `step` from the zero along an axis of the plane, the world
//! shifts by the multiple of `step` nearest the focus:
//!
//! - every component, resource and queued event field the reflected schema
//!   marks `worldspace world` or `root` ([`og_ecs::World::shift`]; a `root`
//!   field only on an entity without og.Parent);
//! - the `hook` fields, whose frame depends on another field ([`HOOKS`]);
//! - every state block's own places ([`og_ecs::StateBlock::shift`]);
//! - the renderer's history kept in the world's coordinates (motion blur's
//!   transforms, debug draws), so a shift draws nothing differently.
//!
//! og.WorldOrigin keeps the zero's place in the world in f64, and
//! og.OriginShifted tells game code by how much its own places move. The
//! render origin ([`crate::present::extract::frame`]) is on a grid of the absolute
//! world, so a shift never moves it.

use crate::engine::Engine;
use crate::host::codec::Layout;
use crate::host::wire::{FromJson, IkSpace, WorldOriginPlane, record};
use glam::{DVec3, Vec3};
use og_core::{FieldSchema, Frame, Worldspace};
use og_ecs::{Entity, QueryDesc, ShiftField, World};
use serde_json::{Value, json};

pub(crate) const RESOURCE: &str = "og.WorldOrigin";
pub(crate) const EVENT: &str = "og.OriginShifted";

/// The fields marked `worldspace hook`: a place in the world only under a
/// condition another field states, moved here ([`hooks`]).
pub(crate) const HOOKS: &[(&str, &str)] = &[
    // A world point while `other` is 0 (the ground body's anchor).
    ("og.Joint", "otherAnchor"),
    // In the skeleton entity's Transform frame with `space` world.
    ("og.IkChain", "target"),
    ("og.IkChain", "pole"),
    ("og.LookAt", "target"),
    // Always in the skeleton entity's Transform frame.
    ("og.IkTarget2D", "target"),
];

/// A field's place in the world: its own schema's `worldspace`, else the
/// IDL's for the record of that name (a module's derived schema carries
/// none).
pub(crate) fn space_of(schema: &str, f: &FieldSchema) -> Option<Worldspace> {
    let text = f
        .worldspace
        .clone()
        .or_else(|| crate::host::codec::idl_worldspace(schema, &f.name))?;
    Worldspace::of(&text, f.ty).ok()
}

/// Every registered field a shift moves by schema ([`Frame::World`] and
/// [`Frame::Root`]), in id order.
fn fields(world: &World) -> Vec<ShiftField> {
    let mut out = Vec::new();
    for s in world.schemas() {
        let Ok(id) = world.id_by_name(&s.name) else {
            continue;
        };
        for (i, f) in s.fields.iter().enumerate() {
            if let Some(space) = space_of(&s.name, f)
                && matches!(space.frame, Frame::World | Frame::Root)
            {
                out.push(ShiftField {
                    schema: id,
                    field: i,
                    space,
                });
            }
        }
    }
    out
}

fn layout(eng: &Engine, name: &str) -> Option<Layout> {
    eng.state.borrow().layouts.get(name).cloned()
}

fn json3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

/// The entities with component `name`, in id order.
fn with(world: &World, name: &str) -> Vec<Entity> {
    let q = QueryDesc {
        read: vec![name.into()],
        ..QueryDesc::default()
    };
    let mut list = Vec::new();
    if let Ok(iter) = world.query(&q) {
        for a in iter {
            list.extend(a.entities().iter().copied());
        }
    }
    og_core::sort::sort_by_key(&mut list, |e| (e.index(), e.generation()));
    list
}

/// Whether entity `e` has og.Parent.
fn is_child(world: &World, e: Entity) -> bool {
    world
        .id_by_name("og.Parent")
        .is_ok_and(|p| world.get_bytes(e, p).is_ok())
}

/// The axes `plane` moves along (og.WorldOriginPlane: xz, xyz, xy).
fn axes(plane: WorldOriginPlane) -> [bool; 3] {
    match plane {
        WorldOriginPlane::Xyz => [true, true, true],
        WorldOriginPlane::Xy => [true, true, false],
        WorldOriginPlane::Xz => [true, false, true],
    }
}

/// How far the world shifts with its focus at `focus`: per axis of the
/// plane, minus the multiple of `step` nearest the focus once the focus is
/// farther than `step` from the zero; zero otherwise.
pub(crate) fn delta(focus: Vec3, step: f64, axes: [bool; 3]) -> DVec3 {
    let f = focus.as_dvec3().to_array();
    let mut d = [0.0f64; 3];
    for a in 0..3 {
        if axes[a] && f[a].is_finite() && f[a].abs() > step {
            d[a] = -(f[a] / step).round() * step;
        }
    }
    DVec3::from_array(d)
}

/// Where the zero follows: og.WorldOrigin.focus's position, else the first
/// og.StreamAnchor's, else og.Camera.position. `Err` names why there is
/// none to follow (a session without a focus or an anchor).
fn focus(
    eng: &Engine,
    world: &World,
    origin: &record::WorldOrigin,
) -> Result<Option<Vec3>, String> {
    let transform = layout(eng, "og.Transform").ok_or("og.Transform has no layout")?;
    let tid = world
        .id_by_name("og.Transform")
        .map_err(|e| e.reason().to_string())?;
    let at = |e: Entity| -> Option<Vec3> {
        let b = world.get_bytes(e, tid).ok()?;
        Some(transform.read::<record::Transform>(&b).ok()?.position)
    };
    let bits = origin.focus;
    if bits != 0 {
        let e = Entity::from_bits(bits);
        if world.is_alive(e) && !is_child(world, e) {
            return Ok(at(e));
        }
        return Err(format!(
            "og.WorldOrigin.focus {bits} is not a live entity without a parent: the zero holds"
        ));
    }
    if world
        .id_by_name(crate::world::stream::cells::ANCHOR)
        .is_ok()
        && let Some(e) = with(world, crate::world::stream::cells::ANCHOR)
            .into_iter()
            .find(|e| !is_child(world, *e))
    {
        return Ok(at(e));
    }
    if crate::world::stream::cells::in_session(eng, world) {
        return Err("og.WorldOrigin in an og.net session follows og.WorldOrigin.focus or an og.StreamAnchor: each peer's camera is its own, so the zero holds".into());
    }
    let camera = layout(eng, "og.Camera").and_then(|l| {
        let id = world.id_by_name("og.Camera").ok()?;
        l.read::<record::Camera>(&world.resource_bytes(id).ok()?)
            .ok()
    });
    Ok(camera.map(|c| c.position))
}

/// og.WorldOrigin on one engine: the last warning it logged (not world
/// state: the log's).
#[derive(Default)]
pub(crate) struct ModuleState(std::cell::RefCell<String>);

/// A warning in the engine's log, once until another one replaces it.
fn warn_once(eng: &Engine, tick: u64, text: String) {
    let fresh = {
        let mut last = eng.modules.origin.0.borrow_mut();
        let same = *last == text;
        *last = text.clone();
        !same
    };
    if fresh {
        eng.state
            .borrow_mut()
            .log
            .push(json!({ "tick": tick, "warning": true, "text": text }));
    }
}

/// The probe's start-of-tick hook: shifts the world when its focus has
/// strayed past og.WorldOrigin.step from the zero.
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Ok(id) = world.id_by_name(RESOURCE) else {
        return;
    };
    let (Some(l), Ok(bytes)) = (layout(eng, RESOURCE), world.resource_bytes(id)) else {
        return;
    };
    let Ok(origin) = l.read::<record::WorldOrigin>(&bytes) else {
        return;
    };
    if origin.off {
        return;
    }
    let step = f64::from(origin.step).round().max(64.0);
    let at = match focus(eng, world, &origin) {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(why) => {
            warn_once(eng, world.time().tick, why);
            return;
        }
    };
    let d = delta(at, step, axes(origin.plane));
    if d != DVec3::ZERO
        && let Err(e) = shift(eng, world, d)
    {
        warn_once(eng, world.time().tick, format!("og.WorldOrigin: {e}"));
    }
}

/// Shifts every place in `world` by `d` (whole metres): the marked fields,
/// the hooks, the state blocks and the renderer's history; then
/// og.WorldOrigin's zero and og.OriginShifted.
pub(crate) fn shift(eng: &Engine, world: &mut World, d: DVec3) -> Result<(), String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let parent = world.id_by_name("og.Parent").ok();
    world.shift(&fields(world), d, parent).map_err(err)?;
    hooks(eng, world, d)?;
    if let Some(ui) = &eng.state.borrow().ui {
        ui.lock().shift_world(d.as_vec3());
    }
    if let Ok(mut rs) = eng.render.try_borrow_mut() {
        rs.shift(d);
    }
    let l = layout(eng, RESOURCE).ok_or("og.WorldOrigin has no layout")?;
    let id = world.id_by_name(RESOURCE).map_err(err)?;
    let mut v = l.decode(&world.resource_bytes(id).map_err(err)?);
    let row = record::WorldOrigin::from_json(&v)?;
    let now = DVec3::new(row.x, row.y, row.z) - d;
    v["x"] = json!(now.x);
    v["y"] = json!(now.y);
    v["z"] = json!(now.z);
    v["shifts"] = json!(u64::from(row.shifts) + 1);
    world
        .insert_resource_bytes(id, &l.encode(None, Some(&v))?)
        .map_err(err)?;
    crate::engine::lazy::ensure(eng, world, EVENT)?;
    let el = layout(eng, EVENT).ok_or("og.OriginShifted has no layout")?;
    let eid = world.id_by_name(EVENT).map_err(err)?;
    let event = json!({ "delta": json3(d.as_vec3()), "x": now.x, "y": now.y, "z": now.z });
    world
        .send_bytes(eid, &el.encode(None, Some(&event))?)
        .map_err(err)
}

/// The [`HOOKS`] fields: each moved where its condition makes it a place
/// in the world.
fn hooks(eng: &Engine, world: &mut World, d: DVec3) -> Result<(), String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let moved = |p: Vec3, d: DVec3| json3((p.as_dvec3() + d).as_vec3());
    let mut names: Vec<&str> = HOOKS.iter().map(|h| h.0).collect();
    names.dedup();
    for name in names {
        let (Ok(id), Some(l)) = (world.id_by_name(name), layout(eng, name)) else {
            continue;
        };
        for e in with(world, name) {
            let Ok(bytes) = world.get_bytes(e, id) else {
                continue;
            };
            let mut v = l.decode(&bytes);
            // The skeleton a constraint is in the frame of moves with a root.
            let skeleton_root = |s: u64| s != 0 && !is_child(world, Entity::from_bits(s));
            let changed = match name {
                "og.Joint" => {
                    let j = record::Joint::from_json(&v)?;
                    if j.other == 0 {
                        v["otherAnchor"] = moved(j.other_anchor, d);
                    }
                    j.other == 0
                }
                "og.IkChain" => {
                    let c = record::IkChain::from_json(&v)?;
                    let on = c.space == IkSpace::World && skeleton_root(c.skeleton);
                    if on {
                        v["target"] = moved(c.target, d);
                        // The zero pole keeps the animated bend: not a place.
                        if c.pole != Vec3::ZERO {
                            v["pole"] = moved(c.pole, d);
                        }
                    }
                    on
                }
                "og.LookAt" => {
                    let a = record::LookAt::from_json(&v)?;
                    let on = a.space == IkSpace::World && skeleton_root(a.skeleton);
                    if on {
                        v["target"] = moved(a.target, d);
                    }
                    on
                }
                "og.IkTarget2D" => {
                    let t = record::IkTarget2D::from_json(&v)?;
                    let on = skeleton_root(t.skeleton);
                    if on {
                        let p = t.target.as_dvec2() + d.truncate();
                        v["target"] = json!({ "x": p.x as f32, "y": p.y as f32 });
                    }
                    on
                }
                _ => false,
            };
            if changed {
                let b = l.encode(Some(&bytes), Some(&v))?;
                world.insert_bytes(e, id, &b).map_err(err)?;
            }
        }
    }
    Ok(())
}

impl crate::present::extract::RenderState {
    /// Moves what the renderer side keeps in the world's coordinates across
    /// ticks by `d`: motion blur's last transforms, the sprites, ribbons and
    /// texts queued for the next frames, and the debug draws still showing.
    /// What the renderer itself keeps is relative to the render origin, on
    /// a grid of the absolute world, so a shift does not move it.
    pub(crate) fn shift(&mut self, d: DVec3) {
        let p = |v: Vec3| (v.as_dvec3() + d).as_vec3();
        let m = |t: &mut glam::Mat4| {
            let w = t.w_axis;
            if w.w != 0.0 {
                t.w_axis = (p(w.truncate() / w.w) * w.w).extend(w.w);
            }
        };
        let fx = &mut self.fx;
        for (_, t) in fx
            .current
            .iter_mut()
            .chain(fx.previous.iter_mut())
            .chain(fx.drawn.iter_mut())
        {
            m(t);
        }
        fx.previous_by_key = None;
        for s in fx.sprites.iter_mut().filter(|s| !s.screen) {
            s.sprite.position = p(s.sprite.position);
        }
        for r in &mut fx.ribbons {
            for q in &mut r.points {
                q.position = p(q.position);
            }
        }
        for t in &mut self.texts {
            t.position = p(t.position);
        }
        self.geo.shift(d);
    }
}

/// Where og.WorldOrigin puts the world's zero (zero without one): what the
/// render origin's grid is laid in.
pub(crate) fn zero(world: &World, layout: Option<&Layout>) -> DVec3 {
    let Some(v) = crate::present::extract::resource(world, layout) else {
        return DVec3::ZERO;
    };
    let Ok(o) = record::WorldOrigin::from_json(&v) else {
        return DVec3::ZERO;
    };
    DVec3::new(o.x, o.y, o.z)
}
