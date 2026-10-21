# og-audio

Audio for the engine (`docs/design.md` §2.2, §2.7; phase 1 M6): a tick-side
system that turns component changes into a deterministic, time-stamped
**command stream**, and a **mixer** that renders that stream to PCM outside
the tick. The mixer never writes the world; `world_hash` covers the command
stream, never the PCM.

```rust
let link = og_audio::install(&mut world, &og_audio::AudioDesc::default())?;   // after the game's systems
og_audio::publish_link(engine_id, link.clone());                            // so the host's sink finds it
let blip = link.bank().insert_qoa(&cooked.id, &cooked.qoa)?;                // a cooked sound's QOA section
world.spawn((og_audio::AudioSource::new(blip),))?;                         // plays once, on the sfx bus
let mut sink = og_audio::HeadlessSink::new(&link, og_audio::HeadlessMode::Capture { sample_rate: 48_000 });
world.step()?;
sink.pump();                                                                // 800 stereo frames at 60 Hz
```

Pure Rust over og-core and og-ecs; builds for `wasm32-unknown-unknown`. The
QOA codec is og-cook's own `src/qoa.rs`, compiled into this crate by path
(`og_audio::qoa`) so audio needs neither og-cook nor its og-render
dependency (Assets: moving `qoa.rs` into a tiny crate would make this an
ordinary dependency).

## API map

| Module | Entry points |
|---|---|
| `system` | `install(&mut World, &AudioDesc) -> AudioLink`, `AudioLink` (`bank`, `attach`, `detach`, `drain`, `next_tick`, `dropped`), `publish_link`/`link`/`retire_link` (engine id → link), `SYSTEM_NAME` = `og.audio` |
| `schema` | `AudioSource`, `AudioListener`, `Audio`, `ReverbZone` (`weight_at`), `SoundStarted`, `SoundEnded`, `end_reason`, engine-owned `AudioPlayback`, `AudioStream`, the `og.Transform` mirror, `Bus` |
| `command` | `Command`, `Op`, `encode`/`decode` (56-byte records), `TickPacket` |
| `sound` | `SoundBank` (`insert_qoa`, `insert_pcm`, `insert_chunked`, `put_chunk`, `keep_chunks`, `set_chunk_loader`, `resident_bytes`, `get`, `info`, `remove`; `with_resident_seconds`), `SoundData` (`chunking`, `resident_chunks`, `resident_bytes`), `chunks_wanted` (§Chunked sounds) |
| `mixer` | `Mixer::new(MixerDesc, SoundBank)` / `Mixer::for_link`, `push`, `pump`, `render`, `render_until_tick`, `frames_until_tick`, `skip`, `stats`; `Clock::{Driven, Free}` |
| `sink` | `HeadlessSink` (`Null` / `Capture`), `wav(pcm, channels, rate)` |

## Records (for `api.ogt`)

Field order, names and storage types are the layout (`og_core::reflect`);
the IDL must declare them exactly so (an IDL record with the same name and
layout is adopted by `install`). `u32` flags hold 0 or 1 until og-core's
one-byte `bool` and `enum` field types land for component columns; then
`loop`, `spatial` and `playing` become `bool` and `bus` an enum (a layout
change both sides make together). `sound` is a `Sound` handle stored as
`u64` (the IDL needs a `column Sound`, `Uint32Array` stride 2, as for
`Geometry`, and the codec's `Kind::parse` a `Sound` → `U64` arm).

### `og.AudioSource` (component)

| # | field | type | default | range / unit | meaning |
|--:|---|---|---|---|---|
| 0 | `sound` | Sound (u64) | 0 | | the sound; 0 plays nothing; a handle not (yet) in the bank waits and starts on the first tick it is there |
| 1 | `bus` | u32 | 2 | 0–3 | 0 master, 1 music, 2 sfx, 3 ui |
| 2 | `volume` | f32 | 1 | 0–4, tunable | linear gain |
| 3 | `pitch` | f32 | 1 | 1/16–16, tunable | playback rate: 2 is an octave up and half as long |
| 4 | `pan` | f32 | 0 | -1–1, tunable | stereo position of a non-spatial source |
| 5 | `loop` | u32 | 0 | 0/1 | loop between `loopStart` and `loopEnd`, seamlessly |
| 6 | `loopStart` | f32 | 0 | s | where a loop restarts |
| 7 | `loopEnd` | f32 | 0 | s | where a loop wraps; 0 is the sound's end |
| 8 | `spatial` | u32 | 0 | 0/1 | attenuate and pan by the entity's `Transform` relative to the listener |
| 9 | `refDistance` | f32 | 1 | m, tunable | full volume up to here |
| 10 | `maxDistance` | f32 | 100 | m, tunable | no quieter past here |
| 11 | `rolloffFactor` | f32 | 1 | tunable | Web Audio's inverse model: `ref / (ref + rolloff · (clamp(d, ref, max) − ref))` |
| 12 | `playing` | u32 | 1 | 0/1 | level-triggered: 1 plays (inserting a source plays it), 0 stops; the engine sets 0 when a one-shot ends |

### `og.AudioListener` (component)

| # | field | type | default | meaning |
|--:|---|---|---|---|
| 0 | `volume` | f32 | 1 (0–4, tunable) | gain on every spatial source it hears |

Position and orientation come from the entity's `og.Transform` (three.js
convention: the listener looks down −Z, +X is right). The first listener in
query order is used; with none, the origin looking down −Z.

