//! The binding symbols, host-neutral: each function is the body of the
//! trait method of the same name (wire arguments in, wire answer out).

use crate::author::probe::{self as rprobe, NameOp, with_probe};
use crate::engine::{
    CommandDecl, DECL, Engine, HANDLES, QueryRec, before_structural, declare_game, declare_system,
    engine, queued_structural, running, world_query_views,
};
use crate::host::codec::{Layout, RecordKind};
use crate::world::assets::{self, Payload, Status, index_of};
use crate::{host::wire, parse};
use og_core::reflect::Reflect;
use og_core::{Rng, Time};
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{Value, json};

mod actions;
mod commands;
mod controls;
mod intents;
mod loading;
mod reactive;
mod render;

use crate::{
    bind::modules, engine, present::environment, present::extract, present::feel, present::fx,
    present::look, present::views,
};
pub use actions::*;
pub use commands::*;
pub use controls::*;
pub use intents::*;
pub use loading::*;
pub use reactive::*;
pub use render::*;

mod components;
mod entities;
mod instantiate;
mod query;
mod reflect;
mod resources;
mod systems;

pub use components::*;
pub use entities::*;
pub use instantiate::*;
pub use query::*;
pub use reflect::*;
pub use resources::*;
pub use systems::*;

fn err(e: og_core::Error) -> String {
    e.reason().to_string()
}

fn entity(n: f64) -> Result<Entity, String> {
    if n.fract() != 0.0 || !(1.0..9_007_199_254_740_992.0).contains(&n) {
        return Err(format!("{n} is not an entity"));
    }
    Ok(Entity::from_bits(n as u64))
}

/// Entity `n`, an argument of `call` that must name a live entity (`alive`
/// answers for the running system or the world, or why it cannot), or the refusal each such
/// call words alike. A number below 2^32 has generation 0, which no entity
/// ever has: a bare index, most often read from `Archetype.entities`, so the
/// refusal says so, and the engine log carries it once per system and call
/// (`og run` prints it), so a Result the game ignores still shows.
fn live(
    eng: &Engine,
    call: &str,
    n: f64,
    alive: impl FnOnce(Entity) -> Result<bool, String>,
) -> Result<Entity, String> {
    if let Ok(e) = entity(n)
        && alive(e)?
    {
        return Ok(e);
    }
    let mut reason = format!("{call}: no live entity has the handle {n}");
    if n.fract() == 0.0 && (0.0..4_294_967_296.0).contains(&n) {
        reason.push_str(
            " (Archetype.entities holds (index, generation) pairs; use a.entity(i) for a handle)",
        );
        let frame = running(eng.id);
        let mut st = eng.state.borrow_mut();
        let who = frame
            .and_then(|f| st.systems.get(f.system))
            .map(|s| format!("system '{}', ", s.name))
            .unwrap_or_default();
        if st
            .undeclared_logged
            .insert(format!("{who}{call} bare index"))
        {
            let tick = match frame {
                // SAFETY: the frame is published while its system runs.
                Some(f) => unsafe { f.ctx() }.time().tick,
                None => eng.world().map(|w| w.time().tick).unwrap_or(0),
            };
            st.log
                .push(json!({ "tick": tick, "warning": true, "text": format!("{who}{reason}") }));
        }
    }
    Err(reason)
}

fn entity_number(e: Entity) -> Value {
    json!(e.to_bits())
}

/// A component layout by schema name, or why the token is not usable.
pub(crate) fn component(eng: &Engine, name: &str) -> Result<(Layout, og_ecs::SchemaId), String> {
    crate::engine::lazy::ensure(eng, eng.world()?, name)?;
    let st = eng.state.borrow();
    let layout = st.layouts.get(name).cloned();
    let warning = st
        .warnings
        .iter()
        .find(|w| w.starts_with(&format!("{name} ")))
        .cloned();
    drop(st);
    let layout = match layout {
        Some(l) if l.kind == RecordKind::Component => l,
        Some(_) => return Err(format!("'{name}' is a resource, not a component")),
        None => {
            return Err(warning.unwrap_or_else(|| {
                format!("'{name}' is not a registered component: declare it with defineComponent")
            }));
        }
    };
    let id = eng.world()?.id_by_name(name).map_err(err)?;
    Ok((layout, id))
}

/// A record's layout and id while a system runs (the world is borrowed by
/// the step, so the id comes from the runtime's own table).
pub(crate) fn running_record(
    eng: &Engine,
    name: &str,
    kind: RecordKind,
) -> Result<(Layout, og_ecs::SchemaId), String> {
    let st = eng.state.borrow();
    let layout = st
        .layouts
        .get(name)
        .filter(|l| l.kind == kind)
        .cloned()
        .ok_or_else(|| format!("'{name}' is not a registered {}", kind.as_str()))?;
    let id = st
        .names_by_id
        .iter()
        .position(|n| n == name)
        .map(|i| og_ecs::SchemaId::from_index(i as u32))
        .ok_or_else(|| format!("'{name}' is not registered with this world"))?;
    Ok((layout, id))
}

/// A running system reached for something it did not declare (`access` is
/// `read` or `write`, `name` the record): the reason, worded the same on
/// every path, and one engine log line per system, access and name, so a
/// publish build (where the TypeScript runtime refuses instead of throwing)
/// says it once in the engine log rather than every tick.
pub(crate) fn undeclared(eng: &Engine, system: usize, access: &str, name: &str) -> String {
    let mut st = eng.state.borrow_mut();
    let sys = st
        .systems
        .get(system)
        .map_or_else(|| "?".to_string(), |s| s.name.clone());
    let list = if access == "write" {
        "writes"
    } else {
        "reads or writes"
    };
    let reason =
        format!("system '{sys}' did not declare '{name}' among its {list} ({access} refused)");
    if st
        .undeclared_logged
        .insert(format!("{sys} {access} {name}"))
    {
        let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
        st.log.push(json!({ "tick": tick, "warning": true, "text": format!("{reason}; declare it in the system's {list}") }));
    }
    reason
}
