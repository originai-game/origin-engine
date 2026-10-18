//! The tick side: `install` registers the records and the `og.audio`
//! system, which turns AudioSource changes into the tick's commands, keeps
//! each playing source's playhead (so `SoundEnded` lands on a tick that only
//! depends on the world), and hands the commands to an attached sink.

use crate::command::{Command, FLAG_LOOP, FLAG_SPATIAL, Op, TickPacket, encode};
use crate::schema::{
    Audio, AudioListener, AudioPlayback, AudioSource, AudioStream, BUS_COUNT, SoundEnded,
    SoundStarted, Transform, end_reason,
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

/// How audio joins a world.
#[derive(Clone, Debug, Default)]
pub struct AudioDesc {
    /// The bank sources' `sound` handles index; a fresh one when `None`.
    pub bank: Option<SoundBank>,
    /// Continue this link (and its bank) instead of making a new one, so a
    /// sink attached to it keeps receiving: a hot reload's rebuilt world
    /// passes the old world's link. Ignored when its tick rate differs.
    pub link: Option<AudioLink>,
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
/// `og.AudioPlayback` and `og.AudioStream`, and `og.Transform` when absent),
/// inserts the resources, and appends the `og.audio` system to the schedule.
/// Call it after the game's systems so the system sees the tick's changes.
pub fn install(world: &mut World, desc: &AudioDesc) -> Result<AudioLink> {
    world.register::<Transform>()?;
    world.register::<AudioSource>()?;
    world.register::<AudioListener>()?;
    world.register::<Audio>()?;
    world.register::<SoundStarted>()?;
    world.register::<SoundEnded>()?;
    world.register::<AudioPlayback>()?;
    world.register::<AudioStream>()?;
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
        },
    };
    let system_link = link.clone();
    world.add_system(
        SystemDesc::new(SYSTEM_NAME)
            .read::<Audio>()
            .read::<AudioListener>()
            .read::<Transform>()
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
            .query(QueryDesc::new().read::<AudioListener>().read::<Transform>()),
        move |ctx| run(ctx, &system_link),
    )?;
    Ok(link)
}

/// The listener's frame: position, rotation, volume.
#[derive(Clone, Copy)]
struct Ear {
    position: Vec3,
    inverse: Quat,
    volume: f32,
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
struct Wanted {
    gain: f32,
    pan: f32,
    pitch: f32,
    bus: u32,
    flags: u32,
    loop_start: u32,
    loop_end: u32,
}

fn wanted(
    ctx: &SystemContext<'_>,
    e: Entity,
    s: &AudioSource,
    ear: &Ear,
    rate: u32,
    frames: u64,
) -> Wanted {
    let (mut gain, mut pan) = (s.volume.max(0.0), s.pan.clamp(-1.0, 1.0));
    let mut flags = if s.looping { FLAG_LOOP as u32 } else { 0 };
    if s.spatial {
        let p = ctx
            .get::<Transform>(e)
            .map(|t| t.position)
            .unwrap_or(Vec3::ZERO);
        let (g, sp) = spatialise(ear, p, s);
        gain *= g;
        pan = sp;
        flags |= FLAG_SPATIAL as u32;
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
        pitch: clamp_pitch(s.pitch),
        bus: s.bus.min(BUS_COUNT as u32 - 1),
        flags,
        loop_start,
        loop_end,
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

fn run(ctx: &mut SystemContext<'_>, link: &AudioLink) -> Result<()> {
    let tick = ctx.time().tick;
    let resync = link.begin(tick);
    let bank = link.bank.clone();
    let mut stream = *ctx.resource::<AudioStream>()?;
    let audio = *ctx.resource::<Audio>()?;
    let tick_rate = stream.tick_rate.max(1);
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
    };
    let listener = ctx.query(3)?.flat_map(|a| a.entities().to_vec()).next();
    if let Some(e) = listener {
        let t = ctx.get::<Transform>(e)?;
        ear.position = t.position;
        ear.inverse = t.quaternion.normalize().inverse();
        ear.volume = ctx.get::<AudioListener>(e)?.volume.max(0.0);
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
    for e in playing {
        let s = ctx.get::<AudioSource>(e)?;
        let mut p = ctx.get::<AudioPlayback>(e)?;
        let stop = |commands: &mut Vec<Command>, voice: u32| {
            let mut c = Command::new(Op::Stop, tick);
            c.voice = voice;
            commands.push(c);
        };
        if !s.playing {
            stop(&mut commands, p.voice);
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
            stop(&mut commands, p.voice);
            ended(ctx, e, &p, end_reason::REPLACED)?;
            ctx.remove::<AudioPlayback>(e)?;
            to_start.push(e);
            continue;
        };
        let w = wanted(ctx, e, &s, &ear, rate, frames);
        let changed = w.gain != p.gain
            || w.pan != p.pan
            || w.pitch != p.pitch
            || w.bus != p.bus
            || w.flags != p.flags
            || w.loop_start != p.loop_start
            || w.loop_end != p.loop_end;
        if changed {
            commands.push(voice_command(Op::Params, tick, p.voice, p.sound, &w, 0.0));
        }
        if resync {
            snapshot.push(voice_command(
                Op::Play,
                tick,
                p.voice,
                p.sound,
                &w,
                p.cursor,
            ));
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
        live.push(p.voice);
        p.cursor = cursor;
        p.gain = w.gain;
        p.pan = w.pan;
        p.pitch = w.pitch;
        p.bus = w.bus;
        p.flags = w.flags;
        p.loop_start = w.loop_start;
        p.loop_end = w.loop_end;
        ctx.set(e, p)?;
    }

    // Sources to start: new ones, then replaced ones.
    let mut fresh: Vec<Entity> = ctx.query(1)?.flat_map(|a| a.entities().to_vec()).collect();
    fresh.extend(to_start);
    for e in fresh {
        let s = ctx.get::<AudioSource>(e)?;
        if !s.playing {
            continue;
        }
        // A sound not (yet) in the bank waits: it starts on the first tick it is there.
        let Some((rate, frames)) = bank.info(s.sound) else {
            continue;
        };
        let w = wanted(ctx, e, &s, &ear, rate, frames);
        let voice = stream.next_voice;
        stream.next_voice = stream.next_voice.wrapping_add(1).max(1);
        let play = voice_command(Op::Play, tick, voice, s.sound, &w, 0.0);
        commands.push(play);
        if resync {
            snapshot.push(play);
        }
        ctx.send(SoundStarted {
            entity: e.to_bits(),
            sound: s.sound,
            voice,
        })?;
        let (cursor, done) = advance(0.0, rate, tick_rate, &w, frames);
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
        live.push(voice);
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
