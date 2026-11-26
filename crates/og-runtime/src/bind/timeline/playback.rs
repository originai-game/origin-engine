//! Timeline playback, snapshot state and scheduled systems.

use super::*;
use og_core::bytes::ByteReader;
use std::collections::BTreeSet;

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
    /// The scenes its `scene` tracks have in the world: (track, key, the
    /// root their entities are under as entity bits; 0: it was refused).
    pub(super) scenes: Vec<(u32, u32, u64)>,
    /// What they have from the next tick's start, when that is not what
    /// `scenes` holds: (track, key, scene asset id). `super::stage` puts
    /// it in place as that tick starts.
    pub(super) due: Option<Vec<(u32, u32, String)>>,
}

#[derive(Default)]
pub(super) struct Shared {
    pub(super) runs: IdMap<u64, Run>,
    pub(super) ops: Vec<(u64, Op)>,
    /// The captions the last tick showed (og.timeline.captions).
    captions: Vec<crate::host::wire::TimelineCaption>,
    /// Entities suppressed this tick (e.g. og.flow's dissolving-out previous deck):
    /// only their video/audio tracks run; camera/captions/events/plate are skipped.
    suppressed: BTreeSet<u64>,
    /// The og.Timeline entities in pre-roll last tick: each asks for the
    /// modules it needs once, as its pre-roll begins. Not saved: a restored
    /// world asks again.
    prerolled: BTreeSet<u64>,
    /// Scene roots of runs that stopped or went: despawned, with what is
    /// under them, as the next tick starts (`super::stage`). Saved.
    pub(super) leaving: Vec<u64>,
    /// The scenes og.timeline asked `crate::world::scenes` to prepare: those
    /// its timelines in pre-roll or playing name. Not saved, as `prerolled`.
    preparing: BTreeSet<String>,
}

impl Shared {
    /// `run` lets go of the scenes it has in the world and of those due.
    fn leave(&mut self, run: &mut Run) {
        let roots = run.scenes.drain(..).map(|s| s.2).filter(|b| *b != 0);
        self.leaving.extend(roots);
        run.due = None;
    }
}

/// Engine `engine`'s runs (fresh ones for an engine that is gone).
pub(super) fn shared(engine: u32) -> Arc<Mutex<Shared>> {
    crate::engine::engine(engine)
        .map(|e| e.modules.timeline.playback.clone())
        .unwrap_or_default()
}

/// Sets the timeline entities suppressed on this tick (called by `og.flow`
/// before `og.timeline` runs for dissolving-out `previous` decks).
pub(crate) fn set_suppressed(engine: u32, entities: BTreeSet<u64>) {
    lock(&shared(engine)).suppressed = entities;
}

/// Queues a seek of `entity`'s og.Timeline to `time`, applied when the
/// timeline system next runs: how a deck loaded paused (og.flow's paused
/// start) is evaluated at its time, since a new run evaluates nothing until a
/// play or a seek.
pub(crate) fn queue_seek(engine: u32, entity: u64, time: f32) {
    lock(&shared(engine)).ops.push((entity, Op::Seek(time)));
}

/// Whether `owner` has a scene queued for placement on the next `before_tick`
/// and no scene placed yet (used by `og.flow` to avoid a 1-frame cut gap).
pub(crate) fn scene_pending(engine: u32, owner: u64) -> bool {
    lock(&shared(engine))
        .runs
        .get(&owner)
        .is_some_and(|r| r.due.as_ref().is_some_and(|d| !d.is_empty()) && r.scenes.is_empty())
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
    Ok(layout.read::<record::Timeline>(&bytes)?.asset)
}

/// The same inside a system, where the step holds the world: the entity's
/// og.Timeline as the running system can read it. `None` when it cannot
/// (it does not read og.Timeline, or inserted it this tick, queued until
/// it returns): the op is queued as it is, and og.timeline, later this
/// tick, warns of one it cannot apply.
fn timeline_in_system(
    eng: &Engine,
    f: &crate::engine::Frame,
    e: Entity,
) -> Result<Option<u64>, String> {
    // SAFETY: the frame is published while its system runs.
    let ctx = unsafe { f.ctx() };
    if !ctx.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    let Ok((layout, id)) =
        crate::host::api::running_record(eng, "og.Timeline", crate::RecordKind::Component)
    else {
        return Ok(None);
    };
    match ctx.get_bytes(e, id) {
        Ok(bytes) => Ok(Some(layout.read::<record::Timeline>(&bytes)?.asset)),
        Err(_) => Ok(None),
    }
}

