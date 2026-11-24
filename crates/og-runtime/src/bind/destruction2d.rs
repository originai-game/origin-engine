//! og.destruction's 2D half: pixel terrain (og.PixelTerrain), the pieces
//! cut from it (og.PixelPiece) and og.PixelDetached, over
//! og_physics::destruction::terrain2d.
//!
//! - **The state** is world state: per og.PixelTerrain its cells
//!   (og_physics::destruction::terrain2d::PixelTerrain), its chunks' collider entities
//!   and meshes; per og.PixelPiece its cells; and the changes queued by
//!   carve2d, fill2d, explode2d and fracture2d, in the `og.destruction2d`
//!   state block (snapshots, restores, replays and the world hash carry
//!   it).
//! - **The system** runs after the game's systems and before og.physics:
//!   it fills a terrain the first tick it sees it, applies the queued
//!   changes in order, cuts loose what no longer stands
//!   (`PixelTerrain::detach`) into dynamic pieces, then rebuilds the
//!   collider of every chunk a change touched: its outline
//!   (`PixelTerrain::chunk_chains`), registered directly as native polyline
//!   segments under a generated handle. Chunk boundaries inside solid
//!   ground add no edges. Detached pieces use exact convex rectangle
//!   compounds, preserving their holes and concavities. Unused meshes leave
//!   physics. Integer and f32 `+ - * /` arithmetic in fixed orders only.
//! - **Drawing** ([`extract`]): each terrain and piece is one sprite of a
//!   texture painted from its cells, repainted only when its cells changed
//!   (or a restore replaced them): presentation, never read by the tick.
//!
//! The records are lazy (`crate::engine::lazy`): a world that never names them has
//! neither the system nor the block, so its saves and hash are unchanged.

use crate::engine::{Engine, Mount, engine, queued_structural, running};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{
    self, Carve2dDesc, Explosion2dDesc, Fill2dDesc, Fracture2dDesc, FromJson, record,
};
use crate::present::extract::{Hierarchy, RenderState};
use glam::{Quat, Vec2, Vec3};
use og_core::block_form::{self as bf, BlockForm, Field, Record};
use og_core::hash::splitmix64 as splitmix;
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_physics::destruction::terrain2d::collider::{chunk_polyline, piece_rectangles};
use og_physics::destruction::terrain2d::cut::{Sub, pushed, voronoi};
use og_physics::destruction::terrain2d::state::{Body, Ground, Look, Op, Piece, State};
use og_physics::destruction::terrain2d::{BEDROCK, Cell, PixelTerrain};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

mod api;
mod draw;
mod system;

pub use api::*;
pub(crate) use draw::*;
pub(crate) use system::*;

pub(crate) const SYSTEM: &str = "og.destruction2d";
pub(crate) const TERRAIN: &str = "og.PixelTerrain";
pub(crate) const PIECE: &str = "og.PixelPiece";
pub(crate) const DETACHED: &str = "og.PixelDetached";
/// Registered together (`crate::engine::lazy`'s TOGETHER).
pub(crate) const RECORDS: [&str; 3] = [TERRAIN, PIECE, DETACHED];

const CAUSE_EXPLOSION: u32 = 2;
const CAUSE_FRACTURE: u32 = 5;
const CAUSE_CARVE: u32 = 6;
/// The cell fill2d and a terrain's first fill write.
const GROUND: Cell = 1;

// ── the state ───────────────────────────────────────────────────────────────

/// An engine's state and how many times a restore replaced it (drawing's
/// cache is good for one epoch only; not world state).
#[derive(Default)]
struct Shared {
    state: State,
    epoch: u64,
}

/// og.destruction2d on one engine: the state its world's block saves,
/// shared with its system, its calls and extraction.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(
    Arc<Mutex<Shared>>,
    pub(crate) Arc<std::sync::atomic::AtomicBool>,
);

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── reading records ─────────────────────────────────────────────────────────

/// A row read through its IDL type, with the tick's error type.
fn row<T: FromJson>(l: &Layout, bytes: &[u8]) -> og_core::Result<T> {
    l.read(bytes).map_err(invalid)
}

fn rgb(c: &wire::Color) -> [f32; 3] {
    [c.r, c.g, c.b]
}

fn jv(p: Vec3) -> Value {
    json!({ "x": p.x, "y": p.y, "z": p.z })
}

/// An og.PixelTerrain as the tick reads it.
struct Settings {
    shape: [u32; 3],
    ground: f32,
    bedrock: u32,
    tolerance: f32,
    min_piece: u32,
    max_pieces: u32,
    look: Look,
    body: Body,
}

impl Settings {
    fn of(v: &record::PixelTerrain) -> Settings {
        Settings {
            shape: [
                v.width.clamp(1, 4096),
                v.height.clamp(1, 4096),
                v.chunk.clamp(4, 256),
            ],
            ground: v.ground.clamp(0.0, 1.0),
            bedrock: v.bedrock,
            tolerance: v.tolerance.clamp(0.0, 4.0),
            min_piece: v.min_piece,
            max_pieces: v.max_pieces.max(1),
            look: Look {
                cell: v.cell_size.max(0.001),
                color: rgb(&v.color),
                bedrock: rgb(&v.bedrock_color),
                grain: v.grain.clamp(0.0, 1.0),
                seed: v.seed,
                layer: v.layer,
            },
            body: Body {
                friction: v.friction.max(0.0),
                density: v.density.max(1.0),
                layers: v.layers,
                mask: v.mask,
            },
        }
    }

