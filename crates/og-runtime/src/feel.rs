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

use super::codec::{Key, Layout, Rec, Row, Slot};
use super::engine::{Engine, engine, queued_structural, running};
use super::extract::{Hierarchy, RenderState};
use super::parse;
use glam::{Quat, Vec2, Vec3};
use og_anim::{FlipbookSheet, FlipbookSystem};
use og_core::IdMap;
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_tilemap::{MapState, TilemapAsset, Tileset};
use og_tween::{Ease, GroupKind, Leaf, Node, Target, TweenSystem};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

mod draw;
mod tiles;
mod timelines;
mod tweens;
pub(crate) mod vcam;

use super::{art, assets, codec};
use draw::facing_byte;
pub(crate) use draw::*;
pub use tiles::*;
use tiles::{TileBlock, tilemap_system};
pub(crate) use timelines::TimelineRef;
pub use timelines::*;
pub use tweens::*;

pub(crate) const TILEMAP_SYSTEM: &str = "og.tilemap";
pub(crate) const CAMERA_SYSTEM: &str = "og.camera";
pub(crate) const SPRITE_SYSTEM: &str = "og.sprite";

/// Records og-anim and og-tween register themselves (strings, or adopted).
pub(crate) const REGISTERED_BY_FEEL: &[&str] =
    &["og.FlipbookEvent", "og.Skeleton2DEvent", "og.TweenEvent"];

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn err(e: Error) -> String {
    e.reason().to_string()
}

fn f<R: Rec + ?Sized, K: Key + ?Sized>(v: &R, key: &K, default: f32) -> f32 {
    v.num(key).map_or(default, |x| x as f32)
}

fn on<R: Rec + ?Sized, K: Key + ?Sized>(v: &R, key: &K, default: bool) -> bool {
    v.flag(key).unwrap_or(default)
}

/// `v2(&v[key], default)` of a record read in place.
fn v2_of<R: Rec + ?Sized, K: Key + ?Sized>(v: &R, key: &K, default: Vec2) -> Vec2 {
    let lane = |l: &str, d: f32| v.lane(key, l).map_or(d, |x| x as f32);
    Vec2::new(lane("x", default.x), lane("y", default.y))
}

/// Colour field `key` of a record read in place, `default`'s lanes where absent.
fn rgb_of<R: Rec + ?Sized, K: Key + ?Sized>(v: &R, key: &K, default: Vec3) -> Vec3 {
    let lane = |l: &str, d: f32| v.lane(key, l).map_or(d, |x| x as f32);
    Vec3::new(
        lane("r", default.x),
        lane("g", default.y),
        lane("b", default.z),
    )
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
    /// `FloatingTextDesc.facing`: a billboard (`RenderScene::text_facing`).
    pub facing: og_render::SpriteFacing,
}

#[derive(Default)]
pub(crate) struct FeelShared {
    pub camera_ops: Vec<CameraOp>,
    pub ghosts: IdMap<u64, Vec<Ghost>>,
    pub floating: Vec<Floating>,
    /// Texts spawned so far (their drift comes from it).
    pub spawned: u64,
    /// Per entity: the last squash and flash tween, cancelled by the next.
    pub squash: IdMap<u64, u64>,
    pub flash: IdMap<u64, u64>,
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
    pub maps: IdMap<u64, Arc<MapAsset>>,
    pub live: IdMap<u64, LiveMap>,
    /// Maps made or resized at run time (og.tilemap.create, resize): saved
    /// whole in the og.tilemap block's trailer (G2-6).
    pub runtime: BTreeSet<u64>,
    /// The generated physics meshes of merged solid regions this engine
    /// registered (not world state: a collider's shape is saved with it).
    pub generated: BTreeSet<u64>,
    /// og.tilemap.create's last handle number (world state, saved with them).
    pub next_runtime: u64,
    /// A resized loaded map's map as loaded, for a restore of a save made
    /// before the resize.
    pub originals: IdMap<u64, Arc<MapAsset>>,
}

