//! The world's content and its persistence: cooked and uncooked loads and
//! scene spawning ([`assets`], [`uncooked`], [`prefab`]), streaming
//! ([`stream`]), the moving origin ([`origin`]) and saves ([`save`],
//! [`save_migrations`]).

pub(crate) mod assets;
pub(crate) mod origin;
pub(crate) mod prefab;
pub(crate) mod save;
pub(crate) mod save_migrations;
pub(crate) mod stream;
pub(crate) mod uncooked;