    fn built(&self) -> [u32; 5] {
        [
            self.look.cell.to_bits(),
            self.tolerance.to_bits(),
            self.body.friction.to_bits(),
            self.body.layers,
            self.body.mask,
        ]
    }

    /// A terrain filled as it starts: bedrock rows, then ground.
    fn filled(&self, origin: [f32; 3]) -> Ground {
        let [w, h, chunk] = self.shape;
        let rows = ((self.ground * h as f32).round() as u32).min(h);
        let rock = self.bedrock.min(h);
        let mut cells = vec![0; (w * h) as usize];
        for y in 0..h {
            let c = if y < rock {
                BEDROCK
            } else if y < rows {
                GROUND
            } else {
                continue;
            };
            cells[(y * w) as usize..((y + 1) * w) as usize].fill(c);
        }
        Ground {
            grid: PixelTerrain::from_cells(w, h, chunk, cells).expect("sized above"),
            shape: self.shape,
            origin,
            built: self.built(),
            look: self.look,
            colliders: BTreeMap::new(),
            revision: 0,
            updates: Vec::new(),
        }
    }
}

// ── the state block ─────────────────────────────────────────────────────────

struct Block(Arc<Mutex<Shared>>);

const MAGIC: &[u8; 8] = b"DESTR2D2";

impl StateBlock for Block {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(MAGIC);
        out.extend(
            bincode::serialize(&lock(&self.0).state)
                .expect("the og.destruction2d state serialises"),
        );
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        let rest = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| invalid("not an og.destruction2d state"))?;
        let s: State = bincode::deserialize(rest).map_err(|e| invalid(e.to_string()))?;
        Ok(Box::new(s))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut g = lock(&self.0);
        g.state = decoded
            .and_then(|d| d.downcast::<State>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        g.epoch += 1;
    }
    /// Per og.PixelTerrain its `terrain` (cells, origin, look, revision)
    /// and per chunk with a collider its `collider` (keyed by chunk: the
    /// collider's entity, its mesh handle, a hash of the mesh); per
    /// og.PixelPiece its `piece` (the terrain it was cut from an entity);
    /// queued carves, fills and fractures as `op` on their entity, keyed
    /// by their place in the queue. Explosions and the mesh set are global.
    fn form(&self, bytes: &[u8]) -> og_core::Result<BlockForm> {
        let s = *self
            .decode(bytes)?
            .downcast::<State>()
            .expect("decoded by the og.destruction2d block");
        let mut out = bf::Records::default();
        for (&e, g) in &s.terrains {
            out.entities.push(
                Record::new(e, "terrain")
                    .field("grid", Field::of(&g.grid))
                    .field("shape", Field::of(&g.shape))
                    .field("origin", Field::of(&g.origin))
                    .field("built", Field::of(&g.built))
                    .field("look", Field::of(&g.look))
                    .field("revision", Field::of(&g.revision)),
            );
            for (&chunk, &(collider, mesh)) in &g.colliders {
                out.entities.push(
                    Record::new(e, "collider")
                        .key(Field::of(&chunk))
                        .field("entity", Field::Entity(collider))
                        .field("mesh", Field::of(&mesh)),
                );
            }
        }
        for (&e, p) in &s.pieces {
            out.entities
                .push(Record::new(e, "piece").members(p, &["terrain", "colliders"]));
        }
        let mut explosions = Vec::new();
        for (i, op) in s.ops.iter().enumerate() {
            match op {
                Op::Carve { entity, .. }
                | Op::Fill { entity, .. }
                | Op::Fracture { entity, .. } => {
                    // `{"Carve": {..}}`: the call's name, then its fields
                    // but the entity (the record's).
                    let mut rec = Record::new(*entity, "op").key(Field::of(&i));
                    if let Ok(Value::Object(tagged)) = serde_json::to_value(op) {
                        for (name, args) in tagged {
                            rec = rec.field("call", Field::of(&name));
                            if let Value::Object(mut args) = args {
                                args.remove("entity");
                                rec = rec.members(&args, &[]);
                            }
                        }
                    }
                    out.entities.push(rec);
                }
                Op::Explode { .. } => explosions.push(json!({ "at": i, "op": op })),
            }
        }
        out.global
            .push(("explosions".into(), Field::of(&explosions)));
        out.global.push(("meshes".into(), Field::of(&s.meshes)));
        Ok(BlockForm::Entities(out))
    }
    /// A terrain's origin is its root entity's og.Transform.position (its
    /// colliders, roots too, move with the shift); queued points are in
    /// the world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut g = lock(&self.0);
        let st = &mut g.state;
        for (&k, t) in st.terrains.iter_mut() {
            t.origin = s.moved_of(k, Vec3::from(t.origin)).to_array();
        }
        let m = |p: &mut [f32; 2]| {
            let v = (Vec3::new(p[0], p[1], 0.0).as_dvec3() + s.d).as_vec3();
            *p = [v.x, v.y];
        };
        for op in &mut st.ops {
            match op {
                Op::Carve { point, .. } | Op::Fill { point, .. } | Op::Explode { point, .. } => {
                    m(point)
                }
                Op::Fracture { .. } => {}
            }
        }
    }
}
