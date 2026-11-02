//! Timeline playback, snapshot state and scheduled systems.

use super::*;

// ── og.timeline: ops and the playback state ─────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Op {
    Play,
    Pause,
    Seek(f32),
    Stop,
}

/// One og.Timeline's playback, kept by the og.timeline state block.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Run {
    pub(super) asset: u64,
    /// The time in steps of 1 / (rate * STEPS) seconds.
    pub(super) num: i64,
    /// The `time` field last written (another value is a seek).
    written: f32,
    /// `playing` as last written.
    playing: bool,
    /// The next tick advances the time (it was evaluated playing).
    advance: bool,
    /// Paused (a play resumes, without og.TimelineStarted).
    paused: bool,
    /// Evaluated and holding what it owns (camera, voices, caption).
    pub(super) active: bool,
    /// Ended last tick: hands back what it owns this tick.
    release: bool,
    /// The camera it found: position, quaternion, fov.
    pub(super) camera: Option<[f32; 8]>,
    /// Per animation track, the key last applied.
    pub(super) anim: Vec<(u32, u32)>,
    /// Voices it started: (track, key, entity bits).
    pub(super) voices: Vec<(u32, u32, u64)>,
    /// og.PostEffects' depth of field as a camera track's focus found it:
    /// on (1 or 0), focus distance, focus range.
    pub(super) focus: Option<[f32; 3]>,
    /// While a shot track holds og.CameraBrain.solo: the solo it found
    /// (handed back) and the one it wrote, as entity bits.
    pub(super) solo: Option<(u64, u64)>,
}

#[derive(Default)]
pub(super) struct Shared {
    pub(super) runs: IdMap<u64, Run>,
    pub(super) ops: Vec<(u64, Op)>,
}

thread_local! {
    static SHARED: RefCell<IdMap<u32, Arc<Mutex<Shared>>>> = const { RefCell::new(IdMap::new()) };
}

pub(super) fn shared(engine: u32) -> Arc<Mutex<Shared>> {
    SHARED.with(|s| s.borrow_mut().entry(engine).or_default().clone())
}

fn has_timeline(eng: &Engine, e: Entity) -> Result<u64, String> {
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    let id = w
        .id_by_name("og.Timeline")
        .map_err(|e| e.reason().to_string())?;
    let bytes = w
        .get_bytes(e, id)
        .map_err(|_| format!("entity {} has no og.Timeline", e.to_bits()))?;
    let layout = eng
        .state
        .borrow()
        .layouts
        .get("og.Timeline")
        .cloned()
        .ok_or("og.Timeline is not registered")?;
    Ok(bits(&layout.decode(&bytes)["asset"]))
}

fn op(world: u32, entity: f64, op: Op) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let asset = has_timeline(&eng, e)?;
        if matches!(op, Op::Play | Op::Seek(_)) {
            let t = lock(&registry(world)).timeline(asset).ok_or_else(|| {
                format!("its og.Timeline.asset {asset} is not a loaded timeline (og.load.timeline)")
            })?;
            if let Op::Seek(s) = op {
                let d = lock(&t).timeline.duration;
                if !(s.is_finite() && (0.0..=d).contains(&s)) {
                    return Err(format!(
                        "time {s} is outside 0 to the timeline's duration {d} s"
                    ));
                }
            }
        }
        lock(&shared(world)).ops.push((e.to_bits(), op));
        Ok(())
    })())
}

pub fn og_timeline_play(world: u32, entity: f64) -> String {
    op(world, entity, Op::Play)
}

pub fn og_timeline_pause(world: u32, entity: f64) -> String {
    op(world, entity, Op::Pause)
}

pub fn og_timeline_seek(world: u32, entity: f64, time: f64) -> String {
    op(world, entity, Op::Seek(time as f32))
}

pub fn og_timeline_stop(world: u32, entity: f64) -> String {
    op(world, entity, Op::Stop)
}

// ── the state block ──────────────────────────────────────────────────────────

struct Bytes<'a>(&'a [u8]);

