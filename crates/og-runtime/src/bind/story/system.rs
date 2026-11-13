//! The og.story system: the tick's conversations, shots and voices.

use super::*;

/// The og.story system, mounted on every world before the game's systems.
/// It does nothing until og.Story is registered; then each tick it runs the
/// choice timers, counts the events objectives count, settles barks, starts
/// and stops voices, sends the events the story queued since the last run
/// and writes og.Story's counters.
pub(crate) fn mount(world: &mut World, mount: crate::engine::Mount<'_>) -> Result<(), String> {
    let engine_id = mount.id;
    world
        .add_system(
            SystemDesc::new(SYSTEM).write::<og_audio::AudioSource>(),
            move |ctx| {
                let Ok(story_id) = ctx.id_by_name(STORY) else {
                    return Ok(());
                };
                let Ok(eng) = engine(engine_id) else {
                    return Ok(());
                };
                let time = ctx.time();
                let tick = time.tick as u32;
                let layout = |name: &str| eng.state.borrow().layouts.get(name).cloned();
                // The world as a story reads it.
                let read = |e: Option<u64>, path: &str| -> Option<Val> {
                    if let Some(f) = path.strip_prefix("og.Time.") {
                        return match f {
                            "tick" => Some(Val::Num(time.tick as f64)),
                            "dt" => Some(Val::Num(f64::from(time.dt))),
                            "elapsed" => Some(Val::Num(time.elapsed)),
                            _ => None,
                        };
                    }
                    let st = eng.state.borrow();
                    let (rec, fields) = record_of(&st.layouts, path)?;
                    let l = st.layouts.get(rec)?;
                    let id = ctx.id_by_name(rec).ok()?;
                    let bytes = match e {
                        Some(e) => ctx.get_bytes(Entity::from_bits(e), id).ok()?,
                        None => ctx.resource_bytes(id).ok()?,
                    };
                    walk(&l.decode(&bytes), fields)
                };
                // Events objectives count, as they were sent.
                let counted = with(engine_id, |rt| Story::counted_events(&rt.lib));
                let mut seen_events: Vec<(String, Value)> = Vec::new();
                for name in &counted {
                    let (Ok(id), Some(l)) = (ctx.id_by_name(name), layout(name)) else {
                        continue;
                    };
                    for b in ctx.events_bytes(id).unwrap_or_default() {
                        seen_events.push((name.clone(), l.decode(&b)));
                    }
                }
                let alive = |e: u64| ctx.is_alive(Entity::from_bits(e));
                let (starts, stops) = with(engine_id, |rt| {
                    let Rt {
                        lib, story, voices, ..
                    } = rt;
                    let s = story.get_or_insert_with(fresh);
                    s.tick(lib, tick);
                    for (name, ev) in &seen_events {
                        let with_event =
                            |e: Option<u64>, path: &str| match path.strip_prefix("event.") {
                                Some(f) => walk(ev, f),
                                None => read(e, path),
                            };
                        s.count_event(lib, name, &with_event);
                    }
                    s.run_barks(lib, tick, &read, &alive);
                    voice_plan(s, voices, tick)
                });
                shot(ctx, &eng, engine_id)?;
                // Voices: stop the ones whose line moved on or ended, start
                // the new lines' (each its own entity with an AudioSource).
                for e in stops {
                    let e = Entity::from_bits(e);
                    if ctx.is_alive(e) {
                        ctx.despawn(e)?;
                    }
                }
                let source = ctx.id_by_name("og.AudioSource")?;
                let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
                let mut spawned = Vec::new();
                if let Some(l) = layout("og.AudioSource") {
                    for (key, seq, handle) in starts {
                        let bytes = l
                            .encode(None, Some(&json!({ "sound": handle })))
                            .map_err(bad)?;
                        let e = ctx.spawn_bytes(vec![(source, bytes)])?;
                        spawned.push((key, seq, e.to_bits()));
                    }
                }
                let (events, counters, warnings) = with(engine_id, |rt| {
                    let s = rt.story.get_or_insert_with(fresh);
                    for (key, seq, e) in spawned {
                        started(s, &key, seq, e, tick);
                    }
                    let counters = json!({
                        "lines": s.seq,
                        "conversations": s.convs.values().filter(|c| c.wait != Wait::Ended).count(),
                        "activeQuests": s.quests.iter().filter(|q| q.status == "active").count(),
                    });
                    (
                        std::mem::take(&mut s.events),
                        counters,
                        std::mem::take(&mut s.warnings),
                    )
                });
                for e in &events {
                    let (name, v) = event_json(e);
                    if let Some(l) = layout(name) {
                        ctx.send_bytes(
                            ctx.id_by_name(name)?,
                            &l.encode(None, Some(&v)).map_err(bad)?,
                        )?;
                    }
                }
                if let Some(l) = layout(STORY) {
                    ctx.set_resource_bytes(
                        story_id,
                        &l.encode(None, Some(&counters)).map_err(bad)?,
                    )?;
                }
                log(&eng, warnings);
                Ok(())
            },
        )
        .map_err(|e| e.reason().to_string())
}

