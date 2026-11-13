//! Systems and the game: declared before an engine exists.

use super::*;

pub fn og_system_define_system(name: String, desc: String, run: u32) -> u32 {
    let desc = parse(Some(&desc)).unwrap_or(Value::Null);
    declare_system(name, &desc, run)
}

/// `f` of a declared system; none for a handle no system has.
pub(super) fn system_field<T>(
    system: u32,
    f: impl FnOnce(&crate::engine::SystemDecl) -> T,
) -> Option<T> {
    DECL.with(|d| d.borrow().systems.get(system as usize).map(f))
}

pub fn og_system_get_name(system: u32) -> String {
    DECL.with(|d| {
        d.borrow()
            .systems
            .get(system as usize)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    })
}

pub fn og_system_get_reads(system: u32) -> Option<Vec<String>> {
    system_field(system, |s| s.reads.iter().map(|t| t.name.clone()).collect())
}

pub fn og_system_get_writes(system: u32) -> Option<Vec<String>> {
    system_field(system, |s| {
        s.writes.iter().map(|t| t.name.clone()).collect()
    })
}

pub fn og_game_game(desc: Option<String>) -> u32 {
    let desc = parse(desc.as_deref()).unwrap_or(Value::Null);
    declare_game(&desc)
}

pub fn og_game_get_tick(game: u32) -> u32 {
    DECL.with(|d| d.borrow().games.get(game as usize).map_or(0, |g| g.tick))
}

pub fn og_game_get_systems(game: u32) -> Vec<u32> {
    DECL.with(|d| {
        d.borrow()
            .games
            .get(game as usize)
            .map(|g| g.systems.clone())
            .unwrap_or_default()
    })
}
