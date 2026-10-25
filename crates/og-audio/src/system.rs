//! The tick side: `install` registers the records and the `og.audio`
//! system, which turns AudioSource changes into the tick's commands, keeps
//! each playing source's playhead (so `SoundEnded` lands on a tick that only
//! depends on the world), and hands the commands to an attached sink.

use crate::command::{Command, FLAG_LOOP, FLAG_SPATIAL, Op, TickPacket, encode, fx};
use crate::schema::{
    Audio, AudioBusFx, AudioListener, AudioPlayback, AudioSource, AudioStream, BUS_COUNT, Parent,
    ReverbZone, SoundEnded, SoundStarted, Transform, end_reason, panning_model,
};
use crate::sound::SoundBank;
use og_core::Result;
use og_core::glam::{Quat, Vec3, Vec4};
use og_core::hash::StableHasher;
use og_ecs::{Entity, QueryDesc, SystemContext, SystemDesc, World};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

/// The system's name in the schedule.
pub const SYSTEM_NAME: &str = "og.audio";

/// Packets an attached sink may leave undrained before the oldest are
/// dropped (about 17 s at 60 Hz).
pub const MAX_QUEUED_PACKETS: usize = 1024;

/// Whether something solid stands between a source and the listener (the
/// host casts a physics ray); `og.AudioSource.occlude` asks it.
pub type Occluder = Arc<dyn Fn(Vec3, Vec3) -> bool + Send + Sync>;

/// How audio joins a world.
#[derive(Clone, Default)]
pub struct AudioDesc {
    /// The bank sources' `sound` handles index; a fresh one when `None`.
    pub bank: Option<SoundBank>,
    /// Continue this link (and its bank) instead of making a new one, so a
    /// sink attached to it keeps receiving: a hot reload's rebuilt world
    /// passes the old world's link. Ignored when its tick rate differs.
    pub link: Option<AudioLink>,
    /// Occlusion's ray (none: nothing occludes).
    pub occluder: Option<Occluder>,
}

impl std::fmt::Debug for AudioDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDesc")
            .field("bank", &self.bank)
            .field("link", &self.link)
            .field("occluder", &self.occluder.is_some())
            .finish()
    }
}

#[derive(Default)]
struct LinkState {
    attached: bool,
    resync: bool,
    last_tick: Option<u64>,
    last_live: Vec<u32>,
    packets: VecDeque<TickPacket>,
    dropped: u64,
    /// The tick the world will run next (the end of the audio produced so far).
    next_tick: u64,
}

/// The connection between one world's audio system and its sink (a mixer
/// on the web Worker, cpal, or the null sink). Cloning shares it.
#[derive(Clone)]
pub struct AudioLink {
    state: Arc<Mutex<LinkState>>,
    bank: SoundBank,
    tick_rate: u32,
    listen: crate::listen::Listen,
}

impl std::fmt::Debug for AudioLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioLink({} Hz, {:?})", self.tick_rate, self.bank)
    }
}

impl AudioLink {
    /// The sounds.
    pub fn bank(&self) -> &SoundBank {
        &self.bank
    }

    /// Ticks per second of the world.
    pub fn tick_rate(&self) -> u32 {
        self.tick_rate
    }

    /// What the sink's mixer rendered last (the probe's `listen`).
    pub fn listen(&self) -> &crate::listen::Listen {
        &self.listen
    }

    /// Starts queueing packets for a sink. The next tick sends a resync
    /// (every playing voice at its playhead) so a sink attached mid-game,
    /// such as an AudioContext unlocked at the first gesture, hears the
    /// music already playing.
    pub fn attach(&self) {
        let mut s = self.state.lock().expect("audio link");
        s.attached = true;
        s.resync = true;
        s.packets.clear();
    }

    /// Stops queueing; the queue is dropped.
    pub fn detach(&self) {
        let mut s = self.state.lock().expect("audio link");
        s.attached = false;
        s.packets.clear();
    }

    pub fn attached(&self) -> bool {
        self.state.lock().expect("audio link").attached
    }

    /// Every packet queued since the last drain, oldest first.
    pub fn drain(&self) -> Vec<TickPacket> {
        self.state
            .lock()
            .expect("audio link")
            .packets
            .drain(..)
            .collect()
    }

    /// The tick the world runs next: audio exists up to this tick's start.
    pub fn next_tick(&self) -> u64 {
        self.state.lock().expect("audio link").next_tick
    }

    /// Packets dropped because the sink did not drain.
    pub fn dropped(&self) -> u64 {
        self.state.lock().expect("audio link").dropped
    }

    /// Whether this tick must resync the sink, and notes the tick.
    fn begin(&self, tick: u64) -> bool {
        let mut s = self.state.lock().expect("audio link");
        let gap = s.last_tick.is_some_and(|t| t + 1 != tick);
        if gap {
            // A restore or a skipped tick: the mixer's voices are not the world's.
            s.resync = true;
        }
        s.last_tick = Some(tick);
        s.next_tick = tick + 1;
        s.attached && std::mem::take(&mut s.resync)
    }

    fn publish(&self, packet: TickPacket, live: Vec<u32>) {
        let mut s = self.state.lock().expect("audio link");
        if !s.attached {
            s.last_live = live;
            return;
        }
        let mut packet = packet;
        if packet.resync || live != s.last_live {
            packet.live = Some(live.clone());
        }
        s.last_live = live;
        if packet.commands.is_empty() && packet.live.is_none() {
            return;
        }
        if s.packets.len() >= MAX_QUEUED_PACKETS {
            s.packets.pop_front();
            s.dropped += 1;
            s.resync = true;
        }
        s.packets.push_back(packet);
    }
}

