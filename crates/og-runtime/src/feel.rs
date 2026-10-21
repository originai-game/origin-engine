//! The E4 2D and game-feel modules on the runtime: time scale and hit stop
//! (`time`), camera feel (`camera`: shake, punch, kick, zones, rails),
//! tweens (`tween`, over og-tween), flipbooks (og-anim's `og.flipbook`),
//! tilemaps (`tilemap`, over og-tilemap), `og.Sprite` with its flips,
//! nine-slice, y sorting, blob shadows and afterimages, and floating text.
//!
//! Systems, in tick order after the game's: `og.tween`, `og.flipbook`
//! (then og-anim's `og.anim`), `og.tilemap` (adopts maps, applies edits to
//! colliders), `og.camera` (queued shakes, punches and kicks, their decay,
//! rails); after physics and audio, `og.sprite` (afterimage samples,
//! floating text expiry). Each keeps its state in a state block, so
//! snapshots, restore and the world hash cover it.
//!
//! Extraction (`extract`) draws og.Sprite rows (with their flipbook frame),
//! afterimages, floating texts and tile layers, and applies the camera's
//! shake, punch, kick and zones to the frame only: rendering never feeds
//! back into the simulation.

use super::codec::Layout;
use super::engine::{Engine, engine, queued_structural, running};
use super::extract::{Hierarchy, RenderState};
use super::{ok, ok_void, parse, refuse};
use glam::{Quat, Vec2, Vec3};
use og_anim::{FlipbookSheet, FlipbookSystem};
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_tilemap::{MapState, TilemapAsset, Tileset};
use og_tween::{Ease, GroupKind, Leaf, Node, Target, TweenSystem};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const TILEMAP_SYSTEM: &str = "og.tilemap";
pub(crate) const CAMERA_SYSTEM: &str = "og.camera";
pub(crate) const SPRITE_SYSTEM: &str = "og.sprite";

/// Records og-anim and og-tween register themselves (strings, or adopted).
pub(crate) const REGISTERED_BY_FEEL: &[&str] = &["og.FlipbookEvent"];

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn err(e: Error) -> String {
    e.reason().to_string()
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn wire_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map_or(default, |x| x as f32)
}

fn on(v: &Value, key: &str, default: bool) -> bool {
    match &v[key] {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        _ => default,
    }
}

fn v2(v: &Value, default: Vec2) -> Vec2 {
    Vec2::new(f(v, "x", default.x), f(v, "y", default.y))
}

fn v3(v: &Value, default: Vec3) -> Vec3 {
    Vec3::new(
        f(v, "x", default.x),
        f(v, "y", default.y),
        f(v, "z", default.z),
    )
}

fn rgb(v: &Value, default: Vec3) -> Vec3 {
    Vec3::new(
        f(v, "r", default.x),
        f(v, "g", default.y),
        f(v, "b", default.z),
    )
}

fn bits(v: &Value) -> u64 {
    v.as_f64().map_or(0, |x| x as u64)
}

// ── state ──────────────────────────────────────────────────────────────────

/// A camera effect queued by og.camera.shake / punch / kick, applied by the
/// og.camera system (in call order).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CameraOp {
    Shake(f32),
    Punch(f32),
    Kick(Vec3),
}

/// A pose an afterimage keeps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ghost {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
    pub sheet: u64,
    pub clip: u32,
    pub frame: u32,
    pub flip: u8,
    pub tick: u64,
}

/// A live floating text.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Floating {
    pub text: String,
    /// The font: a FontPages index, or u32::MAX for the UI's body font.
    pub font: u32,
    pub position: Vec3,
    pub drift: f32,
    pub start: f64,
    pub size: f32,
    pub color: Vec3,
    pub outline: f32,
    pub outline_color: Vec3,
    pub rise: f32,
    pub lifetime: f32,
    pub fade: f32,
    pub pop: f32,
    pub layer: i32,
}

#[derive(Default)]
pub(crate) struct FeelShared {
    pub camera_ops: Vec<CameraOp>,
    pub ghosts: BTreeMap<u64, Vec<Ghost>>,
    pub floating: Vec<Floating>,
    /// Texts spawned so far (their drift comes from it).
    pub spawned: u64,
    /// Per entity: the last squash and flash tween, cancelled by the next.
    pub squash: BTreeMap<u64, u64>,
    pub flash: BTreeMap<u64, u64>,
}

/// A map an og.Tilemap uses, with its tilesets and their textures.
#[derive(Clone)]
pub(crate) struct MapAsset {
    pub asset: TilemapAsset,
    pub tilesets: Vec<Arc<Tileset>>,
    /// Per tileset: the texture index in RenderState::textures, or none.
    pub textures: Vec<Option<usize>>,
}

/// A map an og.Tilemap entity uses, live.
pub(crate) struct LiveMap {
    pub map: u64,
    pub state: MapState,
    pub colliders: Vec<u64>,
    pub origin: Vec3,
    pub collision: bool,
    pub layers: u32,
    pub friction: f32,
    pub dirty: bool,
}

#[derive(Default)]
pub(crate) struct TileShared {
    pub maps: BTreeMap<u64, Arc<MapAsset>>,
    pub live: BTreeMap<u64, LiveMap>,
}

/// The feel modules of one world.
#[derive(Clone)]
pub(crate) struct Feel {
    pub tweens: TweenSystem,
    pub flipbooks: FlipbookSystem,
    pub tiles: Arc<Mutex<TileShared>>,
    pub shared: Arc<Mutex<FeelShared>>,
    /// Per flipbook sheet handle: what its frames are drawn from.
    pub sheet_textures: Arc<Mutex<BTreeMap<u64, SheetImage>>>,
}

/// What a flipbook sheet's frames are drawn from: a region of its texture
/// (an index in RenderState::textures), or, for an `array true` sheet, a
/// layer of its texture array (an index in RenderState::texture_arrays).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SheetImage {
    Texture(usize),
    Array(usize),
}

/// The assets the feel modules hold per engine, replayed into a reloaded world.
#[derive(Clone, Default)]
pub(crate) struct FeelAssets {
    next: u64,
    pub sheets: BTreeMap<u64, (FlipbookSheet, Option<SheetImage>)>,
    pub maps: BTreeMap<u64, Arc<MapAsset>>,
    loaded: BTreeMap<String, Value>,
}

impl FeelAssets {
    fn mint(&mut self) -> u64 {
        self.next += 1;
        self.next | (1u64 << 32)
    }

    pub(crate) fn replay(&self, feel: &Feel) {
        for (h, (s, t)) in &self.sheets {
            let _ = feel.flipbooks.add_sheet(*h, s.clone());
            if let Some(t) = t {
                lock(&feel.sheet_textures).insert(*h, *t);
            }
        }
        let mut t = lock(&feel.tiles);
        for (h, m) in &self.maps {
            t.maps.insert(*h, m.clone());
        }
    }
}

/// Installs tweens and flipbooks (after the game's systems, before og.anim).
pub(crate) fn install_front(world: &mut World, components: &[String]) -> Result<Feel, String> {
    let tweens = og_tween::install(world, components).map_err(err)?;
    let flipbooks = og_anim::install_flipbook(world).map_err(err)?;
    Ok(Feel {
        tweens,
        flipbooks,
        tiles: Arc::new(Mutex::new(TileShared::default())),
        shared: Arc::new(Mutex::new(FeelShared::default())),
        sheet_textures: Arc::new(Mutex::new(BTreeMap::new())),
    })
}

/// The records feel systems read and write, resolved once.
#[derive(Clone)]
pub(crate) struct Records {
    pub transform: (SchemaId, Layout),
    pub collider: (SchemaId, Layout),
    pub tilemap: (SchemaId, Layout),
    pub camera: (SchemaId, Layout),
    pub rail: (SchemaId, Layout),
    pub sprite: (SchemaId, Layout),
    pub flipbook: (SchemaId, Layout),
    pub afterimage: (SchemaId, Layout),
    pub zone: (SchemaId, Layout),
    pub follow: (SchemaId, Layout),
}

impl Records {
    pub(crate) fn of(world: &World, layouts: &BTreeMap<String, Layout>) -> Result<Records, String> {
        let get = |n: &str| -> Result<(SchemaId, Layout), String> {
            let id = world.id_by_name(n).map_err(err)?;
            let layout = match layouts.get(n) {
                Some(l) => l.clone(),
                None => Layout::from_schema(
                    world.schema(id).map_err(err)?,
                    super::codec::RecordKind::Component,
                ),
            };
            Ok((id, layout))
        };
        Ok(Records {
            transform: get("og.Transform")?,
            collider: get("og.Collider")?,
            tilemap: get("og.Tilemap")?,
            camera: get("og.Camera")?,
            rail: get("og.CameraRail")?,
            sprite: get("og.Sprite")?,
            flipbook: get("og.Flipbook")?,
            afterimage: get("og.Afterimage")?,
            zone: get("og.CameraZone")?,
            follow: get("og.Follow2D")?,
        })
    }
}

/// Installs og.tilemap and og.camera (after og.anim, before the UI's end and physics).
pub(crate) fn install_middle(world: &mut World, feel: &Feel, r: &Records) -> Result<(), String> {
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let tiles = feel.tiles.clone();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: TILEMAP_SYSTEM.into(),
                reads: names(&[&r.tilemap]),
                writes: names(&[&r.transform, &r.collider]),
                queries: vec![QueryDesc {
                    read: names(&[&r.tilemap, &r.transform]),
                    ..QueryDesc::default()
                }],
            },
            move |ctx| tilemap_system(ctx, &tiles, &rec),
        )
        .map_err(err)?;
    world
        .add_state_block(TILEMAP_SYSTEM, Box::new(TileBlock(feel.tiles.clone())))
        .map_err(err)?;
    let shared = feel.shared.clone();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: CAMERA_SYSTEM.into(),
                reads: names(&[&r.rail, &r.transform, &r.zone, &r.follow]),
                writes: names(&[&r.camera]),
                queries: vec![
                    QueryDesc {
                        read: names(&[&r.rail, &r.transform]),
                        ..QueryDesc::default()
                    },
                    QueryDesc {
                        read: names(&[&r.zone]),
                        ..QueryDesc::default()
                    },
                    QueryDesc {
                        read: names(&[&r.follow, &r.transform]),
                        ..QueryDesc::default()
                    },
                ],
            },
            move |ctx| camera_system(ctx, &shared, &rec),
        )
        .map_err(err)?;
    world
        .add_state_block(CAMERA_SYSTEM, Box::new(CameraBlock(feel.shared.clone())))
        .map_err(err)?;
    Ok(())
}

