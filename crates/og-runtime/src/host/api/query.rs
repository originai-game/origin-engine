//! Queries and archetype views.

use super::*;

pub fn og_world_query(world: u32, desc: Option<String>) -> Result<serde_json::Value, String> {
    if running(world).is_some() {
        return Err("a system receives its query as its first argument; world.query is for commands and callbacks".into());
    }
    (|| {
        let eng = engine(world)?;
        let desc: wire::QueryDesc = wire::arg(desc.as_deref(), "desc")?;
        let names = |v: &[Value]| -> Vec<String> {
            v.iter()
                .filter_map(|n| n.as_str().map(str::to_string))
                .collect()
        };
        let q = QueryDesc {
            read: Vec::new(),
            write: names(&desc.with),
            without: names(&desc.without),
            changed: names(&desc.changed),
            added: names(&desc.added),
            removed: names(&desc.removed),
        };
        for n in q
            .write
            .iter()
            .chain(&q.without)
            .chain(&q.changed)
            .chain(&q.added)
            .chain(&q.removed)
        {
            component(&eng, n)?;
        }
        // Refusals (a filter on an untracked component) now, not as an
        // empty list later.
        eng.world()?.query(&q).map_err(err)?;
        let h = HANDLES.with(|t| {
            t.borrow_mut().world_queries.push((
                eng.id,
                QueryRec::World {
                    desc: q,
                    views: (0, 0),
                },
            ))
        });
        Ok(json!(h))
    })()
}

/// Writes the archetypes of a query handle, from its `from`th, to the
/// shared buffer at word `word`, and answers how many it wrote. The words:
/// how many archetypes the query has, how many follow, their tick and the
/// layout epoch; then per archetype its view handle, its entity count, how
/// many types it has and their schema id indices. `from` 0 makes a world
/// query's views; a later `from` goes on through the views it made.
pub(crate) fn describe_query(query: u32, from: u32, word: usize) -> u32 {
    let range = HANDLES.with(|t| {
        let t = t.borrow();
        if let Some((_, QueryRec::Frame { first, len })) = t.frame_queries.get(query) {
            return Some(Ok((*first, *len)));
        }
        match t.world_queries.get(query) {
            Some((_, QueryRec::World { views, .. })) if from > 0 => Some(Ok(*views)),
            Some((e, QueryRec::World { desc, .. })) => Some(Err((*e, desc.clone()))),
            _ => None,
        }
    });
    let (first, len) = match range {
        None => (0, 0),
        Some(Ok(views)) => views,
        Some(Err((e, desc))) => {
            let made = engine(e)
                .and_then(|eng| world_query_views(&eng, &desc))
                .unwrap_or((0, 0));
            HANDLES.with(|t| {
                if let Some((_, QueryRec::World { views, .. })) =
                    t.borrow_mut().world_queries.get_mut(query)
                {
                    *views = made;
                }
            });
            made
        }
    };
    HANDLES.with(|t| {
        let t = t.borrow();
        crate::host::shared::with(|b| {
            let mut w = crate::host::shared::Words::at(b, word);
            let head = w.index();
            // Retired views (a stale handle) describe as none.
            let live = t.views.get(first).map(|(_, r)| r.tick);
            w.put(if live.is_some() { len } else { 0 });
            w.put(0);
            w.put(live.unwrap_or(0));
            w.put(crate::engine::layout_epoch());
            let mut written = 0;
            for h in (from..len).map(|k| first.wrapping_add(k)) {
                let Some((_, r)) = t.views.get(h) else { break };
                let (start, n) = r.types;
                if !w.fits(3 + n as usize) {
                    break;
                }
                w.put(h);
                w.put(r.count);
                w.put(n);
                for id in &t.view_types[start as usize..(start + n) as usize] {
                    w.put(*id);
                }
                written += 1;
            }
            w.set(head + 1, written);
            written
        })
    })
}

/// `og_engine_query_archetypes`.
pub fn og_engine_query_archetypes(query: u32, from: u32) -> u32 {
    describe_query(query, from, 0)
}

/// `og_engine_component_id`: the schema id index of a component this
/// world has registered, `u32::MAX` for any other name. Asking registers
/// nothing.
pub fn og_engine_component_id(world: u32, component: String) -> u32 {
    engine(world)
        .ok()
        .and_then(|eng| {
            let st = eng.state.borrow();
            st.layouts
                .get(&component)
                .filter(|l| l.kind == RecordKind::Component)?;
            st.names_by_id.iter().position(|n| *n == component)
        })
        .map_or(u32::MAX, |i| i as u32)
}

pub fn og_query_count(query: u32) -> u32 {
    let world = HANDLES.with(|t| {
        let t = t.borrow();
        if let Some((_, QueryRec::Frame { first, len })) = t.frame_queries.get(query) {
            let counts = (0..*len).filter_map(|k| t.views.get(first.wrapping_add(k)));
            return Err(counts.map(|(_, r)| r.count).sum());
        }
        match t.world_queries.get(query) {
            Some((e, QueryRec::World { desc, .. })) => Ok((*e, desc.clone())),
            _ => Err(0),
        }
    });
    match world {
        Err(count) => count,
        Ok((e, desc)) => engine(e)
            .and_then(|eng| Ok(eng.world()?.query(&desc).map_err(err)?.entity_count() as u32))
            .unwrap_or(0),
    }
}

pub(super) fn view_field<T>(
    view: u32,
    f: impl FnOnce(&crate::engine::ViewRecord) -> T,
) -> Option<T> {
    HANDLES.with(|t| t.borrow().views.get(view).map(|(_, r)| f(r)))
}

pub fn og_archetype_view_get_count(view: u32) -> u32 {
    view_field(view, |r| r.count).unwrap_or(0)
}

pub fn og_archetype_view_get_tick(view: u32) -> u32 {
    view_field(view, |r| r.tick).unwrap_or(0)
}

pub fn og_archetype_view_entity(view: u32, index: u32) -> f64 {
    view_field(view, |r| {
        if index >= r.count {
            return 0.0;
        }
        // SAFETY: the record is live.
        unsafe { (*r.entities.add(index as usize)).to_bits() as f64 }
    })
    .unwrap_or(0.0)
}