static LINKS: Mutex<BTreeMap<u32, AudioLink>> = Mutex::new(BTreeMap::new());

/// Publishes an engine's link so its host's sink can find it by engine id.
pub fn publish_link(engine: u32, link: AudioLink) {
    LINKS.lock().expect("audio links").insert(engine, link);
}

/// The link of an engine, if `publish_link` was called for it.
pub fn link(engine: u32) -> Option<AudioLink> {
    LINKS.lock().expect("audio links").get(&engine).cloned()
}

/// Forgets an engine's link (on destroy); its sink stops receiving.
pub fn retire_link(engine: u32) -> Option<AudioLink> {
    let link = LINKS.lock().expect("audio links").remove(&engine);
    if let Some(l) = &link {
        l.detach();
    }
    link
}

/// Registers audio's records (`og.AudioSource`, `og.AudioListener`,
/// `og.Audio`, `og.SoundStarted`, `og.SoundEnded`, the engine-owned
/// `og.AudioPlayback` and `og.AudioStream`, and `og.Transform` and
/// `og.Parent` when absent),
/// inserts the resources, and appends the `og.audio` system to the schedule.
/// Call it after the game's systems so the system sees the tick's changes.
pub fn install(world: &mut World, desc: &AudioDesc) -> Result<AudioLink> {
    world.register::<Transform>()?;
    world.register::<Parent>()?;
    world.register::<AudioSource>()?;
    world.register::<AudioListener>()?;
    world.register::<Audio>()?;
    world.register::<SoundStarted>()?;
    world.register::<SoundEnded>()?;
    world.register::<AudioPlayback>()?;
    world.register::<AudioStream>()?;
    world.register::<AudioBusFx>()?;
    world.register::<ReverbZone>()?;
    crate::music::register(world)?;
    if !world.has_resource::<Audio>() {
        world.insert_resource(Audio::default())?;
    }
    let dt = world.time().dt;
    let tick_rate = (1.0 / f64::from(dt)).round().max(1.0) as u32;
    world.insert_resource(AudioStream {
        hash: 0,
        commands: 0,
        next_voice: 1,
        voices: 0,
        tick_rate,
        bus_sent: Vec4::splat(-1.0),
        fx: fx_hash(&fx_state(&[None; BUS_COUNT], &Audio::default())),
        listener: 0,
        listener_at: Vec3::ZERO,
    })?;
    let link = match &desc.link {
        Some(l) if l.tick_rate == tick_rate => {
            // The new world's first tick resyncs the sink (a gap in ticks).
            l.state.lock().expect("audio link").resync = true;
            l.clone()
        }
        _ => AudioLink {
            state: Arc::new(Mutex::new(LinkState::default())),
            bank: desc.bank.clone().unwrap_or_default(),
            tick_rate,
            listen: crate::listen::Listen::default(),
        },
    };
    crate::music::install(world)?;
    let system_link = link.clone();
    let occluder = desc.occluder.clone();
    world.add_system(
        SystemDesc::new(SYSTEM_NAME)
            .read::<Audio>()
            .read::<AudioListener>()
            .read::<Transform>()
            .read::<Parent>()
            .read::<AudioBusFx>()
            .write::<ReverbZone>()
            .write::<AudioSource>()
            .write::<AudioPlayback>()
            .write::<AudioStream>()
            .write::<SoundStarted>()
            .write::<SoundEnded>()
            // 0: playing sources, 1: sources not yet started,
            // 2: playheads whose source was removed, 3: listeners.
            .query(
                QueryDesc::new()
                    .write::<AudioSource>()
                    .write::<AudioPlayback>(),
            )
            .query(
                QueryDesc::new()
                    .write::<AudioSource>()
                    .without::<AudioPlayback>(),
            )
            .query(
                QueryDesc::new()
                    .write::<AudioPlayback>()
                    .without::<AudioSource>(),
            )
            .query(QueryDesc::new().read::<AudioListener>().read::<Transform>())
            // 4: bus effects.
            .query(QueryDesc::new().read::<AudioBusFx>())
            // 5: reverb zones.
            .query(QueryDesc::new().write::<ReverbZone>().read::<Transform>())
            // 6: child entities, whose sources, listeners and zones are
            // placed down the og.Parent chain.
            .query(QueryDesc::new().read::<Parent>()),
        move |ctx| run(ctx, &system_link, occluder.as_ref()),
    )?;
    Ok(link)
}

/// Where entities are in the world this tick: each child's `og.Parent`
/// (query 6), so a source, listener or zone on a child entity is placed at
/// its `og.Transform` composed down the chain, as the renderer composes
/// `og.MatrixWorld` and physics places soft bodies (one helper,
/// `og_core::math::world_pose`). A function of the tick's transforms alone.
struct Places {
    /// Parents by child bits; empty when nothing is parented.
    parents: BTreeMap<u64, Entity>,
}

impl Places {
    fn read(ctx: &SystemContext<'_>) -> Result<Places> {
        let mut parents = BTreeMap::new();
        for a in ctx.query(6)? {
            for &e in a.entities() {
                let p = ctx.get::<Parent>(e)?;
                parents.insert(e.to_bits(), Entity::from_bits(p.entity));
            }
        }
        Ok(Places { parents })
    }