/// Installs og.sprite (after physics and audio: the poses of the tick's end).
pub(crate) fn install_back(world: &mut World, feel: &Feel, r: &Records) -> Result<(), String> {
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let shared = feel.shared.clone();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: SPRITE_SYSTEM.into(),
                reads: names(&[&r.afterimage, &r.sprite, &r.transform, &r.flipbook]),
                writes: Vec::new(),
                queries: vec![QueryDesc {
                    read: names(&[&r.afterimage, &r.sprite, &r.transform]),
                    ..QueryDesc::default()
                }],
            },
            move |ctx| sprite_system(ctx, &shared, &rec),
        )
        .map_err(err)?;
    world
        .add_state_block(SPRITE_SYSTEM, Box::new(SpriteBlock(feel.shared.clone())))
        .map_err(err)?;
    Ok(())
}

fn feel_of(eng: &Engine) -> Result<Feel, String> {
    eng.state
        .borrow()
        .feel
        .clone()
        .ok_or_else(|| "this engine has no feel modules".to_string())
}

// ── time ───────────────────────────────────────────────────────────────────

/// The Time fields the time scale adds, for `og.Time` and the systems' `time`.
pub(crate) fn time_scale_json(scale: og_core::TimeScale, time: og_core::Time) -> Value {
    json!({
        "scale": scale.scale,
        "hitStop": scale.hit_stop,
        "scaledDt": scale.dt(&time),
        "scaledElapsed": scale.elapsed,
    })
}

/// The dispatch payload of a system call: `scale,hitStop,scaledDt,scaledElapsed`.
pub(crate) fn time_payload(ctx: &SystemContext<'_>) -> String {
    let s = ctx.time_scale();
    let t = ctx.time();
    format!("{},{},{},{}", s.scale, s.hit_stop, s.dt(&t), s.elapsed)
}

fn with_world<T>(
    eng: &Engine,
    inside: impl FnOnce(&SystemContext<'_>) -> og_core::Result<T>,
    outside: impl FnOnce(&mut World) -> og_core::Result<T>,
) -> Result<T, String> {
    if let Some(fr) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        let ctx = unsafe { fr.ctx() };
        return inside(ctx).map_err(err);
    }
    outside(eng.world()?).map_err(err)
}

pub fn og_time_set_scale(world: u32, scale: f64) -> String {
    wire_void(engine(world).and_then(|eng| {
        let s = scale as f32;
        with_world(&eng, |c| c.set_time_scale(s), |w| w.set_time_scale(s))
    }))
}

pub fn og_time_hit_stop(world: u32, ticks: u32) -> String {
    wire_void(
        engine(world)
            .and_then(|eng| with_world(&eng, |c| c.hit_stop(ticks), |w| w.hit_stop(ticks))),
    )
}

// ── camera ─────────────────────────────────────────────────────────────────

fn camera_op(world: u32, op: CameraOp) -> String {
    wire_void(engine(world).and_then(|eng| {
        let feel = feel_of(&eng)?;
        lock(&feel.shared).camera_ops.push(op);
        Ok(())
    }))
}

pub fn og_camera_shake(world: u32, trauma: f64) -> String {
    if !(trauma.is_finite() && (0.0..=1.0).contains(&trauma)) {
        return refuse(format!("trauma {trauma} is outside 0..1"));
    }
    camera_op(world, CameraOp::Shake(trauma as f32))
}

pub fn og_camera_punch(world: u32, amount: f64) -> String {
    if !(amount.is_finite() && (-0.9..=4.0).contains(&amount)) {
        return refuse(format!("punch {amount} is outside -0.9..4"));
    }
    camera_op(world, CameraOp::Punch(amount as f32))
}

pub fn og_camera_kick(world: u32, direction: String, distance: f64) -> String {
    let d = match parse(Some(&direction)) {
        Ok(v) => v3(&v, Vec3::ZERO),
        Err(e) => return refuse(e),
    };
    if !distance.is_finite() || !d.is_finite() {
        return refuse("kick needs a finite direction and distance");
    }
    camera_op(
        world,
        CameraOp::Kick(d.normalize_or_zero() * distance as f32),
    )
}

/// Applies the queued ops, decays trauma, punch and kick, and moves the
/// camera along a rail.
fn camera_system(
    ctx: &mut SystemContext<'_>,
    shared: &Mutex<FeelShared>,
    r: &Records,
) -> og_core::Result<()> {
    let (cid, layout) = &r.camera;
    let ops = std::mem::take(&mut lock(shared).camera_ops);
    let dt = ctx.time().dt;
    let bytes = ctx.resource_bytes(*cid)?;
    let mut cam = layout.decode(&bytes);
    let mut trauma = f(&cam, "trauma", 0.0);
    let mut punch = f(&cam, "punch", 0.0);
    let mut kick = v3(&cam["kick"], Vec3::ZERO);
    for op in &ops {
        match *op {
            CameraOp::Shake(t) => trauma = (trauma + t).min(1.0),
            CameraOp::Punch(p) => punch = (punch + p).clamp(-0.9, 4.0),
            CameraOp::Kick(k) => kick += k,
        }
    }
    let decay = |v: f32, rate: f32| v * og_core::math::exp(-rate.max(0.0) * dt);
    // The decay starts the tick after the op, so a shake shows at full strength once.
    if ops.is_empty() {
        trauma = (trauma - f(&cam, "traumaDecay", 1.5).max(0.0) * dt).max(0.0);
        punch = decay(punch, f(&cam, "punchDecay", 8.0));
        kick *= og_core::math::exp(-f(&cam, "kickDecay", 12.0).max(0.0) * dt);
    }
    // Below a thousandth they are invisible: snap to rest.
    if punch.abs() < 1e-3 {
        punch = 0.0;
    }
    if kick.length_squared() < 1e-6 {
        kick = Vec3::ZERO;
    }
    cam["trauma"] = json!(trauma);
    cam["punch"] = json!(punch);
    cam["kick"] = json!({ "x": kick.x, "y": kick.y, "z": kick.z });
    camera_zone(ctx, r, &mut cam, dt)?;
    // A rail: the camera eases to the rail point nearest the entity.
    let mut rail: Option<(Entity, Value, Vec3)> = None;
    for a in ctx.query(0)? {
        if let Some(&e) = a.entities().first() {
            let v = r.rail.1.decode(&ctx.get_bytes(e, r.rail.0)?);
            let t = r.transform.1.decode(&ctx.get_bytes(e, r.transform.0)?);
            rail = Some((e, v, v3(&t["position"], Vec3::ZERO)));
            break;
        }
    }
    if let Some((_, v, target)) = rail {
        let n = (v["points"].as_u64().unwrap_or(2) as usize).clamp(2, 8);
        let pts: Vec<Vec3> = (0..n)
            .map(|i| v3(&v[format!("p{i}").as_str()], Vec3::ZERO))
            .collect();
        let goal = rail_nearest(&pts, target);
        let pos = v3(&cam["position"], Vec3::ZERO);
        let smooth = f(&v, "smoothTime", 0.2);
        let k = if smooth > 0.0 {
            (dt / smooth).min(1.0)
        } else {
            1.0
        };
        let ortho = on(&cam, "orthographic", false);
        let mut next = pos + (goal - pos) * k;
        if ortho {
            next.z = pos.z;
        }
        cam["position"] = json!({ "x": next.x, "y": next.y, "z": next.z });
        if !ortho {
            let q = look_rotation(next, target);
            cam["quaternion"] = json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w });
        }
    }
    let out = layout
        .encode(Some(&bytes), Some(&cam))
        .map_err(|e| Error::new(ErrorKind::Invalid, e))?;
    ctx.set_resource_bytes(*cid, &out)
}

/// The og.CameraZone the entity followed by og.Follow2D is in (the highest
/// priority, then the first in query order), into og.Camera's zone fields;
/// entering another one starts a scroll over its scrollTime.
fn camera_zone(
    ctx: &mut SystemContext<'_>,
    r: &Records,
    cam: &mut Value,
    dt: f32,
) -> og_core::Result<()> {
    let mut target: Option<Vec2> = None;
    for a in ctx.query(2)? {
        if let Some(&e) = a.entities().first() {
            let t = r.transform.1.decode(&ctx.get_bytes(e, r.transform.0)?);
            target = Some(v3(&t["position"], Vec3::ZERO).truncate());
            break;
        }
    }
    let mut best: Option<(i64, Vec2, Vec2, f32)> = None;
    if let Some(p) = target {
        for a in ctx.query(1)? {
            for &e in a.entities() {
                let z = r.zone.1.decode(&ctx.get_bytes(e, r.zone.0)?);
                let (lo, hi) = (v2(&z["min"], Vec2::ZERO), v2(&z["max"], Vec2::splat(10.0)));
                if p.x < lo.x || p.x > hi.x || p.y < lo.y || p.y > hi.y {
                    continue;
                }
                let pr = z["priority"].as_i64().unwrap_or(0);
                if best.is_none_or(|b| pr > b.0) {
                    best = Some((pr, lo, hi, f(&z, "scrollTime", 0.0).max(0.0)));
                }
            }
        }
    }
    let active = on(cam, "zoneActive", false);
    let (lo0, hi0) = (
        v2(&cam["zoneMin"], Vec2::ZERO),
        v2(&cam["zoneMax"], Vec2::ZERO),
    );
    let mut scroll = f(cam, "zoneScroll", 1.0).clamp(0.0, 1.0);
    let mut time = f(cam, "zoneScrollTime", 0.0).max(0.0);
    let xy = |v: Vec2| json!({ "x": v.x, "y": v.y });
    match best {
        Some((_, lo, hi, t)) if !active || lo != lo0 || hi != hi0 => {
            let scrolls = active && t > 0.0;
            cam["zoneFromActive"] = json!(scrolls);
            cam["zoneFromMin"] = xy(if scrolls { lo0 } else { Vec2::ZERO });
            cam["zoneFromMax"] = xy(if scrolls { hi0 } else { Vec2::ZERO });
            cam["zoneActive"] = json!(true);
            cam["zoneMin"] = xy(lo);
            cam["zoneMax"] = xy(hi);
            scroll = if scrolls { 0.0 } else { 1.0 };
            time = t;
        }
        Some(_) => {
            if scroll < 1.0 {
                scroll = if time > 0.0 {
                    (scroll + dt / time).min(1.0)
                } else {
                    1.0
                };
            }
        }
        None => {
            cam["zoneActive"] = json!(false);
            cam["zoneFromActive"] = json!(false);
            scroll = 1.0;
        }
    }
    if scroll >= 1.0 && on(cam, "zoneFromActive", false) {
        cam["zoneFromActive"] = json!(false);
    }
    cam["zoneScroll"] = json!(scroll);
    cam["zoneScrollTime"] = json!(time);
    Ok(())
}

