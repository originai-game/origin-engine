//! Streaming admission and prioritized pack and sound reads.

use super::*;

/// At every tick boundary: what is wanted, what is admitted, what loads and
/// unloads.
pub(crate) fn update(eng: &Engine) {
    open(eng);
    if eng.assets.borrow().stream.groups.is_empty() {
        return;
    }
    let tick = tick_of(eng);
    retry_retained(eng);
    let cam = camera(eng);
    // Each volume group's box, in its entity's space (the camera inside it).
    let volumes: BTreeMap<String, bool> = {
        let a = eng.assets.borrow();
        let names: Vec<String> = a
            .stream
            .groups
            .iter()
            .filter_map(|g| match &g.desc.trigger {
                StreamTrigger::Volume(e) => Some(e.clone()),
                _ => None,
            })
            .collect();
        drop(a);
        let mut out = BTreeMap::new();
        if !names.is_empty()
            && let (Some(c), Ok(world)) = (cam, eng.world())
        {
            let mut h = super::extract::Hierarchy::of(world);
            for n in names {
                let bits = eng.state.borrow().names.get(&n).copied();
                let inside = bits.is_some_and(|b| {
                    let local = h.world_of(b).inverse().transform_point3(c);
                    local.abs().max_element() <= 0.5
                });
                out.insert(n, inside);
            }
        }
        out
    };
    let mut unload = Vec::new();
    {
        let mut a = eng.assets.borrow_mut();
        let s = &mut a.stream;
        for g in &mut s.groups {
            let trigger = match &g.desc.trigger {
                // WIP (G3-3): cell groups load with their cells' anchors.
                StreamTrigger::Manual | StreamTrigger::Cell => false,
                StreamTrigger::Distance {
                    anchor,
                    radius,
                    unload,
                } => {
                    let d = cam.map(|c| c.distance(glam::Vec3::from(*anchor)));
                    g.near = match d {
                        Some(d) if d <= *radius => true,
                        Some(d) if d > *unload => false,
                        Some(_) => g.near,
                        None => false,
                    };
                    g.near
                }
                StreamTrigger::Volume(e) => volumes.get(e).copied().unwrap_or(false),
            };
            g.wanted = g.requested || trigger;
        }
        // Unwanted groups go; a failed one is tried again once wanted again.
        for i in 0..s.groups.len() {
            if !s.groups[i].wanted && s.groups[i].state != State::Unloaded {
                if matches!(s.groups[i].state, State::Loading | State::Resident) {
                    unload.push(s.groups[i].desc.name.clone());
                }
                set_state(s, eng.id, i, State::Unloaded, tick, "");
            }
        }
    }
    // They let go before admission, so what they keep for the frame counts.
    for name in std::mem::take(&mut unload) {
        downgrade(eng, &name);
    }
    {
        let mut a = eng.assets.borrow_mut();
        let s = &mut a.stream;
        // Admission: the wanted groups by priority (then name) within the budget.
        let budget = s.budget();
        let mut order: Vec<usize> = (0..s.groups.len())
            .filter(|&i| s.groups[i].wanted)
            .collect();
        og_core::sort::sort_by(&mut order, |&x, &y| {
            let (gx, gy) = (&s.groups[x], &s.groups[y]);
            gy.desc
                .priority
                .cmp(&gx.desc.priority)
                .then(gx.desc.name.cmp(&gy.desc.name))
        });
        let admitted =
            |s: &Streams, i: usize| matches!(s.groups[i].state, State::Loading | State::Resident);
        // A lowered budget evicts the lowest admitted groups first.
        for &i in order.iter().rev() {
            if s.used() <= budget {
                break;
            }
            if admitted(s, i) {
                unload.push(s.groups[i].desc.name.clone());
                set_state(
                    s,
                    eng.id,
                    i,
                    State::Queued,
                    tick,
                    "evicted to keep within the budget",
                );
            }
        }
        for (k, &i) in order.iter().enumerate() {
            if admitted(s, i) {
                continue;
            }
            // A pack that could not be read is not read again while wanted.
            if s.groups[i].state == State::Failed && !s.groups[i].reason.starts_with("needs ") {
                continue;
            }
            let need = s.groups[i].desc.memory;
            if need > budget {
                let why = format!("needs {need} B, more than the budget of {budget} B");
                set_state(s, eng.id, i, State::Failed, tick, &why);
                continue;
            }
            // Room is made from the lowest-priority admitted groups below it.
            let mut free = budget - s.used().min(budget);
            let mut evict = Vec::new();
            for &j in order[k + 1..].iter().rev() {
                if free >= need {
                    break;
                }
                if admitted(s, j) && s.groups[j].desc.priority < s.groups[i].desc.priority {
                    free += s.groups[j].desc.memory;
                    evict.push(j);
                }
            }
            if free < need {
                set_state(
                    s,
                    eng.id,
                    i,
                    State::Queued,
                    tick,
                    "waiting for room in the budget",
                );
                continue;
            }
            for j in evict {
                unload.push(s.groups[j].desc.name.clone());
                let why = format!("evicted for '{}'", s.groups[i].desc.name);
                set_state(s, eng.id, j, State::Queued, tick, &why);
            }
            set_state(s, eng.id, i, State::Loading, tick, "");
            let used = s.used();
            s.peak = s.peak.max(used);
        }
    }
    for name in unload {
        downgrade(eng, &name);
    }
    // Loading groups read their packs (after the sound chunks a playhead
    // needs); one group is put in place per update.
    fetch(eng);
    let ready: Option<String> = {
        let a = eng.assets.borrow();
        a.stream
            .groups
            .iter()
            .find(|g| g.state == State::Loading && !g.files.is_empty())
            .map(|g| g.desc.name.clone())
    };
    if let Some(name) = ready {
        match upgrade(eng, &name) {
            Ok(true) => {
                let mut a = eng.assets.borrow_mut();
                let s = &mut a.stream;
                if let Some(i) = s.find(&name) {
                    s.groups[i].loads += 1;
                    set_state(s, eng.id, i, State::Resident, tick, "");
                }
            }
            // A texture still being transcoded: the next update finishes it.
            Ok(false) => {}
            Err(e) => {
                downgrade(eng, &name);
                let mut a = eng.assets.borrow_mut();
                let s = &mut a.stream;
                if let Some(i) = s.find(&name) {
                    s.groups[i].files.clear();
                    set_state(s, eng.id, i, State::Failed, tick, &e);
                }
            }
        }
    }
    refresh(eng);
}