impl Bytes<'_> {
    fn take<const N: usize>(&mut self) -> og_core::Result<[u8; N]> {
        let (a, b) = self
            .0
            .split_first_chunk::<N>()
            .ok_or_else(|| invalid("the og.timeline state is cut short"))?;
        self.0 = b;
        Ok(*a)
    }
    fn u32(&mut self) -> og_core::Result<u32> {
        self.take::<4>().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> og_core::Result<u64> {
        self.take::<8>().map(u64::from_le_bytes)
    }
    fn f32(&mut self) -> og_core::Result<f32> {
        self.take::<4>().map(f32::from_le_bytes)
    }
}

fn save(sh: &Shared, out: &mut Vec<u8>) {
    let u32_ = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
    let u64_ = |out: &mut Vec<u8>, v: u64| out.extend_from_slice(&v.to_le_bytes());
    u32_(out, sh.ops.len() as u32);
    for (e, op) in &sh.ops {
        u64_(out, *e);
        let (k, t) = match op {
            Op::Play => (0, 0.0f32),
            Op::Pause => (1, 0.0),
            Op::Seek(t) => (2, *t),
            Op::Stop => (3, 0.0),
        };
        u32_(out, k);
        out.extend_from_slice(&t.to_le_bytes());
    }
    u32_(out, sh.runs.len() as u32);
    for (e, r) in &sh.runs {
        u64_(out, *e);
        u64_(out, r.asset);
        u64_(out, r.num as u64);
        out.extend_from_slice(&r.written.to_le_bytes());
        let flags = u32::from(r.playing)
            | u32::from(r.advance) << 1
            | u32::from(r.paused) << 2
            | u32::from(r.active) << 3
            | u32::from(r.release) << 4
            | u32::from(r.camera.is_some()) << 5
            | u32::from(r.focus.is_some()) << 6
            | u32::from(r.solo.is_some()) << 7;
        u32_(out, flags);
        for v in r.camera.unwrap_or_default() {
            out.extend_from_slice(&v.to_le_bytes());
        }
        // Only when a camera track focused: a run without has the bytes it
        // always had.
        for v in r.focus.iter().flatten() {
            out.extend_from_slice(&v.to_le_bytes());
        }
        // Only while a shot track holds the solo, likewise.
        if let Some((found, wrote)) = r.solo {
            u64_(out, found);
            u64_(out, wrote);
        }
        u32_(out, r.anim.len() as u32);
        for (t, k) in &r.anim {
            u32_(out, *t);
            u32_(out, *k);
        }
        u32_(out, r.voices.len() as u32);
        for (t, k, v) in &r.voices {
            u32_(out, *t);
            u32_(out, *k);
            u64_(out, *v);
        }
    }
}

fn load_state(bytes: &[u8]) -> og_core::Result<Shared> {
    let mut b = Bytes(bytes);
    let mut sh = Shared::default();
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let k = b.u32()?;
        let t = b.f32()?;
        sh.ops.push((
            e,
            match k {
                0 => Op::Play,
                1 => Op::Pause,
                2 => Op::Seek(t),
                3 => Op::Stop,
                _ => return Err(invalid("an unknown og.timeline op")),
            },
        ));
    }
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let mut r = Run {
            asset: b.u64()?,
            num: b.u64()? as i64,
            written: b.f32()?,
            ..Run::default()
        };
        let flags = b.u32()?;
        r.playing = flags & 1 != 0;
        r.advance = flags & 2 != 0;
        r.paused = flags & 4 != 0;
        r.active = flags & 8 != 0;
        r.release = flags & 16 != 0;
        let mut cam = [0.0f32; 8];
        for c in &mut cam {
            *c = b.f32()?;
        }
        r.camera = (flags & 32 != 0).then_some(cam);
        if flags & 64 != 0 {
            r.focus = Some([b.f32()?, b.f32()?, b.f32()?]);
        }
        if flags & 128 != 0 {
            r.solo = Some((b.u64()?, b.u64()?));
        }
        for _ in 0..b.u32()? {
            r.anim.push((b.u32()?, b.u32()?));
        }
        for _ in 0..b.u32()? {
            r.voices.push((b.u32()?, b.u32()?, b.u64()?));
        }
        sh.runs.insert(e, r);
    }
    if !b.0.is_empty() {
        return Err(invalid("the og.timeline state has bytes left over"));
    }
    Ok(sh)
}

struct TimelineBlock(Arc<Mutex<Shared>>);

