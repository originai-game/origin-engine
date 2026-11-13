//! The IDL's `destruction` module (its 3D half) in a build without
//! og-runtime's `destruction` feature: og-physics' destruction is not linked, and this
//! module stands in for `destruction.rs`. Every `og.destruction` call that
//! answers a Result answers [`ABSENT`]; `capabilities` answers no visual
//! debris.
//!
//! The world is the full build's while the game never names a destructible:
//! the full module adds its system and its state block only once every one
//! of its lazy records is registered (super::lazy), so a world without them
//! has neither in either build. The records stay lazy in IDL order; naming
//! one is refused with [`ABSENT`], so a save holding them is refused, never
//! restored into props that cannot break. A model's prefractured chunks
//! (`<model>#fracture`) are not read.

use super::engine::{Engine, Mount};
use og_ecs::World;

/// Why destruction answers nothing here.
pub(crate) const ABSENT: &str = "og.destruction is not built into this engine: og-runtime was built without its `destruction` feature (a default feature; og-host-web and og-host-node pass it through as `destruction`)";

pub(crate) const DESTRUCTIBLE: &str = "og.Destructible";
pub(crate) const CHUNK: &str = "og.Chunk";
pub(crate) const FRACTURED: &str = "og.Fractured";
pub(crate) const RESOURCE: &str = "og.Destruction";
/// Registered together (super::lazy's TOGETHER), as the full module's.
pub(crate) const RECORDS: [&str; 4] = [DESTRUCTIBLE, CHUNK, FRACTURED, RESOURCE];

/// A lazy record of the module was named: refused.
pub(crate) fn enable(_world: &mut World, _mount: Mount<'_>) -> og_core::Result<()> {
    Err(og_core::Error::new(og_core::ErrorKind::Invalid, ABSENT))
}

/// The physics the 2D half registers its meshes with (only ever asked by
/// destruction2d, which this feature leaves out with it).
pub(crate) fn set_physics(_m: &ModuleState, _physics: &og_physics::PhysicsSystem) {}

/// og.destruction on one engine: nothing here.
#[derive(Clone, Default)]
pub(crate) struct ModuleState;

/// A model's prefractured chunks: not read here.
pub(crate) fn load_asset(_eng: &Engine, _id: &str, _index: usize) -> Result<(), String> {
    Ok(())
}

/// A fractured prop as extraction draws it (the full module's fields):
/// none is ever made here.
pub(crate) struct Drawn {
    pub entity: u64,
    pub chunks: Vec<(u64, u32, glam::Vec3, u32)>,
    pub debris: Vec<DrawnDebris>,
    pub crumbs: Vec<DrawnCrumb>,
    pub mortar: u32,
}

pub(crate) struct DrawnCrumb {
    pub id: u64,
    pub node: u32,
    pub tick: u64,
    pub point: glam::Vec3,
    pub normal: glam::Vec3,
    pub half: f32,
    pub velocity: glam::Vec3,
}

pub(crate) struct DrawnDebris {
    pub id: u64,
    pub node: u32,
    pub centre: glam::Vec3,
    pub tick: u64,
    pub pose: (glam::Vec3, glam::Quat, glam::Vec3),
    pub velocity: glam::Vec3,
    pub angular: glam::Vec3,
    pub points: Vec<glam::Vec3>,
}

/// No prop is fractured here.
pub(crate) fn drawn(_engine: u32) -> Vec<Drawn> {
    Vec::new()
}

pub fn og_destruction_damage(
    _world: u32,
    _entity: f64,
    _desc: Option<String>,
) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_destruction_explode(_world: u32, _desc: Option<String>) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_destruction_fracture(_world: u32, _entity: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_destruction_info(
    _world: u32,
    _entity: f64,
) -> Result<crate::wire::DestructibleInfo, String> {
    Err(ABSENT.into())
}

pub fn og_destruction_material(
    _world: u32,
    _kind: u32,
) -> Result<crate::wire::DestructibleMaterial, String> {
    Err(ABSENT.into())
}

/// No visual debris: nothing breaks here.
pub fn og_destruction_capabilities(_world: u32) -> crate::wire::DestructionCapabilities {
    crate::wire::DestructionCapabilities {
        visual_debris: false,
        max_visual_debris: 0,
        visual_debris_contacts: "world".into(),
    }
}