fn op(world: u32, entity: f64, op: Op) -> Result<(), String> {
    (|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let asset = match crate::engine::running(world) {
            Some(f) => timeline_in_system(&eng, f, e)?,
            None => Some(has_timeline(&eng, e)?),
        };
        if let Some(asset) = asset
            && matches!(op, Op::Play | Op::Seek(_))
        {
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
    })()
}

pub fn og_timeline_play(world: u32, entity: f64) -> Result<(), String> {
    op(world, entity, Op::Play)
}

pub fn og_timeline_pause(world: u32, entity: f64) -> Result<(), String> {
    op(world, entity, Op::Pause)
}

pub fn og_timeline_seek(world: u32, entity: f64, time: f64) -> Result<(), String> {
    op(world, entity, Op::Seek(time as f32))
}

pub fn og_timeline_stop(world: u32, entity: f64) -> Result<(), String> {
    op(world, entity, Op::Stop)
}

pub fn og_timeline_captions(world: u32) -> Vec<crate::host::wire::TimelineCaption> {
    lock(&shared(world)).captions.clone()
}

// ── the state block ──────────────────────────────────────────────────────────

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
            | u32::from(r.solo.is_some()) << 7
            | u32::from(!r.scenes.is_empty() || r.due.is_some()) << 8;
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
        // Only with a scene track's scene in the world or due, likewise.
        if flags & 256 != 0 {
            u32_(out, r.scenes.len() as u32);
            for (t, k, root) in &r.scenes {
                u32_(out, *t);
                u32_(out, *k);
                u64_(out, *root);
            }
            u32_(out, u32::from(r.due.is_some()));
            if let Some(due) = &r.due {
                u32_(out, due.len() as u32);
                for (t, k, scene) in due {
                    u32_(out, *t);
                    u32_(out, *k);
                    u32_(out, scene.len() as u32);
                    out.extend_from_slice(scene.as_bytes());
                }
            }
        }
    }
    // Only while a stopped run's scenes wait for the tick to start.
    if !sh.leaving.is_empty() {
        u32_(out, sh.leaving.len() as u32);
        for root in &sh.leaving {
            u64_(out, *root);
        }
    }
}