/// The rotation that points -Z from `eye` towards `target`, +Y up.
fn look_rotation(eye: Vec3, target: Vec3) -> Quat {
    let forward = (target - eye).normalize_or_zero();
    if forward == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    let z = -forward;
    let x = Vec3::Y.cross(z).normalize_or(Vec3::X);
    let y = z.cross(x);
    Quat::from_mat3(&glam::Mat3::from_cols(x, y, z))
}

/// A point of the Catmull-Rom spline through `p` (ends repeated) at segment
/// `i`, parameter `t`.
fn catmull(p: &[Vec3], i: usize, t: f32) -> Vec3 {
    let at = |k: isize| p[k.clamp(0, p.len() as isize - 1) as usize];
    let (p0, p1, p2, p3) = (
        at(i as isize - 1),
        at(i as isize),
        at(i as isize + 1),
        at(i as isize + 2),
    );
    let (t2, t3) = (t * t, t * t * t);
    ((p1 * 2.0)
        + (p2 - p0) * t
        + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
        + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
        * 0.5
}

/// The rail point nearest `target`: 32 samples per segment, then a refinement.
pub(crate) fn rail_nearest(p: &[Vec3], target: Vec3) -> Vec3 {
    const S: usize = 32;
    let mut best = (f32::MAX, 0usize, 0.0f32);
    for i in 0..p.len() - 1 {
        for s in 0..=S {
            let t = s as f32 / S as f32;
            let d = catmull(p, i, t).distance_squared(target);
            if d < best.0 {
                best = (d, i, t);
            }
        }
    }
    let (_, i, mut t) = best;
    let mut step = 0.5 / S as f32;
    for _ in 0..8 {
        let (a, b) = ((t - step).max(0.0), (t + step).min(1.0));
        let (da, db, dt) = (
            catmull(p, i, a).distance_squared(target),
            catmull(p, i, b).distance_squared(target),
            catmull(p, i, t).distance_squared(target),
        );
        if da < dt && da <= db {
            t = a;
        } else if db < dt {
            t = b;
        }
        step *= 0.5;
    }
    catmull(p, i, t)
}

struct CameraBlock(Arc<Mutex<FeelShared>>);

impl StateBlock for CameraBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        out.extend_from_slice(&(sh.camera_ops.len() as u32).to_le_bytes());
        for op in &sh.camera_ops {
            let (k, v) = match *op {
                CameraOp::Shake(t) => (0u32, Vec3::new(t, 0.0, 0.0)),
                CameraOp::Punch(p) => (1, Vec3::new(p, 0.0, 0.0)),
                CameraOp::Kick(v) => (2, v),
            };
            out.extend_from_slice(&k.to_le_bytes());
            for x in v.to_array() {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let mut r = Reader::new(bytes, "og.camera");
        let n = r.u32()?;
        let mut ops = Vec::new();
        for _ in 0..n {
            let k = r.u32()?;
            let v = Vec3::new(r.f32()?, r.f32()?, r.f32()?);
            ops.push(match k {
                0 => CameraOp::Shake(v.x),
                1 => CameraOp::Punch(v.x),
                _ => CameraOp::Kick(v),
            });
        }
        r.end()?;
        Ok(Box::new(ops))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        lock(&self.0).camera_ops = decoded
            .map(|d| {
                *d.downcast::<Vec<CameraOp>>()
                    .expect("decoded by CameraBlock")
            })
            .unwrap_or_default();
    }
}

/// A little-endian reader for the feel state blocks.
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    what: &'static str,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8], what: &'static str) -> Reader<'a> {
        Reader { bytes, at: 0, what }
    }
    fn bad(&self) -> Error {
        Error::new(
            ErrorKind::Parse,
            format!("{} state block: truncated or malformed", self.what),
        )
    }
    pub fn take(&mut self, n: usize) -> og_core::Result<&'a [u8]> {
        let b = self
            .bytes
            .get(self.at..self.at + n)
            .ok_or_else(|| self.bad())?;
        self.at += n;
        Ok(b)
    }
    pub fn u32(&mut self) -> og_core::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }
    pub fn i32(&mut self) -> og_core::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }
    pub fn u64(&mut self) -> og_core::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }
    pub fn f32(&mut self) -> og_core::Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }
    pub fn f64(&mut self) -> og_core::Result<f64> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }
    pub fn bytes(&mut self) -> og_core::Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    pub fn end(&self) -> og_core::Result<()> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(self.bad())
        }
    }
}

fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b);
}

// ── tweens ─────────────────────────────────────────────────────────────────

/// A token or schema name of a component.
fn token_name(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("name").and_then(|n| n.as_str()).map(str::to_string),
        _ => None,
    }
}

/// The schema id index and schema of a component, by name.
fn component_schema(eng: &Engine, name: &str) -> Result<(u32, og_core::Schema), String> {
    let st = eng.state.borrow();
    let layout = st
        .layouts
        .get(name)
        .filter(|l| l.kind == super::codec::RecordKind::Component)
        .ok_or_else(|| format!("'{name}' is not a registered component"))?;
    let index = st
        .names_by_id
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| format!("'{name}' is not registered in this world"))?;
    Ok((index as u32, layout.schema()))
}

fn entity_of(v: &Value) -> Result<u64, String> {
    let e = v.as_f64().ok_or("desc.entity is the entity to tween")?;
    if e <= 0.0 || e.fract() != 0.0 {
        return Err(format!("{e} is not an entity"));
    }
    Ok(e as u64)
}

/// Whether `e` is alive and has component `index` (inside a system too:
/// archetype membership needs no declaration).
fn has(eng: &Engine, e: u64, index: u32) -> Result<bool, String> {
    let id = SchemaId::from_index(index);
    let ent = Entity::from_bits(e);
    let types = with_world(
        eng,
        |c| {
            if c.is_alive(ent) {
                c.components_of(ent)
            } else {
                Ok(Vec::new())
            }
        },
        |w| {
            if w.is_alive(ent) {
                w.components_of(ent)
            } else {
                Ok(Vec::new())
            }
        },
    )?;
    Ok(types.contains(&id))
}

fn ease_of(v: &Value, default: Ease) -> Result<Ease, String> {
    match v.as_str() {
        None => Ok(default),
        Some(n) => Ease::from_name(n).ok_or_else(|| {
            format!(
                "ease '{n}' is not one of {}",
                og_tween::EASE_NAMES.join(", ")
            )
        }),
    }
}

fn floats(v: &Value, what: &str) -> Result<Vec<f32>, String> {
    match v {
        Value::Number(n) => Ok(vec![n.as_f64().unwrap_or(0.0) as f32]),
        Value::Array(a) => a
            .iter()
            .map(|x| {
                x.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| format!("{what} holds numbers"))
            })
            .collect(),
        _ => Err(format!("{what} is a number or a list of numbers")),
    }
}

/// A TweenDesc as a leaf (checked against the entity and the field).
fn leaf_of(eng: &Engine, d: &Value) -> Result<Leaf, String> {
    let entity = entity_of(&d["entity"])?;
    let name = if d["component"].is_null() {
        "og.Transform".to_string()
    } else {
        token_name(&d["component"])
            .ok_or("desc.component is a component token, such as og.Sprite")?
    };
    let (index, schema) = component_schema(eng, &name)?;
    if !has(eng, entity, index)? {
        return Err(format!("entity {entity} has no {name} (or is not alive)"));
    }
    let field = d["field"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("desc.field names the field, such as `position.y`")?;
    let target = Target::resolve(&schema, index, entity, field).map_err(err)?;
    let mut leaf = Leaf::new(target, floats(&d["to"], "desc.to")?)
        .duration(f(d, "duration", 0.3))
        .delay(f(d, "delay", 0.0))
        .ease(ease_of(&d["ease"], Ease::QuadOut)?)
        .relative(on(d, "relative", false))
        .loops(
            d["loops"].as_i64().unwrap_or(0) as i32,
            on(d, "yoyo", false),
        );
    if !d["from"].is_null() {
        leaf = leaf.from(floats(&d["from"], "desc.from")?);
    }
    Ok(leaf)
}

fn start(eng: &Engine, node: Node, unscaled: bool, tag: u32) -> Result<Value, String> {
    let feel = feel_of(eng)?;
    let h = feel.tweens.start(node, unscaled, tag).map_err(err)?;
    Ok(json!(h as f64))
}

pub fn og_tween_to(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let leaf = leaf_of(&eng, &d)?;
        start(
            &eng,
            Node::Leaf(leaf),
            on(&d, "unscaled", false),
            d["tag"].as_u64().unwrap_or(0) as u32,
        )
    }))
}

fn group(world: u32, kind: GroupKind, list: &str, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let list = parse(Some(list))?;
        let d = parse(desc.as_deref())?;
        let leaves = list
            .as_array()
            .ok_or("the tweens are a list of TweenDesc")?
            .iter()
            .enumerate()
            .map(|(i, t)| leaf_of(&eng, t).map_err(|e| format!("tween {i}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;
        let node = Node::group(kind, leaves, d["loops"].as_i64().unwrap_or(0) as i32);
        start(
            &eng,
            node,
            on(&d, "unscaled", false),
            d["tag"].as_u64().unwrap_or(0) as u32,
        )
    }))
}

pub fn og_tween_sequence(world: u32, steps: String, desc: Option<String>) -> String {
    group(world, GroupKind::Sequence, &steps, desc)
}

pub fn og_tween_parallel(world: u32, tweens: String, desc: Option<String>) -> String {
    group(world, GroupKind::Parallel, &tweens, desc)
}

pub fn og_tween_cancel(world: u32, tween: f64) -> bool {
    engine(world)
        .and_then(|e| feel_of(&e))
        .is_ok_and(|f| f.tweens.cancel(tween as u64))
}

pub fn og_tween_cancel_entity(world: u32, entity: f64) -> u32 {
    engine(world)
        .and_then(|e| feel_of(&e))
        .map_or(0, |f| f.tweens.cancel_entity(entity as u64))
}

