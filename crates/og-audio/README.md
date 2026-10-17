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
| `schema` | `AudioSource`, `AudioListener`, `Audio`, `SoundStarted`, `SoundEnded`, `end_reason`, engine-owned `AudioPlayback`, `AudioStream`, the `og.Transform` mirror, `Bus` |
| `command` | `Command`, `Op`, `encode`/`decode` (56-byte records), `TickPacket` |
| `sound` | `SoundBank` (`insert_qoa`, `insert_pcm`, `get`, `info`, `remove`; `with_resident_seconds`), `SoundData` |
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
  boundary or a loop seam never decodes twice.
- **Voices**: linear-interpolation resampling at `pitch × source rate /
  output rate`; at a loop seam the interpolation reads `loopStart` as the
  frame after `loopEnd − 1`, so a sample-continuous loop region is seamless.
  Gain, pan and bus changes ramp over 256 frames; stops fade over 256; a
  resynced voice fades in; a fresh one starts at full gain (attacks kept).
  Mono sources pan with Web Audio's equal-power law, stereo ones with its
  balance law; a source's first two channels are used.
- **Buses** fold into each voice's gain (master × bus): no per-bus effects
  yet.
- **Limiter**: stereo-linked peak envelope (instant attack, 100 ms release)
  through a tanh soft knee from 0.8: output peaks stay below 1.0.

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

## Status 2026-10-18 (what phase 1 still owes)

At 49ed935 the mixer, the command stream and both hosts' sink modules
(`og-host-web/src/audio.rs`, `og-host-node/src/audio.rs`,
`packages/engine/src/{web,node}/audio*.ts`) are committed, but the wiring
listed under §Host contract is not done: the `og-api` runtime does not
`install` og-audio or publish the link, `og run` and the web shell/Worker
open no sink, and `api.ogt` has `load.sound`/`Sound` but no
`AudioSource`/`AudioListener`/`Audio` records or `SoundStarted`/`SoundEnded`
events, so games cannot play sound yet. §Records' note about `u32` flags is
partly stale (`spatial` and `playing` are `bool` now; `bus` is still `u32`).
Evidence of the crate itself: `tests/evidence/M6/audio-*`.

## What phase 2 owes

- **Effects**: per-bus insert chains (filters, reverb send, compressor),
  ducking music under dialogue, a real bus graph instead of folded gains.
- **Occlusion and propagation**: ray-cast occlusion against physics,
  low-pass by distance and obstruction, Doppler, HRTF for headphones, cone
  directivity, `MatrixWorld` (parented sources and listeners) instead of
  the local `Transform`.
- **Music transitions**: tempo-aware cues, crossfades and stingers on bar
  boundaries, layered stems, sample-accurate scheduled starts (`startAt`).
- Better resampling (windowed sinc), per-voice priority and a voice cap
  with virtualisation, retrigger without a level change, seek/offset.
- Streaming chunks from the network for very long tracks (today a streamed
  sound's QOA bytes are in memory), and decoding off the engine Worker.
- Audio capture into the probe (`og run --audio`), and a `listen` probe
  member returning the last second as a WAV.