fn load_state(bytes: &[u8]) -> og_core::Result<Shared> {
    let mut b = ByteReader::new(bytes, "the og.timeline state");
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
        if flags & 256 != 0 {
            for _ in 0..b.u32()? {
                r.scenes.push((b.u32()?, b.u32()?, b.u64()?));
            }
            if b.u32()? != 0 {
                let mut due = Vec::new();
                for _ in 0..b.u32()? {
                    due.push((b.u32()?, b.u32()?, b.string()?));
                }
                r.due = Some(due);
            }
        }
        sh.runs.insert(e, r);
    }
    if !b.is_done() {
        for _ in 0..b.u32()? {
            sh.leaving.push(b.u64()?);
        }
    }
    if !b.is_done() {
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
        let mut next = decoded
            .and_then(|d| d.downcast::<Shared>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        let mut now = lock(&self.0);
        // What is being prepared is the engine's, not the world's: kept,
        // so the next tick lets go of what the restored world no longer
        // names.
        next.preparing = std::mem::take(&mut now.preparing);
        *now = next;
    }

    /// Per og.Timeline entity: its `run` (the asset handle a value; the
    /// shot solo, voices' players and scene roots as entities) and its
    /// queued `op`s, keyed by their place in the queue. The scene roots
    /// waiting to leave are global, as entities.
    fn form(&self, bytes: &[u8]) -> og_core::Result<og_core::block_form::BlockForm> {
        use og_core::block_form::{BlockForm, Field, Record, Records};
        let sh = load_state(bytes)?;
        let mut out = Records::default();
        for (i, (e, op)) in sh.ops.iter().enumerate() {
            let (name, time) = match op {
                Op::Play => ("play", None),
                Op::Pause => ("pause", None),
                Op::Seek(t) => ("seek", Some(*t)),
                Op::Stop => ("stop", None),
            };
            out.entities.push(
                Record::new(*e, "op")
                    .key(Field::of(&i))
                    .field("op", Field::of(&name))
                    .field("time", Field::of(&time)),
            );
        }
        for (&e, r) in &sh.runs {
            let (found, wrote) = r.solo.unwrap_or_default();
            let tk = |v: &[(u32, u32, u64)]| v.iter().map(|&(t, k, _)| [t, k]).collect::<Vec<_>>();
            let ids = |v: &[(u32, u32, u64)]| v.iter().map(|x| x.2).collect::<Vec<_>>();
            out.entities.push(
                Record::new(e, "run")
                    .field("asset", Field::of(&r.asset))
                    .field("num", Field::of(&r.num))
                    .field("written", Field::of(&r.written))
                    .field("playing", Field::of(&r.playing))
                    .field("advance", Field::of(&r.advance))
                    .field("paused", Field::of(&r.paused))
                    .field("active", Field::of(&r.active))
                    .field("release", Field::of(&r.release))
                    .field("camera", Field::of(&r.camera))
                    .field("focus", Field::of(&r.focus))
                    .field("solo", Field::of(&r.solo.is_some()))
                    .field("soloFound", Field::Entity(found))
                    .field("soloWrote", Field::Entity(wrote))
                    .field("anim", Field::of(&r.anim))
                    .field("voiceKeys", Field::of(&tk(&r.voices)))
                    .field("voices", Field::Entities(ids(&r.voices)))
                    .field("sceneKeys", Field::of(&tk(&r.scenes)))
                    .field("scenes", Field::Entities(ids(&r.scenes)))
                    .field("due", Field::of(&r.due)),
            );
        }
        out.global
            .push(("leaving".into(), Field::Entities(sh.leaving.clone())));
        Ok(BlockForm::Entities(out))
    }

    /// The camera a run hands back is a place in the world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut sh = lock(&self.0);
        for r in sh.runs.values_mut() {
            if let Some(c) = &mut r.camera {
                let p = s.moved(glam::Vec3::new(c[0], c[1], c[2]));
                c[..3].copy_from_slice(&p.to_array());
            }
        }
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
    /// og.Vfx's and og.Parent's, for a vfx track, likewise.
    pub(super) vfx: Option<Layout>,
    pub(super) parent: Option<Layout>,
    /// og.VideoPlayer's layout, for a video track (registered lazily).
    pub(super) video: Option<Layout>,
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
    mount: crate::engine::Mount<'_>,
    layouts: &BTreeMap<String, Layout>,
) -> Result<(), String> {
    let engine_id = mount.id;
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
        brain: layouts.get(crate::present::feel::vcam::BRAIN).cloned(),
        vfx: layouts.get(crate::bind::vfx::COMPONENT).cloned(),
        parent: layouts.get("og.Parent").cloned(),
        video: layouts.get(crate::bind::video::PLAYER).cloned(),
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
    let sh = mount.modules.timeline.playback.clone();
    *lock(&sh) = Shared::default();
    let rec = r.clone();
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
            move |ctx| timeline_system(ctx, engine_id, &state, &rec),
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
        let row = record::PathFollower::from_json(&f).map_err(invalid)?;
        let h = row.path;
        if h == 0 || !row.playing {
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
        let speed = row.speed;
        let mut d = row.distance + speed * dt;
        let mut finished = false;
        if len > 0.0 {
            if row.r#loop {
                d = d.rem_euclid(len);
            } else if d >= len || d <= 0.0 {
                d = d.clamp(0.0, len);
                finished = (speed > 0.0 && d >= len) || (speed < 0.0 && d <= 0.0);
            }
        }
        let s = p.sample(if len > 0.0 { d / len } else { 0.0 });
        let offset = row.offset;
        let align = row.align;
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
) -> og_core::Result<()> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    if rows.is_empty() {
        let st = lock(state);
        if st.ops.is_empty()
            && st.runs.is_empty()
            && st.captions.is_empty()
            && st.suppressed.is_empty()
            && st.prerolled.is_empty()
            && st.preparing.is_empty()
        {
            return Ok(());
        }
    }
    step(ctx, engine_id, state, r, rows)
}