    /// `e`'s transform in the world: its own composed down `og.Parent` (a
    /// parent without `og.Transform` ends the chain).
    fn world(&self, ctx: &SystemContext<'_>, e: Entity) -> Result<Transform> {
        let t = ctx.get::<Transform>(e)?;
        if self.parents.is_empty() {
            return Ok(t);
        }
        let (position, quaternion, scale) = og_core::math::world_pose(
            e,
            (t.position, t.quaternion, t.scale),
            |c| self.parents.get(&c.to_bits()).copied(),
            |p| {
                ctx.get::<Transform>(p)
                    .ok()
                    .map(|t| (t.position, t.quaternion, t.scale))
            },
        );
        Ok(Transform {
            position,
            quaternion,
            scale,
        })
    }
}

/// The listener's frame: position, rotation, volume, and what Doppler and
/// the HRTF read.
#[derive(Clone, Copy)]
struct Ear {
    position: Vec3,
    inverse: Quat,
    volume: f32,
    /// Metres per second: the world position's change since last tick.
    velocity: Vec3,
    /// `AudioListener.dopplerFactor`.
    doppler: f32,
    /// The listener's panning model is the HRTF.
    hrtf: bool,
}

/// A tick velocity in m/s: the change of a world position over one tick. A
/// jump at or past the speed of sound is a teleport (a warp, a respawn) and
/// counts as standing still, so it shifts nothing.
fn tick_velocity(prev: Vec3, now: Vec3, tick_rate: u32, speed: f32) -> Vec3 {
    let v = (now - prev) * tick_rate as f32;
    if v.length() < speed { v } else { Vec3::ZERO }
}

/// Doppler's pitch ratio for a source at `p` moving at `v`: OpenAL 1.1's
/// model (Web Audio's former PannerNode one), `(c - DF·vl) / (c - DF·vs)`,
/// with `vs` and `vl` the source's and the listener's speeds along the line
/// from the source to the listener, `c` the speed of sound and `DF` the
/// source's `dopplerFactor` times the listener's. Each `DF·v` is held
/// within ±c/2, so the ratio stays within 1/3..3. Exactly 1 when nothing
/// moves along that line or `DF` is 0.
fn doppler(ear: &Ear, p: Vec3, v: Vec3, s: &AudioSource, speed: f32) -> f32 {
    let df = s.doppler_factor.max(0.0) * ear.doppler;
    if !(df > 0.0 && speed > 0.0) {
        return 1.0;
    }
    let d = ear.position - p;
    let dist = d.length();
    if dist <= 1e-6 {
        return 1.0;
    }
    let u = d / dist;
    let half = 0.5 * speed;
    let vs = (df * v.dot(u)).clamp(-half, half);
    let vl = (df * ear.velocity.dot(u)).clamp(-half, half);
    (speed - vl) / (speed - vs)
}

/// Web Audio's PannerNode cone gain (three.js PositionalAudio's
/// `setDirectionalCone`): 1 while the listener is within half of
/// `coneInnerAngle` of the source's forward (its world +Z, as three.js
/// orients a PositionalAudio), `coneOuterGain` past half of
/// `coneOuterAngle`, linear in the angle between. Exactly 1 for the
/// default 360/360. The angle is og-core's deterministic `atan2`.
fn cone_gain(s: &AudioSource, t: &Transform, ear: Vec3) -> f32 {
    let inner = s.cone_inner_angle.clamp(0.0, 360.0);
    let outer = s.cone_outer_angle.clamp(0.0, 360.0);
    if inner >= 360.0 && outer >= 360.0 {
        return 1.0;
    }
    let to = (ear - t.position).normalize_or_zero();
    let forward = (t.quaternion.normalize() * Vec3::Z).normalize_or_zero();
    if to == Vec3::ZERO || forward == Vec3::ZERO {
        return 1.0;
    }
    let c = to.dot(forward).clamp(-1.0, 1.0);
    let angle = og_core::math::atan2((1.0 - c * c).max(0.0).sqrt(), c).to_degrees();
    let (hi, ho) = (inner * 0.5, outer * 0.5);
    let outer_gain = s.cone_outer_gain.clamp(0.0, 1.0);
    if angle <= hi {
        1.0
    } else if angle >= ho {
        outer_gain
    } else {
        let x = (angle - hi) / (ho - hi);
        (1.0 - x) + outer_gain * x
    }
}

