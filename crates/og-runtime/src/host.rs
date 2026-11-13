//! The host boundary: the binding symbols both hosts forward to ([`api`],
//! [`host_impl`]), the wire types and layouts that cross it ([`wire`],
//! [`codec`], [`shared`]), and what only a host can answer ([`platform`],
//! [`ui_cursor`]).

pub(crate) mod api;
pub(crate) mod codec;
pub mod host_impl;
pub(crate) mod platform;
pub mod shared;
pub mod ui_cursor;
pub mod wire;