/// Upgrades the assets loaded after their group became resident.
pub(crate) fn refresh(eng: &Engine) {
    let names: Vec<String> = {
        let a = eng.assets.borrow();
        a.stream
            .slots
            .iter()
            .filter(|(g, slots)| a.stream.resident(g) && slots.iter().any(|(_, up)| !up))
            .map(|(g, _)| g.clone())
            .collect()
    };
    for name in names {
        let _ = upgrade(eng, &name);
    }
}

/// The chunks the playheads want that are not in the bank (dropping the
/// others), each with how soon it is needed.
fn wanted_chunks(eng: &Engine) -> Vec<Wanted> {
    let Some(link) = og_audio::link(eng.id) else {
        return Vec::new();
    };
    if !eng.assets.borrow().sounds.iter().any(|s| s.chunked()) {
        return Vec::new();
    }
    let Ok(world) = eng.world() else {
        return Vec::new();
    };
    let bank = link.bank();
    let distances = og_audio::chunk_distances(world, bank);
    let sets: IdMap<u64, _> = distances
        .iter()
        .map(|(h, chunks)| (*h, og_core::sort::set(chunks.keys().copied())))
        .collect();
    let a = eng.assets.borrow();
    let mut out = Vec::new();
    for (handle, id, n) in bank.keep_chunks(&sets) {
        let chunk = og_bundle::runtime::chunk_id(&id, n);
        let Ok((bundle, dir)) = a.bundle_for(&chunk) else {
            continue;
        };
        let Ok(o) = bundle.output(&chunk) else {
            continue;
        };
        let seconds = distances
            .get(&handle)
            .and_then(|c| c.get(&n))
            .copied()
            .unwrap_or(f64::INFINITY);
        out.push(Wanted {
            path: format!("{dir}/{}", o.file),
            need: Need::Chunk {
                handle,
                id,
                n,
                seconds,
            },
        });
    }
    out
}