/// Gain and pan of a spatial source at `p` (Web Audio's inverse distance
/// model, equal-power pan by the source's direction in the listener's frame).
fn spatialise(ear: &Ear, p: Vec3, s: &AudioSource) -> (f32, f32) {
    let d = p - ear.position;
    let dist = d.length();
    let refd = s.ref_distance.max(0.0);
    let maxd = s.max_distance.max(refd);
    let clamped = dist.clamp(refd, maxd);
    let denom = refd + s.rolloff_factor.max(0.0) * (clamped - refd);
    let attenuation = if denom > 0.0 { refd / denom } else { 1.0 };
    let pan = if dist > 1e-6 {
        let local = ear.inverse * d;
        (local.x / dist).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    (attenuation * ear.volume, pan)
}

/// What one playing source sends this tick.
#[derive(Clone, Copy)]
struct Wanted {
    gain: f32,
    pan: f32,
    pitch: f32,
    bus: u32,
    flags: u32,
    loop_start: u32,
    loop_end: u32,
    lowpass: f32,
    /// The source's world position (a spatial one's; zero otherwise).
    position: Vec3,
    /// The HRTF panner's direction in the listener's frame; zero on the
    /// stereo panner.
    direction: Vec3,
}

/// The lower of two cutoffs, 0 being open.
fn min_cutoff(a: f32, b: f32) -> f32 {
    match (a > 0.0, b > 0.0) {
        (true, true) => a.min(b),
        (true, false) => a,
        _ => b,
    }
}

/// A spatial source's cutoff by distance: 20 kHz at `refDistance`, `at_max`
/// at `maxDistance`, log-linear between.
fn distance_cutoff(dist: f32, s: &AudioSource) -> f32 {
    if s.distance_cutoff <= 0.0 {
        return 0.0;
    }
    let refd = s.ref_distance.max(0.0);
    let maxd = s.max_distance.max(refd + 1e-3);
    let t = ((dist - refd) / (maxd - refd)).clamp(0.0, 1.0);
    if t <= 0.0 {
        return 0.0;
    }
    let (lo, hi) = (s.distance_cutoff.clamp(20.0, 20_000.0).ln(), 20_000f32.ln());
    (hi + (lo - hi) * t).exp()
}

/// Where a source is heard from this tick, beside its own record: its world
/// position last tick (none on its first) and the tick rate, for Doppler.
#[derive(Clone, Copy)]
struct Motion {
    prev: Option<Vec3>,
    tick_rate: u32,
}

#[allow(clippy::too_many_arguments)]
fn wanted(
    ctx: &SystemContext<'_>,
    places: &Places,
    e: Entity,
    s: &AudioSource,
    ear: &Ear,
    audio: &Audio,
    occluder: Option<&Occluder>,
    motion: Motion,
    rate: u32,
    frames: u64,
) -> Wanted {
    let (mut gain, mut pan) = (s.volume.max(0.0), s.pan.clamp(-1.0, 1.0));
    let mut flags = if s.looping { FLAG_LOOP as u32 } else { 0 };
    let mut lowpass = if s.lowpass > 0.0 {
        s.lowpass.clamp(20.0, 20_000.0)
    } else {
        0.0
    };
    let mut pitch = clamp_pitch(s.pitch);
    let (mut position, mut direction) = (Vec3::ZERO, Vec3::ZERO);
    if s.spatial {
        let t = places.world(ctx, e).unwrap_or_default();
        let p = t.position;
        position = p;
        let (g, sp) = spatialise(ear, p, s);
        gain *= g * cone_gain(s, &t, ear.position);
        pan = sp;
        let speed = audio.speed_of_sound;
        let v = motion
            .prev
            .map_or(Vec3::ZERO, |q| tick_velocity(q, p, motion.tick_rate, speed));
        pitch = clamp_pitch(s.pitch * doppler(ear, p, v, s, speed));
        if ear.hrtf {
            direction = (ear.inverse * (p - ear.position)).normalize_or_zero();
            if direction == Vec3::ZERO {
                // At the listener: straight ahead.
                direction = Vec3::NEG_Z;
            }
        }
        flags |= FLAG_SPATIAL as u32;
        lowpass = min_cutoff(lowpass, distance_cutoff(p.distance(ear.position), s));
        if s.occlude && occluder.is_some_and(|o| o(p, ear.position)) {
            gain *= audio.occlusion_gain.clamp(0.0, 1.0);
            lowpass = min_cutoff(lowpass, audio.occlusion_cutoff.clamp(20.0, 20_000.0));
        }
    }
    let to_frames = |sec: f32| -> u32 {
        let f = (f64::from(sec.max(0.0)) * f64::from(rate)).round();
        f.min(frames as f64) as u32
    };
    let loop_start = to_frames(s.loop_start);
    let mut loop_end = if s.loop_end > 0.0 {
        to_frames(s.loop_end)
    } else {
        frames as u32
    };
    if loop_end <= loop_start {
        loop_end = frames as u32;
    }
    let loop_start = loop_start.min(loop_end.saturating_sub(1));
    Wanted {
        gain,
        pan,
        pitch,
        bus: s.bus.min(BUS_COUNT as u32 - 1),
        flags,
        loop_start,
        loop_end,
        lowpass,
        position,
        direction,
    }
}

/// Pitch as the mixer will use it.
pub fn clamp_pitch(p: f32) -> f32 {
    if p.is_finite() {
        p.clamp(1.0 / 16.0, 16.0)
    } else {
        1.0
    }
}

/// The `hrtf` command that puts `voice` at `direction` (zero: back on the
/// stereo panner).
fn hrtf_command(tick: u64, voice: u32, direction: Vec3) -> Command {
    let mut c = Command::new(Op::Hrtf, tick);
    c.voice = voice;
    c.gain = direction.x;
    c.pitch = direction.y;
    c.pan = direction.z;
    c
}

fn voice_command(op: Op, tick: u64, voice: u32, sound: u64, w: &Wanted, position: f64) -> Command {
    Command {
        op,
        flags: w.flags as u8,
        bus: w.bus as u8,
        voice,
        tick,
        sound,
        gain: w.gain,
        pitch: w.pitch,
        pan: w.pan,
        loop_start: w.loop_start,
        loop_end: w.loop_end,
        lowpass: w.lowpass,
        position,
    }
}

/// The playhead after one tick: frames of the sound per tick at this pitch.
/// Returns the new cursor and whether a one-shot reached its end.
fn advance(cursor: f64, rate: u32, tick_rate: u32, w: &Wanted, frames: u64) -> (f64, bool) {
    let step = f64::from(rate) * f64::from(w.pitch) / f64::from(tick_rate);
    let mut c = cursor + step;
    if w.flags & FLAG_LOOP as u32 != 0 {
        let (a, b) = (f64::from(w.loop_start), f64::from(w.loop_end));
        if b > a && c >= b {
            c = a + (c - a) % (b - a);
        }
        (c, false)
    } else {
        (c, c >= frames as f64)
    }
}

fn ended(ctx: &SystemContext<'_>, e: Entity, p: &AudioPlayback, reason: u32) -> Result<()> {
    ctx.send(SoundEnded {
        entity: e.to_bits(),
        sound: p.sound,
        voice: p.voice,
        reason,
    })
}

/// Every bus's effect parameters as the mixer takes them ([`fx`] ids),
/// with the shared reverb's room and damping on master.
pub(crate) fn fx_state(
    chosen: &[Option<AudioBusFx>; BUS_COUNT],
    audio: &Audio,
) -> Vec<(u8, u32, f32)> {
    let mut out = Vec::with_capacity(BUS_COUNT * 13 + 3);
    for (b, f) in chosen.iter().enumerate() {
        let f = f.unwrap_or_default();
        let b = b as u8;
        let duck_key = if f.duck {
            f.duck_bus.min(BUS_COUNT as u32 - 1) + 1
        } else {
            0
        };
        out.extend_from_slice(&[
            (b, fx::LOWPASS, f.lowpass.max(0.0)),
            (b, fx::HIGHPASS, f.highpass.max(0.0)),
            (b, fx::REVERB, f.reverb.clamp(0.0, 1.0)),
            (b, fx::THRESHOLD, f.threshold.min(0.0)),
            (b, fx::RATIO, f.ratio.max(1.0)),
            (b, fx::ATTACK, f.attack.max(0.0)),
            (b, fx::RELEASE, f.release.max(0.0)),
            (b, fx::MAKEUP, f.makeup),
            (b, fx::DUCK_KEY, duck_key as f32),
            (b, fx::DUCK_DEPTH, f.duck_depth.max(0.0)),
            (b, fx::DUCK_THRESHOLD, f.duck_threshold),
            (b, fx::DUCK_ATTACK, f.duck_attack.max(0.0)),
            (b, fx::DUCK_RELEASE, f.duck_release.max(0.0)),
        ]);
    }
    out.push((0, fx::ROOM, audio.reverb_room.clamp(0.0, 1.0)));
    out.push((0, fx::DAMPING, audio.reverb_damping.clamp(0.0, 1.0)));
    out.push((0, fx::RESAMPLER, (audio.resampler != 0) as u32 as f32));
    out
}

pub(crate) fn fx_hash(state: &[(u8, u32, f32)]) -> u64 {
    let mut h = StableHasher::new();
    for (b, k, v) in state {
        h.write_u32(u32::from(*b) << 16 | *k);
        h.write_u32(v.to_bits());
    }
    h.finish()
}

fn fx_commands(state: &[(u8, u32, f32)], tick: u64) -> impl Iterator<Item = Command> + '_ {
    state.iter().map(move |&(b, k, v)| {
        let mut c = Command::new(Op::Fx, tick);
        c.bus = b;
        c.voice = k;
        c.gain = v;
        c
    })
}

