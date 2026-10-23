<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `audio` module

Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).

The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.

Spatial gain and pan are computed in the tick from the source's world transform relative to the first AudioListener's (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z). A source, listener or zone on a child entity is placed at its og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a child is heard where its parent carries it, a listener under a camera rig hears from the rig.

- **TypeScript**: functions are the namespace `og.audio`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-audio`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module audio {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 8067</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`SOUND_FINISHED`](#const-SOUND_FINISHED) | constant | SoundEnded.reason: a one-shot played to its end. |
| [`SOUND_STOPPED`](#const-SOUND_STOPPED) | constant | SoundEnded.reason: `playing` was set to false. |
| [`SOUND_REMOVED`](#const-SOUND_REMOVED) | constant | SoundEnded.reason: the AudioSource was removed from a living entity. |
| [`SOUND_REPLACED`](#const-SOUND_REPLACED) | constant | SoundEnded.reason: `sound` changed while it played. |
| [`SOUND_CULLED`](#const-SOUND_CULLED) | constant | SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age). |
| [`AudioBus`](#type-AudioBus) | enum | The bus a source plays on; every bus passes through master. |
| [`AudioBus()`](#column-AudioBus) | column kind | An AudioBus field: the variant's index. |
| [`og.AudioSource`](#record-AudioSource) | component | A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded. |
| [`og.AudioListener`](#record-AudioListener) | component | Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used. |
| [`og.Audio`](#record-Audio) | resource | Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer. |
| [`og.AudioBusFx`](#record-AudioBusFx) | component | A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through. |
| [`og.ReverbZone`](#record-ReverbZone) | component | A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener. |
| [`og.Music`](#record-Music) | component | A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat. |
| [`og.MusicBeat`](#record-MusicBeat) | event | A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after). |
| [`og.SoundStarted`](#record-SoundStarted) | event | A source started, sent in the tick it happens (systems read it the tick after). |
| [`og.SoundEnded`](#record-SoundEnded) | event | A sound ended, sent in the tick it happens (systems read it the tick after). |
| [`og.audio.outputLatency`](#fn-outputLatency) | function | How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand). |

## Functions

<a id="fn-outputLatency"></a>

### `og.audio.outputLatency`

How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand).

Inside a system og.Audio must be among its reads or writes.

```ts
og.audio.outputLatency(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: Seconds; 0 headless.

Binding symbol `og_audio_output_latency` · publish builds: kept

```ogt
fn outputLatency {
  doc "How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand)."
  doc "Inside a system og.Audio must be among its reads or writes."
  param world World "The world."
  returns f64 "Seconds; 0 headless."
}
```

<sub>`api.ogt` line 8447</sub>

## Components, resources and events

<a id="record-AudioSource"></a>

### `og.AudioSource` (component)

A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded.

A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event.

Token `og.AudioSource`, registered under the schema name `og.AudioSource`; its value type is `AudioSource`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sound` | `Sound` | `0` | — | — | — | The sound, from og.load.sound; 0 plays nothing. A handle not yet loaded waits and starts on the first tick it is there. |
| `bus` | `AudioBus` | `"sfx"` | — | — | — | The bus it plays on. |
| `volume` | `number` | `1` | 0 … 4 | — | tunable | Linear gain. |
| `pitch` | `number` | `1` | 0.0625 … 16 | — | tunable | Playback rate: 2 is an octave up and half as long. |
| `pan` | `number` | `0` | -1 … 1 | — | tunable | Stereo position of a non-spatial source, -1 left to 1 right. |
| `loop` | `boolean` | `false` | — | — | — | Loop between loopStart and loopEnd, seamlessly. |
| `loopStart` | `number` | `0` | ≥ 0 | `s` | — | Where a loop restarts. |
| `loopEnd` | `number` | `0` | ≥ 0 | `s` | — | Where a loop wraps; 0 is the sound's end. |
| `spatial` | `boolean` | `false` | — | — | — | Attenuate and pan by the entity's world transform (its og.Transform down og.Parent) relative to the listener. |
| `refDistance` | `number` | `1` | ≥ 0 | `m` | tunable | Full volume up to this distance, as Web Audio's PannerNode.refDistance. |
| `maxDistance` | `number` | `100` | ≥ 0 | `m` | tunable | No quieter past this distance, as Web Audio's PannerNode.maxDistance. |
| `rolloffFactor` | `number` | `1` | ≥ 0 | — | tunable | How fast volume falls with distance, in Web Audio's inverse model: ref / (ref + rolloff * (clamp(d, ref, max) - ref)). |
| `playing` | `boolean` | `true` | — | — | — | Plays while true; the engine sets it false when a one-shot ends. |
| `priority` | `number` | `128` | 0 … 255 | — | — | When more than Audio.maxVoices would play, the lowest priority is stopped first (then the quietest, then the oldest), with SoundEnded reason SOUND_CULLED. |
| `lowpass` | `number` | `0` | 0 … 20000 | `hz` | tunable | A low-pass filter on this source, its cutoff; 0 is off. |
| `occlude` | `boolean` | `false` | — | — | — | A spatial source behind a collider (a physics ray to the listener) plays at Audio.occlusionGain through a low-pass at Audio.occlusionCutoff. |
| `distanceCutoff` | `number` | `0` | 0 … 20000 | `hz` | tunable | A spatial source's low-pass by distance: the cutoff at maxDistance, from 20 kHz at refDistance (log-linear between); 0 is off. |
| `startAt` | `number` | `0` | ≥ 0 | `s` | — | Starts at this world time (seconds since tick 0), to the sample, even inside a tick; 0 or a time past starts at once. Music.nextBar and Music.nextBeat give musical times. |
| `startOffset` | `number` | `0` | ≥ 0 | `s` | — | Where in the sound it starts, in seconds of the sound: started at startAt (or at once) from this far in, to the sample and at full gain, so a resumed phrase continues instead of restarting. Past a one-shot's end it plays nothing; a loop wraps into its region. |
| `fade` | `number` | `0` | 0 … 30 | `s` | tunable | Starts, stops and volume changes ramp over this long: a crossfade is one source stopped and another started with the same fade; 0 is 5 ms. |

```ogt
component AudioSource {
  doc "A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded."
  doc "A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event."
  field sound Sound "The sound, from og.load.sound; 0 plays nothing. A handle not yet loaded waits and starts on the first tick it is there."
  field bus AudioBus "The bus it plays on." {
    default sfx
  }
  field volume f32 "Linear gain." {
    default 1
    min 0
    max 4
    tunable true
  }
  field pitch f32 "Playback rate: 2 is an octave up and half as long." {
    default 1
    min 0.0625
    max 16
    tunable true
  }
  field pan f32 "Stereo position of a non-spatial source, -1 left to 1 right." {
    min -1
    max 1
    tunable true
  }
  field loop bool "Loop between loopStart and loopEnd, seamlessly."
  field loopStart f32 "Where a loop restarts." {
    min 0
    unit s
  }
  field loopEnd f32 "Where a loop wraps; 0 is the sound's end." {
    min 0
    unit s
  }
  field spatial bool "Attenuate and pan by the entity's world transform (its og.Transform down og.Parent) relative to the listener."
  field refDistance f32 "Full volume up to this distance, as Web Audio's PannerNode.refDistance." {
    default 1
    min 0
    unit m
    tunable true
  }
  field maxDistance f32 "No quieter past this distance, as Web Audio's PannerNode.maxDistance." {
    default 100
    min 0
    unit m
    tunable true
  }
  field rolloffFactor f32 "How fast volume falls with distance, in Web Audio's inverse model: ref / (ref + rolloff * (clamp(d, ref, max) - ref))." {
    default 1
    min 0
    tunable true
  }
  field playing bool "Plays while true; the engine sets it false when a one-shot ends." {
    default true
  }
  field priority u32 "When more than Audio.maxVoices would play, the lowest priority is stopped first (then the quietest, then the oldest), with SoundEnded reason SOUND_CULLED." {
    default 128
    min 0
    max 255
  }
  field lowpass f32 "A low-pass filter on this source, its cutoff; 0 is off." {
    min 0
    max 20000
    unit hz
    tunable true
  }
  field occlude bool "A spatial source behind a collider (a physics ray to the listener) plays at Audio.occlusionGain through a low-pass at Audio.occlusionCutoff."
  field distanceCutoff f32 "A spatial source's low-pass by distance: the cutoff at maxDistance, from 20 kHz at refDistance (log-linear between); 0 is off." {
    min 0
    max 20000
    unit hz
    tunable true
  }
  field startAt f64 "Starts at this world time (seconds since tick 0), to the sample, even inside a tick; 0 or a time past starts at once. Music.nextBar and Music.nextBeat give musical times." {
    min 0
    unit s
  }
  field startOffset f64 "Where in the sound it starts, in seconds of the sound: started at startAt (or at once) from this far in, to the sample and at full gain, so a resumed phrase continues instead of restarting. Past a one-shot's end it plays nothing; a loop wraps into its region." {
    min 0
    unit s
  }
  field fade f32 "Starts, stops and volume changes ramp over this long: a crossfade is one source stopped and another started with the same fade; 0 is 5 ms." {
    min 0
    max 30
    unit s
    tunable true
  }
}
```

<sub>`api.ogt` line 8094</sub>

<a id="record-AudioListener"></a>

### `og.AudioListener` (component)

Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used.

Token `og.AudioListener`, registered under the schema name `og.AudioListener`; its value type is `AudioListener`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `volume` | `number` | `1` | 0 … 4 | — | tunable | A gain on every spatial source this listener hears. |

```ogt
component AudioListener {
  doc "Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used."
  field volume f32 "A gain on every spatial source this listener hears." {
    default 1
    min 0
    max 4
    tunable true
  }
}
```

<sub>`api.ogt` line 8182</sub>

<a id="record-Audio"></a>

### `og.Audio` (resource)

Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer.

Token `og.Audio`, registered under the schema name `og.Audio`; its value type is `Audio`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `master` | `number` | `1` | 0 … 2 | — | tunable | Gain of the master bus, applied to every sound. |
| `music` | `number` | `1` | 0 … 2 | — | tunable | Gain of the music bus. |
| `sfx` | `number` | `1` | 0 … 2 | — | tunable | Gain of the sfx bus. |
| `ui` | `number` | `1` | 0 … 2 | — | tunable | Gain of the ui bus. |
| `maxVoices` | `number` | `32` | 1 … 256 | — | — | The most sources that play at once; past it the lowest AudioSource.priority is stopped (SoundEnded reason SOUND_CULLED). |
| `occlusionGain` | `number` | `0.35` | 0 … 1 | — | tunable | The gain of an occluded source (AudioSource.occlude). |
| `occlusionCutoff` | `number` | `900` | 20 … 20000 | `hz` | tunable | The low-pass cutoff of an occluded source. |
| `reverbRoom` | `number` | `0.6` | 0 … 1 | — | tunable | The shared reverb's room size (the AudioBusFx.reverb sends feed it). |
| `reverbDamping` | `number` | `0.4` | 0 … 1 | — | tunable | The shared reverb's damping of high frequencies. |
| `outputLatency` | `number` | `0` | ≥ 0 | `s` | read only | Seconds from a sample's world time to the speaker, as the host reports it (og.audio.outputLatency). |

```ogt
resource Audio {
  doc "Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer."
  field master f32 "Gain of the master bus, applied to every sound." {
    default 1
    min 0
    max 2
    tunable true
  }
  field music f32 "Gain of the music bus." {
    default 1
    min 0
    max 2
    tunable true
  }
  field sfx f32 "Gain of the sfx bus." {
    default 1
    min 0
    max 2
    tunable true
  }
  field ui f32 "Gain of the ui bus." {
    default 1
    min 0
    max 2
    tunable true
  }
  field maxVoices u32 "The most sources that play at once; past it the lowest AudioSource.priority is stopped (SoundEnded reason SOUND_CULLED)." {
    default 32
    min 1
    max 256
  }
  field occlusionGain f32 "The gain of an occluded source (AudioSource.occlude)." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field occlusionCutoff f32 "The low-pass cutoff of an occluded source." {
    default 900
    min 20
    max 20000
    unit hz
    tunable true
  }
  field reverbRoom f32 "The shared reverb's room size (the AudioBusFx.reverb sends feed it)." {
    default 0.6
    min 0
    max 1
    tunable true
  }
  field reverbDamping f32 "The shared reverb's damping of high frequencies." {
    default 0.4
    min 0
    max 1
    tunable true
  }
  field outputLatency f64 "Seconds from a sample's world time to the speaker, as the host reports it (og.audio.outputLatency)." {
    min 0
    unit s
    readonly true
  }
}
```

<sub>`api.ogt` line 8192</sub>

<a id="record-AudioBusFx"></a>

### `og.AudioBusFx` (component)

A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through.

Changes reach the mixer through the command stream, so they are deterministic and in the world hash; the PCM is the mixer's.

Token `og.AudioBusFx`, registered under the schema name `og.AudioBusFx`; its value type is `AudioBusFx`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bus` | `AudioBus` | `"music"` | — | — | — | The bus these effects are on. |
| `lowpass` | `number` | `0` | 0 … 20000 | `hz` | tunable | Low-pass cutoff; 0 is off. |
| `highpass` | `number` | `0` | 0 … 20000 | `hz` | tunable | High-pass cutoff; 0 is off. |
| `reverb` | `number` | `0` | 0 … 1 | — | tunable | How much of the bus goes to the reverb. |
| `threshold` | `number` | `0` | -60 … 0 | `db` | tunable | The compressor's threshold; 0 dB is off. |
| `ratio` | `number` | `4` | 1 … 30 | — | tunable | The compressor's ratio: dB in over the threshold per dB out. |
| `attack` | `number` | `0.005` | 0 … 1 | `s` | tunable | The compressor's attack. |
| `release` | `number` | `0.15` | 0 … 3 | `s` | tunable | The compressor's release. |
| `makeup` | `number` | `0` | -24 … 24 | `db` | tunable | Gain after the compressor. |
| `duck` | `boolean` | `false` | — | — | — | Turns this bus down while duckBus is loud: music under effects or dialogue. |
| `duckBus` | `AudioBus` | `"sfx"` | — | — | — | The bus that ducks this one. |
| `duckDepth` | `number` | `9` | 0 … 60 | `db` | tunable | How far it is turned down. |
| `duckThreshold` | `number` | `-36` | -80 … 0 | `db` | tunable | How loud the key bus must be. |
| `duckAttack` | `number` | `0.02` | 0 … 2 | `s` | tunable | How fast it goes down. |
| `duckRelease` | `number` | `0.4` | 0 … 5 | `s` | tunable | How fast it comes back. |

```ogt
component AudioBusFx {
  doc "A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through."
  doc "Changes reach the mixer through the command stream, so they are deterministic and in the world hash; the PCM is the mixer's."
  field bus AudioBus "The bus these effects are on." {
    default music
  }
  field lowpass f32 "Low-pass cutoff; 0 is off." {
    min 0
    max 20000
    unit hz
    tunable true
  }
  field highpass f32 "High-pass cutoff; 0 is off." {
    min 0
    max 20000
    unit hz
    tunable true
  }
  field reverb f32 "How much of the bus goes to the reverb." {
    min 0
    max 1
    tunable true
  }
  field threshold f32 "The compressor's threshold; 0 dB is off." {
    min -60
    max 0
    unit db
    tunable true
  }
  field ratio f32 "The compressor's ratio: dB in over the threshold per dB out." {
    default 4
    min 1
    max 30
    tunable true
  }
  field attack f32 "The compressor's attack." {
    default 0.005
    min 0
    max 1
    unit s
    tunable true
  }
  field release f32 "The compressor's release." {
    default 0.15
    min 0
    max 3
    unit s
    tunable true
  }
  field makeup f32 "Gain after the compressor." {
    min -24
    max 24
    unit db
    tunable true
  }
  field duck bool "Turns this bus down while duckBus is loud: music under effects or dialogue."
  field duckBus AudioBus "The bus that ducks this one." {
    default sfx
  }
  field duckDepth f32 "How far it is turned down." {
    default 9
    min 0
    max 60
    unit db
    tunable true
  }
  field duckThreshold f32 "How loud the key bus must be." {
    default -36
    min -80
    max 0
    unit db
    tunable true
  }
  field duckAttack f32 "How fast it goes down." {
    default 0.02
    min 0
    max 2
    unit s
    tunable true
  }
  field duckRelease f32 "How fast it comes back." {
    default 0.4
    min 0
    max 5
    unit s
    tunable true
  }
}
```

<sub>`api.ogt` line 8255</sub>

<a id="record-ReverbZone"></a>

### `og.ReverbZone` (component)

A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener.

Deterministic: each tick the listener's weight in every zone is a function of its position (written to `weight`), the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream and the world hash. Audio.reverbRoom, Audio.reverbDamping and the bus's AudioBusFx.reverb stay the open air's: the zone never writes them. The listener and the zone are placed at their world transforms (og.Transform down og.Parent): a zone on a child of a moving room moves with it.

Token `og.ReverbZone`, registered under the schema name `og.ReverbZone`; its value type is `ReverbZone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector3` | `{ x: 10, y: 4, z: 10 }` | — | `m` | — | The box's full size along the entity's axes, centred on it (times og.Transform.scale). |
| `radius` | `number` | `0` | ≥ 0 | `m` | tunable | A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box. |
| `fade` | `number` | `2` | ≥ 0 | `m` | tunable | How far past the volume's surface the zone fades out, linearly; 0 is a hard edge. |
| `bus` | `AudioBus` | `"sfx"` | — | — | — | The bus whose reverb send the zone sets. |
| `send` | `number` | `0.5` | 0 … 1 | — | tunable | That bus's send to the reverb inside (its AudioBusFx.reverb outside). |
| `room` | `number` | `0.85` | 0 … 1 | — | tunable | The reverb's room size inside (Audio.reverbRoom outside). |
| `damping` | `number` | `0.3` | 0 … 1 | — | tunable | The reverb's damping of high frequencies inside (Audio.reverbDamping outside). |
| `weight` | `number` | `0` | 0 … 1 | — | read only | How much of this zone the listener hears, 0 to 1: 1 inside, 0 past the fade (written by og.audio each tick). |

```ogt
component ReverbZone {
  doc "A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener."
  doc "Deterministic: each tick the listener's weight in every zone is a function of its position (written to `weight`), the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream and the world hash. Audio.reverbRoom, Audio.reverbDamping and the bus's AudioBusFx.reverb stay the open air's: the zone never writes them. The listener and the zone are placed at their world transforms (og.Transform down og.Parent): a zone on a child of a moving room moves with it."
  field size Vector3 "The box's full size along the entity's axes, centred on it (times og.Transform.scale)." {
    default 10 4 10
    unit m
  }
  field radius f32 "A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box." {
    min 0
    unit m
    tunable true
  }
  field fade f32 "How far past the volume's surface the zone fades out, linearly; 0 is a hard edge." {
    default 2
    min 0
    unit m
    tunable true
  }
  field bus AudioBus "The bus whose reverb send the zone sets." {
    default sfx
  }
  field send f32 "That bus's send to the reverb inside (its AudioBusFx.reverb outside)." {
    default 0.5
    min 0
    max 1
    tunable true
  }
  field room f32 "The reverb's room size inside (Audio.reverbRoom outside)." {
    default 0.85
    min 0
    max 1
    tunable true
  }
  field damping f32 "The reverb's damping of high frequencies inside (Audio.reverbDamping outside)." {
    default 0.3
    min 0
    max 1
    tunable true
  }
  field weight f32 "How much of this zone the listener hears, 0 to 1: 1 inside, 0 past the fade (written by og.audio each tick)." {
    min 0
    max 1
    readonly true
  }
}
```

<sub>`api.ogt` line 8344</sub>

<a id="record-Music"></a>

### `og.Music` (component)

A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat.

Token `og.Music`, registered under the schema name `og.Music`; its value type is `Music`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bpm` | `number` | `120` | 20 … 400 | `bpm` | tunable | Beats per minute. |
| `beatsPerBar` | `number` | `4` | 1 … 16 | — | — | Beats per bar. |
| `start` | `number` | `0` | ≥ 0 | `s` | — | The world time of bar 1's first beat. |
| `bar` | `number` | `0` | — | — | read only | The bar the next tick starts in, from 1 (0 before start; written by og.music). |
| `beat` | `number` | `0` | — | — | read only | The beat in that bar, from 1 (written by og.music). |
| `nextBeat` | `number` | `0` | — | `s` | read only | The world time of the next beat from the next tick on (written by og.music). |
| `nextBar` | `number` | `0` | — | `s` | read only | The world time of the next bar line from the next tick on (written by og.music). |

```ogt
component Music {
  doc "A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat."
  field bpm f32 "Beats per minute." {
    default 120
    min 20
    max 400
    unit bpm
    tunable true
  }
  field beatsPerBar u32 "Beats per bar." {
    default 4
    min 1
    max 16
  }
  field start f64 "The world time of bar 1's first beat." {
    min 0
    unit s
  }
  field bar u32 "The bar the next tick starts in, from 1 (0 before start; written by og.music)." {
    readonly true
  }
  field beat u32 "The beat in that bar, from 1 (written by og.music)." {
    readonly true
  }
  field nextBeat f64 "The world time of the next beat from the next tick on (written by og.music)." {
    unit s
    readonly true
  }
  field nextBar f64 "The world time of the next bar line from the next tick on (written by og.music)." {
    unit s
    readonly true
  }
}
```

<sub>`api.ogt` line 8390</sub>

<a id="record-MusicBeat"></a>

### `og.MusicBeat` (event)

A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after).

Token `og.MusicBeat`, registered under the schema name `og.MusicBeat`; its value type is `MusicBeat`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the Music. |
| `bar` | `number` | `0` | — | — | — | The bar, from 1. |
| `beat` | `number` | `0` | — | — | — | The beat in the bar, from 1 (1 is the bar line). |
| `time` | `number` | `0` | — | — | — | Its world time. |

```ogt
event MusicBeat {
  doc "A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after)."
  field entity Entity "The entity with the Music."
  field bar u32 "The bar, from 1."
  field beat u32 "The beat in the bar, from 1 (1 is the bar line)."
  field time f64 "Its world time."
}
```

<sub>`api.ogt` line 8424</sub>

<a id="record-SoundStarted"></a>

### `og.SoundStarted` (event)

A source started, sent in the tick it happens (systems read it the tick after).

Token `og.SoundStarted`, registered under the schema name `og.SoundStarted`; its value type is `SoundStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose AudioSource started. |
| `sound` | `Sound` | `0` | — | — | — | The sound. |
| `voice` | `number` | `0` | — | — | — | The voice id in the command stream. |

```ogt
event SoundStarted {
  doc "A source started, sent in the tick it happens (systems read it the tick after)."
  field entity Entity "The entity whose AudioSource started."
  field sound Sound "The sound."
  field voice u32 "The voice id in the command stream."
}
```

<sub>`api.ogt` line 8432</sub>

<a id="record-SoundEnded"></a>

### `og.SoundEnded` (event)

A sound ended, sent in the tick it happens (systems read it the tick after).

Token `og.SoundEnded`, registered under the schema name `og.SoundEnded`; its value type is `SoundEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose AudioSource ended. |
| `sound` | `Sound` | `0` | — | — | — | The sound. |
| `voice` | `number` | `0` | — | — | — | The voice id in the command stream. |
| `reason` | `number` | `0` | — | — | — | Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3) or SOUND_CULLED (4). |

```ogt
event SoundEnded {
  doc "A sound ended, sent in the tick it happens (systems read it the tick after)."
  field entity Entity "The entity whose AudioSource ended."
  field sound Sound "The sound."
  field voice u32 "The voice id in the command stream."
  field reason u32 "Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3) or SOUND_CULLED (4)."
}
```

<sub>`api.ogt` line 8439</sub>

## Types

<a id="type-AudioBus"></a>

### `AudioBus` (enum)

The bus a source plays on; every bus passes through master.

A string: one of the variants below. The zero value is `"master"`.

| Variant | Doc |
|---|---|
| `"master"` | The master bus: what every bus passes through. |
| `"music"` | Music. |
| `"sfx"` | Sound effects. |
| `"ui"` | Interface sounds. |

```ogt
enum AudioBus {
  doc "The bus a source plays on; every bus passes through master."
  variant master "The master bus: what every bus passes through."
  variant music "Music."
  variant sfx "Sound effects."
  variant ui "Interface sounds."
}
```

<sub>`api.ogt` line 8079</sub>

## Column kinds

Each is a factory, `AudioBus(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-AudioBus"></a>`AudioBus` | `AudioBus` | `Uint32Array` | 1 | An AudioBus field: the variant's index. |

```ogt
column AudioBus {
  doc "An AudioBus field: the variant's index."
  array Uint32Array
  stride 1
  value AudioBus
}
```

<sub>`api.ogt` line 8087</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-SOUND_FINISHED"></a>`SOUND_FINISHED` | `number` | `0` | SoundEnded.reason: a one-shot played to its end. |
| <a id="const-SOUND_STOPPED"></a>`SOUND_STOPPED` | `number` | `1` | SoundEnded.reason: `playing` was set to false. |
| <a id="const-SOUND_REMOVED"></a>`SOUND_REMOVED` | `number` | `2` | SoundEnded.reason: the AudioSource was removed from a living entity. |
| <a id="const-SOUND_REPLACED"></a>`SOUND_REPLACED` | `number` | `3` | SoundEnded.reason: `sound` changed while it played. |
| <a id="const-SOUND_CULLED"></a>`SOUND_CULLED` | `number` | `4` | SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age). |

```ogt
const SOUND_FINISHED u32 0 "SoundEnded.reason: a one-shot played to its end."
```

<sub>`api.ogt` line 8073</sub>

```ogt
const SOUND_STOPPED u32 1 "SoundEnded.reason: `playing` was set to false."
```

<sub>`api.ogt` line 8074</sub>

```ogt
const SOUND_REMOVED u32 2 "SoundEnded.reason: the AudioSource was removed from a living entity."
```

<sub>`api.ogt` line 8075</sub>

```ogt
const SOUND_REPLACED u32 3 "SoundEnded.reason: `sound` changed while it played."
```

<sub>`api.ogt` line 8076</sub>

```ogt
const SOUND_CULLED u32 4 "SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age)."
```

<sub>`api.ogt` line 8077</sub>