/// The feel modules of one world.
#[derive(Clone)]
pub(crate) struct Feel {
    pub tweens: TweenSystem,
    pub flipbooks: FlipbookSystem,
    pub tiles: Arc<Mutex<TileShared>>,
    pub shared: Arc<Mutex<FeelShared>>,
    /// Per flipbook sheet handle: what its frames are drawn from.
    pub sheet_textures: Arc<Mutex<IdMap<u64, SheetImage>>>,
    /// og.skeleton2d, and per rig handle its texture (an index in RenderState::textures).
    pub skeletons: og_anim::Skeleton2dSystem,
    pub rig_textures: Arc<Mutex<IdMap<u64, usize>>>,
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
    pub sheets: IdMap<u64, (FlipbookSheet, Option<SheetImage>)>,
    pub maps: IdMap<u64, Arc<MapAsset>>,
    /// 2D rigs and their textures, by handle.
    pub rigs: IdMap<u64, (og_anim::Rig2dAsset, usize)>,
    /// Graphs for flipbooks and 2D skeletons (og.anim.graph's `sprite`),
    /// by their og.anim handle.
    pub graphs: IdMap<u64, Arc<og_anim::SpriteGraph>>,
    pub(crate) loaded: BTreeMap<String, Value>,
}

impl FeelAssets {
    pub(crate) fn mint(&mut self) -> u64 {
        self.next += 1;
        self.next | (1u64 << 32)
    }

    // Called by the reloader, which a publish build has not.
    #[cfg_attr(feature = "publish", allow(dead_code))]
    pub(crate) fn replay(&self, feel: &Feel) {
        for (h, (s, t)) in &self.sheets {
            let _ = feel.flipbooks.add_sheet(*h, s.clone());
            if let Some(t) = t {
                lock(&feel.sheet_textures).insert(*h, *t);
            }
        }
        for (h, (rig, t)) in &self.rigs {
            let _ = feel.skeletons.add_rig(*h, rig.clone());
            lock(&feel.rig_textures).insert(*h, *t);
        }
        for (h, g) in &self.graphs {
            let _ = feel.flipbooks.add_graph(*h, g.clone());
            let _ = feel.skeletons.add_graph(*h, g.clone());
        }
        let mut t = lock(&feel.tiles);
        for (h, m) in &self.maps {
            t.maps.insert(*h, m.clone());
        }
    }
}