pub fn og_tween_active(world: u32, tween: f64) -> bool {
    engine(world)
        .and_then(|e| feel_of(&e))
        .is_ok_and(|f| f.tweens.is_active(tween as u64))
}

/// A preset shows its impact whole on the tick it is started in: every
/// tween in it waits one tick (it then holds its start value), so the
/// recovery begins on the next.
fn hold_first_tick(node: Node, tick: f32) -> Node {
    match node {
        Node::Leaf(l) => {
            let d = l.delay;
            Node::Leaf(l.delay(d + tick))
        }
        Node::Group(mut g) => {
            for c in &mut g.children {
                c.delay += tick;
            }
            Node::Group(g)
        }
    }
}

pub fn og_tween_squash(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let entity = entity_of(&d["entity"])?;
        let (index, schema) = component_schema(&eng, "og.Transform")?;
        if !has(&eng, entity, index)? {
            return Err(format!(
                "entity {entity} has no og.Transform (or is not alive)"
            ));
        }
        let target = Target::resolve(&schema, index, entity, "scale").map_err(err)?;
        let impact = v3(&d["impact"], Vec3::new(1.25, 0.8, 0.0));
        let rest = v3(&d["rest"], Vec3::ONE);
        let tick = 1.0 / eng.state.borrow().rate.max(1) as f32;
        let squash = og_tween::presets::Squash {
            impact: impact.to_array(),
            rest: rest.to_array(),
            duration: f(&d, "duration", 0.18),
            ease: ease_of(&d["ease"], Ease::BackOut)?,
        };
        let feel = feel_of(&eng)?;
        if let Some(old) = lock(&feel.shared).squash.remove(&entity) {
            feel.tweens.cancel(old);
        }
        let h = feel
            .tweens
            .start(
                hold_first_tick(og_tween::presets::squash(target, &squash), tick),
                false,
                0,
            )
            .map_err(err)?;
        lock(&feel.shared).squash.insert(entity, h);
        Ok(json!(h as f64))
    }))
}

pub fn og_tween_flash(world: u32, desc: Option<String>) -> String {
    wire(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let entity = entity_of(&d["entity"])?;
        let mut found = None;
        // A glTF instance flashes alone: og.Model's flash rides in its
        // instances' records (kestrel-stair #11).
        for name in ["og.Sprite", "og.Mesh", "og.Model"] {
            let (index, schema) = component_schema(&eng, name)?;
            if has(&eng, entity, index)? {
                found = Some((index, schema));
                break;
            }
        }
        let (index, schema) = found.ok_or_else(|| {
            format!("entity {entity} has none of og.Sprite, og.Mesh and og.Model (or is not alive)")
        })?;
        let amount = Target::resolve(&schema, index, entity, "flash").map_err(err)?;
        let color = Target::resolve(&schema, index, entity, "flashColor").map_err(err)?;
        let c = rgb(&d["color"], Vec3::ONE);
        let tick = 1.0 / eng.state.borrow().rate.max(1) as f32;
        let flash = og_tween::presets::Flash {
            color: c.to_array(),
            amount: f(&d, "amount", 1.0),
            duration: f(&d, "duration", 0.12),
            ease: ease_of(&d["ease"], Ease::QuadIn)?,
        };
        let feel = feel_of(&eng)?;
        if let Some(old) = lock(&feel.shared).flash.remove(&entity) {
            feel.tweens.cancel(old);
        }
        let h = feel
            .tweens
            .start(
                hold_first_tick(og_tween::presets::flash(amount, Some(color), &flash), tick),
                false,
                0,
            )
            .map_err(err)?;
        lock(&feel.shared).flash.insert(entity, h);
        Ok(json!(h as f64))
    }))
}

// ── floating text ──────────────────────────────────────────────────────────

pub fn og_render_floating_text(world: u32, desc: Option<String>) -> String {
    wire_void(engine(world).and_then(|eng| {
        let d = parse(desc.as_deref())?;
        let text = d["text"].as_str().unwrap_or_default().to_string();
        let font = match d["font"].as_f64() {
            None => {
                let has_ui = eng.render.borrow().ui.as_ref().is_some_and(|u| u.lock().fonts().body.handle.is_some());
                if !has_ui {
                    return Err("desc.font is null and the UI has no body font: load one with og.load.font and pass it".into());
                }
                u32::MAX
            }
            Some(h) => {
                let i = super::assets::index_of(h).ok_or_else(|| format!("{h} is not a font handle"))?;
                if i >= eng.render.borrow().fonts.len() {
                    return Err(format!("font {h} is not alive"));
                }
                i as u32
            }
        };
        let elapsed = with_world(&eng, |c| Ok(c.time_scale().elapsed), |w| Ok(w.time_scale().elapsed))?;
        let crit = on(&d, "crit", false);
        let scale = if crit { f(&d, "critScale", 1.6).max(1.0) } else { 1.0 };
        let feel = feel_of(&eng)?;
        let mut sh = lock(&feel.shared);
        // The drift: a hash of the spawn count, so equal runs drift equally.
        let n = sh.spawned;
        sh.spawned += 1;
        let h = (n.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as u32;
        let unit = (h % 2001) as f32 / 1000.0 - 1.0;
        sh.floating.push(Floating {
            text,
            font,
            position: v3(&d["position"], Vec3::ZERO),
            drift: unit * f(&d, "spread", 0.3).max(0.0),
            start: elapsed,
            size: f(&d, "size", 0.5).max(0.0) * scale,
            color: rgb(&d["color"], Vec3::ONE),
            outline: f(&d, "outline", 0.6).clamp(0.0, 1.0),
            outline_color: rgb(&d["outlineColor"], Vec3::new(0.02, 0.02, 0.03)),
            rise: f(&d, "rise", 1.0),
            lifetime: f(&d, "lifetime", 0.8).max(0.05),
            fade: f(&d, "fade", 0.4).clamp(0.0, 1.0),
            pop: f(&d, "pop", 0.4).max(0.0) * if crit { 1.5 } else { 1.0 },
            layer: d["layer"].as_i64().unwrap_or(100) as i32,
        });
        Ok(())
    }))
}

// ── loading ────────────────────────────────────────────────────────────────

/// The sampling a sheet or tileset states for its texture; it wins over the
/// texture's own (`kind pixel` loads nearest, `filter linear` smooths it).
fn set_filter(eng: &Engine, texture: usize, nearest: bool) {
    let mut rs = eng.render.borrow_mut();
    if let Some(&h) = rs.textures.get(texture) {
        let filter = if nearest {
            og_render::TextureFilter::Nearest
        } else {
            og_render::TextureFilter::Linear
        };
        rs.scene.set_texture_filter(h, filter);
    }
}

fn load_flipbook(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    let key = format!("flipbook:{id}");
    if let Some(v) = eng.state.borrow().feel_assets.loaded.get(&key) {
        return Ok(v.clone());
    }
    let sheet = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .flipbook(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // An `array true` sheet draws from the texture array the cook cut its
    // frames into; any other from regions of its texture.
    let image = if sheet.array {
        let array_id = format!("{id}#array");
        let i = super::assets::load_texture_array_index(eng, &array_id)
            .map_err(|e| format!("{id}: texture array {array_id}: {e}"))?;
        SheetImage::Array(i)
    } else {
        let texture = super::assets::load_texture(eng, &sheet.texture)
            .map_err(|e| format!("{id}: texture {}: {e}", sheet.texture))?;
        set_filter(eng, texture, sheet.filter == og_anim::SheetFilter::Nearest);
        SheetImage::Texture(texture)
    };
    let feel = feel_of(eng)?;
    let mut st = eng.state.borrow_mut();
    let handle = st.feel_assets.mint();
    feel.flipbooks
        .add_sheet(handle, sheet.clone())
        .map_err(|e| format!("{id}: {}", e.reason()))?;
    lock(&feel.sheet_textures).insert(handle, image);
    let clips: Vec<Value> = sheet
        .clips
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut events: Vec<(u32, String)> = c.events.clone();
            events.sort();
            json!({
                "clip": i,
                "name": c.name,
                "frames": c.frames.len(),
                "duration": c.duration(),
                "mode": c.mode.name(),
                "events": events.into_iter().map(|e| e.1).collect::<Vec<_>>(),
            })
        })
        .collect();
    let first = sheet.frames.first().map_or([0; 4], |f| f.rect);
    let answer = json!({
        "sheet": handle as f64,
        "texture": match image {
            SheetImage::Texture(t) => super::assets::handle(t) as f64,
            SheetImage::Array(_) => 0.0,
        },
        "array": match image {
            SheetImage::Texture(_) => 0.0,
            SheetImage::Array(a) => super::assets::handle(a) as f64,
        },
        "clips": clips,
        "frames": sheet.frames.len(),
        "frameWidth": first[2],
        "frameHeight": first[3],
    });
    st.feel_assets.sheets.insert(handle, (sheet, Some(image)));
    st.feel_assets.loaded.insert(key, answer.clone());
    Ok(answer)
}

pub fn og_load_flipbook(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_flipbook(&eng, &path)))
}