/// A line's shot (R10): the newest running conversation's current line's
/// shot is og.CameraBrain.solo while it shows. The solo the brain had when
/// the story took it is kept (Story.camera) and handed back when no line
/// has a shot, unless the game set another meanwhile, which is then the
/// one handed back (and a solo the game sets while the story holds it is
/// replaced at the next line's shot only).
pub(super) fn shot(
    ctx: &mut SystemContext<'_>,
    eng: &crate::engine::Engine,
    engine_id: u32,
) -> og_core::Result<()> {
    if !with(engine_id, |rt| rt.granted.contains(BRAIN)) {
        return Ok(());
    }
    let (Ok(bid), Ok(vid)) = (ctx.id_by_name(BRAIN), ctx.id_by_name(VCAM)) else {
        return Ok(());
    };
    let layouts = {
        let st = eng.state.borrow();
        (
            st.layouts.get(BRAIN).cloned(),
            st.layouts.get(VCAM).cloned(),
        )
    };
    let (Some(bl), Some(vl)) = layouts else {
        return Ok(());
    };
    if !ctx.has_resource_id(bid)? {
        return Ok(());
    }
    let (wanted, held) = with(engine_id, |rt| {
        let s = rt.story.as_ref();
        let wanted = s.and_then(|s| {
            let (id, shot) = s.shot()?;
            Some((shot.to_string(), s.convs.get(&id)?.cast.clone()))
        });
        (wanted, s.and_then(|s| s.camera))
    });
    let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
    let is_vcam = |e: Entity| ctx.is_alive(e) && ctx.get_bytes(e, vid).is_ok();
    // A name: that entity; a cast role: its entity's own virtual camera,
    // else the first (in query order) that looks at it.
    let want = match &wanted {
        None => None,
        Some((shot, cast)) => {
            let found = match cast.get(shot.as_str()) {
                Some(&bits) => {
                    let actor = Entity::from_bits(bits);
                    if is_vcam(actor) {
                        Some(actor)
                    } else {
                        let q = QueryDesc {
                            read: vec![VCAM.into()],
                            ..QueryDesc::default()
                        };
                        let mut hit = None;
                        'all: for a in ctx.query_with(&q)? {
                            for &e in a.entities() {
                                let v: record::VirtualCamera =
                                    vl.read(&ctx.get_bytes(e, vid)?).map_err(bad)?;
                                if v.look_at == bits {
                                    hit = Some(e);
                                    break 'all;
                                }
                            }
                        }
                        hit
                    }
                }
                None => ctx.find(shot).filter(|e| is_vcam(*e)),
            };
            if found.is_none() {
                let first = with(engine_id, |rt| rt.unresolved.insert(shot.clone()));
                if first {
                    log(
                        eng,
                        vec![format!(
                            "a line's shot '{shot}' names no virtual camera: no entity of that name with an og.VirtualCamera, and no cast role whose entity has one or is one's lookAt; the line shows without a shot"
                        )],
                    );
                }
            }
            found.map(Entity::to_bits)
        }
    };
    let bytes = ctx.resource_bytes(bid)?;
    let mut brain = bl.decode(&bytes);
    let cur = record::CameraBrain::from_json(&brain).map_err(bad)?.solo;
    let (camera, write) = match (held, want) {
        (None, None) => return Ok(()),
        (None, Some(e)) => (Some((cur, e)), Some(e)),
        (Some((found, wrote)), Some(e)) => {
            let found = if cur == wrote { found } else { cur };
            (Some((found, e)), (cur != e).then_some(e))
        }
        (Some((found, wrote)), None) => (None, (cur == wrote).then_some(found)),
    };
    if let Some(solo) = write {
        brain["solo"] = json!(solo);
        brain["soloBlend"] = json!(-1.0);
        let next = bl.encode(Some(&bytes), Some(&brain)).map_err(bad)?;
        ctx.set_resource_bytes(bid, &next)?;
    }
    with(engine_id, |rt| {
        if let Some(s) = rt.story.as_mut() {
            s.camera = camera;
        }
    });
    Ok(())
}