/// Installs tweens, flipbooks and 2D skeletons (after the game's systems,
/// which set their clips and IK targets, before og.anim).
pub(crate) fn install_front(world: &mut World, components: &[String]) -> Result<Feel, String> {
    let tweens = og_tween::install(world, components).map_err(err)?;
    let flipbooks = og_anim::install_flipbook(world).map_err(err)?;
    let skeletons = og_anim::install_skeleton2d(world).map_err(err)?;
    Ok(Feel {
        tweens,
        flipbooks,
        skeletons,
        rig_textures: Arc::new(Mutex::new(IdMap::new())),
        tiles: Arc::new(Mutex::new(TileShared::default())),
        shared: Arc::new(Mutex::new(FeelShared::default())),
        sheet_textures: Arc::new(Mutex::new(IdMap::new())),
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
    /// og.VirtualCamera, og.CameraBrain and og.CameraCut (lazy), and
    /// og.PostEffects for a shot's focus.
    pub vcams: vcam::Layouts,
    /// The fields extraction reads of each sprite, flipbook, afterimage
    /// and tile layer row, resolved once against their layouts.
    pub slots: Slots,
}

/// Declares the fields of a record that per-frame extraction reads,
/// resolved once per layout ([`Slot`]): each row is then read by offset.
macro_rules! slots {
    ($(#[$m:meta])* $name:ident { $($field:ident: $key:literal),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug)]
        pub(crate) struct $name {
            $($field: Slot,)*
        }

        impl $name {
            fn of(layout: &Layout) -> $name {
                $name {
                    $($field: Slot::of(layout, $key),)*
                }
            }
        }
    };
}

slots!(
    /// og.Sprite's fields.
    SpriteSlots {
        visible: "visible",
        width: "width",
        height: "height",
        flip_x: "flipX",
        flip_y: "flipY",
        texture: "texture",
        array: "array",
        array_layer: "arrayLayer",
        uv_min: "uvMin",
        uv_max: "uvMax",
        anchor: "anchor",
        color: "color",
        opacity: "opacity",
        layer: "layer",
        sort_y: "sortY",
        flash_color: "flashColor",
        flash: "flash",
        glow: "glow",
        blend: "blend",
        tone_mapped: "toneMapped",
        shadow: "shadow",
        shadow_size: "shadowSize",
        shadow_offset: "shadowOffset",
        slice_x: "sliceX",
        slice_y: "sliceY",
        slice_scale: "sliceScale",
        cast_shadow: "castShadow",
        drop_shadow: "dropShadow",
        drop_shadow_height: "dropShadowHeight",
        layers: "layers",
        facing: "facing",
        outline: "outline",
        outline_color: "outlineColor",
        palette: "palette",
        palette_row: "paletteRow",
        material: "material",
    }
);

slots!(
    /// og.Flipbook's fields.
    FlipbookSlots {
        sheet: "sheet",
        clip: "clip",
        frame: "frame",
    }
);

slots!(
    /// og.Afterimage's fields.
    AfterimageSlots {
        count: "count",
        interval: "interval",
        color: "color",
        opacity: "opacity",
        additive: "additive",
    }
);

slots!(
    /// og.Tilemap's fields.
    TilemapSlots {
        visible: "visible",
        map: "map",
        color: "color",
        opacity: "opacity",
        layer: "layer",
        palette: "palette",
        palette_row: "paletteRow",
    }
);

/// [`Records::slots`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct Slots {
    sprite: SpriteSlots,
    flipbook: FlipbookSlots,
    afterimage: AfterimageSlots,
    tilemap: TilemapSlots,
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
        let (tilemap, sprite) = (get("og.Tilemap")?, get("og.Sprite")?);
        let (flipbook, afterimage) = (get("og.Flipbook")?, get("og.Afterimage")?);
        let slots = Slots {
            sprite: SpriteSlots::of(&sprite.1),
            flipbook: FlipbookSlots::of(&flipbook.1),
            afterimage: AfterimageSlots::of(&afterimage.1),
            tilemap: TilemapSlots::of(&tilemap.1),
        };
        Ok(Records {
            transform: get("og.Transform")?,
            collider: get("og.Collider")?,
            tilemap,
            camera: get("og.Camera")?,
            rail: get("og.CameraRail")?,
            sprite,
            flipbook,
            afterimage,
            zone: get("og.CameraZone")?,
            follow: get("og.Follow2D")?,
            vcams: vcam::Layouts::of(layouts),
            slots,
        })
    }
}

/// Installs og.tilemap and og.camera (after og.anim, before the UI's end and physics).
pub(crate) fn install_middle(
    world: &mut World,
    engine_id: u32,
    feel: &Feel,
    r: &Records,
    physics: &og_physics::PhysicsSystem,
    profile: u32,
) -> Result<(), String> {
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let tiles = feel.tiles.clone();
    let rec = r.clone();
    let camera_physics = physics.clone();
    let physics = physics.clone();
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
                ..Default::default()
            },
            move |ctx| tilemap_system(ctx, &tiles, &rec, &physics, profile),
        )
        .map_err(err)?;
    world
        .add_state_block(
            TILEMAP_SYSTEM,
            Box::new(TileBlock(feel.tiles.clone(), engine_id)),
        )
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
                ..Default::default()
            },
            move |ctx| camera_system(ctx, engine_id, &shared, &rec, &camera_physics),
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
                ..Default::default()
            },
            move |ctx| sprite_system(ctx, &shared, &rec),
        )
        .map_err(err)?;
    world
        .add_state_block(SPRITE_SYSTEM, Box::new(SpriteBlock(feel.shared.clone())))
        .map_err(err)?;
    Ok(())
}