/// A voice that plays on this tick, for the voice cap.
struct Sounding {
    e: Entity,
    voice: u32,
    priority: u32,
    gain: f32,
    /// Virtual now (past the cap with `virtualize`): silent in the mixer.
    is_virtual: bool,
    /// Its source's `virtualize`.
    virtualize: bool,
    /// The play that makes it heard again at its playhead (the tick's start).
    play: Command,
    direction: Vec3,
}

/// A source waiting to start this tick.
struct Fresh {
    e: Entity,
    s: AudioSource,
    w: Wanted,
    rate: u32,
    frames: u64,
    /// Seconds into the tick it starts at.
    delay: f64,
    /// Past the cap with `virtualize`: it starts virtual (its playhead runs, unheard).
    is_virtual: bool,
}

/// A voice's standing under the voice cap: higher priority, then louder,
/// then newer is kept.
type Standing = (u32, u32, u64);

fn standing(priority: u32, gain: f32, newness: u64) -> Standing {
    (priority, gain.max(0.0).to_bits(), newness)
}

/// The reverb zones around a listener at `ear` (query 5): each zone's
/// weight written back when it changed, and the strongest (ties to the
/// first in query order) easing the shared reverb's room and damping in
/// `room` and its bus's send in `chosen` towards its own by its weight.
fn zones(
    ctx: &SystemContext<'_>,
    places: &Places,
    ear: Vec3,
    chosen: &mut [Option<AudioBusFx>; BUS_COUNT],
    room: &mut Audio,
) -> Result<()> {
    let entities: Vec<Entity> = ctx.query(5)?.flat_map(|a| a.entities().to_vec()).collect();
    let mut best: Option<(ReverbZone, f32)> = None;
    for e in entities {
        let z = ctx.get::<ReverbZone>(e)?;
        let w = z.weight_at(&places.world(ctx, e)?, ear);
        if w.to_bits() != z.weight.to_bits() {
            ctx.set(e, ReverbZone { weight: w, ..z })?;
        }
        if w > 0.0 && best.is_none_or(|(_, b)| w > b) {
            best = Some((z, w));
        }
    }
    let Some((z, w)) = best else {
        return Ok(());
    };
    let ease = |open: f32, zone: f32| open + (zone.clamp(0.0, 1.0) - open) * w;
    room.reverb_room = ease(room.reverb_room.clamp(0.0, 1.0), z.room);
    room.reverb_damping = ease(room.reverb_damping.clamp(0.0, 1.0), z.damping);
    let b = z.bus.min(BUS_COUNT as u32 - 1) as usize;
    let mut f = chosen[b].unwrap_or(AudioBusFx {
        bus: b as u32,
        ..AudioBusFx::default()
    });
    f.reverb = ease(f.reverb.clamp(0.0, 1.0), z.send);
    chosen[b] = Some(f);
    Ok(())
}