/// The streaming reads, in order: the sound chunks a playhead needs
/// soonest first, then the loading groups' packs by priority and name; at
/// most [`MAX_FETCHES`] in flight (a read the host answers `Pending` holds
/// its slot until it lands). A chunk no playhead wants any more is still
/// asked after until it lands (then dropped), so its slot comes back; a
/// pack whose group let go of it frees its slot at once (the host drops it
/// when it lands).
pub(crate) fn fetch(eng: &Engine) {
    let mut wants = wanted_chunks(eng);
    og_core::sort::sort_by(&mut wants, |a, b| match (&a.need, &b.need) {
        (
            Need::Chunk {
                id: ia,
                n: na,
                seconds: sa,
                ..
            },
            Need::Chunk {
                id: ib,
                n: nb,
                seconds: sb,
                ..
            },
        ) => sa.total_cmp(sb).then(ia.cmp(ib)).then(na.cmp(nb)),
        _ => std::cmp::Ordering::Equal,
    });
    {
        let a = eng.assets.borrow();
        let mut packs: Vec<&Group> = a
            .stream
            .groups
            .iter()
            .filter(|g| g.state == State::Loading && g.files.is_empty())
            .collect();
        og_core::sort::sort_by(&mut packs, |x, y| {
            y.desc
                .priority
                .cmp(&x.desc.priority)
                .then(x.desc.name.cmp(&y.desc.name))
        });
        wants.extend(packs.into_iter().map(|g| Wanted {
            path: format!("{}/{}", g.dir, g.desc.file),
            need: Need::Pack {
                group: g.desc.name.clone(),
                priority: g.desc.priority,
            },
        }));
    }
    let read = |path: &str| match hooks() {
        Some(h) => h.read_asset(path),
        None => super::assets::Fetch::Failed("no host is installed to read assets".into()),
    };
    // In flight, no longer wanted: asked after so the slot comes back.
    let stale: Vec<String> = {
        let a = eng.assets.borrow();
        a.stream
            .fetch
            .in_flight
            .keys()
            .filter(|p| !wants.iter().any(|w| &w.path == *p))
            .cloned()
            .collect()
    };
    for path in stale {
        if !matches!(read(&path), super::assets::Fetch::Pending) {
            let f = &mut eng.assets.borrow_mut().stream.fetch;
            f.in_flight.remove(&path);
            f.finished += 1;
        }
    }
    let mut queued = Vec::new();
    for w in wants {
        {
            let a = eng.assets.borrow();
            let f = &a.stream.fetch;
            if !f.in_flight.contains_key(&w.path) && f.in_flight.len() >= MAX_FETCHES {
                drop(a);
                queued.push(w);
                continue;
            }
        }
        let result = read(&w.path);
        {
            let mut a = eng.assets.borrow_mut();
            let f = &mut a.stream.fetch;
            let flying = f.in_flight.contains_key(&w.path);
            if !flying {
                f.started += 1;
            }
            if matches!(result, super::assets::Fetch::Pending) {
                f.in_flight.insert(w.path.clone(), w.need.clone());
                f.peak = f.peak.max(f.in_flight.len());
                continue;
            }
            f.in_flight.remove(&w.path);
            f.finished += 1;
        }
        let bytes = match result {
            super::assets::Fetch::Ready(b) => Ok(b),
            super::assets::Fetch::Failed(e) => Err(e),
            super::assets::Fetch::Pending => unreachable!("held in flight above"),
        };
        deliver(eng, &w, bytes);
    }
    eng.assets.borrow_mut().stream.fetch.queued = queued;
}

/// A read landed: a chunk into the bank, a pack into its loading group (a
/// failure fails the group and lets go of what the host put).
fn deliver(eng: &Engine, w: &Wanted, bytes: Result<Vec<u8>, String>) {
    let path = &w.path;
    match &w.need {
        Need::Chunk { handle, id, n, .. } => {
            let bytes = match bytes {
                Ok(b) => b,
                Err(e) => return warn_once(eng, format!("{path}: {e}")),
            };
            let chunk = og_bundle::runtime::chunk_id(id, *n);
            let decoded = {
                let a = eng.assets.borrow();
                let Ok((bundle, _)) = a.bundle_for(&chunk) else {
                    return;
                };
                bundle.sound_chunk(&chunk, super::assets::aligned(&bytes).bytes())
            };
            match decoded {
                Ok((k, qoa)) if k == *n => {
                    if let Some(link) = og_audio::link(eng.id)
                        && let Err(e) = link.bank().put_chunk(*handle, *n, &qoa)
                    {
                        warn_once(eng, e.reason().to_string());
                    }
                }
                Ok((k, _)) => warn_once(eng, format!("{path} holds chunk {k}, not {n}")),
                Err(e) => warn_once(eng, e.to_string()),
            }
        }
        Need::Pack { group, .. } => {
            let tick = tick_of(eng);
            {
                let mut a = eng.assets.borrow_mut();
                let s = &mut a.stream;
                let Some(i) = s.find(group) else { return };
                // Let go of while it was fetched: the host dropped it.
                if s.groups[i].state != State::Loading {
                    return;
                }
                match bytes.and_then(|b| og_bundle::runtime::unpack(&b).map_err(|e| e.to_string()))
                {
                    Ok(files) => {
                        s.groups[i].files = files.into_iter().collect();
                        return;
                    }
                    Err(e) => set_state(s, eng.id, i, State::Failed, tick, &format!("{path}: {e}")),
                }
            }
            // Textures the host put before the pack failed go with it.
            release(eng, group);
        }
    }
}

/// `dump_stream`'s `fetch`: the slots, what is in flight and what waits.
pub(super) fn fetch_json(s: &Streams) -> Value {
    let f = &s.fetch;
    json!({
        "max": MAX_FETCHES,
        "inFlight": f.in_flight.iter().map(|(path, need)| Wanted { path: path.clone(), need: need.clone() }.json()).collect::<Vec<_>>(),
        "queued": f.queued.iter().map(Wanted::json).collect::<Vec<_>>(),
        "started": f.started as f64,
        "finished": f.finished as f64,
        "peak": f.peak,
    })
}

fn warn_once(eng: &Engine, text: String) {
    let mut st = eng.state.borrow_mut();
    if st.ui_warned.insert(text.clone()) {
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": text }));
    }
}