impl StateBlock for TimelineBlock {
    fn save(&self, out: &mut Vec<u8>) {
        save(&lock(&self.0), out);
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ok(Box::new(load_state(bytes)?))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let next = decoded
            .and_then(|d| d.downcast::<Shared>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        *lock(&self.0) = next;
    }
}

// ── installing ───────────────────────────────────────────────────────────────

/// The records the systems read and write.
#[derive(Clone)]
pub(super) struct Rec {
    pub(super) transform: (SchemaId, Layout),
    follower: (SchemaId, Layout),
    timeline: (SchemaId, Layout),
    pub(super) camera: (SchemaId, Layout),
    pub(super) model: (SchemaId, Layout),
    pub(super) skeleton: (SchemaId, Layout),
    pub(super) player: (SchemaId, Layout),
    pub(super) source: (SchemaId, Layout),
    /// og.PostEffects, for a camera track's focus.
    pub(super) post: Option<(SchemaId, Layout)>,
    /// og.CameraBrain's layout, for a shot track (registered lazily: its
    /// id is looked up when the track plays).
    pub(super) brain: Option<Layout>,
}

/// Registers the timeline events (their layouts into `layouts`).
pub(crate) fn register_records(
    world: &mut World,
    layouts: &mut BTreeMap<String, Layout>,
) -> Result<(), String> {
    let err = |e: Error| e.reason().to_string();
    world.register::<TimelineStarted>().map_err(err)?;
    world.register::<TimelineMarker>().map_err(err)?;
    world.register::<TimelineEnded>().map_err(err)?;
    for n in REGISTERED_BY_TIMELINE {
        let id = world.id_by_name(n).map_err(err)?;
        let layout = Layout::from_schema(world.schema(id).map_err(err)?, RecordKind::Event);
        layouts.insert(layout.schema.clone(), layout);
    }
    Ok(())
}

/// Installs og.path and og.timeline (after the game's systems and og.tween,
/// before og.anim) and the og.timeline state block.
pub(crate) fn install(
    world: &mut World,
    engine_id: u32,
    layouts: &BTreeMap<String, Layout>,
    ui: &og_ui::SharedUi,
) -> Result<(), String> {
    let err = |e: Error| e.reason().to_string();
    let get = |n: &str| -> Result<(SchemaId, Layout), String> {
        let id = world.id_by_name(n).map_err(err)?;
        let layout = layouts.get(n).cloned().unwrap_or_else(|| {
            Layout::from_schema(
                world.schema(id).expect("a registered schema"),
                RecordKind::Component,
            )
        });
        Ok((id, layout))
    };
    let r = Rec {
        transform: get("og.Transform")?,
        follower: get("og.PathFollower")?,
        timeline: get("og.Timeline")?,
        camera: get("og.Camera")?,
        model: get("og.Model")?,
        skeleton: get("og.Skeleton")?,
        player: get("og.AnimationPlayer")?,
        source: get("og.AudioSource")?,
        post: get("og.PostEffects").ok(),
        brain: layouts.get(crate::feel::vcam::BRAIN).cloned(),
    };
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: PATH_SYSTEM.into(),
                reads: Vec::new(),
                writes: names(&[&r.follower, &r.transform]),
                queries: vec![QueryDesc {
                    write: names(&[&r.follower, &r.transform]),
                    ..QueryDesc::default()
                }],
                ..Default::default()
            },
            move |ctx| path_system(ctx, engine_id, &rec),
        )
        .map_err(err)?;
    let sh = shared(engine_id);
    *lock(&sh) = Shared::default();
    let rec = r.clone();
    let ui = ui.clone();
    let state = sh.clone();
    let mut writes = names(&[
        &r.timeline,
        &r.transform,
        &r.camera,
        &r.skeleton,
        &r.player,
        &r.source,
    ]);
    writes.extend(REGISTERED_BY_TIMELINE.iter().map(|s| s.to_string()));
    world
        .add_system(
            SystemDesc {
                name: TIMELINE_SYSTEM.into(),
                reads: names(&[&r.model]),
                writes,
                queries: vec![QueryDesc {
                    write: names(&[&r.timeline]),
                    ..QueryDesc::default()
                }],
                ..Default::default()
            },
            move |ctx| timeline_system(ctx, engine_id, &state, &rec, &ui),
        )
        .map_err(err)?;
    world
        .add_state_block(TIMELINE_SYSTEM, Box::new(TimelineBlock(sh)))
        .map_err(err)?;
    Ok(())
}

