<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `audio` module

Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).

The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.

Spatial gain and pan are computed in the tick from the source's og.Transform relative to the first AudioListener (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z).

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module audio {
  doc "Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively)."
  doc "The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends."
  doc "Spatial gain and pan are computed in the tick from the source's og.Transform relative to the first AudioListener (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z)."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 5262</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`SOUND_FINISHED`](#const-SOUND_FINISHED) | constant | SoundEnded.reason: a one-shot played to its end. |
| [`SOUND_STOPPED`](#const-SOUND_STOPPED) | constant | SoundEnded.reason: `playing` was set to false. |
| [`SOUND_REMOVED`](#const-SOUND_REMOVED) | constant | SoundEnded.reason: the AudioSource was removed from a living entity. |
| [`SOUND_REPLACED`](#const-SOUND_REPLACED) | constant | SoundEnded.reason: `sound` changed while it played. |
| [`AudioBus`](#type-AudioBus) | enum | The bus a source plays on; every bus passes through master. |
| [`AudioBus()`](#column-AudioBus) | column kind | An AudioBus field: the variant's index. |
| [`og.AudioSource`](#record-AudioSource) | component | A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded. |
| [`og.AudioListener`](#record-AudioListener) | component | Where spatial sources are heard from: the entity's og.Transform, looking down its -Z with +X to the right. The first listener in query order is used. |
| [`og.Audio`](#record-Audio) | resource | Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer. |
| [`og.SoundStarted`](#record-SoundStarted) | event | A source started, sent in the tick it happens (systems read it the tick after). |
| [`og.SoundEnded`](#record-SoundEnded) | event | A sound ended, sent in the tick it happens (systems read it the tick after). |

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
| `spatial` | `boolean` | `false` | — | — | — | Attenuate and pan by the entity's og.Transform relative to the listener. |
| `refDistance` | `number` | `1` | ≥ 0 | `m` | tunable | Full volume up to this distance, as Web Audio's PannerNode.refDistance. |
| `maxDistance` | `number` | `100` | ≥ 0 | `m` | tunable | No quieter past this distance, as Web Audio's PannerNode.maxDistance. |
| `rolloffFactor` | `number` | `1` | ≥ 0 | — | tunable | How fast volume falls with distance, in Web Audio's inverse model: ref / (ref + rolloff * (clamp(d, ref, max) - ref)). |
| `playing` | `boolean` | `true` | — | — | — | Plays while true; the engine sets it false when a one-shot ends. |

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
  field spatial bool "Attenuate and pan by the entity's og.Transform relative to the listener."
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
}
```

<sub>`api.ogt` line 5287</sub>

<a id="record-AudioListener"></a>

### `og.AudioListener` (component)

Where spatial sources are heard from: the entity's og.Transform, looking down its -Z with +X to the right. The first listener in query order is used.

Token `og.AudioListener`, registered under the schema name `og.AudioListener`; its value type is `AudioListener`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `volume` | `number` | `1` | 0 … 4 | — | tunable | A gain on every spatial source this listener hears. |

```ogt
component AudioListener {
  doc "Where spatial sources are heard from: the entity's og.Transform, looking down its -Z with +X to the right. The first listener in query order is used."
  field volume f32 "A gain on every spatial source this listener hears." {
    default 1
    min 0
    max 4
    tunable true
  }
}
```

<sub>`api.ogt` line 5343</sub>

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
}
```

<sub>`api.ogt` line 5353</sub>

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

<sub>`api.ogt` line 5381</sub>

<a id="record-SoundEnded"></a>

### `og.SoundEnded` (event)

A sound ended, sent in the tick it happens (systems read it the tick after).

Token `og.SoundEnded`, registered under the schema name `og.SoundEnded`; its value type is `SoundEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose AudioSource ended. |
| `sound` | `Sound` | `0` | — | — | — | The sound. |
| `voice` | `number` | `0` | — | — | — | The voice id in the command stream. |
| `reason` | `number` | `0` | — | — | — | Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2) or SOUND_REPLACED (3). |

```ogt
event SoundEnded {
  doc "A sound ended, sent in the tick it happens (systems read it the tick after)."
  field entity Entity "The entity whose AudioSource ended."
  field sound Sound "The sound."
  field voice u32 "The voice id in the command stream."
  field reason u32 "Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2) or SOUND_REPLACED (3)."
}
```

<sub>`api.ogt` line 5388</sub>

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

<sub>`api.ogt` line 5272</sub>

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

<sub>`api.ogt` line 5280</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-SOUND_FINISHED"></a>`SOUND_FINISHED` | `number` | `0` | SoundEnded.reason: a one-shot played to its end. |
| <a id="const-SOUND_STOPPED"></a>`SOUND_STOPPED` | `number` | `1` | SoundEnded.reason: `playing` was set to false. |
| <a id="const-SOUND_REMOVED"></a>`SOUND_REMOVED` | `number` | `2` | SoundEnded.reason: the AudioSource was removed from a living entity. |
| <a id="const-SOUND_REPLACED"></a>`SOUND_REPLACED` | `number` | `3` | SoundEnded.reason: `sound` changed while it played. |

```ogt
const SOUND_FINISHED u32 0 "SoundEnded.reason: a one-shot played to its end."
```

<sub>`api.ogt` line 5267</sub>

```ogt
const SOUND_STOPPED u32 1 "SoundEnded.reason: `playing` was set to false."
```

<sub>`api.ogt` line 5268</sub>

```ogt
const SOUND_REMOVED u32 2 "SoundEnded.reason: the AudioSource was removed from a living entity."
```

<sub>`api.ogt` line 5269</sub>

```ogt
const SOUND_REPLACED u32 3 "SoundEnded.reason: `sound` changed while it played."
```

<sub>`api.ogt` line 5270</sub>