// A guarded entry into the engine's `timeline` module (wasm-modules.json).
#[inline(never)]
fn step(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    state: &Mutex<Shared>,
    r: &Rec,
    rows: Vec<Entity>,
) -> og_core::Result<()> {
    let time = ctx.time();
    let rate = i64::from(time.rate);
    let den = rate * STEPS;
    let ops = std::mem::take(&mut lock(state).ops);
    // A play or seek a system queued without reading the entity's
    // og.Timeline (`timeline_in_system`): said here when it has none.
    for (k, o) in &ops {
        if matches!(o, Op::Play | Op::Seek(_)) && !rows.iter().any(|e| e.to_bits() == *k) {
            note(
                engine_id,
                format!("og.timeline: entity {k} has no og.Timeline: nothing plays"),
            );
        }
    }
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
            lock(state).leave(&mut run);
        }
    }
    // Runs whose asset changed hand back what they own before any run
    // takes it: when og.flow cuts from one deck to another in a tick, the
    // camera the new segment takes is the one the old one hands back, in
    // whichever order their rows come.
    for e in &rows {
        let row =
            record::Timeline::from_json(&r.timeline.1.decode(&ctx.get_bytes(*e, r.timeline.0)?))
                .map_err(invalid)?;
        let mut st = lock(state);
        if let Some(run) = st.runs.get_mut(&e.to_bits())
            && run.asset != row.asset
        {
            let mut run = std::mem::take(run);
            drop(st);
            release(ctx, r, &mut run)?;
            lock(state).runs.insert(e.to_bits(), run);
        }
    }
    let reg = registry(engine_id);
    // The caption keys showing this tick, in the timelines' order: the
    // UI's `$captions` (docs/ui.md §2). Nothing here draws them.
    let mut captions: Vec<crate::host::wire::TimelineCaption> = Vec::new();
    let suppressed_set = std::mem::take(&mut lock(state).suppressed);
    // The plate a `layer under` video shows, and every timeline's own
    // player's texture (what a plate may have left on the camera).
    let mut plate: Option<u64> = None;
    let mut owned: BTreeSet<u64> = BTreeSet::new();
    let player = ctx.id_by_name(crate::bind::video::PLAYER).ok();
    let was = std::mem::take(&mut lock(state).prerolled);
    let mut prerolled = BTreeSet::new();
    let mut wanted: Vec<String> = Vec::new();
    // The scenes the timelines in pre-roll or playing name.
    let mut scenes: BTreeSet<String> = BTreeSet::new();
    for e in rows {
        let key = e.to_bits();
        let suppressed = suppressed_set.contains(&key);
        if let (Some(id), Some(l)) = (player, &r.video)
            && let Ok(b) = ctx.get_bytes(e, id)
            && let Ok(p) = l.read::<record::VideoPlayer>(&b)
            && p.texture != 0
        {
            owned.insert(p.texture);
        }
        let cb = ctx.get_bytes(e, r.timeline.0)?;
        let mut comp = r.timeline.1.decode(&cb);
        let row = record::Timeline::from_json(&comp).map_err(invalid)?;
        let asset = row.asset;
        let mut run = lock(state).runs.remove(&key).unwrap_or_default();
        let loaded = lock(&reg).timeline(asset);
        // A new run starts where its `time` field is, stopped: nothing is
        // evaluated (the camera is not taken) until a play or a seek.
        let field_time = row.time;
        if run.asset != asset {
            release(ctx, r, &mut run)?;
            lock(state).leave(&mut run);
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
        let mut playing = row.playing;
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
            lock(state).leave(&mut run);
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
        let has_sub_loop = row.r#loop && row.loop_end > row.loop_start && row.loop_start >= 0.0;
        let loop_a = if has_sub_loop {
            ((f64::from(row.loop_start) * den as f64).round() as i64).clamp(0, dur)
        } else {
            0
        };
        let loop_b = if has_sub_loop {
            ((f64::from(row.loop_end) * den as f64).round() as i64).clamp(loop_a, dur)
        } else {
            dur
        };
        let sub_loop_valid = has_sub_loop && loop_b > loop_a;
        let from = run.num;
        if playing {
            if run.advance {
                let speed = f64::from(row.speed).clamp(0.0, 16.0);
                run.num += (speed * STEPS as f64).round() as i64;
            }
            run.advance = true;
        } else {
            run.advance = false;
        }
        let mut ended = false;
        let mut wrapped = false;
        let mut wrap_from = dur;
        let mut wrap_to = 0i64;
        if sub_loop_valid && playing && run.num >= loop_b {
            let span = loop_b - loop_a;
            run.num = loop_a + (run.num - loop_a).rem_euclid(span);
            wrapped = true;
            wrap_from = loop_b;
            wrap_to = loop_a;
        } else if run.num >= dur {
            if playing && row.r#loop && dur > 0 {
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
        if starting && !run.paused && !suppressed {
            ctx.send(TimelineStarted { entity: key, asset })?;
        }
        if playing || seeked || run.active || ended {
            run.active = true;
        }
        let secs = |n: i64| n as f64 / den as f64;
        let t64 = secs(run.num);
        let t = t64 as f32;
        let loop_range = sub_loop_valid.then(|| (secs(loop_a), secs(loop_b)));
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
                spans.push((lo, secs(wrap_from)));
                spans.push((secs(wrap_to) - 1e-9, t64));
            } else {
                spans.push((lo, t64));
            }
        }
        // A forward seek runs the retroactive events it jumps over.
        let retro = (seeked && run.num > before).then(|| (secs(before), t64));
        drop_ended_voices(ctx, r, &mut run, playing)?;
        if run.active {
            let mut out = Evaluated::default();
            evaluate(
                ctx,
                engine_id,
                r,
                &mut lt,
                &mut run,
                &Step {
                    owner: e,
                    t,
                    t64,
                    playing,
                    spans: &spans,
                    retro,
                    dt: time.dt(),
                    speed: row.speed,
                    loop_range,
                    suppressed,
                    base: 0,
                    depth: 0,
                },
                &mut out,
            )?;
            let Evaluated {
                touched,
                captions: showing,
                plate: shown,
            } = out;
            // A held or paused segment keeps its plate; a warming deck
            // shows none.
            if !row.preroll && !suppressed {
                plate = plate.or(shown);
            }
            if !showing.is_empty() {
                for (track, k, mut cap) in showing {
                    cap.id = format!("{key}:{track}:{k}");
                    captions.push(cap);
                }
            }
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
        // What its scene tracks have in the world at this time, put in
        // place as the next tick starts when it is not there already.
        if run.active && !(lt.scenes.is_empty() && run.scenes.is_empty()) {
            let mut at = Vec::new();
            super::evaluate::scenes_at(&lt, t, 0, 0, &mut at);
            let placed =
                |a: &(u32, u32, String)| run.scenes.iter().any(|s| (s.0, s.1) == (a.0, a.1));
            let same = at.len() == run.scenes.len() && at.iter().all(placed);
            run.due = (!same).then_some(at);
        }
        // Pre-roll: stopped and holding nothing, it prepares what its time
        // starts with instead of showing it.
        let prerolling = row.preroll && !playing && !run.active;
        if prerolling {
            preroll(ctx, r, &lt, e, t)?;
            prerolled.insert(key);
            if !was.contains(&key) {
                wanted.push(lt.id.clone());
            }
        }
        if prerolling || playing || run.active || !run.scenes.is_empty() || run.due.is_some() {
            scenes.extend(lt.scenes.iter().cloned());
        }
        if !suppressed {
            for (a, b) in &spans {
                markers(ctx, key, asset, &lt, *a, *b, "", 0)?;
            }
        }
        if ended {
            if !suppressed {
                ctx.send(TimelineEnded { entity: key, asset })?;
            }
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
    super::evaluate::plate(ctx, r, plate, &owned)?;
    {
        let mut sh = lock(state);
        sh.captions = captions;
        sh.prerolled = prerolled;
    }
    // The modules a pre-rolled timeline needs (its videos' and its
    // story's), asked for once as its pre-roll begins.
    if !wanted.is_empty()
        && let Ok(eng) = crate::engine::engine(engine_id)
    {
        for id in wanted {
            crate::world::needs::asset(&eng, &id, None);
        }
    }
    // The scenes those timelines name are prepared from the tick one
    // starts to name them until none does: loaded with what they spawn
    // with, never placed here (`super::stage` places them).
    let before = std::mem::replace(&mut lock(state).preparing, scenes.clone());
    if before != scenes
        && let Ok(eng) = crate::engine::engine(engine_id)
    {
        use crate::world::scenes::{By, prepare, release};
        for id in scenes.difference(&before) {
            if let Err(e) = prepare(&eng, id, By::Timeline) {
                note(engine_id, format!("og.timeline: scene {id}: {e}"));
            }
        }
        for id in before.difference(&scenes) {
            let _ = release(&eng, id, By::Timeline);
        }
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
