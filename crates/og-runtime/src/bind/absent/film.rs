//! og.film in a build without the `film` feature (or without `story` or
//! `video`, which it needs): every call answers the refusal, and naming
//! og.Film, og.FilmDeck or a film event is refused (crate::engine::lazy),
//! so a world without films keeps the full build's type table and hash.

use crate::host::wire::{FilmChart, FilmInfo, FilmView};
use og_ecs::World;
use serde_json::Value;

/// Why og.film answers nothing here.
pub(crate) const ABSENT: &str = "og.film is not built into this engine: og-runtime was built without its `film` feature (a default feature, which needs `story` and `video`; og-host-web and og-host-node pass it through as `film`)";

pub(crate) const FILM: &str = "og.Film";
pub(crate) const DECK: &str = "og.FilmDeck";
pub(crate) const RECORDS: [&str; 6] = [
    FILM,
    DECK,
    "og.FilmSegment",
    "og.FilmChoice",
    "og.FilmChapter",
    "og.FilmEnded",
];

/// og.film on one engine: nothing here.
#[derive(Default)]
pub(crate) struct ModuleState;

/// A lazy og.film record was named: refused.
pub(crate) fn enable(_world: &mut World, _mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    Err(og_core::Error::new(og_core::ErrorKind::Invalid, ABSENT))
}

/// No `film:` command here.
pub(crate) fn register_commands(_probe: &mut og_probe::Probe, _engine: u32) {}

pub fn og_film_load(_world: u32, _dialogue: String) -> Result<FilmInfo, String> {
    Err(ABSENT.into())
}

pub fn og_film_start(_world: u32, _desc: Option<String>) -> Result<Value, String> {
    Err(ABSENT.into())
}

pub fn og_film_view(_world: u32, _film: f64) -> Option<FilmView> {
    None
}

pub fn og_film_choose(_world: u32, _film: f64, _index: u32) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_film_play(_world: u32, _film: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_film_pause(_world: u32, _film: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_film_stop(_world: u32, _film: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_film_skip(_world: u32, _film: f64) -> Result<(), String> {
    Err(ABSENT.into())
}

pub fn og_film_chart(_world: u32, _dialogue: String) -> Result<FilmChart, String> {
    Err(ABSENT.into())
}