fn load_tilemap(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    let key = format!("tilemap:{id}");
    if let Some(v) = eng.state.borrow().feel_assets.loaded.get(&key) {
        return Ok(v.clone());
    }
    let (asset, tilesets) = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        let asset = bundle
            .tilemap(&id, bytes.bytes())
            .map_err(|e| e.to_string())?;
        let mut tilesets = Vec::new();
        for t in &asset.tilesets {
            tilesets.push(Arc::new(match t {
                og_tilemap::TilesetRef::Embedded(t) => t.clone(),
                og_tilemap::TilesetRef::Asset(tid) => {
                    let (b, bundle) = assets
                        .blob(tid)
                        .map_err(|e| format!("{id}: tileset {tid}: {e}"))?;
                    bundle
                        .tileset(tid, b.bytes())
                        .map_err(|e| format!("{id}: tileset {tid}: {e}"))?
                }
            }));
        }
        (asset, tilesets)
    };
    // Check the terrains and every cell against the tilesets now, not at the first tick.
    MapState::new(&asset, &tilesets).map_err(|e| format!("{id}: {}", e.reason()))?;
    let mut textures = Vec::new();
    for t in &tilesets {
        if t.texture.is_empty() {
            textures.push(None);
            continue;
        }
        let i = super::assets::load_texture(eng, &t.texture)
            .map_err(|e| format!("{id}: texture {}: {e}", t.texture))?;
        set_filter(eng, i, t.filter == og_tilemap::Filter::Nearest);
        textures.push(Some(i));
    }
    let tile = asset.tile_size();
    let layers: Vec<Value> = asset
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| {
            json!({
                "name": l.name, "index": i, "z": l.z,
                "parallax": { "x": l.parallax.x, "y": l.parallax.y },
                "collision": l.collision, "terrain": l.terrain.clone().unwrap_or_default(), "visible": l.visible,
                "lit": l.lit,
            })
        })
        .collect();
    let objects: Vec<Value> = asset
        .objects
        .iter()
        .map(|o| {
            let props: serde_json::Map<String, Value> = o
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), json!(v)))
                .collect();
            json!({
                "name": o.name, "kind": o.kind,
                "position": { "x": o.x, "y": o.y },
                "size": { "x": o.width, "y": o.height },
                "center": { "x": o.x + o.width / 2.0, "y": o.y - o.height / 2.0 },
                "properties": props,
            })
        })
        .collect();
    let feel = feel_of(eng)?;
    let map = Arc::new(MapAsset {
        asset: asset.clone(),
        tilesets,
        textures,
    });
    let mut st = eng.state.borrow_mut();
    let handle = st.feel_assets.mint();
    st.feel_assets.maps.insert(handle, map.clone());
    lock(&feel.tiles).maps.insert(handle, map);
    let answer = json!({
        "map": handle as f64,
        "width": asset.width, "height": asset.height,
        "tileWidth": tile.x, "tileHeight": tile.y,
        "layers": layers, "objects": objects,
    });
    st.feel_assets.loaded.insert(key, answer.clone());
    Ok(answer)
}

pub fn og_load_tilemap(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load_tilemap(&eng, &path)))
}

// ── tilemap calls ──────────────────────────────────────────────────────────

fn with_live<T>(
    world: u32,
    entity: f64,
    f: impl FnOnce(&mut LiveMap) -> Result<T, String>,
) -> Result<T, String> {
    let eng = engine(world)?;
    let feel = feel_of(&eng)?;
    let mut t = lock(&feel.tiles);
    let live = t
        .live
        .get_mut(&(entity as u64))
        .ok_or_else(|| format!("entity {entity} has no adopted og.Tilemap (og.tilemap adopts a map in the first tick it exists)"))?;
    f(live)
}

fn layer_index(live: &LiveMap, name: &str) -> Result<usize, String> {
    live.state.asset().layer(name).ok_or_else(|| {
        let names: Vec<_> = live
            .state
            .asset()
            .layers
            .iter()
            .map(|l| l.name.as_str())
            .collect();
        format!(
            "the map has no layer '{name}' (its layers: {})",
            names.join(", ")
        )
    })
}

pub fn og_tilemap_set_tile(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let d = parse(desc.as_deref())?;
        let entity = d["entity"]
            .as_f64()
            .ok_or("desc.entity is the entity with the og.Tilemap")?;
        with_live(world, entity, |live| {
            let layer = layer_index(live, d["layer"].as_str().unwrap_or_default())?;
            let (x, y) = (
                d["x"].as_u64().unwrap_or(0) as u32,
                d["y"].as_u64().unwrap_or(0) as u32,
            );
            let tile = d["tile"].as_i64().unwrap_or(-1) as i32;
            live.state.set(layer, x, y, tile).map_err(err)?;
            if live.state.asset().layers[layer].collision {
                live.dirty = true;
            }
            Ok(())
        })
    })())
}

pub fn og_tilemap_get_tile(world: u32, entity: f64, layer: String, x: u32, y: u32) -> i32 {
    with_live(world, entity, |live| {
        Ok(live.state.get(layer_index(live, &layer)?, x, y))
    })
    .unwrap_or(-1)
}

pub fn og_tilemap_cell_at(world: u32, entity: f64, point: String) -> String {
    let p = parse(Some(&point))
        .map(|v| v3(&v, Vec3::ZERO))
        .unwrap_or_default();
    let cell = with_live(world, entity, |live| {
        let local = Vec2::new(p.x - live.origin.x, p.y - live.origin.y);
        Ok(og_tilemap::world_to_cell(live.state.asset(), local))
    })
    .ok()
    .flatten();
    match cell {
        Some((x, y)) => json!({ "x": x, "y": y, "inside": true }),
        None => json!({ "x": 0, "y": 0, "inside": false }),
    }
    .to_string()
}

pub fn og_tilemap_cell_center(world: u32, entity: f64, x: u32, y: u32) -> String {
    let c = with_live(world, entity, |live| {
        let c = og_tilemap::cell_center(live.state.asset(), x, y);
        Ok(Vec3::new(
            live.origin.x + c.x,
            live.origin.y + c.y,
            live.origin.z,
        ))
    })
    .unwrap_or(Vec3::ZERO);
    json!({ "x": c.x, "y": c.y, "z": c.z }).to_string()
}

// ── the og.tilemap system ──────────────────────────────────────────────────

/// A collider entity's packed components.
type Spawn = Vec<(SchemaId, Vec<u8>)>;

fn tilemap_system(
    ctx: &mut SystemContext<'_>,
    tiles: &Mutex<TileShared>,
    r: &Records,
) -> og_core::Result<()> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut t = lock(tiles);
    let mut seen = BTreeSet::new();
    let mut gone: Vec<u64> = Vec::new();
    for e in rows {
        let key = e.to_bits();
        let v = r.tilemap.1.decode(&ctx.get_bytes(e, r.tilemap.0)?);
        let tr = r.transform.1.decode(&ctx.get_bytes(e, r.transform.0)?);
        let map = bits(&v["map"]);
        let Some(asset) = t.maps.get(&map).cloned() else {
            continue;
        };
        seen.insert(key);
        let origin = v3(&tr["position"], Vec3::ZERO);
        let collision = on(&v, "collision", true);
        let layers = v["colliderLayers"].as_u64().unwrap_or(1) as u32;
        let friction = f(&v, "friction", 0.5);
        let fresh = t.live.get(&key).is_none_or(|l| l.map != map);
        if fresh {
            let old = t.live.remove(&key).map(|l| l.colliders).unwrap_or_default();
            gone.extend(old);
            let state = MapState::new(&asset.asset, &asset.tilesets)?;
            t.live.insert(
                key,
                LiveMap {
                    map,
                    state,
                    colliders: Vec::new(),
                    origin,
                    collision,
                    layers,
                    friction,
                    dirty: true,
                },
            );
        }
        let live = t.live.get_mut(&key).expect("inserted above");
        if live.origin != origin
            || live.collision != collision
            || live.layers != layers
            || live.friction != friction
        {
            live.origin = origin;
            live.collision = collision;
            live.layers = layers;
            live.friction = friction;
            live.dirty = true;
        }
    }
    let dropped: Vec<u64> = t
        .live
        .keys()
        .filter(|k| !seen.contains(k))
        .copied()
        .collect();
    for k in dropped {
        gone.extend(t.live.remove(&k).map(|l| l.colliders).unwrap_or_default());
    }
    let mut spawns: Vec<(u64, Spawn)> = Vec::new();
    for (&key, live) in t.live.iter_mut().filter(|(_, l)| l.dirty) {
        live.dirty = false;
        gone.append(&mut live.colliders);
        if !live.collision {
            continue;
        }
        for rect in og_tilemap::collision_rects_all(&live.state) {
            let c = (rect.min + rect.max) * 0.5;
            let size = rect.max - rect.min;
            let tj = json!({ "position": { "x": live.origin.x + c.x, "y": live.origin.y + c.y, "z": live.origin.z } });
            let cj = json!({
                "shape": 0, "size": { "x": size.x, "y": size.y, "z": 1.0 },
                "friction": live.friction, "layers": live.layers,
            });
            let enc = |l: &Layout, v: &Value| {
                l.encode(None, Some(v))
                    .map_err(|e| Error::new(ErrorKind::Invalid, e))
            };
            spawns.push((
                key,
                vec![
                    (r.transform.0, enc(&r.transform.1, &tj)?),
                    (r.collider.0, enc(&r.collider.1, &cj)?),
                ],
            ));
        }
    }
    for g in gone {
        let e = Entity::from_bits(g);
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
            queued_structural();
        }
    }
    for (key, comps) in spawns {
        let e = ctx.spawn_bytes(comps)?;
        queued_structural();
        if let Some(live) = t.live.get_mut(&key) {
            live.colliders.push(e.to_bits());
        }
    }
    Ok(())
}

struct TileBlock(Arc<Mutex<TileShared>>);

impl StateBlock for TileBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let t = lock(&self.0);
        out.extend_from_slice(&(t.live.len() as u32).to_le_bytes());
        for (k, l) in &t.live {
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&l.map.to_le_bytes());
            for x in l.origin.to_array() {
                out.extend_from_slice(&x.to_le_bytes());
            }
            out.extend_from_slice(&(l.collision as u32 | (l.dirty as u32) << 1).to_le_bytes());
            out.extend_from_slice(&l.layers.to_le_bytes());
            out.extend_from_slice(&l.friction.to_le_bytes());
            out.extend_from_slice(&(l.colliders.len() as u32).to_le_bytes());
            for c in &l.colliders {
                out.extend_from_slice(&c.to_le_bytes());
            }
            let mut s = Vec::new();
            l.state.encode(&mut s);
            put_bytes(out, &s);
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let maps = lock(&self.0).maps.clone();
        let mut r = Reader::new(bytes, "og.tilemap");
        let n = r.u32()?;
        let mut live = BTreeMap::new();
        for _ in 0..n {
            let key = r.u64()?;
            let map = r.u64()?;
            let origin = Vec3::new(r.f32()?, r.f32()?, r.f32()?);
            let flags = r.u32()?;
            let layers = r.u32()?;
            let friction = r.f32()?;
            let colliders = (0..r.u32()?)
                .map(|_| r.u64())
                .collect::<og_core::Result<Vec<_>>>()?;
            let s = r.bytes()?;
            let asset = maps.get(&map).ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("og.tilemap state block: map {map} is not loaded in this engine"),
                )
            })?;
            let state = MapState::decode(&asset.asset, &asset.tilesets, s)?;
            live.insert(
                key,
                LiveMap {
                    map,
                    state,
                    colliders,
                    origin,
                    collision: flags & 1 != 0,
                    layers,
                    friction,
                    dirty: flags & 2 != 0,
                },
            );
        }
        r.end()?;
        Ok(Box::new(LiveMaps(live)))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        lock(&self.0).live = decoded
            .map(|d| d.downcast::<LiveMaps>().expect("decoded by TileBlock").0)
            .unwrap_or_default();
    }
}