### `og.Audio` (resource)

| # | field | type | default | meaning |
|--:|---|---|---|---|
| 0 | `master` | f32 | 1 (0–2, tunable) | applied to every bus |
| 1 | `music` | f32 | 1 | music bus |
| 2 | `sfx` | f32 | 1 | sfx bus |
| 3 | `ui` | f32 | 1 | ui bus |

### `og.SoundStarted`, `og.SoundEnded` (events)

| # | field | type | meaning |
|--:|---|---|---|
| 0 | `entity` | Entity (u64) | the source's entity |
| 1 | `sound` | Sound (u64) | the sound |
| 2 | `voice` | u32 | the voice id in the command stream |
| 3 | `reason` (`SoundEnded` only) | u32 | 0 finished, 1 stopped (`playing` = 0), 2 removed (the AudioSource left a living entity), 3 replaced (`sound` changed while playing) |

Sent during the tick it happens; systems read them the tick after (og-ecs
event semantics). A despawned entity's voice is silenced without an event.

### Engine-owned (readonly; transcribe as `readonly true` if the probe should show them)

`og.AudioPlayback` (component, present while a source plays): `voice u32`,
`sound u64`, `cursor f64` (playhead at the next tick's start, frames of the
sound), `gain f32`, `pan f32`, `pitch f32`, `bus u32`, `flags u32`,
`loopStart u32`, `loopEnd u32` (what the mixer was last told).

`og.AudioStream` (resource): `hash u64` (the stream hash), `commands u64`,
`nextVoice u32`, `voices u32`, `tickRate u32`, `busSent Vector4` (gains
last sent; negative before the first tick).

## Tick semantics

- `install` appends `og.audio` to the schedule; install it **after** the
  game's systems so it sees the tick's changes. It reads `og.Audio`,
  `og.AudioListener`, `og.Transform`; writes `og.AudioSource` (only to reset
  `playing`), `og.AudioPlayback`, `og.AudioStream` and the two events.
- Per tick, in order: bus commands for changed `og.Audio` gains; stops for
  playheads whose AudioSource was removed; for each playing source (query
  order) stop / replace / params-if-changed, then the playhead advances by
  `rate × pitch / tickRate` frames (wrapping inside the loop region); then
  new sources start, in query order, taking voice ids from
  `AudioStream.nextVoice`.
- A one-shot of `F` frames at rate `R` and pitch `p` started in tick `S`
  sends `SoundEnded` in tick `S + ceil(F·tickRate / (R·p)) − 1`, the tick
  whose audio interval contains its last frame (0.5 s at 60 Hz: tick S+29).
- Spatial gain and pan are computed here, in f32 with glam's scalar math,
  so they are part of the stream and identical on every host. Pan is the
  x component of the listener-space direction (−1 hard left, 1 hard right).
- Every tick with commands chains `FNV-1a(previous hash ‖ encoded
  commands)` into `AudioStream.hash`; the resource and the playheads are
  world state, so snapshots, `restore` and `world_hash` cover the stream.
  Nothing a sink does reaches the world: attaching, resyncing and dropping
  packets leave the hash unchanged.