/// The voice each speaking line or bark should have now: `c<conversation>`
/// for a conversation's current line, `b<entity>` for a bark, with its seq,
/// clip and length.
pub(super) fn voice_wants(s: &Story) -> BTreeMap<String, (u32, String, u32, Option<u32>)> {
    let mut want = BTreeMap::new();
    for (id, c) in &s.convs {
        if c.wait == Wait::Ended {
            continue;
        }
        if let Some(l) = c.history.last().filter(|l| !l.voice.is_empty()) {
            want.insert(
                format!("c{id}"),
                (l.seq, l.voice.clone(), l.voice_ticks, l.voice_start),
            );
        }
    }
    for (e, b) in &s.barks {
        if !b.voice.is_empty() {
            want.insert(
                format!("b{e}"),
                (b.seq, b.voice.clone(), b.voice_ticks, b.voice_start),
            );
        }
    }
    want
}

/// Which voice entities to despawn (their line moved on, ended or played
/// out) and which voices to start: (key, seq, sound handle).
pub(super) fn voice_plan(
    s: &mut Story,
    handles: &BTreeMap<String, u64>,
    tick: u32,
) -> (Vec<(String, u32, u64)>, Vec<u64>) {
    let want = voice_wants(s);
    let mut stops = Vec::new();
    s.voices.retain(|k, (seq, e)| {
        let keep = want
            .get(k)
            .is_some_and(|w| w.0 == *seq && w.3.is_some_and(|t| t.saturating_add(w.2) > tick));
        if !keep {
            stops.push(*e);
        }
        keep
    });
    let starts = want
        .into_iter()
        .filter(|(k, w)| w.3.is_none() && w.2 > 0 && !s.voices.contains_key(k))
        .filter_map(|(k, (seq, clip, _, _))| handles.get(&clip).map(|h| (k, seq, *h)))
        .collect();
    (starts, stops)
}

/// A voice started at `tick` on entity `e`: the line or bark it is for
/// times its end from now.
pub(super) fn started(s: &mut Story, key: &str, seq: u32, e: u64, tick: u32) {
    s.voices.insert(key.to_string(), (seq, e));
    let rest = &key[1..];
    if key.starts_with('c') {
        if let Some(l) = rest
            .parse::<u32>()
            .ok()
            .and_then(|id| s.convs.get_mut(&id))
            .and_then(|c| c.history.last_mut())
            .filter(|l| l.seq == seq)
        {
            l.voice_start = Some(tick);
        }
    } else if let Some(b) = rest
        .parse::<u64>()
        .ok()
        .and_then(|e| s.barks.get_mut(&e))
        .filter(|b| b.seq == seq)
    {
        b.voice_start = Some(tick);
    }
}