fn run(ctx: &mut SystemContext<'_>, link: &AudioLink, occluder: Option<&Occluder>) -> Result<()> {
    let tick = ctx.time().tick;
    let resync = link.begin(tick);
    let bank = link.bank.clone();
    let mut stream = *ctx.resource::<AudioStream>()?;
    let audio = *ctx.resource::<Audio>()?;
    let tick_rate = stream.tick_rate.max(1);
    let t0 = tick as f64 / f64::from(tick_rate);
    let t1 = (tick + 1) as f64 / f64::from(tick_rate);
    let mut commands: Vec<Command> = Vec::new();
    // In a resync tick: the voices that sound during this tick, at their
    // playheads at its start.
    let mut snapshot: Vec<Command> = Vec::new();
    let mut live: Vec<u32> = Vec::new();

    // Bus gains.
    let gains = audio.gains();
    let sent = stream.bus_sent.to_array();
    for (b, (&g, &was)) in gains.iter().zip(sent.iter()).enumerate() {
        if g != was {
            let mut c = Command::new(Op::Bus, tick);
            c.bus = b as u8;
            c.gain = g.max(0.0);
            commands.push(c);
        }
    }
    stream.bus_sent = Vec4::from_array(gains.map(|g| g.max(0.0)));

    // The listener.
    let mut ear = Ear {
        position: Vec3::ZERO,
        inverse: Quat::IDENTITY,
        volume: 1.0,
        velocity: Vec3::ZERO,
        doppler: 1.0,
        hrtf: false,
    };
    let places = Places::read(ctx)?;
    let listener = ctx.query(3)?.flat_map(|a| a.entities().to_vec()).next();
    if let Some(e) = listener {
        let t = places.world(ctx, e)?;
        let l = ctx.get::<AudioListener>(e)?;
        ear.position = t.position;
        ear.inverse = t.quaternion.normalize().inverse();
        ear.volume = l.volume.max(0.0);
        ear.doppler = l.doppler_factor.max(0.0);
        ear.hrtf = l.panning_model == panning_model::HRTF;
        if stream.listener == e.to_bits() {
            ear.velocity = tick_velocity(
                stream.listener_at,
                t.position,
                tick_rate,
                audio.speed_of_sound,
            );
        }
        stream.listener = e.to_bits();
        stream.listener_at = t.position;
    } else {
        stream.listener = 0;
        stream.listener_at = Vec3::ZERO;
    }

    // Bus effects: every parameter again when any changed.
    let mut chosen: [Option<AudioBusFx>; BUS_COUNT] = [None; BUS_COUNT];
    let fx_entities: Vec<Entity> = ctx.query(4)?.flat_map(|a| a.entities().to_vec()).collect();
    for e in fx_entities {
        let f = ctx.get::<AudioBusFx>(e)?;
        let b = f.bus.min(BUS_COUNT as u32 - 1) as usize;
        if chosen[b].is_none() {
            chosen[b] = Some(f);
        }
    }
    let mut room = audio;
    zones(ctx, &places, ear.position, &mut chosen, &mut room)?;
    let fx_now = fx_state(&chosen, &room);
    let fx_now_hash = fx_hash(&fx_now);
    if fx_now_hash != stream.fx {
        commands.extend(fx_commands(&fx_now, tick));
        stream.fx = fx_now_hash;
    }

    // Playheads whose AudioSource was removed.
    let orphans: Vec<Entity> = ctx.query(2)?.flat_map(|a| a.entities().to_vec()).collect();
    for e in orphans {
        let p = ctx.get::<AudioPlayback>(e)?;
        let mut c = Command::new(Op::Stop, tick);
        c.voice = p.voice;
        commands.push(c);
        ended(ctx, e, &p, end_reason::REMOVED)?;
        ctx.remove::<AudioPlayback>(e)?;
    }

    // Playing sources.
    let playing: Vec<Entity> = ctx.query(0)?.flat_map(|a| a.entities().to_vec()).collect();
    let mut to_start: Vec<Entity> = Vec::new();
    // The voices that play on, heard or virtual.
    let mut sounding: Vec<Sounding> = Vec::new();
    for e in playing {
        let s = ctx.get::<AudioSource>(e)?;
        let mut p = ctx.get::<AudioPlayback>(e)?;
        let stop = |commands: &mut Vec<Command>, voice: u32, fade: f32| {
            let mut c = Command::new(Op::Stop, tick);
            c.voice = voice;
            c.position = f64::from(fade.max(0.0));
            commands.push(c);
        };
        if s.retrigger {
            // From the top in this tick: this voice ends, a new one starts.
            if !p.is_virtual {
                stop(&mut commands, p.voice, s.fade);
            }
            ended(ctx, e, &p, end_reason::RETRIGGERED)?;
            ctx.remove::<AudioPlayback>(e)?;
            ctx.set(
                e,
                AudioSource {
                    playing: true,
                    retrigger: false,
                    ..s
                },
            )?;
            to_start.push(e);
            continue;
        }
        if !s.playing {
            stop(&mut commands, p.voice, s.fade);
            ended(ctx, e, &p, end_reason::STOPPED)?;
            ctx.remove::<AudioPlayback>(e)?;
            continue;
        }
        let info = if s.sound == p.sound {
            bank.info(s.sound)
        } else {
            None
        };
        let Some((rate, frames)) = info else {
            // Another sound (or this one left the bank): end this voice and
            // start the new sound as a fresh source.
            stop(&mut commands, p.voice, 0.0);
            ended(ctx, e, &p, end_reason::REPLACED)?;
            ctx.remove::<AudioPlayback>(e)?;
            to_start.push(e);
            continue;
        };
        let motion = Motion {
            // A source that was not spatial last tick has no position yet.
            prev: (p.flags & FLAG_SPATIAL as u32 != 0).then_some(p.position),
            tick_rate,
        };
        let w = wanted(
            ctx, &places, e, &s, &ear, &audio, occluder, motion, rate, frames,
        );
        let changed = w.gain != p.gain
            || w.pan != p.pan
            || w.pitch != p.pitch
            || w.bus != p.bus
            || w.flags != p.flags
            || w.loop_start != p.loop_start
            || w.loop_end != p.loop_end
            || w.lowpass != p.lowpass;
        // A virtual voice is not in the mixer: it is told nothing until it
        // is heard again (then a play at its playhead, below).
        let heard = !p.is_virtual;
        if changed && heard {
            commands.push(voice_command(
                Op::Params,
                tick,
                p.voice,
                p.sound,
                &w,
                f64::from(s.fade.max(0.0)),
            ));
        }
        if w.direction != p.direction && heard {
            commands.push(hrtf_command(tick, p.voice, w.direction));
        }
        let play = voice_command(Op::Play, tick, p.voice, p.sound, &w, p.cursor);
        if resync && heard {
            snapshot.push(voice_command(
                Op::Play,
                tick,
                p.voice,
                p.sound,
                &w,
                p.cursor,
            ));
            if w.direction != Vec3::ZERO {
                snapshot.push(hrtf_command(tick, p.voice, w.direction));
            }
        }
        let (cursor, done) = advance(p.cursor, rate, tick_rate, &w, frames);
        if done {
            ended(ctx, e, &p, end_reason::FINISHED)?;
            ctx.set(
                e,
                AudioSource {
                    playing: false,
                    ..s
                },
            )?;
            ctx.remove::<AudioPlayback>(e)?;
            continue;
        }
        if heard {
            live.push(p.voice);
        }
        sounding.push(Sounding {
            e,
            voice: p.voice,
            priority: s.priority,
            gain: w.gain,
            is_virtual: p.is_virtual,
            virtualize: s.virtualize,
            play,
            direction: w.direction,
        });
        p.cursor = cursor;
        p.gain = w.gain;
        p.pan = w.pan;
        p.pitch = w.pitch;
        p.bus = w.bus;
        p.flags = w.flags;
        p.loop_start = w.loop_start;
        p.loop_end = w.loop_end;
        p.lowpass = w.lowpass;
        p.position = w.position;
        p.direction = w.direction;
        ctx.set(e, p)?;
    }

    // Sources to start: new ones, then replaced ones; a source whose
    // startAt is after this tick waits.
    let mut fresh_entities: Vec<Entity> =
        ctx.query(1)?.flat_map(|a| a.entities().to_vec()).collect();
    fresh_entities.extend(to_start);
    let mut fresh: Vec<Fresh> = Vec::new();
    for e in fresh_entities {
        let mut s = ctx.get::<AudioSource>(e)?;
        if s.retrigger {
            // Not playing (stopped, ended, culled or never started): it
            // plays from the top, `playing` set back to true.
            s.retrigger = false;
            s.playing = true;
            ctx.set(e, s)?;
        }
        if !s.playing || s.start_at >= t1 {
            continue;
        }
        // A sound not (yet) in the bank waits: it starts on the first tick it is there.
        let Some((rate, frames)) = bank.info(s.sound) else {
            continue;
        };
        let motion = Motion {
            prev: None,
            tick_rate,
        };
        let w = wanted(
            ctx, &places, e, &s, &ear, &audio, occluder, motion, rate, frames,
        );
        let delay = if s.start_at > t0 {
            s.start_at - t0
        } else {
            0.0
        };
        fresh.push(Fresh {
            e,
            s,
            w,
            rate,
            frames,
            delay,
            is_virtual: false,
        });
    }

    // The voice cap: the lowest standing is culled (or goes virtual, with
    // `virtualize`), playing or not yet started; a virtual voice back
    // within the cap is heard again from its playhead.
    let cap = audio.max_voices.max(1) as usize;
    if sounding.len() + fresh.len() > cap || sounding.iter().any(|v| v.is_virtual) {
        // (standing, index into sounding, index into fresh)
        let mut all: Vec<(Standing, Option<usize>, Option<usize>)> = Vec::new();
        for (i, v) in sounding.iter().enumerate() {
            all.push((
                standing(v.priority, v.gain, u64::from(v.voice)),
                Some(i),
                None,
            ));
        }
        for (i, f) in fresh.iter().enumerate() {
            all.push((
                standing(f.s.priority, f.w.gain, u64::MAX / 2 + i as u64),
                None,
                Some(i),
            ));
        }
        all.sort_by_key(|a| std::cmp::Reverse(a.0));
        let mut cull_fresh = vec![false; fresh.len()];
        for (rank, (_, playing, starting)) in all.into_iter().enumerate() {
            let within = rank < cap;
            if let Some(i) = playing {
                let v = &sounding[i];
                let set_virtual = |ctx: &SystemContext<'_>, is_virtual: bool| -> Result<()> {
                    let p = ctx.get::<AudioPlayback>(v.e)?;
                    ctx.set(v.e, AudioPlayback { is_virtual, ..p })
                };
                if within && v.is_virtual {
                    // Heard again where it would have been (a resumed play fades in).
                    commands.push(v.play);
                    if resync {
                        snapshot.push(v.play);
                    }
                    if v.direction != Vec3::ZERO {
                        let h = hrtf_command(tick, v.voice, v.direction);
                        commands.push(h);
                        if resync {
                            snapshot.push(h);
                        }
                    }
                    live.push(v.voice);
                    set_virtual(ctx, false)?;
                } else if !within && !v.is_virtual {
                    let mut c = Command::new(Op::Stop, tick);
                    c.voice = v.voice;
                    commands.push(c);
                    live.retain(|x| *x != v.voice);
                    snapshot.retain(|c| c.voice != v.voice);
                    if v.virtualize {
                        set_virtual(ctx, true)?;
                    } else {
                        let p = ctx.get::<AudioPlayback>(v.e)?;
                        ended(ctx, v.e, &p, end_reason::CULLED)?;
                        let s = ctx.get::<AudioSource>(v.e)?;
                        ctx.set(
                            v.e,
                            AudioSource {
                                playing: false,
                                ..s
                            },
                        )?;
                        ctx.remove::<AudioPlayback>(v.e)?;
                    }
                }
            }
            if let Some(i) = starting
                && !within
            {
                if fresh[i].s.virtualize {
                    fresh[i].is_virtual = true;
                } else {
                    cull_fresh[i] = true;
                }
            }
        }
        let mut k = 0;
        fresh.retain(|f| {
            let keep = !cull_fresh[k];
            k += 1;
            if !keep {
                let _ = ctx.send(SoundEnded {
                    entity: f.e.to_bits(),
                    sound: f.s.sound,
                    voice: 0,
                    reason: end_reason::CULLED,
                });
                let _ = ctx.set(
                    f.e,
                    AudioSource {
                        playing: false,
                        ..f.s
                    },
                );
            }
            keep
        });
    }

    for Fresh {
        e,
        s,
        w,
        rate,
        frames,
        delay,
        is_virtual,
    } in fresh
    {
        let voice = stream.next_voice;
        stream.next_voice = stream.next_voice.wrapping_add(1).max(1);
        // A start inside the tick: that many frames of the sound later.
        let position = -(delay * f64::from(rate) * f64::from(w.pitch));
        // A start part-way into the sound (startOffset): a cue after the play.
        let cue = if s.start_offset > 0.0 && s.start_offset.is_finite() {
            s.start_offset * f64::from(rate)
        } else {
            0.0
        };
        let mut started = Vec::with_capacity(3);
        if s.fade > 0.0 {
            // Fade in: start silent, then ramp to the gain.
            let silent = Wanted { gain: 0.0, ..w };
            started.push(voice_command(
                Op::Play,
                tick,
                voice,
                s.sound,
                &silent,
                position,
            ));
            started.push(voice_command(
                Op::Params,
                tick,
                voice,
                s.sound,
                &w,
                f64::from(s.fade),
            ));
        } else {
            started.push(voice_command(Op::Play, tick, voice, s.sound, &w, position));
        }
        if cue > 0.0 {
            started.insert(1, voice_command(Op::Cue, tick, voice, s.sound, &w, cue));
        }
        if w.direction != Vec3::ZERO {
            started.push(hrtf_command(tick, voice, w.direction));
        }
        if !is_virtual {
            commands.extend_from_slice(&started);
            if resync {
                snapshot.extend_from_slice(&started);
            }
        }
        ctx.send(SoundStarted {
            entity: e.to_bits(),
            sound: s.sound,
            voice,
        })?;
        // The playhead as if it had started `cue` frames earlier: the sound
        // reaches frame f at delay + (f - cue) / rate, so a one-shot ends on
        // the tick its rest ends in.
        let (cursor, done) = advance(position + cue, rate, tick_rate, &w, frames);
        if done {
            // Shorter than one tick: it starts and ends in this tick.
            ctx.send(SoundEnded {
                entity: e.to_bits(),
                sound: s.sound,
                voice,
                reason: end_reason::FINISHED,
            })?;
            ctx.set(
                e,
                AudioSource {
                    playing: false,
                    ..s
                },
            )?;
            continue;
        }
        if !is_virtual {
            live.push(voice);
        }
        ctx.insert(
            e,
            AudioPlayback {
                voice,
                sound: s.sound,
                cursor,
                gain: w.gain,
                pan: w.pan,
                pitch: w.pitch,
                bus: w.bus,
                flags: w.flags,
                loop_start: w.loop_start,
                loop_end: w.loop_end,
                lowpass: w.lowpass,
                position: w.position,
                direction: w.direction,
                is_virtual,
            },
        )?;
    }

    // The stream hash covers every tick that has commands.
    if !commands.is_empty() {
        let mut h = StableHasher::new();
        h.write_u64(stream.hash);
        h.write(&encode(&commands));
        stream.hash = h.finish();
        stream.commands += commands.len() as u64;
    }
    live.sort_unstable();
    stream.voices = live.len() as u32;
    *ctx.resource_mut::<AudioStream>()? = stream;

    let packet = if resync {
        let mut all = vec![Command::new(Op::StopAll, tick)];
        for (b, &g) in stream.bus_sent.to_array().iter().enumerate() {
            let mut c = Command::new(Op::Bus, tick);
            c.bus = b as u8;
            c.gain = g;
            all.push(c);
        }
        all.extend(fx_commands(&fx_now, tick));
        all.extend(snapshot);
        TickPacket {
            tick,
            commands: all,
            live: None,
            resync: true,
        }
    } else {
        TickPacket {
            tick,
            commands,
            live: None,
            resync: false,
        }
    };
    link.publish(packet, live);
    Ok(())
}