// ── og.path's system ─────────────────────────────────────────────────────────

fn path_system(ctx: &mut SystemContext<'_>, engine_id: u32, r: &Rec) -> og_core::Result<()> {
    let dt = ctx.scaled_dt();
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    for e in rows {
        let fb = ctx.get_bytes(e, r.follower.0)?;
        let mut f = r.follower.1.decode(&fb);
        let h = bits(&f["path"]);
        if h == 0 || !f["playing"].as_bool().unwrap_or(true) {
            continue;
        }
        let Some(p) = path_of(engine_id, h) else {
            note(
                engine_id,
                format!(
                    "og.PathFollower.path {h} is not a live path (og.path.create or og.path.find): it does not move"
                ),
            );
            continue;
        };
        let len = p.length();
        let speed = f["speed"].as_f64().unwrap_or(1.0) as f32;
        let mut d = f["distance"].as_f64().unwrap_or(0.0) as f32 + speed * dt;
        let mut finished = false;
        if len > 0.0 {
            if f["loop"].as_bool().unwrap_or(true) {
                d = d.rem_euclid(len);
            } else if d >= len || d <= 0.0 {
                d = d.clamp(0.0, len);
                finished = (speed > 0.0 && d >= len) || (speed < 0.0 && d <= 0.0);
            }
        }
        let s = p.sample(if len > 0.0 { d / len } else { 0.0 });
        let offset = v3(&f["offset"]);
        let align = f["align"].as_bool().unwrap_or(false);
        let tb = ctx.get_bytes(e, r.transform.0)?;
        let mut t = r.transform.1.decode(&tb);
        let pos = if align {
            let (m, q) = facing(&s);
            t["quaternion"] = jq(q);
            s.position + m * offset
        } else {
            s.position + offset
        };
        t["position"] = jv3(pos);
        f["distance"] = json!(d);
        f["finished"] = json!(finished);
        let tb = r.transform.1.encode(Some(&tb), Some(&t)).map_err(invalid)?;
        ctx.set_bytes(e, r.transform.0, &tb)?;
        let fb = r.follower.1.encode(Some(&fb), Some(&f)).map_err(invalid)?;
        ctx.set_bytes(e, r.follower.0, &fb)?;
    }
    Ok(())
}

// ── og.timeline's system ─────────────────────────────────────────────────────

