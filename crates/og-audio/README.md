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
| `schema` | `AudioSource`, `AudioListener`, `Audio`, `ReverbZone` (`weight_at`), `SoundStarted`, `SoundEnded`, `end_reason`, `panning_model`, engine-owned `AudioPlayback`, `AudioStream`, the `og.Transform` mirror, `Bus` |
| `command` | `Command`, `Op`, `encode`/`decode` (56-byte records), `TickPacket` |
| `sound` | `SoundBank` (`insert_qoa`, `insert_pcm`, `insert_chunked`, `put_chunk`, `keep_chunks`, `set_chunk_loader`, `resident_bytes`, `get`, `info`, `remove`; `with_resident_seconds`), `SoundData` (`chunking`, `resident_chunks`, `resident_bytes`), `chunks_wanted` (§Chunked sounds) |
| `decode` | `SoundBank::start_decode_thread`, `take_decode_jobs` → `DecodeJob` (`key`, `frame`, `qoa`, `decode`), `put_decoded`, `decode_queued`, `decode_stats` → `DecodeStats`; `DECODE_AHEAD_SECONDS`, `DECODE_AHEAD_FRAMES` (§Decoding beside the tick) |
| `resample` | the windowed sinc's constants (`SINC_ZEROS`, `SINC_TAPS`, `SINC_BETA`, `SINC_PHASES`, `SINC_MAX_SPREAD`, `WIDE_PHASES`) |
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
| 8 | `spatial` | u32 | 0 | 0/1 | attenuate and pan by the entity's world transform (its `Transform` down `og.Parent`) relative to the listener |
| 9 | `refDistance` | f32 | 1 | m, tunable | full volume up to here |
| 10 | `maxDistance` | f32 | 100 | m, tunable | no quieter past here |
| 11 | `rolloffFactor` | f32 | 1 | tunable | Web Audio's inverse model: `ref / (ref + rolloff · (clamp(d, ref, max) − ref))` |
| 12 | `playing` | u32 | 1 | 0/1 | level-triggered: 1 plays (inserting a source plays it), 0 stops; the engine sets 0 when a one-shot ends |

### `og.AudioListener` (component)

| # | field | type | default | meaning |
|--:|---|---|---|---|
| 0 | `volume` | f32 | 1 (0–4, tunable) | gain on every spatial source it hears |

Position and orientation come from the entity's world transform: its
`og.Transform` composed down `og.Parent` (§Tick semantics), so a listener
under a camera rig hears from the rig (three.js convention: the listener
looks down −Z, +X is right). The first listener in query order is used;
with none, the origin looking down −Z.

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
| 3 | `reason` (`SoundEnded` only) | u32 | 0 finished, 1 stopped (`playing` = 0), 2 removed (the AudioSource left a living entity), 3 replaced (`sound` changed while playing), 4 culled (§E6), 5 retriggered (§Resampling, virtual voices, retrigger) |

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
  `og.AudioListener`, `og.Transform`, `og.Parent`; writes `og.AudioSource` (only to reset
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
- Sources, the listener and reverb zones are placed at their world
  transforms: an entity's `og.Transform` composed down its `og.Parent`
  chain (`og_core::math::world_pose`, the helper og-physics places soft
  bodies with, as the renderer composes `og.MatrixWorld`; a parent's scale
  scales the child's offset, a parent without a Transform ends the
  chain). A parented source follows its parent: a sound on a child
  entity is heard where its parent carries it, every tick, as a function
  of that tick's transforms (row 86). Nothing parented, nothing composed.
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
| 0 | u8 | `op` | 1 play, 2 stop, 3 params, 4 bus, 5 stop-all, 6 fx, 7 cue, 8 hrtf (§Propagation) |
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
- **Sounds**: nothing is decoded at insert. A QOA sound's frames (5120
  samples, self-contained) are decoded beside the tick into the sound's
  frame cache (§Decoding beside the tick): a sound up to 6 s
  (`DEFAULT_RESIDENT_SECONDS`) whole, and it stays decoded; a longer one
  streams, its frames decoded ahead of the playheads and dropped behind
  them. A voice reads the cache two frames at a time, so a frame boundary
  or a loop seam never looks twice, and decodes a frame nobody decoded
  yet itself, to the same samples. A sound cooked in chunks keeps only
  some of its QOA bytes in memory (§Chunked sounds).
