//! The IDL's `nav` module in a build without og-runtime's `nav` feature:
//! og-nav (and og-bundle's navmesh loaders) are not linked, and this module
//! stands in for `nav.rs`. Every `og.nav` call that answers a Result
//! answers [`ABSENT`], so a cooked navmesh loaded here is refused by name,
//! never dropped; the others answer what the full module answers with no
//! grid, navmesh or flow field (an outside cell, a zero direction, null,
//! false).
//!
//! The world is the full build's while the game never uses og.nav: the
//! og.nav system and its state block are mounted idle (super::e6_idle),
//! and og.NavFilter, og.Crowd and og.NavLinkReached stay lazy records whose
//! registration is refused with [`ABSENT`].

use og_ecs::{Entity, World};
use serde_json::Value;

/// Why navigation answers nothing here.
pub(crate) const ABSENT: &str = "og.nav is not built into this engine: og-runtime was built without its `nav` feature (a default feature; og-host-web and og-host-node pass it through as `nav`)";

/// `dump_nav`: no agent plans here.
pub(crate) fn path_json(_engine: u32, _world: &World, _e: Entity) -> Option<Value> {
    None
}

pub fn og_nav_grid(_world: u32, _desc: Option<String>) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_nav_grid_from_tilemap(
    _world: u32,
    _desc: Option<String>,
) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_nav_set_cost(
    _world: u32,
    _grid: f64,
    _x: u32,
    _y: u32,
    _cost: u32,
) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_nav_cell_at(_world: u32, _grid: f64, _point: String) -> crate::wire::NavCell {
    crate::wire::NavCell {
        x: 0,
        y: 0,
        inside: false,
    }
}

pub fn og_nav_find_path(
    _world: u32,
    _desc: Option<String>,
) -> Result<crate::wire::NavPath, String> {
    Err(ABSENT.into())
}

pub fn og_nav_flow_field(_world: u32, _desc: Option<String>) -> Result<Value, String> {
    Err(ABSENT.into())
}

pub fn og_nav_set_flow_targets(_world: u32, _field: f64, _targets: String) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_nav_flow_at(_world: u32, _field: f64, _point: String) -> glam::Vec3 {
    glam::Vec3::ZERO
}

pub fn og_nav_load(
    _world: u32,
    _path: String,
    _profile: Option<String>,
) -> Result<crate::wire::NavMeshInfo, String> {
    Err(ABSENT.into())
}

pub fn og_nav_build(
    _world: u32,
    _desc: Option<String>,
) -> Result<crate::wire::NavMeshInfo, String> {
    Err(ABSENT.into())
}

pub fn og_nav_set_obstacle(
    _world: u32,
    _mesh: f64,
    _id: u32,
    _obstacle: Option<String>,
) -> Result<crate::wire::NavRebuild, String> {
    Err(ABSENT.into())
}

pub fn og_nav_carve(
    _world: u32,
    _mesh: f64,
    _id: u32,
    _shape: Option<String>,
) -> Result<crate::wire::NavRebuild, String> {
    Err(ABSENT.into())
}

pub fn og_nav_set_area(
    _world: u32,
    _mesh: f64,
    _id: u32,
    _desc: Option<String>,
) -> Result<crate::wire::NavRebuild, String> {
    Err(ABSENT.into())
}

pub fn og_nav_set_link(
    _world: u32,
    _mesh: f64,
    _id: u32,
    _desc: Option<String>,
) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_nav_set_link_open(
    _world: u32,
    _mesh: f64,
    _id: u32,
    _open: bool,
) -> Result<serde_json::Value, String> {
    Err(ABSENT.into())
}

pub fn og_nav_finish_link(_world: u32, _entity: f64) -> bool {
    false
}

pub fn og_nav_nearest(
    _world: u32,
    _mesh: f64,
    _point: String,
    _extent: Option<String>,
) -> Option<glam::Vec3> {
    None
}

pub fn og_nav_path_of(_world: u32, _entity: f64) -> String {
    Value::Null.to_string()
}

fn refused() -> og_core::Error {
    og_core::Error::new(og_core::ErrorKind::Invalid, ABSENT)
}

/// og.NavFilter, og.Crowd, og.NavLinkReached: refused, there is no og.nav.
pub(crate) fn enable_filters(_w: &mut World) -> og_core::Result<()> {
    Err(refused())
}

pub(crate) fn enable_crowds(_w: &mut World) -> og_core::Result<()> {
    Err(refused())
}

pub(crate) fn enable_links(_w: &mut World) -> og_core::Result<()> {
    Err(refused())
}