struct LiveMaps(BTreeMap<u64, LiveMap>);

// ── the og.sprite system ───────────────────────────────────────────────────

/// The rotation about z of a quaternion (deterministic arithmetic).
fn yaw_z(q: Quat) -> f32 {
    og_core::math::atan2(
        2.0 * (q.w * q.z + q.x * q.y),
        1.0 - 2.0 * (q.y * q.y + q.z * q.z),
    )
}

fn sprite_system(
    ctx: &mut SystemContext<'_>,
    shared: &Mutex<FeelShared>,
    r: &Records,
) -> og_core::Result<()> {
    let tick = ctx.time().tick;
    let now = ctx.time_scale().elapsed;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let mut sh = lock(shared);
    sh.floating
        .retain(|t| now - t.start <= f64::from(t.lifetime));
    let mut seen = BTreeSet::new();
    for e in rows {
        let key = e.to_bits();
        seen.insert(key);
        let a = r.afterimage.1.decode(&ctx.get_bytes(e, r.afterimage.0)?);
        let s = r.sprite.1.decode(&ctx.get_bytes(e, r.sprite.0)?);
        let t = r.transform.1.decode(&ctx.get_bytes(e, r.transform.0)?);
        let count = a["count"].as_u64().unwrap_or(4).min(16) as usize;
        let interval = a["interval"].as_u64().unwrap_or(3).max(1);
        let list = sh.ghosts.entry(key).or_default();
        // Ghosts older than the whole trail have faded out.
        let span = interval * count as u64;
        list.retain(|g| tick.saturating_sub(g.tick) < span.max(1));
        if on(&a, "active", true) && tick.is_multiple_of(interval) && count > 0 {
            let (sheet, clip, frame) = if ctx.components_of(e)?.contains(&r.flipbook.0) {
                let fb = r.flipbook.1.decode(&ctx.get_bytes(e, r.flipbook.0)?);
                (
                    bits(&fb["sheet"]),
                    fb["clip"].as_u64().unwrap_or(0) as u32,
                    fb["frame"].as_u64().unwrap_or(0) as u32,
                )
            } else {
                (0, 0, 0)
            };
            let q = &t["quaternion"];
            let quat = Quat::from_xyzw(
                f(q, "x", 0.0),
                f(q, "y", 0.0),
                f(q, "z", 0.0),
                f(q, "w", 1.0),
            );
            let scale = v3(&t["scale"], Vec3::ONE);
            list.insert(
                0,
                Ghost {
                    position: v3(&t["position"], Vec3::ZERO),
                    rotation: yaw_z(quat),
                    scale: Vec2::new(scale.x, scale.y),
                    sheet,
                    clip,
                    frame,
                    flip: on(&s, "flipX", false) as u8 | (on(&s, "flipY", false) as u8) << 1,
                    tick,
                },
            );
            list.truncate(count);
        }
    }
    sh.ghosts.retain(|k, v| seen.contains(k) && !v.is_empty());
    Ok(())
}

struct SpriteBlock(Arc<Mutex<FeelShared>>);

impl StateBlock for SpriteBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        let u32_ = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
        let f32_ = |out: &mut Vec<u8>, v: f32| out.extend_from_slice(&v.to_le_bytes());
        out.extend_from_slice(&sh.spawned.to_le_bytes());
        u32_(out, sh.ghosts.len() as u32);
        for (k, list) in &sh.ghosts {
            out.extend_from_slice(&k.to_le_bytes());
            u32_(out, list.len() as u32);
            for g in list {
                for x in g.position.to_array() {
                    f32_(out, x);
                }
                f32_(out, g.rotation);
                f32_(out, g.scale.x);
                f32_(out, g.scale.y);
                out.extend_from_slice(&g.sheet.to_le_bytes());
                u32_(out, g.clip);
                u32_(out, g.frame);
                u32_(out, u32::from(g.flip));
                out.extend_from_slice(&g.tick.to_le_bytes());
            }
        }
        u32_(out, sh.floating.len() as u32);
        for t in &sh.floating {
            put_bytes(out, t.text.as_bytes());
            u32_(out, t.font);
            for x in t.position.to_array() {
                f32_(out, x);
            }
            f32_(out, t.drift);
            out.extend_from_slice(&t.start.to_le_bytes());
            f32_(out, t.size);
            for x in t.color.to_array() {
                f32_(out, x);
            }
            f32_(out, t.outline);
            for x in t.outline_color.to_array() {
                f32_(out, x);
            }
            for x in [t.rise, t.lifetime, t.fade, t.pop] {
                f32_(out, x);
            }
            out.extend_from_slice(&t.layer.to_le_bytes());
        }
        for map in [&sh.squash, &sh.flash] {
            u32_(out, map.len() as u32);
            for (k, v) in map {
                out.extend_from_slice(&k.to_le_bytes());
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let mut r = Reader::new(bytes, "og.sprite");
        let mut sh = FeelShared {
            spawned: r.u64()?,
            ..FeelShared::default()
        };
        let v3r = |r: &mut Reader<'_>| -> og_core::Result<Vec3> {
            Ok(Vec3::new(r.f32()?, r.f32()?, r.f32()?))
        };
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let mut list = Vec::new();
            for _ in 0..r.u32()? {
                list.push(Ghost {
                    position: v3r(&mut r)?,
                    rotation: r.f32()?,
                    scale: Vec2::new(r.f32()?, r.f32()?),
                    sheet: r.u64()?,
                    clip: r.u32()?,
                    frame: r.u32()?,
                    flip: r.u32()? as u8,
                    tick: r.u64()?,
                });
            }
            sh.ghosts.insert(k, list);
        }
        for _ in 0..r.u32()? {
            let text = String::from_utf8(r.bytes()?.to_vec()).map_err(|_| {
                Error::new(ErrorKind::Parse, "og.sprite state block: text is not UTF-8")
            })?;
            sh.floating.push(Floating {
                text,
                font: r.u32()?,
                position: v3r(&mut r)?,
                drift: r.f32()?,
                start: r.f64()?,
                size: r.f32()?,
                color: v3r(&mut r)?,
                outline: r.f32()?,
                outline_color: v3r(&mut r)?,
                rise: r.f32()?,
                lifetime: r.f32()?,
                fade: r.f32()?,
                pop: r.f32()?,
                layer: r.i32()?,
            });
        }
        for i in 0..2 {
            let mut map = BTreeMap::new();
            for _ in 0..r.u32()? {
                map.insert(r.u64()?, r.u64()?);
            }
            if i == 0 {
                sh.squash = map;
            } else {
                sh.flash = map;
            }
        }
        r.end()?;
        Ok(Box::new(sh))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut sh = lock(&self.0);
        let ops = std::mem::take(&mut sh.camera_ops);
        *sh = decoded
            .map(|d| *d.downcast::<FeelShared>().expect("decoded by SpriteBlock"))
            .unwrap_or_default();
        // The camera ops are the og.camera block's.
        sh.camera_ops = ops;
    }
}

// ── extraction ─────────────────────────────────────────────────────────────

/// A layer's chunks as drawn: the map, the chunk versions they were built at.
type TileCache = (u64, Vec<u64>, Arc<Vec<og_render::TileChunk>>);

/// What extraction keeps between frames for the feel modules.
#[derive(Default)]
pub(crate) struct FeelRender {
    pub feel: Option<(Feel, Records)>,
    /// The soft round texture of blob shadows, made on first use.
    blob: Option<og_render::TextureHandle>,
    /// Per (entity, layer): the map handle, the chunk versions and the chunks.
    tiles: BTreeMap<(u64, usize), TileCache>,
}

fn decoded_rows(world: &World, id: SchemaId, layout: &Layout) -> Vec<(u64, Value)> {
    let q = QueryDesc {
        read: vec![layout.schema.clone()],
        ..QueryDesc::default()
    };
    let Ok(all) = world.query(&q) else {
        return Vec::new();
    };
    let entities: Vec<Entity> = all.flat_map(|a| a.entities().to_vec()).collect();
    entities
        .into_iter()
        .filter_map(|e| {
            world
                .get_bytes(e, id)
                .ok()
                .map(|b| (e.to_bits(), layout.decode(&b)))
        })
        .collect()
}

fn blob_texture(scene: &mut og_render::RenderScene) -> og_render::TextureHandle {
    const N: u32 = 64;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let u = (x as f32 + 0.5) / N as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / N as f32 * 2.0 - 1.0;
            // A solid core with a soft rim (the outer quarter fades).
            let d = ((1.0 - (u * u + v * v).sqrt()) * 4.0).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[255, 255, 255, (d * 255.0).round() as u8]);
        }
    }
    scene.add_texture(og_render::TextureData {
        width: N,
        height: N,
        rgba,
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    })
}

struct SpriteOut {
    sprite: og_render::Sprite,
    key: Option<f32>,
    flash: [f32; 4],
    glow: f32,
    additive: bool,
    cast_shadow: bool,
    /// Drawn from this layer of a texture array instead of its texture.
    array: Option<(og_render::TextureArrayHandle, u32)>,
    /// `Sprite.dropShadow`: a flattened dark copy on the ground (0 none),
    /// and `Sprite.dropShadowHeight`, the level ground under it (a higher
    /// terrain wins, `terrain::ground_drop_shadows`).
    drop_shadow: f32,
    drop_shadow_height: f32,
    /// `Sprite.toneMapped`: false keeps it out of exposure, bloom and the
    /// tone curve ([`og_render::SpriteFx::tone_mapped`]).
    tone_mapped: bool,
}

fn push(scene: &mut og_render::RenderScene, s: SpriteOut) {
    let index = scene.sprites.len() as u32;
    scene.sprites.push(s.sprite);
    if let Some((array, layer)) = s.array {
        scene.sprite_arrays.push(og_render::SpriteArray {
            sprite: index,
            array,
            layer,
        });
    }
    if let Some(key) = s.key {
        scene
            .sprite_order
            .push(og_render::SpriteOrder { sprite: index, key });
    }
    if s.cast_shadow {
        scene
            .occluders_2d
            .push(og_render::Occluder2d::from_sprite(&s.sprite, 0.0));
    }
    if s.drop_shadow > 0.0 {
        scene.sprite_shadows.push(og_render::SpriteShadow {
            sprite: index,
            opacity: s.drop_shadow.min(1.0),
            ground: og_render::ShadowGround::level(s.drop_shadow_height),
        });
    }
    if s.flash[3] > 0.0 || s.glow > 0.0 || s.additive || !s.tone_mapped {
        scene.sprite_fx.push(og_render::SpriteFx {
            sprite: index,
            flash: s.flash,
            glow: s.glow,
            blend: if s.additive {
                og_render::SpriteBlend::Additive
            } else {
                og_render::SpriteBlend::Alpha
            },
            tone_mapped: s.tone_mapped,
        });
    }
}