- **Voices**: resampling at `pitch × source rate / output rate`, by a
  windowed sinc (`og.Audio.resampler` sinc, the default) or linear
  interpolation (`linear`, the mix before round 4; §Resampling, virtual
  voices, retrigger); at a loop seam both read `loopStart` as the frame
  after `loopEnd − 1`, so a sample-continuous loop region is seamless.
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

## Propagation

Doppler, cone directivity and the HRTF panner (round 4). Doppler and the
cone are computed in the tick, like gain and pan, so they are part of the
stream and the world hash; the HRTF is the mixer's, steered by `hrtf`
commands.

Records (`schema.rs`, IDL module `audio`):

| record | field | type | default | meaning |
|---|---|---|---|---|
| `og.AudioSource` | `dopplerFactor` | f32 | 1 (0–10, tunable) | how much motion shifts a spatial source's pitch; 0 off |
| | `coneInnerAngle` | f32 | 360 (deg) | full volume within this full angle about the source's +Z |
| | `coneOuterAngle` | f32 | 360 (deg) | `coneOuterGain` past this full angle |
| | `coneOuterGain` | f32 | 0 (0–1) | the gain outside the outer angle |
| `og.AudioListener` | `dopplerFactor` | f32 | 1 (0–10) | scales every source's shift; 0 turns Doppler off for the listener |
| | `panningModel` | enum | `equalpower` | `equalpower` (speakers) or `hrtf` (headphones) |
| `og.Audio` | `speedOfSound` | f32 | 343.3 (m/s) | what Doppler divides by |
| `og.AudioPlayback` (engine) | `position`, `direction` | Vector3 | | the source's world position last tick; the direction last sent to the HRTF (zero: the panner) |
| `og.AudioStream` (engine) | `listener`, `listenerAt` | u64, Vector3 | | the listener heard last tick (0: none) and where it was |

- **Doppler** (OpenAL 1.1's model, Web Audio's former PannerNode one):
  pitch × `(c − DF·vl) / (c − DF·vs)`, `vs` and `vl` the source's and the
  listener's velocities along the line from the source to the listener,
  `c` = `og.Audio.speedOfSound`, `DF` = the source's `dopplerFactor` × the
  listener's. Velocities are tick velocities: the change of the world
  position (its `og.Transform` down `og.Parent`, §Tick semantics) since
  the last tick × the tick rate, so a source carried by a moving parent
  shifts as one placed there would. A voice's first tick, a source that
  was not spatial last tick and a listener that was not the listener last
  tick have no velocity; a jump at or past the speed of sound in one tick
  is a teleport (a warp, a respawn) and counts as still. Each `DF·v` is
  held within ±c/2, so the shift stays within ×1/3..×3. The shifted pitch
  is the voice's pitch in the stream, so the playhead (and the tick a
  one-shot ends on) follows it. Static scenes compute a ratio of exactly
  1: their stream is the one before Doppler, bit for bit.
- **Cones** (Web Audio's PannerNode cone, three.js PositionalAudio's
  `setDirectionalCone`): with θ the angle between the source's world +Z
  (three.js orients a PositionalAudio by +Z) and the listener, the gain is
  1 for θ ≤ inner/2, `coneOuterGain` for θ ≥ outer/2, linear in θ
  between; 360/360 is exactly 1. θ is og-core's deterministic `atan2`.