fn timeline_system(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    state: &Mutex<Shared>,
    r: &Rec,
    ui: &og_ui::SharedUi,
) -> og_core::Result<()> {
    let time = ctx.time();
    let rate = (1.0 / f64::from(time.dt)).round().max(1.0) as i64;
    let den = rate * STEPS;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let ops = std::mem::take(&mut lock(state).ops);
    // Runs of entities that lost their og.Timeline (or died): let go.
    let gone: Vec<u64> = lock(state)
        .runs
        .keys()
        .copied()
        .filter(|k| !rows.iter().any(|e| e.to_bits() == *k))
        .collect();
    for k in gone {
        let run = lock(state).runs.remove(&k);
        if let Some(mut run) = run {
            release(ctx, r, &mut run)?;
        }
    }
    let reg = registry(engine_id);
    for e in rows {
        let key = e.to_bits();
        let cb = ctx.get_bytes(e, r.timeline.0)?;
        let mut comp = r.timeline.1.decode(&cb);
        let asset = bits(&comp["asset"]);
        let mut run = lock(state).runs.remove(&key).unwrap_or_default();
        let loaded = lock(&reg).timeline(asset);
        // A new run starts where its `time` field is, stopped: nothing is
        // evaluated (the camera is not taken) until a play or a seek.
        let field_time = comp["time"].as_f64().unwrap_or(0.0) as f32;
        if run.asset != asset {
            release(ctx, r, &mut run)?;
            run = Run {
                asset,
                written: field_time,
                num: (f64::from(field_time.max(0.0)) * den as f64).round() as i64,
                ..Run::default()
            };
        }
        let Some(loaded) = loaded else {
            if asset != 0 {
                note(
                    engine_id,
                    format!(
                        "og.Timeline.asset {asset} is not a loaded timeline (og.load.timeline): it plays nothing"
                    ),
                );
            }
            lock(state).runs.insert(key, run);
            continue;
        };
        let mut playing = comp["playing"].as_bool().unwrap_or(false);
        let mut stop = false;
        let mut seek: Option<f32> = None;
        if field_time != run.written {
            seek = Some(field_time);
        }
        for (_, o) in ops.iter().filter(|(k, _)| *k == key) {
            match *o {
                Op::Play => playing = true,
                Op::Pause => playing = false,
                Op::Seek(t) => seek = Some(t),
                Op::Stop => {
                    stop = true;
                    playing = false;
                    seek = None;
                }
            }
        }
        let mut lt = lock(&loaded);
        let duration = lt.timeline.duration;
        let dur = (f64::from(duration) * den as f64).round() as i64;
        if stop {
            release(ctx, r, &mut run)?;
            run = Run {
                asset,
                ..Run::default()
            };
            comp["time"] = json!(0.0);
            comp["playing"] = json!(false);
            run.written = 0.0;
            // A stop resets; a play or seek queued after it (the
            // command `timeline:play` queues Stop then Play) still applies.
            if !playing && seek.is_none() {
                let out = r
                    .timeline
                    .1
                    .encode(Some(&cb), Some(&comp))
                    .map_err(invalid)?;
                ctx.set_bytes(e, r.timeline.0, &out)?;
                lock(state).runs.insert(key, run);
                continue;
            }
        }
        // A timeline that ended last tick hands back what it owns now
        // (its last frame showed the end).
        if run.release && !playing && seek.is_none() {
            release(ctx, r, &mut run)?;
            run.release = false;
            run.active = false;
        }
        let starting = playing && !run.playing;
        let before = run.num;
        let mut seeked = false;
        if let Some(s) = seek {
            run.num = ((f64::from(s.max(0.0)) * den as f64).round() as i64).clamp(0, dur);
            run.advance = false;
            seeked = true;
        }
        if starting && !seeked && run.num >= dur {
            // Played again after its end: from the start.
            run.num = 0;
        }
        let from = run.num;
        if playing {
            if run.advance {
                let speed = comp["speed"].as_f64().unwrap_or(1.0).clamp(0.0, 16.0);
                run.num += (speed * STEPS as f64).round() as i64;
            }
            run.advance = true;
        } else {
            run.advance = false;
        }
        let mut ended = false;
        let mut wrapped = false;
        if run.num >= dur {
            if playing && comp["loop"].as_bool().unwrap_or(false) && dur > 0 {
                run.num %= dur;
                wrapped = true;
            } else {
                run.num = dur;
                if playing {
                    ended = true;
                    playing = false;
                }
            }
        }
        if starting && !run.paused {
            ctx.send(TimelineStarted { entity: key, asset })?;
        }
        if playing || seeked || run.active || ended {
            run.active = true;
        }
        let secs = |n: i64| n as f64 / den as f64;
        let t = secs(run.num) as f32;
        // What playback passed this tick: (from, to] (a start includes its
        // first time), across the end when it wrapped.
        let mut spans: Vec<(f64, f64)> = Vec::new();
        if (playing || ended) && !seeked {
            let lo = if starting {
                secs(from) - 1e-9
            } else {
                secs(from)
            };
            if wrapped {
                spans.push((lo, secs(dur)));
                spans.push((-1e-9, secs(run.num)));
            } else {
                spans.push((lo, secs(run.num)));
            }
        }
        // A forward seek runs the retroactive events it jumps over.
        let retro = (seeked && run.num > before).then(|| (secs(before), secs(run.num)));
        drop_ended_voices(ctx, r, &mut run, playing)?;
        if run.active {
            let mut touched = Vec::new();
            evaluate(
                ctx,
                engine_id,
                r,
                ui,
                &mut lt,
                &mut run,
                &Step {
                    t,
                    playing,
                    spans: &spans,
                    retro,
                    dt: time.dt,
                    speed: comp["speed"].as_f64().unwrap_or(1.0) as f32,
                    base: 0,
                    depth: 0,
                },
                &mut touched,
            )?;
            // Voices of a nested timeline whose key ended go with it.
            let mut gone = Vec::new();
            run.voices.retain(|v| {
                let keep = v.0 & NESTED == 0 || touched.contains(&(v.0 & !(NESTED_TRACKS - 1)));
                if !keep {
                    gone.push(v.2);
                }
                keep
            });
            for v in gone {
                let e = Entity::from_bits(v);
                if ctx.is_alive(e) {
                    ctx.despawn(e)?;
                }
            }
        }
        for (a, b) in &spans {
            markers(ctx, key, asset, &lt, *a, *b, "", 0)?;
        }
        if ended {
            ctx.send(TimelineEnded { entity: key, asset })?;
            run.release = true;
            run.paused = false;
        } else if !playing && run.playing && !stop {
            run.paused = true;
        }
        if playing {
            run.paused = false;
        }
        drop(lt);
        comp["time"] = json!(t);
        comp["playing"] = json!(playing);
        run.written = t;
        run.playing = playing;
        let out = r
            .timeline
            .1
            .encode(Some(&cb), Some(&comp))
            .map_err(invalid)?;
        ctx.set_bytes(e, r.timeline.0, &out)?;
        lock(state).runs.insert(key, run);
    }
    Ok(())
}