## Command stream

One 56-byte little-endian record per command (`command.rs`):

| offset | type | field | meaning |
|---:|---|---|---|
| 0 | u8 | `op` | 1 play, 2 stop, 3 params, 4 bus, 5 stop-all |
| 1 | u8 | `flags` | bit 0 loop, bit 1 spatial |
| 2 | u8 | `bus` | 0 master, 1 music, 2 sfx, 3 ui |
| 3 | u8 | reserved | 0 |
| 4 | u32 | `voice` | voice id (0 for bus, stop-all) |
| 8 | u64 | `tick` | the tick whose start it takes effect at |
| 16 | u64 | `sound` | sound handle (play) |
| 24 | f32 | `gain` | voice gain (volume × distance × listener volume), or the bus gain |
| 28 | f32 | `pitch` | playback rate (clamped to 1/16–16) |
| 32 | f32 | `pan` | −1 … 1 |
| 36 | u32 | `loop_start` | frames of the sound |
| 40 | u32 | `loop_end` | frames, exclusive; 0 = the sound's end |
| 44 | u32 | reserved | 0 |
| 48 | f64 | `position` | where a play starts, frames of the sound |

A `TickPacket` is one tick's commands plus, when the set changed, the
sorted voice ids alive after it (`live`; the mixer fades the others at the
tick's **end**, so a one-shot that finished in the tick keeps its tail) and
a `resync` flag. A resync packet — the first after `attach`, after a
`restore` (any gap in ticks) or after the queue overflowed — is stop-all,
the four bus gains, then a play per voice sounding in that tick at its
playhead. Only the regular commands are hashed.

## Mixer

- **Clock**: the first packet anchors its tick to the mixer's frame; tick
  `t` starts at `anchor + (t − t₀) · rate / tickRate`, so commands are
  sample-accurate. `Clock::Driven` (web Worker, null, capture) is rendered
  exactly to the world's next tick, so its PCM is a function of the stream.
  `Clock::Free { latency_frames }` (cpal) is pulled by the device; a packet
  more than 100 ms late or 500 ms early re-anchors.
- **Sounds**: a sound up to 6 s (`DEFAULT_RESIDENT_SECONDS`) is decoded at
  insert and shared; a longer one streams: each voice decodes the QOA frames
  (5120 samples, self-contained) it reaches into two slots, so a frame
  boundary or a loop seam never decodes twice. A sound cooked in chunks
  keeps only some of its QOA bytes in memory (§Chunked sounds).
- **Voices**: linear-interpolation resampling at `pitch × source rate /
  output rate`; at a loop seam the interpolation reads `loopStart` as the
  frame after `loopEnd − 1`, so a sample-continuous loop region is seamless.
  Gain, pan and bus changes ramp over 256 frames; stops fade over 256; a
  resynced voice fades in; a fresh one starts at full gain (attacks kept).
  Mono sources pan with Web Audio's equal-power law, stereo ones with its
  balance law; a source's first two channels are used.
- **Buses** fold into each voice's gain (master × bus). Without any
  `og.AudioBusFx` every voice sums straight into the output; with one, each
  voice sums into its bus, music, sfx and ui run their inserts (12 dB/oct
  high-pass, low-pass, a stereo-linked compressor with makeup, then side-chain
  ducking by their key bus's pre-insert level) and their reverb send, sum
  into master, which runs its own inserts; the shared reverb (Freeverb's
  eight combs and four all-passes per channel, scaled to the rate) returns
  into master before the limiter.
- **Voice filter**: each voice has a one-pole low-pass (its `lowpass`, its
  distance cutoff and its occlusion, the lowest of them) whose coefficient
  glides with the gain ramp.
- **Scheduled starts**: a play at a negative position starts that many
  source frames later (silent, its ramps held), so `startAt` lands on its
  sample inside a tick. `params` and `stop` carry their ramp in seconds
  (`AudioSource.fade`); a bus change keeps a longer fade in progress.
- **Limiter**: stereo-linked peak envelope (instant attack, 100 ms release)
  through a tanh soft knee from 0.8: output peaks stay below 1.0.

## E6: effects, music, priority, occlusion, listen

Records (`schema.rs`, IDL module `audio`):

- `og.AudioSource` gains `priority` (0–255, 128), `lowpass` (Hz), `occlude`,
  `distanceCutoff` (Hz at `maxDistance`, from 20 kHz at `refDistance`,
  log-linear), `startAt` (world seconds, to the sample) and `fade` (s).
- `og.Audio` gains `maxVoices` (32), `occlusionGain` (0.35),
  `occlusionCutoff` (900 Hz), `reverbRoom` (0.6) and `reverbDamping` (0.4).
- `og.AudioBusFx` (component, one per bus): `bus`, `lowpass`, `highpass`,
  `reverb`, `threshold`/`ratio`/`attack`/`release`/`makeup`, `duck`,
  `duckBus`, `duckDepth`, `duckThreshold`, `duckAttack`, `duckRelease`.
- `og.Music` (component): `bpm`, `beatsPerBar`, `start`; written by
  `og.music` (before `og.audio`): `bar`, `beat`, `nextBeat`, `nextBar`
  (from the next tick on). `og.MusicBeat` (event) per beat, in the tick
  whose interval holds it.
- `og.SoundEnded.reason` 4 (`SOUND_CULLED`): the voice cap.
- `og.ReverbZone` (component, wave 11; kestrel-stair #21): a box (`size`
  along the entity's axes) or a sphere (`radius`) with `fade` (m), `bus`,
  `send`, `room`, `damping`, and `weight` written back. Each tick the
  listener's weight in each zone is `ReverbZone::weight_at`: 1 inside, then
  linear to 0 at `fade` metres from the surface. The strongest zone (ties to
  the first in query order) eases `reverbRoom`, `reverbDamping` and its
  bus's `AudioBusFx.reverb` towards its own by that weight before the fx
  state is hashed and sent, so a zone is the same fx commands as a game
  writing those values: in the stream, the world hash and every resync, a
  function of the listener's position alone (no state of its own; a
  restore continues as the walk did). og.Audio and og.AudioBusFx keep the
  open air's values: the zone never writes them.

Tick side: bus effects are sent as `fx` commands (op 6: `bus`, the
parameter in `voice`, the value in `gain`) whenever any changed, and in
every resync; `AudioStream.fx` holds the hash of what was sent. Past
`maxVoices`, voices are ranked by priority, then gain, then age (newer
wins) and the lowest are stopped or never started (`SOUND_CULLED`, `playing`
false). A spatial `occlude` source asks the host's `Occluder` (the runtime
casts a physics ray between it and the listener, clear of both bodies)
and, when blocked, plays at `occlusionGain` through `occlusionCutoff`. A
source whose `startAt` is past the tick waits; inside the tick it plays at
a negative position. With `fade`, a start is a silent play and a params
ramping to the gain over `fade`; a stop and every change ramp over it.

`listen`: every mixer made by `Mixer::for_link` records what it renders
into the link's two-second ring (`AudioLink::listen`); the null sink now
renders at 48 kHz into that ring only, so the probe's `listen` member and
the command channel's `listen[:path][:seconds]` action have the last
seconds a person would have heard, headless too. `og run --audio out.wav`
captures the whole run (its SHA-256 is in `--json`).

Tests: `tests/mix.rs` (each broken once, `tests/evidence/E6/break-and-restore.md`):
bus filters per bus, ducking 12 dB and back, the compressor's 10.5 dB, the
reverb tail, a reverb zone (the open air, half way through its fade, inside,
nothing sent standing still, the open air back; the same walk twice and
across a restore; a click rings inside and not outside), scheduled starts on their sample and stems aligned, the music
clock and a stinger on the bar line, a two-second crossfade, the voice cap,
occlusion and distance dulling, and the same PCM twice with `listen`
returning its last second (`$OG_EVIDENCE_DIR/M6/e6-mix.{wav,json}`).

## Chunked sounds

og-cook cooks a long sound (`sound { stream true chunk 4 }` in its import
settings, or any whose QOA file would pass 1 MiB unless `stream false`) in
chunks of whole QOA frames: the sound's blob holds the QOA header and the
first chunk, each other chunk is a blob `<id>#chunk<n>`. A QOA frame
carries its predictor's state, so the chunks are the whole file's frames
cut apart, and decode to its samples exactly.

- `SoundBank::insert_chunked(id, head, per_chunk, count)` adds one with its
  first chunk, which stays resident (a play starts at once);
  `put_chunk(handle, n, frames)` puts chunk `n` in place (its shape
  checked); a voice reaching a chunk that is not there plays silence and
  looks again on its next frame.
- `chunks_wanted(world, bank)` reads the playheads (`og.AudioPlayback`):
  per voice of a chunked sound, the chunks from `CHUNK_BEHIND_SECONDS`
  (0.5) behind its cursor to `CHUNK_AHEAD_SECONDS` (4) ahead at its pitch,
  and for a loop its start's chunk and the next. `keep_chunks(&wanted)`
  drops every other chunk and returns the ones to read. og-runtime calls
  both at each tick boundary and after the ticks, before the host pumps
  the mixer, so a voice started in the tick (part-way too) has its chunk
  before its first sample; natively the read completes at once, on the web
  the chunk is fetched like a streaming group's pack (not preloaded, taken
  from the Worker's files when read, cached by the service worker) and put
  in place at a later call, four seconds being the fetch's margin.
- `chunk_distances(world, bank)` is `chunks_wanted` with, per chunk, how
  soon a playhead needs it: 0 for the chunk a voice is in (and one it has
  just left, which a free-running device may still be rendering), else
  the seconds of the device until the nearest voice reaches the chunk's
  first frame at its pitch, and for a loop's start chunk (and the next)
  the time to the wrap plus their offset from the loop start. og-runtime
  reads every streamed file through one queue (its README's streaming
  groups, `stream.rs`): these chunks first, the smallest distance first,
  then the loading groups' packs by priority, at most four in flight, so
  on the web the chunk a playhead is about to reach is never queued
  behind a pack or a chunk four seconds away. Natively each read lands at
  once and only the order shows (og-runtime's `tests/stream_fetch.rs`).
- A voice holds the chunk it reads and the next, so one fading out after
  its playhead is gone (a crossfade's outgoing track) keeps its samples
  while the bank drops them.
- `set_chunk_loader(handle, loader)`: natively og-runtime gives each
  chunked sound a loader that reads a missing chunk's file on the mixing
  thread (type and cook stamp checked), so a long `advance` mixed at once
  (og run) still hears every sample.
- `resident_bytes()` sums what the sounds hold in memory (decoded PCM, a
  streamed sound's QOA file, a chunked sound's resident chunks).

Nothing here reaches the world: the tick side times a sound by its rate
and length alone, so `world_hash` is the same chunked or whole.

Tests: `tests/chunks.rs` (the distances of a voice looping from 20 s,
30 s in, and of a second at twice the pitch: the chunk under each
playhead 0, the next by its seconds, the loop start's by the wrap; each
broken once: without the voice's held
chunks the crossfade's tail went silent at sample 5,765,120; with chunks
indexed past their last frame a wrong-length chunk was accepted). A
3-minute stereo 48 kHz track (6.99 MB of QOA, 46 chunks of 3.9 s) looping
from 20 s with a 120 bpm `og.Music` clock and, at 60 s, a two-second
crossfade to a second voice of it started 100 s in (wrapping at 180 s),
190 s in all: the chunked mix is sample-identical to the whole-file mix
(9,120,000 frames), the music clock beats the same 380 times, and at most
918,200 B (6 chunks) are resident at once against the whole file's
6,989,848 B; with a loader and the mixer pumped every 600 ticks it is
identical again; a missing chunk plays silence, then sound once put;
wrong-length chunks and wrong chunk counts are refused
(`$OG_EVIDENCE_DIR/M6/chunks.json`). og-cook's `tests/sound_chunks.rs`
checks the cook (the chunks laid end to end are the whole file's bytes,
the settings re-cook only that sound, the threshold, a chunked sound
refused in a stream group), and `packages/engine/tests/sound-chunks.test.ts`
runs the idle archetype's music cooked whole and in 2-second chunks
through `og run --audio` for 35 s (the loop wraps): the same WAV.

## Ring (web)

`packages/engine/src/web/audio-worklet.ts` defines it; `audio.ts` writes
it. One SharedArrayBuffer: an Int32 header of 8 slots, then `capacity × 2`
f32 (interleaved stereo) from byte 32.

| slot | name | |
|---:|---|---|
| 0 | `write` | frames written, monotonic int32 (wraps) |
| 1 | `read` | frames read, monotonic int32 (wraps) |
| 2 | `capacity` | frames, a power of two (default 8192: 170 ms at 48 kHz) |
| 3 | `channels` | 2 |
| 4 | `underruns` | silent frames played because the ring ran dry |
| 5 | `sampleRate` | the AudioContext's rate |
| 6 | `primed` | 1 once `prime` frames were buffered; 0 again after a dry spell |
| 7 | `prime` | frames to buffer before (re)starting (default 2048: 43 ms) |

Single producer (the engine Worker) and single consumer (the worklet):
data first, then the index with `Atomics.store`. The producer never writes
more than the free space (it asks the mixer for at most that many frames);
the worklet outputs silence and counts underruns when the ring is dry.
Without cross-origin isolation the Worker posts each pump's PCM as a
transferred `Float32Array` on a MessagePort the main thread handed to the
worklet; the worklet queues up to 0.5 s and drops the oldest beyond that.

## Host contract

| | web (`crates/og-host-web/src/audio.rs`) | native (`crates/og-host-node/src/audio.rs`) |
|---|---|---|
| open | `og_web_audio_open(engine, sampleRate)` → Result | `og_node_audio_open(engine, "null" \| "capture" \| "device", sampleRate?)` → Result |
| per frame | `og_web_audio_pump(engine, maxFrames)` → `Float32Array` | `og_node_audio_pump(engine)` → Result of stats |
| sounds | `og_web_audio_insert_qoa(engine, id, bytes)` → Result of the handle | `og_node_audio_insert_qoa(engine, id, Buffer)` → Result of the handle |
| other | `og_web_audio_stats`, `og_web_audio_close` | `og_node_audio_stats`, `og_node_audio_wav` (capture → WAV Buffer), `og_node_audio_close` |
| TypeScript | `web/audio.ts`: `AudioShell` (main thread: `listen()`, `unlock()`), `WorkerAudio.attach(glue, engine, msg)` → `pump()`; `web/audio-worklet.ts`: processor `og-audio` | `node/audio.ts`: `openAudio(engine, mode)` → `pump()`, `stats()`, `wav()`; `insertSound` |

Rules: one sink per engine; the sink attaches the link (the next tick
resyncs); pump after every `advance`/`frame`; a headless run opens `null`
(or `capture`) so the queue never grows (an unpumped attached link keeps
1024 packets, then drops the oldest and resyncs). `device` needs the addon
built with `--features native-audio` (cpal; ALSA headers on Linux), else it
is refused with that reason.

**Wiring the Tools agent owns** (the only lines needed outside this crate's
hosts files):

1. `og-api` runtime, `create_inner`, after the game's systems are added:
   `let link = og_audio::install(&mut world, &og_audio::AudioDesc::default()).map_err(err)?;`
   and once the engine id is known, `og_audio::publish_link(id, link)`;
   in `destroy`, `og_audio::retire_link(id)` (`og-api` gains the
   dependency under its `runtime` feature).
2. `og.load.sound`: insert the cooked sound into the engine's bank
   (`og_audio::link(id).bank().insert_qoa(&cooked.id, &cooked.qoa)`) and
   return that handle, so `AudioSource.sound` and `og.load.soundInfo` name
   the same sound (today's asset index handles coincide with the bank's
   only while sounds are never removed).
3. Web: the shell creates `new AudioShell({ worker, worklet }).listen()`
   (one unlock, on the first gesture); the Worker handles
   `{ type: "audio" }` with `WorkerAudio.attach(glue, engine.id, msg)` and
   calls `audio.pump()` after each `engine.frame()`/`advance`; the build
   bundles `audio-worklet.ts` as its own module and passes its URL.
4. Native: `og run` opens `openAudio(engine.id, "null")` (or `capture` for
   `--audio out.wav`, `device` when asked) and pumps after each step.
5. `hello`/capabilities: `modules` lists `audio`.

## Tests

`cargo test -p og-audio` (`tests/audio.rs`, sounds synthesised and QOA-encoded
in `tests/support/`). Each was broken once and failed, then restored:

| Test | Asserts | Broken by |
|---|---|---|
| `command_stream_is_deterministic_and_in_the_world_hash` | the scripted scene (music loop, a spatial drone crossing, Rng-timed blips despawned on `SoundEnded`) gives the same stream hash, world hash and received hash twice; the same without a sink; one changed volume at tick 300 changes the stream hash after 300 only, and the world hash; a snapshot at 300 restored into a fresh world ends identical | not chaining the hash |
| `a_one_shot_ends_on_its_tick` | 0.5 s started in tick 10 ends in tick 39, one start and one end event, `playing` reset, playhead removed; PCM from frame 8000 to 32000 | `>` for `>=` at the end |
| `a_loop_crosses_its_seam_without_a_discontinuity` | a 375 Hz loop region (300 whole periods, DC after `loopEnd`) streamed across QOA frames and resident, pitch 1 and 1.37: no step above 1.35 × the sine's slope, the DC never plays, at pitch 1 within 0.03 of the ideal sine over 3 wraps | wrapping one frame late; interpolating past the seam |
| `pitch_two_halves_the_duration` | 30 vs 15 ticks to `SoundEnded`; 24000 vs 12000 audible frames | ignoring pitch in the resampler |
| `a_spatial_source_moving_left_to_right_pans_left_to_right` | pan rises monotonically from < −0.95 to > 0.95; left quarter L > 3R, right quarter R > 3L, middle balanced | inverting the pan |
| `the_limiter_keeps_summed_peaks_at_or_below_one` | 8 full-scale sines: ≤ 1.0 limited (rms > 0.25), > 2.0 unlimited | skipping the limiter |
| `one_second_of_a_busy_mix_renders_in_bounded_time` | 32 voices (16 streamed stereo QOA, half spatial, assorted pitch), 1 s at 48 kHz in 128-frame quanta: < 50 ms release, < 1000 ms debug | disabling the QOA frame cache |
| `a_late_sink_hears_the_playing_loop_and_a_despawn_silences_it` | a sink attached at tick 100 gets one resync and hears the loop within 16 frames; attaching changes no hash; a despawn silences within a tick and a fade | no resync on attach; ignoring `live` |
| `the_scripted_scene_renders_to_a_wav` | 2.5 s of the scene, sound from frame 0, no sample above 1.0; writes `audio-scene.wav` and its waveform | — |

Measured on an M4 Pro: 1 s of the 32-voice mix renders in 17.5 ms
(release, 1.7 % of real time) and 336 ms (debug). Evidence:
`tests/evidence/M6/audio-*`.

```sh
export CARGO_TARGET_DIR=$PWD/target/agent-audio
cargo test -p og-audio && cargo test --release -p og-audio --test audio one_second
cargo check --target wasm32-unknown-unknown -p og-audio
cargo clippy -p og-audio --all-targets --all-features -- -D warnings
```

## Status 2026-10-20

The wiring §Host contract lists is in: og-runtime installs og-audio in
every engine (`modules.rs`) and publishes the link, `og.load.sound` puts
the cooked sound in the engine's bank and answers that handle, `og run`
opens a `null` sink (`capture` with `--audio out.wav`, `device` when
asked), the web shell and engine Worker attach `WorkerAudio` and pump
after each frame, `hello` lists `audio`, and `api.ogt`'s `audio` module
declares the records, resources and events above (E6 added its own,
§E6). Of §Records' note, `loop`, `spatial` and `playing` are `bool` now;
`bus` is still a `u32` index. Evidence of the crate:
`tests/evidence/M6/audio-*`, the E6 mix (`tests/evidence/E6/`) and
chunked sounds (`tests/evidence/P4/cook/sound-chunks.md`).

## What phase 2 owes

Effects, ducking, ray-cast occlusion, distance low-pass, music clock and
crossfades, scheduled starts, priority and the voice cap, `listen`, `og run
--audio` and chunked streaming of long sounds have landed (§E6, §Chunked
sounds). Still owed:

- **Propagation**: Doppler, HRTF for headphones, cone directivity,
  `MatrixWorld` (parented sources and listeners) instead of the local
  `Transform`.
- Better resampling (windowed sinc), voice virtualisation (a culled voice
  resuming where it would be), retrigger without a level change.
- Decoding off the engine Worker (the mixer decodes QOA frames as voices
  reach them). (A chunk's fetch is prioritised by how soon its playhead
  needs it since the cook round three: §Chunked sounds.)
