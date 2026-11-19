//! The world's content and its persistence: cooked and uncooked loads and
//! scene spawning ([`assets`], [`uncooked`], [`prefab`]), the http(s)
//! images documents name ([`remote`]), streaming
//! ([`stream`]), the moving origin ([`origin`]), the regions' shape
//! ([`region`]) and saves ([`save`], [`save_migrations`]).

pub(crate) mod assets;
pub(crate) mod data;
pub(crate) mod origin;
pub(crate) mod prefab;
pub(crate) mod region;
pub(crate) mod remote;
pub(crate) mod save;
pub(crate) mod save_migrations;
pub(crate) mod scenes;
pub(crate) mod stream;
pub(crate) mod uncooked;