pub(crate) fn feel_of(eng: &Engine) -> Result<Feel, String> {
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
    time_payload_of(s.scale, s.hit_stop, s.dt(&t), s.elapsed)
}

/// [`time_payload`]'s text, `{}`'s numbers: the scaled elapsed time
/// formatted with zmij, whose shortest round-trip digits are core's (as
/// JavaScript's `Number` reads them back) where core's `{}` falls back to
/// Dragon4 for most elapsed times, a fifth of a millisecond of every tick
/// of the reference scene, measured on the retired WebGL2 tier
/// (the former test sample
/// The f32 fields keep `{}`: on an exact tie between two shortest
/// candidates zmij rounds to even and core up.
fn time_payload_of(scale: f32, hit_stop: u32, dt: f32, elapsed: f64) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(48);
    let _ = write!(out, "{scale},{hit_stop},{dt},");
    if elapsed.is_finite() {
        out.push_str(zmij::Buffer::new().format_finite(elapsed));
    } else {
        let _ = write!(out, "{elapsed}");
    }
    out
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

pub fn og_time_set_scale(world: u32, scale: f64) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let s = scale as f32;
        with_world(&eng, |c| c.set_time_scale(s), |w| w.set_time_scale(s))
    })
}

pub fn og_time_hit_stop(world: u32, ticks: u32) -> Result<(), String> {
    engine(world).and_then(|eng| with_world(&eng, |c| c.hit_stop(ticks), |w| w.hit_stop(ticks)))
}

pub fn og_time_set_group_scale(world: u32, group: u32, scale: f64) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let s = scale as f32;
        let name = "og.TimeGroups";
        if running(eng.id).is_none() {
            crate::lazy::ensure(&eng, eng.world()?, name)?;
        }
        with_world(
            &eng,
            |c| {
                c.set_group_scale(group, s).map_err(|e| {
                    if e.kind() == og_core::ErrorKind::NotFound {
                        og_core::Error::new(
                            og_core::ErrorKind::NotFound,
                            format!(
                                "{name} is not registered yet: inside a system, name og.TimeGroup or {name} in a system's reads or writes, or call og.time.setGroupScale once in setup"
                            ),
                        )
                    } else {
                        e
                    }
                })
            },
            |w| w.set_group_scale(group, s),
        )
    })
}

pub fn og_time_group_dt(world: u32, entity: f64) -> f64 {
    let e = Entity::from_bits(entity as u64);
    engine(world)
        .and_then(|eng| {
            with_world(
                &eng,
                |c| Ok(c.group_dt(c.time_groups().as_ref(), e, c.scaled_dt())),
                |w| {
                    let dt = w.scaled_dt();
                    Ok(w.time_groups().map_or(dt, |g| dt * w.group_scale(&g, e)))
                },
            )
        })
        .map_or(0.0, f64::from)
}

// ── camera ─────────────────────────────────────────────────────────────────

fn camera_op(world: u32, op: CameraOp) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let feel = feel_of(&eng)?;
        lock(&feel.shared).camera_ops.push(op);
        Ok(())
    })
}

pub fn og_camera_shake(world: u32, trauma: f64) -> Result<(), String> {
    if !(trauma.is_finite() && (0.0..=1.0).contains(&trauma)) {
        return Err(format!("trauma {trauma} is outside 0..1"));
    }
    camera_op(world, CameraOp::Shake(trauma as f32))
}

pub fn og_camera_punch(world: u32, amount: f64) -> Result<(), String> {
    if !(amount.is_finite() && (-0.9..=4.0).contains(&amount)) {
        return Err(format!("punch {amount} is outside -0.9..4"));
    }
    camera_op(world, CameraOp::Punch(amount as f32))
}