/// What a sprite is drawn from: a texture (`None`: solid), or a layer of a
/// texture array.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Image {
    Texture(Option<og_render::TextureHandle>),
    Array(og_render::TextureArrayHandle, u32),
}

impl Image {
    fn texture(self) -> Option<og_render::TextureHandle> {
        match self {
            Image::Texture(t) => t,
            Image::Array(..) => None,
        }
    }

    fn array(self) -> Option<(og_render::TextureArrayHandle, u32)> {
        match self {
            Image::Texture(_) => None,
            Image::Array(a, layer) => Some((a, layer)),
        }
    }
}

/// The image, uv region and anchor a flipbook frame gives a sprite: a
/// region of the sheet's texture, or the whole layer of an `array true`
/// sheet's array (`None` while that array is not in the renderer).
fn sheet_frame(
    feel: &Feel,
    rs: (&[og_render::TextureHandle], &[super::assets::LoadedArray]),
    sheet: u64,
    clip: u32,
    frame: u32,
) -> Option<(Image, [f32; 4], Vec2)> {
    let s = feel.flipbooks.sheet(sheet)?;
    let c = s.clips.get(clip as usize)?;
    let index = *c.frames.get(frame as usize)?;
    let f = s.frames.get(index as usize)?;
    let pivot = Vec2::new(f.pivot[0], f.pivot[1]);
    match *lock(&feel.sheet_textures).get(&sheet)? {
        SheetImage::Texture(t) => Some((Image::Texture(Some(*rs.0.get(t)?)), s.uv(index), pivot)),
        SheetImage::Array(a) => {
            let loaded = rs.1.get(a)?;
            let layer = index.min(loaded.layers.saturating_sub(1));
            Some((
                Image::Array(loaded.handle?, layer),
                [0.0, 0.0, 1.0, 1.0],
                pivot,
            ))
        }
    }
}

/// An og.Sprite's own image: its texture array's layer when it names one
/// (`None` while the array's streaming group is not resident: nothing is
/// drawn), else its texture.
fn sprite_image(
    v: &Value,
    textures: &[og_render::TextureHandle],
    arrays: &[super::assets::LoadedArray],
) -> Option<Image> {
    match super::fx::handle_index(&v["array"]) {
        Some(i) => {
            let loaded = arrays.get(i)?;
            let layer = v["arrayLayer"].as_u64().unwrap_or(0) as u32;
            Some(Image::Array(
                loaded.handle?,
                layer.min(loaded.layers.saturating_sub(1)),
            ))
        }
        None => Some(Image::Texture(
            super::fx::handle_index(&v["texture"]).and_then(|i| textures.get(i).copied()),
        )),
    }
}

fn flipped(mut uv: [f32; 4], mut anchor: Vec2, flip: u8) -> ([f32; 4], Vec2) {
    if flip & 1 != 0 {
        uv.swap(0, 2);
        anchor.x = 1.0 - anchor.x;
    }
    if flip & 2 != 0 {
        uv.swap(1, 3);
        anchor.y = 1.0 - anchor.y;
    }
    (uv, anchor)
}

/// og.Sprite rows (with their flipbook frames, nine-slices, shadows and
/// afterimages), then the floating texts and the tile layers.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let Some((feel, r)) = rs.feel.feel.clone() else {
        return;
    };
    let scene_tick = world.time().tick;
    let scaled_now = world.time_scale().elapsed;
    let flips: BTreeMap<u64, Value> = decoded_rows(world, r.flipbook.0, &r.flipbook.1)
        .into_iter()
        .collect();
    let sprites = decoded_rows(world, r.sprite.0, &r.sprite.1);
    let afterimages: BTreeMap<u64, Value> = decoded_rows(world, r.afterimage.0, &r.afterimage.1)
        .into_iter()
        .collect();
    let ghosts = lock(&feel.shared).ghosts.clone();
    if rs.feel.blob.is_none() && sprites.iter().any(|(_, v)| f(v, "shadow", 0.0) > 0.0) {
        rs.feel.blob = Some(blob_texture(&mut rs.scene));
    }
    let blob = rs.feel.blob;
    let textures = rs.textures.clone();
    let arrays = &rs.texture_arrays;
    let scene = &mut rs.scene;
    for (e, v) in sprites {
        if !on(&v, "visible", true) {
            continue;
        }
        let m = h.world_of(e);
        let pos = m.w_axis.truncate();
        let xa = m.x_axis.truncate();
        let rotation = xa.y.atan2(xa.x);
        let scale = Vec2::new(xa.length(), m.y_axis.truncate().length());
        let size = Vec2::new(f(&v, "width", 1.0), f(&v, "height", 1.0));
        let flip = on(&v, "flipX", false) as u8 | (on(&v, "flipY", false) as u8) << 1;
        // A texture array whose streaming group is not resident draws nothing.
        let Some(mut image) = sprite_image(&v, &textures, arrays) else {
            continue;
        };
        let lo = v2(&v["uvMin"], Vec2::ZERO);
        let hi = v2(&v["uvMax"], Vec2::ONE);
        let mut uv = [lo.x, lo.y, hi.x, hi.y];
        let mut anchor = v2(&v["anchor"], Vec2::splat(0.5));
        let flipbook = flips.get(&e).map(|fb| {
            (
                bits(&fb["sheet"]),
                fb["clip"].as_u64().unwrap_or(0) as u32,
                fb["frame"].as_u64().unwrap_or(0) as u32,
            )
        });
        if let Some((img, fuv, pivot)) =
            flipbook.and_then(|(s, c, fr)| sheet_frame(&feel, (&textures, arrays), s, c, fr))
        {
            image = img;
            uv = fuv;
            anchor = pivot;
        }
        let (uv, anchor) = flipped(uv, anchor, flip);
        let c = rgb(&v["color"], Vec3::ONE);
        let opacity = f(&v, "opacity", 1.0);
        let layer = v["layer"].as_i64().unwrap_or(0) as i32;
        let key = on(&v, "sortY", false).then_some(-pos.y);
        let fc = rgb(&v["flashColor"], Vec3::ONE);
        let flash = [fc.x, fc.y, fc.z, f(&v, "flash", 0.0).clamp(0.0, 1.0)];
        let glow = f(&v, "glow", 0.0).max(0.0);
        let additive = v["blend"].as_str() == Some("additive");
        let tone_mapped = on(&v, "toneMapped", true);
        let base = og_render::Sprite {
            texture: image.texture(),
            position: pos,
            size: size * scale,
            anchor,
            rotation,
            color: [c.x, c.y, c.z, opacity],
            uv_rect: uv,
            layer,
            clip: None,
        };
        // The blob shadow: under every sprite of the layer.
        let shadow = f(&v, "shadow", 0.0);
        if shadow > 0.0 {
            let mut ss = v2(&v["shadowSize"], Vec2::ZERO);
            if ss == Vec2::ZERO {
                ss = Vec2::new(0.8, 0.3) * base.size.x;
            }
            let off = v2(&v["shadowOffset"], Vec2::ZERO);
            push(
                scene,
                SpriteOut {
                    sprite: og_render::Sprite {
                        texture: blob,
                        position: pos + off.extend(0.0),
                        size: ss,
                        anchor: Vec2::splat(0.5),
                        rotation: 0.0,
                        color: [0.0, 0.0, 0.0, shadow.min(1.0)],
                        uv_rect: [0.0, 0.0, 1.0, 1.0],
                        layer,
                        clip: None,
                    },
                    key: Some(f32::MIN),
                    flash: [1.0, 1.0, 1.0, 0.0],
                    glow: 0.0,
                    additive: false,
                    cast_shadow: false,
                    array: None,
                    drop_shadow: 0.0,
                    drop_shadow_height: 0.0,
                    tone_mapped: true,
                },
            );
        }
        // Afterimages: oldest first, under the sprite.
        if let (Some(a), Some(list)) = (afterimages.get(&e), ghosts.get(&e)) {
            let count = a["count"].as_u64().unwrap_or(4).max(1) as f32;
            let interval = a["interval"].as_u64().unwrap_or(3).max(1) as f32;
            let tint = rgb(&a["color"], Vec3::ONE);
            let op = f(a, "opacity", 0.5);
            let add = on(a, "additive", false);
            for g in list.iter().rev() {
                let age = scene_tick.saturating_sub(g.tick) as f32 / (count * interval);
                let alpha = op * (1.0 - age).clamp(0.0, 1.0);
                if alpha <= 0.0 {
                    continue;
                }
                let (gimage, guv, ganchor) =
                    sheet_frame(&feel, (&textures, arrays), g.sheet, g.clip, g.frame).unwrap_or((
                        image,
                        [lo.x, lo.y, hi.x, hi.y],
                        v2(&v["anchor"], Vec2::splat(0.5)),
                    ));
                let (guv, ganchor) = flipped(guv, ganchor, g.flip);
                push(
                    scene,
                    SpriteOut {
                        sprite: og_render::Sprite {
                            texture: gimage.texture(),
                            position: g.position,
                            size: size * g.scale,
                            anchor: ganchor,
                            rotation: g.rotation,
                            color: [c.x * tint.x, c.y * tint.y, c.z * tint.z, opacity * alpha],
                            uv_rect: guv,
                            layer,
                            clip: None,
                        },
                        key: key.map(|k| k - 1e-4),
                        flash: [1.0, 1.0, 1.0, 0.0],
                        glow: 0.0,
                        additive: add,
                        cast_shadow: false,
                        array: gimage.array(),
                        drop_shadow: 0.0,
                        drop_shadow_height: 0.0,
                        tone_mapped,
                    },
                );
            }
        }
        let sx = v2(&v["sliceX"], Vec2::ZERO);
        let sy = v2(&v["sliceY"], Vec2::ZERO);
        let size_of = match image {
            Image::Texture(t) => t.and_then(|t| scene.texture_size(t)),
            Image::Array(h, _) => arrays.iter().find(|a| a.handle == Some(h)).map(|a| a.size),
        };
        let pieces = match size_of {
            Some(tsize) if sx != Vec2::ZERO || sy != Vec2::ZERO => og_render::nine_slice(
                &base,
                [sx.x, sy.x, sx.y, sy.y],
                tsize,
                f(&v, "sliceScale", 0.0625),
            ),
            _ => vec![base],
        };
        let cast = on(&v, "castShadow", false);
        let drop_shadow = f(&v, "dropShadow", 0.0).clamp(0.0, 1.0);
        let drop_shadow_height = f(&v, "dropShadowHeight", 0.0);
        for (i, p) in pieces.into_iter().enumerate() {
            push(
                scene,
                SpriteOut {
                    sprite: p,
                    key,
                    flash,
                    glow,
                    additive,
                    cast_shadow: cast && i == 0,
                    array: image.array(),
                    drop_shadow,
                    drop_shadow_height,
                    tone_mapped,
                },
            );
        }
    }
    floating(rs, &feel, scaled_now);
    tile_layers(world, rs, &feel, &r, h, scaled_now);
}

