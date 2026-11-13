//! The IDL's `destruction` module (its 2D half, og.PixelTerrain) in a
//! build without og-runtime's `destruction` feature: og-physics' destruction is not
//! linked, and this module stands in for `destruction2d.rs`. Every 2D
//! `og.destruction` call answers [`crate::bind::destruction::ABSENT`].
//!
//! As the 3D half: the full module adds its system and state block only
//! once its lazy records are registered, so a world without them is the
//! same in both builds; naming one is refused.

use crate::engine::Mount;
use crate::present::extract::{Hierarchy, RenderState};
use og_ecs::World;

pub(crate) const TERRAIN: &str = "og.PixelTerrain";
pub(crate) const PIECE: &str = "og.PixelPiece";
pub(crate) const DETACHED: &str = "og.PixelDetached";
/// Registered together (crate::engine::lazy's TOGETHER), as the full module's.
pub(crate) const RECORDS: [&str; 3] = [TERRAIN, PIECE, DETACHED];

/// A lazy record of the module was named: refused.
pub(crate) fn enable(world: &mut World, mount: Mount<'_>) -> og_core::Result<()> {
    crate::bind::destruction::enable(world, mount)
}

/// og.destruction2d on one engine: nothing here.
#[derive(Clone, Default)]
pub(crate) struct ModuleState;

pub(crate) fn extract(_world: &World, _rs: &mut RenderState, _h: &mut Hierarchy) {}

pub fn og_destruction_carve2d(
    _world: u32,
    _entity: f64,
    _desc: Option<String>,
) -> Result<(), String> {
    Err(crate::bind::destruction::ABSENT.into())
}

pub fn og_destruction_fill2d(
    _world: u32,
    _entity: f64,
    _desc: Option<String>,
) -> Result<(), String> {
    Err(crate::bind::destruction::ABSENT.into())
}

pub fn og_destruction_explode2d(_world: u32, _desc: Option<String>) -> Result<(), String> {
    Err(crate::bind::destruction::ABSENT.into())
}

pub fn og_destruction_fracture2d(
    _world: u32,
    _entity: f64,
    _desc: Option<String>,
) -> Result<(), String> {
    Err(crate::bind::destruction::ABSENT.into())
}

pub fn og_destruction_cell2d(
    _world: u32,
    _entity: f64,
    _point: String,
) -> Result<serde_json::Value, String> {
    Err(crate::bind::destruction::ABSENT.into())
}
