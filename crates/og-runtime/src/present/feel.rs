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

use crate::engine::{Engine, engine, queued_structural, running};
use crate::host::codec::{ColRow, Key, Layout, Rec, Slot};
use crate::host::wire::{FromJson, record};
use crate::parse;
use crate::present::extract::{Hierarchy, RenderState, column_rows};
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

use crate::{host::codec, present::art, world::assets};
use draw::facing_byte;
pub(crate) use draw::*;
pub use tiles::*;
use tiles::{TileBlock, tilemap_system};
pub(crate) use timelines::TimelineRef;
pub use timelines::*;
pub use tweens::*;

mod camera;
mod floating_text;
mod install;
mod sprites;
mod sway;
mod time;

pub use camera::*;
pub use floating_text::*;
pub(crate) use install::*;
use sprites::*;
pub(crate) use sway::SwayTexture;
pub use time::*;

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

/// A row the typed reader refused, as a system's error.
fn invalid(e: String) -> Error {
    Error::new(ErrorKind::Invalid, e)
}

fn f<R: Rec + ?Sized, K: Key + ?Sized>(v: &R, key: &K, default: f32) -> f32 {
    v.num(key).map_or(default, |x| x as f32)
}

fn on<R: Rec + ?Sized, K: Key + ?Sized>(v: &R, key: &K, default: bool) -> bool {
    v.flag(key).unwrap_or(default)
}

/// Vector field `key` of a record read in place, `default`'s lanes where absent.
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

fn v3(v: &Value, default: Vec3) -> Vec3 {
    Vec3::new(
        f(v, "x", default.x),
        f(v, "y", default.y),
        f(v, "z", default.z),
    )
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
    /// Its font's family (a CSS family list; `sans-serif` the UI's body).
    pub family: String,
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
        sway: "sway",
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
        interval: "intervalTicks",
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

pub(crate) fn feel_of(eng: &Engine) -> Result<Feel, String> {
    eng.state
        .borrow()
        .feel
        .clone()
        .ok_or_else(|| "this engine has no feel modules".to_string())
}

// ── time ───────────────────────────────────────────────────────────────────

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

// ── camera ─────────────────────────────────────────────────────────────────

/// The feel state blocks read with og-core's bounds-checked reader.
pub(crate) use og_core::bytes::ByteReader as Reader;

fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b);
}

// ── the og.sprite system ───────────────────────────────────────────────────