/// Sends og.TimelineMarker for each marker key in `(a, b]`, and those of the
/// timelines it nests (named `<nested>/<marker>`) in the child's span.
#[allow(clippy::too_many_arguments)]
fn markers(
    ctx: &mut SystemContext<'_>,
    entity: u64,
    asset: u64,
    lt: &Loaded,
    a: f64,
    b: f64,
    prefix: &str,
    depth: usize,
) -> og_core::Result<()> {
    for tr in &lt.timeline.tracks {
        match tr {
            Track::Marker(ks) => {
                for k in ks.iter().filter(|k| crossed(k.time, a, b)) {
                    let name = format!("{prefix}{}", k.name);
                    ctx.send(TimelineMarker {
                        entity,
                        asset,
                        name: Str64::new(&name).unwrap_or_default(),
                        time: k.time,
                    })?;
                }
            }
            Track::Timeline(keys) if depth < og_timeline::NEST_DEPTH => {
                for (ki, k) in keys.iter().enumerate() {
                    let until = keys.get(ki + 1).map_or(lt.timeline.duration, |n| n.time);
                    let Some(child) = lt.nested.get(&k.play).cloned() else {
                        continue;
                    };
                    let cl = lock(&child);
                    let d = cl.timeline.duration;
                    let p = format!("{prefix}{}/", short(&k.play));
                    for (ca, cb) in og_timeline::eval::child_spans(k, until, d, a, b) {
                        markers(ctx, entity, asset, &cl, ca, cb, &p, depth + 1)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Lets go of what a run owns: the camera back, its voices stopped.
fn release(ctx: &mut SystemContext<'_>, r: &Rec, run: &mut Run) -> og_core::Result<()> {
    if let (Some(f), Some(post)) = (run.focus.take(), &r.post)
        && let Ok(bytes) = ctx.resource_bytes(post.0)
    {
        let mut v = post.1.decode(&bytes);
        v["depthOfField"] = json!(f[0] != 0.0);
        v["dofFocusDistance"] = json!(f[1]);
        v["dofFocusRange"] = json!(f[2]);
        let out = post.1.encode(Some(&bytes), Some(&v)).map_err(invalid)?;
        ctx.set_resource_bytes(post.0, &out)?;
    }
    if let Some(c) = run.camera.take() {
        let bytes = ctx.resource_bytes(r.camera.0)?;
        let mut cam = r.camera.1.decode(&bytes);
        cam["position"] = jv3(Vec3::new(c[0], c[1], c[2]));
        cam["quaternion"] = jq(Quat::from_xyzw(c[3], c[4], c[5], c[6]));
        cam["fov"] = json!(c[7]);
        let out = r
            .camera
            .1
            .encode(Some(&bytes), Some(&cam))
            .map_err(invalid)?;
        ctx.set_resource_bytes(r.camera.0, &out)?;
    }
    if let Some((found, _)) = run.solo {
        super::evaluate::set_solo(ctx, r, found, -1.0, og_tween::Ease::CubicInOut)?;
        run.solo = None;
    }
    for (_, _, v) in std::mem::take(&mut run.voices) {
        let e = Entity::from_bits(v);
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
        }
    }
    run.anim.clear();
    run.active = false;
    Ok(())
}