pub fn og_camera_kick(world: u32, direction: String, distance: f64) -> Result<(), String> {
    let d: Vec3 = super::wire::arg(Some(&direction), "direction")?;
    if !distance.is_finite() || !d.is_finite() {
        return Err("kick needs a finite direction and distance".into());
    }
    camera_op(
        world,
        CameraOp::Kick(d.normalize_or_zero() * distance as f32),
    )
}

/// Applies the queued ops, decays trauma, punch and kick, moves the
/// camera along a rail, then lets the brain blend it to the live virtual
/// camera (vcam).
fn camera_system(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    shared: &Mutex<FeelShared>,
    r: &Records,
    physics: &og_physics::PhysicsSystem,
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
        // A path handle (og.path) replaces the inline points.
        let path = v["path"].as_f64().map_or(0, |h| h as u64);
        let goal = match super::timeline::path_of(engine_id, path).filter(|_| path != 0) {
            Some(p) => p.nearest(target).position,
            None => {
                let n = (v["points"].as_u64().unwrap_or(2) as usize).clamp(2, 8);
                let pts: Vec<Vec3> = (0..n)
                    .map(|i| v3(&v[format!("p{i}").as_str()], Vec3::ZERO))
                    .collect();
                rail_nearest(&pts, target)
            }
        };
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
    vcam::brain(ctx, engine_id, r, physics, &mut cam, dt)?;
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
pub(crate) fn look_rotation(eye: Vec3, target: Vec3) -> Quat {
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

    /// Shakes, punches and kicks are offsets: nothing in the world.
    fn shift(&mut self, _: &og_ecs::Shift) {}
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
    pub fn at_end(&self) -> bool {
        self.at == self.bytes.len()
    }
    pub fn u8(&mut self) -> og_core::Result<u8> {
        Ok(self.take(1)?[0])
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
    /// The bytes not read yet (a block's trailer), taken.
    pub fn rest(&mut self) -> &'a [u8] {
        let r = &self.bytes[self.at..];
        self.at = self.bytes.len();
        r
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

// ── floating text ──────────────────────────────────────────────────────────

pub fn og_render_floating_text(world: u32, desc: Option<String>) -> Result<(), String> {
    engine(world).and_then(|eng| {
        let d: super::wire::FloatingTextDesc = super::wire::arg(desc.as_deref(), "desc")?;
        let text = d.text;
        let font = match d.font {
            None => {
                let has_ui = eng.render.borrow().ui.as_ref().is_some_and(|u| u.lock().fonts().body.handle.is_some());
                if !has_ui {
                    return Err("desc.font is null and the UI has no body font: load one with og.load.font and pass it".into());
                }
                u32::MAX
            }
            Some(h) => {
                let i = super::assets::index_of(h as f64).ok_or_else(|| format!("{h} is not a font handle"))?;
                if i >= eng.render.borrow().fonts.len() {
                    return Err(format!("font {h} is not alive"));
                }
                i as u32
            }
        };
        let elapsed = with_world(&eng, |c| Ok(c.time_scale().elapsed), |w| Ok(w.time_scale().elapsed))?;
        // Its missing glyphs warned of now, on the CPU, renderer or not.
        let tick = with_world(&eng, |c| Ok(c.time().tick), |w| Ok(w.time().tick))?;
        let missing = {
            let mut rs = eng.render.borrow_mut();
            if font == u32::MAX {
                crate::glyphs::ui_lacking(&mut rs, &text)
            } else {
                crate::glyphs::lacking(&mut rs, font as usize, &text).into_iter().collect()
            }
        };
        crate::glyphs::warn_missing(&eng, tick, missing);
        let crit = d.crit;
        let scale = if crit { d.crit_scale.max(1.0) } else { 1.0 };
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
            position: d.position,
            drift: unit * d.spread.max(0.0),
            start: elapsed,
            size: d.size.max(0.0) * scale,
            color: Vec3::new(d.color.r, d.color.g, d.color.b),
            outline: d.outline.clamp(0.0, 1.0),
            outline_color: Vec3::new(d.outline_color.r, d.outline_color.g, d.outline_color.b),
            rise: d.rise,
            lifetime: d.lifetime.max(0.05),
            fade: d.fade.clamp(0.0, 1.0),
            pop: d.pop.max(0.0) * if crit { 1.5 } else { 1.0 },
            layer: d.layer,
            facing: facing_named(Some(d.facing.as_str())),
        });
        Ok(())
    })
}

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
        // The floating texts' facings, a byte each, only when one faces the
        // camera: a block without is the one engines before
        // FloatingTextDesc.facing wrote and read, and its hash unchanged.
        if sh
            .floating
            .iter()
            .any(|t| t.facing != og_render::SpriteFacing::None)
        {
            out.extend(sh.floating.iter().map(|t| facing_byte(t.facing)));
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
                facing: og_render::SpriteFacing::None,
            });
        }
        for i in 0..2 {
            let mut map = IdMap::new();
            for _ in 0..r.u32()? {
                map.insert(r.u64()?, r.u64()?);
            }
            if i == 0 {
                sh.squash = map;
            } else {
                sh.flash = map;
            }
        }
        if !r.at_end() {
            for t in &mut sh.floating {
                t.facing = match r.u8()? {
                    1 => og_render::SpriteFacing::Camera,
                    2 => og_render::SpriteFacing::CameraY,
                    _ => og_render::SpriteFacing::None,
                };
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

    /// Afterimages and floating texts are at places in the world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut sh = lock(&self.0);
        for list in sh.ghosts.values_mut() {
            for g in list {
                g.position = s.moved(g.position);
            }
        }
        for t in &mut sh.floating {
            t.position = s.moved(t.position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::time_payload_of;

    /// What JavaScript's `Number` reads from each field of a payload (a
    /// correctly rounded parse to f64, as Rust's), by bits; NaN for text
    /// it does not read (`inf`).
    fn numbers(payload: &str) -> Vec<u64> {
        payload
            .split(',')
            .map(|f| f.parse::<f64>().unwrap_or(f64::NAN))
            .map(|x| {
                if x.is_nan() {
                    f64::NAN.to_bits()
                } else {
                    x.to_bits()
                }
            })
            .collect()
    }

    /// The payload's numbers are those of `{}`'s text, so the game's
    /// systems read the same scaled time: an f32 field carries the f32's
    /// own shortest digits (not the f64 it widens to), over the elapsed
    /// times of long runs, random bit patterns and the odd values.
    #[test]
    fn the_time_payload_reads_back_as_the_formatted_numbers() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut cases: Vec<(f32, u32, f32, f64)> = Vec::new();
        // Elapsed as the engine sums it: f32 ticks added to an f64.
        for (scale, dt) in [
            (1.0f32, 1.0f32 / 60.0),
            (0.5, 1.0 / 60.0),
            (0.3, 1.0 / 30.0),
            (1.7, 1.0 / 120.0),
        ] {
            let mut elapsed = 0.0f64;
            for _ in 0..20_000 {
                cases.push((scale, 0, dt * scale, elapsed));
                elapsed += f64::from(dt * scale);
            }
        }
        for _ in 0..50_000 {
            let r = next();
            let (a, b) = (f32::from_bits(r as u32), f32::from_bits((r >> 32) as u32));
            cases.push((a, (r >> 40) as u32, b, f64::from_bits(next())));
        }
        for x in [
            0.0,
            -0.0,
            1e-45,
            3.4e38,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            1e-7,
        ] {
            cases.push((x, u32::MAX, x, f64::from(x)));
        }
        cases.push((1.0, 0, 0.0, f64::MIN_POSITIVE));
        cases.push((1.0, 0, 0.0, f64::MAX));
        for (scale, hit_stop, dt, elapsed) in cases {
            let ours = time_payload_of(scale, hit_stop, dt, elapsed);
            let display = format!("{scale},{hit_stop},{dt},{elapsed}");
            assert_eq!(
                numbers(&ours),
                numbers(&display),
                "{ours} against {display}"
            );
        }
    }
}
