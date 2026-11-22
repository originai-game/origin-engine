//! The engine wasm modules content needs, fetched as it is prepared
//! (docs/web-export.md §Modules): the cook records them in the manifest
//! (`modules` lines, og-cook's `pipeline/needs.rs`), and the host is asked
//! for them when a bundle opens and when a scene loads (og.load.scene,
//! og.scene.preload and og.scene.change, a sub-scene, a timeline's
//! pre-roll), so they arrive before the first call that would load one
//! where it stands. Presentation only: a module that arrives late is
//! loaded at its first call with the same results (docs/web-export.md
//! §Modules), so nothing here reaches a tick.

use crate::engine::{Engine, hooks};
use og_bundle::runtime::Bundle;

/// Asks the host for `modules` (none: nothing).
pub(crate) fn fetch(modules: &[String]) {
    if modules.is_empty() {
        return;
    }
    if let Some(h) = hooks() {
        h.prefetch_modules(modules);
    }
}

/// Asks for the modules opening `bundle` needs (its boot scene's).
pub(crate) fn bundle(bundle: &Bundle) {
    fetch(&bundle.manifest().modules);
}

/// Asks for the modules asset `id` needs, as the bundle it loads from
/// (`owner` first, as [`crate::world::assets::Assets::source`])
/// records them; an asset no open bundle holds needs none it can name.
pub(crate) fn asset(eng: &Engine, id: &str, owner: Option<usize>) {
    fetch(&of(eng, id, owner));
}

/// The modules asset `id` needs, as the bundle it loads from records
/// them; none when no open bundle holds it.
pub(crate) fn of(eng: &Engine, id: &str, owner: Option<usize>) -> Vec<String> {
    let a = eng.assets.borrow();
    let Ok(i) = a.source(id, owner) else {
        return Vec::new();
    };
    match &a.entries[i].payload {
        crate::world::assets::Payload::Bundle { bundle, .. } => bundle
            .asset(id)
            .map(|asset| asset.modules.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}