fn floating(rs: &mut RenderState, feel: &Feel, now: f64) {
    let list = lock(&feel.shared).floating.clone();
    let ui = rs.ui.clone();
    for t in list {
        // No font of its own: the UI's for this text (its body face, or the
        // locale's or a fallback when the body face lacks a character).
        let body = ui.as_ref().and_then(|u| {
            u.lock()
                .fonts()
                .for_text(og_ui::FontRole::Body, &t.text)
                .handle
        });
        let age = (now - t.start) as f32;
        if age < 0.0 || age > t.lifetime {
            continue;
        }
        let k = age / t.lifetime;
        let rise = t.rise * (1.0 - (1.0 - k) * (1.0 - k));
        let pop = 1.0 + t.pop * (1.0 - (k / 0.2).min(1.0)).powi(2);
        let size = t.size * pop;
        let alpha = if k > 1.0 - t.fade && t.fade > 0.0 {
            ((1.0 - k) / t.fade).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let pages: Vec<og_render::FontHandle> = if t.font == u32::MAX {
            match body {
                Some(b) => rs.ui_pages.get(&b).cloned().unwrap_or_else(|| vec![b]),
                None => continue,
            }
        } else {
            match rs.fonts.get(t.font as usize) {
                Some(p) => p.pages.clone(),
                None => continue,
            }
        };
        let Some(first) = pages.first().and_then(|p| rs.scene.font(*p)) else {
            continue;
        };
        let width: f32 = first.measure(&t.text) * size;
        let position = Vec3::new(
            t.position.x + t.drift * k - width / 2.0,
            t.position.y + rise,
            t.position.z,
        );
        for page in pages {
            let index = rs.scene.texts.len() as u32;
            rs.scene.texts.push(og_render::Text {
                font: page,
                text: t.text.clone(),
                position,
                size,
                rotation: 0.0,
                color: [t.color.x, t.color.y, t.color.z, alpha],
                layer: t.layer,
                clip: None,
                tracking: 0.0,
                weight: 0.0,
                glow: None,
                placements: Vec::new(),
            });
            if t.outline > 0.0 {
                rs.scene.text_outlines.push(og_render::TextOutline {
                    text: index,
                    width: t.outline,
                    color: [
                        t.outline_color.x,
                        t.outline_color.y,
                        t.outline_color.z,
                        alpha,
                    ],
                });
            }
        }
    }
}

fn tile_layers(
    world: &World,
    rs: &mut RenderState,
    feel: &Feel,
    r: &Records,
    h: &mut Hierarchy,
    now: f64,
) {
    let rows = decoded_rows(world, r.tilemap.0, &r.tilemap.1);
    if rows.is_empty() {
        return;
    }
    let tiles = lock(&feel.tiles);
    let cam = rs.scene.camera.position;
    let RenderState {
        scene,
        feel: fr,
        textures,
        ..
    } = rs;
    let mut keep = BTreeSet::new();
    for (e, v) in rows {
        if !on(&v, "visible", true) {
            continue;
        }
        let (Some(live), map) = (tiles.live.get(&e), bits(&v["map"])) else {
            continue;
        };
        if live.map != map {
            continue;
        }
        let Some(asset) = tiles.maps.get(&map) else {
            continue;
        };
        let origin = h.world_of(e).w_axis.truncate();
        let tint = rgb(&v["color"], Vec3::ONE);
        let opacity = f(&v, "opacity", 1.0);
        let base = v["layer"].as_i64().unwrap_or(-10) as i32;
        let (cw, ch) = live.state.chunks();
        let world_size = asset.asset.world_size();
        for (li, layer) in asset.asset.layers.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            let Some(Some(ti)) = asset.textures.get(layer.tileset as usize) else {
                continue;
            };
            let Some(&texture) = textures.get(*ti) else {
                continue;
            };
            let Some((tw, th)) = scene.texture_size(texture) else {
                continue;
            };
            keep.insert((e, li));
            let versions: Vec<u64> = (0..ch)
                .flat_map(|cy| (0..cw).map(move |cx| (cx, cy)))
                .map(|(cx, cy)| live.state.chunk_version(li, cx, cy))
                .collect();
            let fresh = fr
                .tiles
                .get(&(e, li))
                .is_some_and(|(m, vs, _)| *m == map && *vs == versions);
            if !fresh {
                let mut chunks = Vec::new();
                for cy in 0..ch {
                    for cx in 0..cw {
                        let q = og_tilemap::chunk_quads(&live.state, li, cx, cy, tw, th);
                        if q.quads.is_empty() {
                            continue;
                        }
                        let mut min = Vec2::splat(f32::MAX);
                        let mut max = Vec2::splat(f32::MIN);
                        for t in &q.quads {
                            min = min.min(t.min);
                            max = max.max(t.min + t.size);
                        }
                        chunks.push(og_render::TileChunk {
                            min,
                            max,
                            quads: q
                                .quads
                                .iter()
                                .map(|t| og_render::TileQuad {
                                    min: t.min,
                                    size: t.size,
                                    uv: t.uv,
                                })
                                .collect(),
                            animated: q
                                .animated
                                .iter()
                                .map(|a| og_render::TileAnimation {
                                    quad: a.quad,
                                    uvs: a.frames.clone(),
                                    durations: a.durations.clone(),
                                })
                                .collect(),
                        });
                    }
                }
                fr.tiles.insert((e, li), (map, versions, Arc::new(chunks)));
            }
            let chunks = fr.tiles[&(e, li)].2.clone();
            let p = layer.parallax;
            let at = Vec3::new(
                origin.x + layer.offset.x + cam.x * (1.0 - p.x),
                origin.y + layer.offset.y + cam.y * (1.0 - p.y),
                origin.z,
            );
            scene.tile_layers.push(og_render::TileLayer {
                texture,
                chunks,
                origin: at,
                layer: base + layer.z,
                color: [tint.x, tint.y, tint.z, opacity * layer.opacity],
                repeat: [
                    layer.repeat_x.then_some(world_size.x),
                    layer.repeat_y.then_some(world_size.y),
                ],
                time: now as f32,
                lit: layer.lit,
            });
        }
    }
    fr.tiles.retain(|k, _| keep.contains(k));
}

/// The camera feel on the frame only: trauma shake, punch, kick, and the
/// zone the followed entity is in (visual: the world's camera is untouched).
pub(crate) fn camera_frame(world: &World, rs: &mut RenderState, aspect: f32) {
    let Some((_, r)) = rs.feel.feel.clone() else {
        return;
    };
    let Ok(bytes) = world.resource_bytes(r.camera.0) else {
        return;
    };
    let cam = r.camera.1.decode(&bytes);
    let scene = &mut rs.scene;
    let trauma = f(&cam, "trauma", 0.0);
    let held = scene.shake != og_render::CameraShake::default();
    if trauma > 0.0 && !held {
        scene.shake = og_render::CameraShake::trauma(
            trauma,
            scene.tick,
            scene.tick_rate,
            0,
            f(&cam, "shakeOffset", 0.04),
            f(&cam, "shakeRoll", 2.0),
            f(&cam, "shakeFrequency", 15.0),
        );
    }
    let punch = f(&cam, "punch", 0.0);
    if punch != 0.0 {
        let k = (1.0 + punch).max(0.1);
        scene.camera.projection = match scene.camera.projection {
            og_render::Projection::Orthographic { height } => {
                og_render::Projection::Orthographic { height: height / k }
            }
            og_render::Projection::Perspective { fov_y_degrees } => {
                let half = (fov_y_degrees.to_radians() / 2.0).tan() / k;
                og_render::Projection::Perspective {
                    fov_y_degrees: (2.0 * half.atan()).to_degrees(),
                }
            }
        };
    }
    let mut shift = v3(&cam["kick"], Vec3::ZERO);
    // The zone og.camera picked (and the one a room transition leaves).
    if let og_render::Projection::Orthographic { height } = scene.camera.projection
        && on(&cam, "zoneActive", false)
    {
        let half = Vec2::new(height * 0.5 * aspect, height * 0.5);
        let c = scene.camera.position.truncate() + shift.truncate();
        let clamp = |v: f32, a: f32, b: f32, hf: f32| {
            if b - a <= 2.0 * hf {
                (a + b) * 0.5
            } else {
                v.clamp(a + hf, b - hf)
            }
        };
        let inside = |lo: Vec2, hi: Vec2| {
            Vec2::new(
                clamp(c.x, lo.x, hi.x, half.x),
                clamp(c.y, lo.y, hi.y, half.y),
            )
        };
        let mut n = inside(
            v2(&cam["zoneMin"], Vec2::ZERO),
            v2(&cam["zoneMax"], Vec2::ZERO),
        );
        let k = f(&cam, "zoneScroll", 1.0).clamp(0.0, 1.0);
        if k < 1.0 && on(&cam, "zoneFromActive", false) {
            let from = inside(
                v2(&cam["zoneFromMin"], Vec2::ZERO),
                v2(&cam["zoneFromMax"], Vec2::ZERO),
            );
            // Eased in and out over the scroll.
            let e = k * k * (3.0 - 2.0 * k);
            n = from + (n - from) * e;
        }
        shift += (n - c).extend(0.0);
    }
    if shift != Vec3::ZERO {
        scene.camera.position += shift;
        scene.camera.target += shift;
    }
}