- **HRTF** (`panningModel` hrtf): the tick sends op 8 `hrtf` per spatial
  voice after its play (and its params) whenever its direction in the
  listener's frame changed (x, y, z in `gain`, `pitch`, `pan`), and with
  every resynced play; the zero vector (the listener back on
  `equalpower`) returns the voice to the stereo panner. The mixer's voice
  then folds to mono and runs Brown and Duda's spherical-head model (IEEE
  Trans. Speech and Audio Processing 6(5), 1998, its head part; `dsp.rs`
  `Hrtf`): per ear an interaural delay by Woodworth's formula (up to
  (a/c)(1 + π/2) = 0.66 ms, 31.5 frames at 48 kHz; read from a 128-frame
  delay line with linear interpolation) and the head-shadow filter
  `(1 + α(θ)s/2ω₀)/(1 + s/2ω₀)` (a = 8.75 cm, ω₀ = c/a, α from 2 at the
  ear's axis to 0.1 at 150° from it), by the bilinear transform; the
  output is scaled by √½ so a centred source keeps the equal-power
  panner's low-frequency power. Delays and coefficients glide over the
  256-frame ramp. No pinna or shoulder echoes: sources move between the
  ears, front/back and up/down are not resolved. Non-spatial sources keep
  their pan. With `equalpower` no `hrtf` command is sent and the voice
  path is the one before the HRTF, sample for sample.

Tests: `tests/propagation.rs` (Doppler against OpenAL's formula in f64 at
20 m/s coming and going, heard 440 Hz → 467/416 Hz, factor 0 on either
side, factor 2, a 200 m teleport, a listener walking at 5 m/s; a 60°/180°
cone with outer gain 0.2 at eight angles, turned to face −Z, and the
omnidirectional default; the HRTF's far-ear lag of 31–32 frames by
cross-correlation, its highs 5× down and its lows not, a source ahead
identical in both ears, no `hrtf` command and hard right on `equalpower`,
and the panner back after switching mid-run; a walk with every feature on
the same twice, samples and stream, and across a restore at tick 117) and
`tests/parented.rs` (a carried source's pitch bits equal a placed one's).

## Resampling, virtual voices, retrigger

Records: `og.Audio.resampler` (`AudioResampler`: `linear`, `sinc`;
default `sinc`), `og.AudioSource.virtualize` and `retrigger` (bool, false),
engine-owned `og.AudioPlayback.virtual` (bool), `SOUND_RETRIGGERED` (5).

- **Windowed sinc** (`resample.rs`): a Kaiser window, β = 7 (about 70 dB
  of stopband), 8 zero crossings each side: 16 taps at pitch ≤ 1 from a
  table of 2048 phases (nearest phase: at most 1/4096 frame of time
  error), each phase normalised to sum to 1. A voice stepping faster than
  one source frame per output frame widens the kernel by the step rounded
  up to an eighth (cutoff at the output's Nyquist), up to 4× (64 taps),
  from a 512-phase table built on first use; past 4× it aliases, as
  linear always did. A voice keeps a 128-frame history around its
  playhead, filled one source frame at a time and across the loop seam
  as the linear path reads it, so a frame costs one decoded frame and its
  taps. On a source frame exactly (an unpitched sound at the output's
  rate) it returns the frame: such a voice is the linear path's samples,
  bit for bit. Tables are built from `+ − × ÷` and `sqrt` (series for sin
  and I0), the same on every host. The choice reaches the mixer as an fx
  command (`fx::RESAMPLER` 22, on master) with the others, so it is in
  the stream, the hash and every resync. `linear` keeps the old output
  exactly (a scratch scene of seven pitched voices at three rates hashed
  the same as on the base); measured on a 15 kHz tone from 44.1 kHz:
  linear −3.4 dB on the tone and its image at −11.5 dB, sinc 0.0 dB and
  −74 dB; pitched 2.5× from 6 kHz, the folded image −28 dB linear,
  −90 dB sinc.
- **Virtual voices**: past `maxVoices` a source with `virtualize` is not
  culled: its voice is stopped in the mixer and `AudioPlayback.virtual`
  set; its playhead (the tick side's `cursor`) runs on, a one-shot ends on
  the tick it would have (`SoundEnded` finished), nothing is sent for it.
  Each tick the cap ranks heard and virtual voices together; a virtual
  one back within it is played at its cursor (a resumed play, fading in
  over 256 frames, with its `hrtf` direction), so it is heard where it
  would have been, reading the clock, not restarting. One that is heard
  and falls outside goes virtual; one without `virtualize` is culled as
  before. Fresh sources past the cap start virtual.
- **Retrigger**: `retrigger` true starts the sound again from the top in
  that tick (its `startOffset`, `startAt` and `fade` as a fresh start):
  a playing voice is stopped (`SOUND_RETRIGGERED`) and a new voice id
  plays it; a source not playing (ended, stopped, culled) plays; either
  way the engine clears `retrigger` and sets `playing` true. No level
  change on `playing` is needed (a `Voices` system no longer has to stop
  a voice one tick and start it the next).

Tests: `tests/voices.rs` (the sinc against linear on the 15 kHz tone and
2.5× pitch-up, and identical on its own frames; a virtual voice at
priority 100 under a cap of 2 runs its playhead 800 frames a tick, is
never sent, is played once at tick 30 from frame 24 000 when it outranks
another and is heard at the swell's level there, a virtual one-shot
ending on tick 29; a retrigger stopping the old voice and playing a new
one from 0 in one tick, flag cleared, and an ended one-shot played again;
a run with every feature the same twice and across a restore at tick 87).

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
- `resident_bytes()` sums what the sounds hold in memory (PCM, a QOA
  sound's file, a chunked sound's resident chunks, and the QOA frames
  decoded from them). A chunk dropped takes its decoded frames with it.

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
the settings re-cook only that sound, the threshold, a chunked sound in a
stream group packing its head alone), `tests/chunks.rs` a chunked sound
taking its silent stand-in's handle (`replace_chunked`) and a chunk landing
after its group unloaded let go, og-runtime's `tests/stream_chunked.rs`
that no chunk of a grouped one is read before its group is resident nor
after it is let go, and `packages/engine/tests/sound-chunks.test.ts`
runs the idle archetype's music cooked whole and in 2-second chunks
through `og run --audio` for 35 s (the loop wraps): the same WAV.

## Decoding beside the tick

Debt round 3 (`tests/evidence/debt3/audio-decode/NOTES.md`). Before it a
short sound was decoded whole when it entered the bank (inside
`og.load.sound`, on the engine thread: 6 s of 48 kHz stereo is a whole
file's decode in one call) and a long one by each voice as the mixer
reached its frames, in the pump after the frame (on the web, the engine
Worker). Now neither happens on the engine thread unless a frame is
needed before anything decoded it.

- **The frame cache.** A QOA or chunked sound is indexed at insert (its
  frame table, checked as the decoder reads it) and holds one slot per
  QOA frame. The frames of a sound up to the bank's resident seconds are
  pinned (once decoded they stay, as the whole decode did); of a longer
  one only the first is, the others are held while the decode-ahead
  wants them. `insert_pcm` sounds are PCM as before.
- **What is decoded is planned from the world.** Each sound queues its
  pinned frames when it enters the bank (`insert_*`, `replace_*`). The
  `og.audio` system, at the start of every tick with a sink attached,
  plans the decode-ahead from the playheads (`og.AudioPlayback`, not
  virtual): per voice the QOA frame its cursor is in and the next ones
  up to `DECODE_AHEAD_SECONDS` (0.25) of the device at its pitch, at
  most `DECODE_AHEAD_FRAMES` (4) past the cursor's frame, and across a
  loop's wrap the loop start's frames for the rest of the window; soonest
  first, ahead of the pins. Frames it planned before and no longer wants
  are dropped, so a voice holds at most 5 decoded frames on each side of
  a wrap. The plan reads the world and writes nothing in it: the stream
  and `world_hash` are unchanged.
- **The helper** takes the queue's jobs (a frame's one-frame QOA file)
  and answers with its samples: natively `start_decode_thread`, one
  thread per bank that og-runtime starts for every engine (it ends with
  the bank); on the web `web/audio-decoder.ts`, a decoder Worker the
  engine Worker starts from a `blob:` URL of `decodeQoa`'s own source
  and feeds after each pump (`og_web_audio_decode_jobs`, at most 32 in
  flight), answering the bank with `og_web_audio_put_decoded`. The same
  page works isolated or not (no SharedArrayBuffer, no threaded wasm);
  where the Worker cannot start or fails (a CSP without `worker-src
  blob:`) no decoder runs. A job not answered within 120 plans is handed
  out again; an answer of another length, or for a key not in flight, is
  refused.
- **A miss is decoded where it is read.** A voice reaching a frame not in
  the cache decodes it itself (today's path), to the same samples, and
  keeps a pinned one for the others; `DecodeStats.misses` counts them.
  So the PCM is a function of the command stream whatever the helper
  did: no helper, a slow one, a racing thread. The misses that remain
  with a helper that keeps up are the frames nothing could foresee: the
  first frame of a start part-way into a long sound (`startOffset`, a
  virtual voice heard again, a resync after attach) and a long `advance`
  mixed at once (og run's playtests: the plan covers a quarter second).
  Nothing about a sound's start depends on its data being ready: the
  tick times a sound by its rate and length alone, as with chunks.
- **Two decoders, one result.** The web's decoder is a TypeScript port
  of `qoa.rs`'s decoder (integer arithmetic, operation for operation).
  `tests/decode.rs` and `packages/engine/tests/audio-decoder.test.ts`
  decode one fixture (`packages/engine/tests/fixtures/qoa/frames.qoa`,
  written by this crate's encoder: full-scale squares with noise, a
  sweep, silence, impulses; stereo, two frames and a short one) to one
  FNV-1a hash, whole and as the jobs' one-frame files, the bun test also
  after the export's minifying build.

Tests: `tests/decode.rs` (each broken once and failing: a miss playing
silence, the plan stopping at the loop's end, an answer of another
length taken, the pins decoded on the inserting thread), and
`packages/engine/tests/audio-decoder.test.ts` (the TS decoder against the
fixture's hash, its Worker source after minifying, `AudioDecoder` feeding
a real Worker and answering a bank).

| Test | Asserts |
|---|---|
| `the_mix_is_the_same_whatever_the_helper_did` | 10 s of a looping 44.1 kHz track (wrapping), a pad pitched 1.37, blips and a second voice started 3 s into the track: no helper, a helper that keeps up, a racing thread and a late web-like helper (four jobs a tick, each answered three ticks later) give the same stream hash and bit-identical PCM; without a helper over 100 misses, with one that keeps up exactly one (the start part-way in) |
| `a_sound_is_indexed_at_insert_and_decoded_by_the_helper` | inserting 6 s of stereo queues its 57 frames and decodes none (resident bytes: the QOA file alone); a long sound queues its first; after the helper the bytes add the PCM, and a voice pitched 1.1 mixes bit for bit what the whole file decoded at once mixes, with no miss |
| `the_decode_ahead_is_bounded_and_crosses_the_wrap` | a voice pitched 16 plans its cursor's frame and four more; a loop of 1.0–1.5 s plans its start's frame a quarter second before the wrap, never past its end, and holds at most its window |
| `answers_of_another_length_or_key_are_refused` | a short answer, a second answer and an unknown key are refused; the refused frame is decoded where it is read (one miss), the mix bit for bit the whole file's |
| `the_shared_fixture_decodes_to_its_hash_frame_by_frame` | the encoder still writes the fixture; it decodes to `080bf5513188bc60` whole and as the jobs' one-frame files |

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
| decoding (§Decoding beside the tick) | `og_web_audio_decode_jobs(engine, max)` → `[key, frame, Uint8Array, …]`, `og_web_audio_put_decoded(engine, key, Int16Array)` → bool | the bank's decode thread, started by og-runtime for every engine |
| other | `og_web_audio_stats`, `og_web_audio_close` | `og_node_audio_stats`, `og_node_audio_wav` (capture → WAV Buffer), `og_node_audio_close` |
| TypeScript | `web/audio.ts`: `AudioShell` (main thread: `listen()`, `unlock()`), `WorkerAudio.attach(glue, engine, msg)` → `pump()` (then feeds the decoder); `web/audio-decoder.ts`: `AudioDecoder`, `decodeQoa`; `web/audio-worklet.ts`: processor `og-audio` | `node/audio.ts`: `openAudio(engine, mode)` → `pump()`, `stats()`, `wav()`; `insertSound` |

Both hosts' stats carry `decodedAhead` (frames the helper decoded),
`decodeMisses` (frames a voice decoded where it read them) and
`decodeQueued` (the web's also `decodeInFlight`).

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
| `tests/parented.rs`: `a_source_on_a_child_follows_its_moving_parent` | a source 2 m ahead of a cart moving x −10 to 10 sends, every tick, the same gain, pan and cutoff bits as a source placed there itself, the same stream hash, twice | each entity's own `Transform` again (`Places::world`) |
| `a_listener_under_a_rig_hears_from_the_rig` | a listener under a rig at x 10 turned +90° about Y hears a source at (10, 0, 3) hard left at gain 1/3; the rig moved and turned back, 6 m right and 3 m behind | the same |
| `a_zone_under_a_parent_is_where_its_parent_puts_it` | a radius-1 zone under a hall at x 20 scaled 2: weight 0 at the origin, 1 at 1.5 m from the hall; the hall moved to the origin takes its zone | the same |

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
--audio`, chunked streaming of long sounds, propagation (Doppler, cones,
the HRTF), the windowed sinc, virtual voices, retrigger and decoding
beside the tick have landed (§E6, §Chunked sounds, §Propagation,
§Resampling, virtual voices, retrigger, §Decoding beside the tick).
Nothing is owed. The last item, decoding off the engine Worker, landed in
debt round 3; round 4 had left it because a decode on another Worker that
lands late would play silence and make the PCM depend on timing. The
miss path answers that (a frame not decoded yet is decoded where it is
read, to the same samples), and no threaded wasm is needed: the decoder
Worker shares no memory with the engine.
