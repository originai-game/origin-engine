<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `audio` module

Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).

The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.

Spatial gain and pan are computed in the tick from the source's world transform relative to the first AudioListener's (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z). A source, listener or zone on a child entity is placed at its og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a child is heard where its parent carries it, a listener under a camera rig hears from the rig.

Propagation, all in the tick and so in the stream: a spatial source or listener that moves shifts the pitch (Doppler, from their tick velocities: AudioSource.dopplerFactor, AudioListener.dopplerFactor, Audio.speedOfSound), a source with a cone is turned down away from its +Z (coneInnerAngle, coneOuterAngle, coneOuterGain, as three.js's setDirectionalCone), and a listener whose panningModel is hrtf places sources for headphones (interaural delay and head shadow) instead of the stereo panner.

- **TypeScript**: functions are the namespace `og.audio`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-audio`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module audio {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 11573</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`SOUND_FINISHED`](#const-SOUND_FINISHED) | constant | SoundEnded.reason: a one-shot played to its end. |
| [`SOUND_STOPPED`](#const-SOUND_STOPPED) | constant | SoundEnded.reason: `playing` was set to false. |
| [`SOUND_REMOVED`](#const-SOUND_REMOVED) | constant | SoundEnded.reason: the AudioSource was removed from a living entity. |
| [`SOUND_REPLACED`](#const-SOUND_REPLACED) | constant | SoundEnded.reason: `sound` changed while it played. |
| [`SOUND_CULLED`](#const-SOUND_CULLED) | constant | SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age). |
| [`SOUND_RETRIGGERED`](#const-SOUND_RETRIGGERED) | constant | SoundEnded.reason: AudioSource.retrigger started the sound again from the top; a new voice plays it. |
| [`AudioBus`](#type-AudioBus) | enum | The bus a source plays on; every bus passes through master. |
| [`AudioBus()`](#column-AudioBus) | column kind | An AudioBus field: the variant's index. |
| [`PanningModel`](#type-PanningModel) | enum | How an AudioListener places spatial sources between the ears, as Web Audio's PannerNode.panningModel. |
| [`AudioResampler`](#type-AudioResampler) | enum | How the mixer resamples a voice whose pitch or sound rate differs from the output's. |
| [`AudioResampler()`](#column-AudioResampler) | column kind | An AudioResampler field: the variant's index. |
| [`PanningModel()`](#column-PanningModel) | column kind | A PanningModel field: the variant's index. |
| [`og.AudioSource`](#record-AudioSource) | component | A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded. |
| [`og.AudioListener`](#record-AudioListener) | component | Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used. |
| [`og.Audio`](#record-Audio) | resource | Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer. |
| [`og.AudioBusFx`](#record-AudioBusFx) | component | A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through. |
| [`og.AudioBusNode`](#record-AudioBusNode) | component | A user bus of the bus graph: bus `index` (4 to 15) sums the sources whose AudioSource.userBus (or AudioEvent.userBus) names it, runs its og.AudioBusFx (the one whose userBus is the index), and plays into its `parent`: a built-in bus or a user bus of a lower index, so the graph has no cycles (any other parent is master). Its volume is multiplied down the chain to master, so a footsteps bus under sfx is turned down with sfx. The first node in query order for an index counts; a removed node's sources play on their `bus`. |
| [`og.AudioSnapshot`](#record-AudioSnapshot) | component | A mixer snapshot (combat, underwater, paused): per-bus volumes and filters the mix blends towards while `active`, and back from when not, over `fade` seconds. Each tick og.audio moves `weight` by tickDuration / fade towards 1 or 0 and sends the blend in the stream; the bus gains ramp linearly across each tick, so a blend is a straight line to the sample, not a staircase. |
| [`AudioEventOrder`](#type-AudioEventOrder) | enum | How a AudioEvent chooses its next sound. |
| [`AudioEventOrder()`](#column-AudioEventOrder) | column kind | A AudioEventOrder field: the variant's index. |
| [`og.AudioEvent`](#record-AudioEvent) | component | An audio event: one of a set of sounds played on each trigger, with pitch and volume jitter, a cooldown and a voice limit (footsteps, impacts, gunshots). Set `trigger` to play it; the engine sets it back to false. |
| [`SoundDesc`](#type-SoundDesc) | desc struct | PCM a game made itself (og.audio.createSound): a tone, a procedural effect, a recording it synthesised. |
| [`og.ReverbZone`](#record-ReverbZone) | component | A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener. |
| [`og.Music`](#record-Music) | component | A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat. |
| [`og.MusicBeat`](#record-MusicBeat) | event | A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after). |
| [`og.SoundStarted`](#record-SoundStarted) | event | A source started, sent in the tick it happens (systems read it the tick after). |
| [`og.SoundEnded`](#record-SoundEnded) | event | A sound ended, sent in the tick it happens (systems read it the tick after). |
| [`og.audio.createSound`](#fn-createSound) | function | Make a sound from PCM the game computed: it plays, loops and pitches as a loaded one (AudioSource.sound, AudioEvent.sounds), always resident, stored as 16-bit PCM, and og.load.soundInfo reports it. Between ticks or inside a system alike; the handle is the next one the bank gives, so a game that makes its sounds in the same order on every launch (in GameDesc.start) names the same handles in its saves. |
| [`og.audio.outputLatency`](#fn-outputLatency) | function | How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand). |

## Functions

<a id="fn-createSound"></a>

### `og.audio.createSound`

Make a sound from PCM the game computed: it plays, loops and pitches as a loaded one (AudioSource.sound, AudioEvent.sounds), always resident, stored as 16-bit PCM, and og.load.soundInfo reports it. Between ticks or inside a system alike; the handle is the next one the bank gives, so a game that makes its sounds in the same order on every launch (in GameDesc.start) names the same handles in its saves.

```ts
og.audio.createSound(world: World, desc?: SoundDesc): Result<Sound>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the sound. |
| `desc` | `SoundDesc` | yes | The samples and their format. |

**Returns** `Result<Sound>`: Refused for no samples, a length that is not whole frames, a rate or channel count out of range, or a number that is not finite.

Binding symbol `og_audio_create_sound` · publish builds: kept

```ogt
fn createSound {
  doc "Make a sound from PCM the game computed: it plays, loops and pitches as a loaded one (AudioSource.sound, AudioEvent.sounds), always resident, stored as 16-bit PCM, and og.load.soundInfo reports it. Between ticks or inside a system alike; the handle is the next one the bank gives, so a game that makes its sounds in the same order on every launch (in GameDesc.start) names the same handles in its saves."
  param world World "The world that owns the sound."
  param desc SoundDesc "The samples and their format."
  returns Result.Sound "Refused for no samples, a length that is not whole frames, a rate or channel count out of range, or a number that is not finite."
}
```

<sub>`api.ogt` line 12269</sub>

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

<sub>`api.ogt` line 12276</sub>

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
| `dopplerFactor` | `number` | `1` | 0 … 10 | — | tunable | Doppler: how much a spatial source's and the listener's motion along the line between them shifts its pitch, from their tick velocities (world positions down og.Parent): pitch times (c - DF * vListener) / (c - DF * vSource), c being Audio.speedOfSound and DF this times AudioListener.dopplerFactor (OpenAL's model). 1 is physical, 0 is off. A jump at or past the speed of sound in a tick is a teleport and shifts nothing; DF * v is held within c/2, so the shift stays within a factor of 3. |
| `coneInnerAngle` | `number` | `360` | 0 … 360 | `deg` | tunable | Directivity, as three.js PositionalAudio.setDirectionalCone and Web Audio's PannerNode: a spatial source plays at full volume while the listener is within this full angle about the source's +Z (its world transform's, three.js's PositionalAudio orientation); 360 with coneOuterAngle 360 is omnidirectional. |
| `coneOuterAngle` | `number` | `360` | 0 … 360 | `deg` | tunable | Past this full angle about the source's +Z it plays at coneOuterGain; between coneInnerAngle and this the gain falls linearly with the angle. |
| `coneOuterGain` | `number` | `0` | 0 … 1 | — | tunable | The gain outside coneOuterAngle. |
| `virtualize` | `boolean` | `false` | — | — | — | Past Audio.maxVoices it goes virtual instead of being culled: silent, its playhead running on (a one-shot still ends on its tick), and heard again from where it would be, fading in, once it stands within the cap. Without it the source is culled (SOUND_CULLED, playing false). |
| `retrigger` | `boolean` | `false` | — | — | — | Set true to start the sound again from the top (its startOffset, its startAt) this tick, playing or not, without a level change on `playing`: the voice playing ends (SOUND_RETRIGGERED) and a new one starts in the same tick. The engine sets it back to false and `playing` true. |
| `userBus` | `number` | `0` | 0 … 15 | — | — | A user bus of the bus graph (4 to 15, declared by an og.AudioBusNode) it plays into instead of `bus`; 0, or an index no node declares, plays on `bus`. |

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
  field dopplerFactor f32 "Doppler: how much a spatial source's and the listener's motion along the line between them shifts its pitch, from their tick velocities (world positions down og.Parent): pitch times (c - DF * vListener) / (c - DF * vSource), c being Audio.speedOfSound and DF this times AudioListener.dopplerFactor (OpenAL's model). 1 is physical, 0 is off. A jump at or past the speed of sound in a tick is a teleport and shifts nothing; DF * v is held within c/2, so the shift stays within a factor of 3." {
    default 1
    min 0
    max 10
    tunable true
  }
  field coneInnerAngle f32 "Directivity, as three.js PositionalAudio.setDirectionalCone and Web Audio's PannerNode: a spatial source plays at full volume while the listener is within this full angle about the source's +Z (its world transform's, three.js's PositionalAudio orientation); 360 with coneOuterAngle 360 is omnidirectional." {
    default 360
    min 0
    max 360
    unit deg
    tunable true
  }
  field coneOuterAngle f32 "Past this full angle about the source's +Z it plays at coneOuterGain; between coneInnerAngle and this the gain falls linearly with the angle." {
    default 360
    min 0
    max 360
    unit deg
    tunable true
  }
  field coneOuterGain f32 "The gain outside coneOuterAngle." {
    min 0
    max 1
    tunable true
  }
  field virtualize bool "Past Audio.maxVoices it goes virtual instead of being culled: silent, its playhead running on (a one-shot still ends on its tick), and heard again from where it would be, fading in, once it stands within the cap. Without it the source is culled (SOUND_CULLED, playing false)."
  field retrigger bool "Set true to start the sound again from the top (its startOffset, its startAt) this tick, playing or not, without a level change on `playing`: the voice playing ends (SOUND_RETRIGGERED) and a new one starts in the same tick. The engine sets it back to false and `playing` true."
  field userBus u32 "A user bus of the bus graph (4 to 15, declared by an og.AudioBusNode) it plays into instead of `bus`; 0, or an index no node declares, plays on `bus`." {
    min 0
    max 15
  }
}
```

<sub>`api.ogt` line 11628</sub>

<a id="record-AudioListener"></a>

### `og.AudioListener` (component)

Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used.

Token `og.AudioListener`, registered under the schema name `og.AudioListener`; its value type is `AudioListener`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `volume` | `number` | `1` | 0 … 4 | — | tunable | A gain on every spatial source this listener hears. |
| `dopplerFactor` | `number` | `1` | 0 … 10 | — | tunable | Scales every spatial source's Doppler shift (AudioSource.dopplerFactor); 0 turns Doppler off for this listener. |
| `panningModel` | `PanningModel` | `"equalpower"` | — | — | — | How spatial sources are placed between the ears: equalpower (speakers) or hrtf (headphones). Non-spatial sources keep their pan either way; the stereo path is unchanged while it is equalpower. |

```ogt
component AudioListener {
  doc "Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used."
  field volume f32 "A gain on every spatial source this listener hears." {
    default 1
    min 0
    max 4
    tunable true
  }
  field dopplerFactor f32 "Scales every spatial source's Doppler shift (AudioSource.dopplerFactor); 0 turns Doppler off for this listener." {
    default 1
    min 0
    max 10
    tunable true
  }
  field panningModel PanningModel "How spatial sources are placed between the ears: equalpower (speakers) or hrtf (headphones). Non-spatial sources keep their pan either way; the stereo path is unchanged while it is equalpower." {
    default equalpower
  }
}
```

<sub>`api.ogt` line 11747</sub>

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
| `speedOfSound` | `number` | `343.3` | 1 … 10000 | `mps` | tunable | The speed of sound Doppler shifts are computed against (AudioSource.dopplerFactor). |
| `resampler` | `AudioResampler` | `"sinc"` | — | — | — | How voices are resampled: sinc (the default) or linear, the mix before round 4, for a game that relies on its exact output. A sound at the output's rate played at pitch 1 is its own samples either way. |

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
  field speedOfSound f32 "The speed of sound Doppler shifts are computed against (AudioSource.dopplerFactor)." {
    default 343.3
    min 1
    max 10000
    unit mps
    tunable true
  }
  field resampler AudioResampler "How voices are resampled: sinc (the default) or linear, the mix before round 4, for a game that relies on its exact output. A sound at the output's rate played at pitch 1 is its own samples either way." {
    default sinc
  }
}
```

<sub>`api.ogt` line 11766</sub>

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
| `userBus` | `number` | `0` | 0 … 15 | — | — | A user bus (4 to 15, an og.AudioBusNode) these effects are on instead of `bus`; 0 uses `bus`. |
| `eqLow` | `number` | `0` | -24 … 24 | `db` | tunable | The equaliser's low shelf (RBJ, 12 dB/octave): its gain below eqLowFreq; 0 dB is flat. |
| `eqLowFreq` | `number` | `200` | 20 … 2000 | `hz` | tunable | The low shelf's corner frequency. |
| `eqMid` | `number` | `0` | -24 … 24 | `db` | tunable | The equaliser's mid band (a peak, or a notch below 0 dB): its gain at eqMidFreq; 0 dB is flat. |
| `eqMidFreq` | `number` | `1000` | 40 … 16000 | `hz` | tunable | The mid band's centre frequency. |
| `eqMidQ` | `number` | `0.7` | 0.1 … 10 | — | tunable | The mid band's Q: higher is narrower. |
| `eqHigh` | `number` | `0` | -24 … 24 | `db` | tunable | The equaliser's high shelf: its gain above eqHighFreq; 0 dB is flat. |
| `eqHighFreq` | `number` | `5000` | 1000 … 20000 | `hz` | tunable | The high shelf's corner frequency. |
| `delay` | `number` | `0` | 0 … 2 | `s` | tunable | An echo after the compressor: the delay's time, to the sample; 0 is off. |
| `delayFeedback` | `number` | `0.35` | 0 … 0.95 | — | tunable | How much of each echo feeds the next. |
| `delayMix` | `number` | `0.35` | 0 … 1 | — | tunable | How loud the echoes are, added to the dry bus. |
| `distortion` | `number` | `0` | 0 … 1 | — | tunable | A tanh waveshaper after the equaliser: its drive, 0 (off) to 1 (heavy: 20x into the curve), scaled so a full-scale input stays at full scale. |
| `distortionMix` | `number` | `1` | 0 … 1 | — | tunable | How much of the distorted signal replaces the clean one. |

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
  field userBus u32 "A user bus (4 to 15, an og.AudioBusNode) these effects are on instead of `bus`; 0 uses `bus`." {
    min 0
    max 15
  }
  field eqLow f32 "The equaliser's low shelf (RBJ, 12 dB/octave): its gain below eqLowFreq; 0 dB is flat." {
    min -24
    max 24
    unit db
    tunable true
  }
  field eqLowFreq f32 "The low shelf's corner frequency." {
    default 200
    min 20
    max 2000
    unit hz
    tunable true
  }
  field eqMid f32 "The equaliser's mid band (a peak, or a notch below 0 dB): its gain at eqMidFreq; 0 dB is flat." {
    min -24
    max 24
    unit db
    tunable true
  }
  field eqMidFreq f32 "The mid band's centre frequency." {
    default 1000
    min 40
    max 16000
    unit hz
    tunable true
  }
  field eqMidQ f32 "The mid band's Q: higher is narrower." {
    default 0.7
    min 0.1
    max 10
    tunable true
  }
  field eqHigh f32 "The equaliser's high shelf: its gain above eqHighFreq; 0 dB is flat." {
    min -24
    max 24
    unit db
    tunable true
  }
  field eqHighFreq f32 "The high shelf's corner frequency." {
    default 5000
    min 1000
    max 20000
    unit hz
    tunable true
  }
  field delay f32 "An echo after the compressor: the delay's time, to the sample; 0 is off." {
    min 0
    max 2
    unit s
    tunable true
  }
  field delayFeedback f32 "How much of each echo feeds the next." {
    default 0.35
    min 0
    max 0.95
    tunable true
  }
  field delayMix f32 "How loud the echoes are, added to the dry bus." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field distortion f32 "A tanh waveshaper after the equaliser: its drive, 0 (off) to 1 (heavy: 20x into the curve), scaled so a full-scale input stays at full scale." {
    min 0
    max 1
    tunable true
  }
  field distortionMix f32 "How much of the distorted signal replaces the clean one." {
    default 1
    min 0
    max 1
    tunable true
  }
}
```

<sub>`api.ogt` line 11839</sub>

<a id="record-AudioBusNode"></a>

### `og.AudioBusNode` (component)

A user bus of the bus graph: bus `index` (4 to 15) sums the sources whose AudioSource.userBus (or AudioEvent.userBus) names it, runs its og.AudioBusFx (the one whose userBus is the index), and plays into its `parent`: a built-in bus or a user bus of a lower index, so the graph has no cycles (any other parent is master). Its volume is multiplied down the chain to master, so a footsteps bus under sfx is turned down with sfx. The first node in query order for an index counts; a removed node's sources play on their `bus`.

The graph reaches the mixer as commands in the stream (only what changed each tick), so it is deterministic and in the world hash.

Token `og.AudioBusNode`, registered under the schema name `og.AudioBusNode`; its value type is `AudioBusNode`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `4` | 4 … 15 | — | — | The bus's index, 4 to 15. |
| `parent` | `number` | `0` | 0 … 15 | — | — | The bus it plays into: 0 master, 1 music, 2 sfx, 3 ui, or a user bus below `index`. |
| `volume` | `number` | `1` | 0 … 2 | — | tunable | The bus's gain. |

```ogt
component AudioBusNode {
  doc "A user bus of the bus graph: bus `index` (4 to 15) sums the sources whose AudioSource.userBus (or AudioEvent.userBus) names it, runs its og.AudioBusFx (the one whose userBus is the index), and plays into its `parent`: a built-in bus or a user bus of a lower index, so the graph has no cycles (any other parent is master). Its volume is multiplied down the chain to master, so a footsteps bus under sfx is turned down with sfx. The first node in query order for an index counts; a removed node's sources play on their `bus`."
  doc "The graph reaches the mixer as commands in the stream (only what changed each tick), so it is deterministic and in the world hash."
  field index u32 "The bus's index, 4 to 15." {
    default 4
    min 4
    max 15
  }
  field parent u32 "The bus it plays into: 0 master, 1 music, 2 sfx, 3 ui, or a user bus below `index`." {
    min 0
    max 15
  }
  field volume f32 "The bus's gain." {
    default 1
    min 0
    max 2
    tunable true
  }
}
```

<sub>`api.ogt` line 12006</sub>

<a id="record-AudioSnapshot"></a>

### `og.AudioSnapshot` (component)

A mixer snapshot (combat, underwater, paused): per-bus volumes and filters the mix blends towards while `active`, and back from when not, over `fade` seconds. Each tick og.audio moves `weight` by tickDuration / fade towards 1 or 0 and sends the blend in the stream; the bus gains ramp linearly across each tick, so a blend is a straight line to the sample, not a staircase.

At weight w a bus's gain is multiplied by 1 + w * (volume - 1); its low-pass moves from 20 kHz and its high-pass from 20 Hz towards the snapshot's cutoffs in octaves (log2-linear in w). Several snapshots multiply their volumes and take the lowest low-pass and the highest high-pass. Audio's bus volumes and the AudioBusFx filters stay what the game set: the snapshot gain multiplies them, and of its cutoffs and the AudioBusFx's the lower low-pass and the higher high-pass count.

Token `og.AudioSnapshot`, registered under the schema name `og.AudioSnapshot`; its value type is `AudioSnapshot`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `active` | `boolean` | `false` | — | — | — | Blends in while true, out while false. |
| `fade` | `number` | `0.5` | 0 … 30 | `s` | tunable | Seconds a full blend in or out takes; 0 is at once. |
| `weight` | `number` | `0` | 0 … 1 | — | read only | How far it is blended in, 0 to 1 (written by og.audio each tick). |
| `volume` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per bus, by index (master, music, sfx, ui, then user buses 4 to 15), the gain it multiplies in at full weight; buses past the list's length keep 1: `{ volume: [1, 0.3] }` turns the music down. |
| `lowpass` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per bus, a low-pass cutoff in Hz at full weight; 0 or past the list's length is none: `{ lowpass: [0, 0, 600] }` muffles sfx (underwater). |
| `highpass` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per bus, a high-pass cutoff in Hz at full weight; 0 or past the list's length is none. |

```ogt
component AudioSnapshot {
  doc "A mixer snapshot (combat, underwater, paused): per-bus volumes and filters the mix blends towards while `active`, and back from when not, over `fade` seconds. Each tick og.audio moves `weight` by tickDuration / fade towards 1 or 0 and sends the blend in the stream; the bus gains ramp linearly across each tick, so a blend is a straight line to the sample, not a staircase."
  doc "At weight w a bus's gain is multiplied by 1 + w * (volume - 1); its low-pass moves from 20 kHz and its high-pass from 20 Hz towards the snapshot's cutoffs in octaves (log2-linear in w). Several snapshots multiply their volumes and take the lowest low-pass and the highest high-pass. Audio's bus volumes and the AudioBusFx filters stay what the game set: the snapshot gain multiplies them, and of its cutoffs and the AudioBusFx's the lower low-pass and the higher high-pass count."
  field active bool "Blends in while true, out while false."
  field fade f32 "Seconds a full blend in or out takes; 0 is at once." {
    default 0.5
    min 0
    max 30
    unit s
    tunable true
  }
  field weight f32 "How far it is blended in, 0 to 1 (written by og.audio each tick)." {
    min 0
    max 1
    readonly true
  }
  field volume List.f32.16 "Per bus, by index (master, music, sfx, ui, then user buses 4 to 15), the gain it multiplies in at full weight; buses past the list's length keep 1: `{ volume: [1, 0.3] }` turns the music down."
  field lowpass List.f32.16 "Per bus, a low-pass cutoff in Hz at full weight; 0 or past the list's length is none: `{ lowpass: [0, 0, 600] }` muffles sfx (underwater)."
  field highpass List.f32.16 "Per bus, a high-pass cutoff in Hz at full weight; 0 or past the list's length is none."
}
```

<sub>`api.ogt` line 12026</sub>

<a id="record-AudioEvent"></a>

### `og.AudioEvent` (component)

An audio event: one of a set of sounds played on each trigger, with pitch and volume jitter, a cooldown and a voice limit (footsteps, impacts, gunshots). Set `trigger` to play it; the engine sets it back to false.

The og.audioEvents system (before og.audio) picks the sound and the jitter from the engine's own random stream, forked from og.Rng's seed with the label og.audio and kept in the world (never Math.random), so the same seed and the same triggers play the same sounds at the same pitches on every host and every replay. Each play spawns a voice entity (an og.AudioSource, parented to this entity with og.Parent, so a spatial event is heard where its entity is) that is despawned when it ends; nothing of a voice feeds back into the simulation but `played`, `last`, `index`, `bag` and `voices`.

Token `og.AudioEvent`, registered under the schema name `og.AudioEvent`; its value type is `AudioEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sounds` | `readonly Sound[]` | `[]` | — | — | a list of at most 8 `Sound` (`List.Sound.8`): reads a frozen copy, writes a whole array | The variations (og.load.sound or og.audio.createSound); 0s are skipped. |
| `weights` | `readonly number[]` | `[]` | — | — | a list of at most 8 `f32` (`List.f32.8`): reads a frozen copy, writes a whole array | Weights of a random pick, by position in `sounds`; past the list's length 1. |
| `order` | `AudioEventOrder` | `"random"` | — | — | — | How the next sound is chosen. |
| `volume` | `number` | `1` | 0 … 4 | — | tunable | Linear gain. |
| `volumeJitter` | `number` | `0` | 0 … 24 | `db` | tunable | Each play's volume varies uniformly by up to this many dB either way. |
| `pitch` | `number` | `1` | 0.0625 … 16 | — | tunable | Playback rate. |
| `pitchJitter` | `number` | `0` | 0 … 24 | — | tunable | Each play's pitch varies uniformly by up to this many semitones either way. |
| `cooldown` | `number` | `0` | 0 … 60 | `s` | tunable | A trigger sooner than this after the last play is ignored (and still reset). |
| `maxVoices` | `number` | `0` | 0 … 64 | — | — | The most voices of this event at once; 0 is no limit. |
| `steal` | `boolean` | `true` | — | — | — | At maxVoices, the oldest voice is stopped (SOUND_STOPPED) for the new one; false ignores the trigger instead. |
| `bus` | `AudioBus` | `"sfx"` | — | — | — | The bus its voices play on. |
| `userBus` | `number` | `0` | 0 … 15 | — | — | A user bus its voices play into instead (AudioSource.userBus). |
| `spatial` | `boolean` | `false` | — | — | — | Its voices are spatial, heard at this entity's world transform. |
| `refDistance` | `number` | `1` | ≥ 0 | `m` | tunable | AudioSource.refDistance of its voices. |
| `maxDistance` | `number` | `100` | ≥ 0 | `m` | tunable | AudioSource.maxDistance of its voices. |
| `priority` | `number` | `128` | 0 … 255 | — | — | AudioSource.priority of its voices. |
| `trigger` | `boolean` | `false` | — | — | — | Set true to play it this tick; the engine sets it back to false. |
| `played` | `number` | `0` | — | — | read only | Plays so far (written by og.audioEvents). |
| `last` | `number` | `0` | — | `s` | read only | The world time of the last play (written by og.audioEvents). |
| `index` | `number` | `0` | — | — | read only | The position in `sounds` played last (written by og.audioEvents). |
| `bag` | `number` | `0` | — | — | read only | A shuffle's sounds already played this round, one bit each (written by og.audioEvents). |
| `voices` | `number` | `0` | — | — | read only | Its voices playing now (written by og.audioEvents). |

```ogt
component AudioEvent {
  doc "An audio event: one of a set of sounds played on each trigger, with pitch and volume jitter, a cooldown and a voice limit (footsteps, impacts, gunshots). Set `trigger` to play it; the engine sets it back to false."
  doc "The og.audioEvents system (before og.audio) picks the sound and the jitter from the engine's own random stream, forked from og.Rng's seed with the label og.audio and kept in the world (never Math.random), so the same seed and the same triggers play the same sounds at the same pitches on every host and every replay. Each play spawns a voice entity (an og.AudioSource, parented to this entity with og.Parent, so a spatial event is heard where its entity is) that is despawned when it ends; nothing of a voice feeds back into the simulation but `played`, `last`, `index`, `bag` and `voices`."
  field sounds List.Sound.8 "The variations (og.load.sound or og.audio.createSound); 0s are skipped."
  field weights List.f32.8 "Weights of a random pick, by position in `sounds`; past the list's length 1."
  field order AudioEventOrder "How the next sound is chosen." {
    default random
  }
  field volume f32 "Linear gain." {
    default 1
    min 0
    max 4
    tunable true
  }
  field volumeJitter f32 "Each play's volume varies uniformly by up to this many dB either way." {
    min 0
    max 24
    unit db
    tunable true
  }
  field pitch f32 "Playback rate." {
    default 1
    min 0.0625
    max 16
    tunable true
  }
  field pitchJitter f32 "Each play's pitch varies uniformly by up to this many semitones either way." {
    min 0
    max 24
    tunable true
  }
  field cooldown f32 "A trigger sooner than this after the last play is ignored (and still reset)." {
    min 0
    max 60
    unit s
    tunable true
  }
  field maxVoices u32 "The most voices of this event at once; 0 is no limit." {
    min 0
    max 64
  }
  field steal bool "At maxVoices, the oldest voice is stopped (SOUND_STOPPED) for the new one; false ignores the trigger instead." {
    default true
  }
  field bus AudioBus "The bus its voices play on." {
    default sfx
  }
  field userBus u32 "A user bus its voices play into instead (AudioSource.userBus)." {
    min 0
    max 15
  }
  field spatial bool "Its voices are spatial, heard at this entity's world transform."
  field refDistance f32 "AudioSource.refDistance of its voices." {
    default 1
    min 0
    unit m
    tunable true
  }
  field maxDistance f32 "AudioSource.maxDistance of its voices." {
    default 100
    min 0
    unit m
    tunable true
  }
  field priority u32 "AudioSource.priority of its voices." {
    default 128
    min 0
    max 255
  }
  field trigger bool "Set true to play it this tick; the engine sets it back to false."
  field played u32 "Plays so far (written by og.audioEvents)." {
    readonly true
  }
  field last f64 "The world time of the last play (written by og.audioEvents)." {
    unit s
    readonly true
  }
  field index u32 "The position in `sounds` played last (written by og.audioEvents)." {
    readonly true
  }
  field bag u32 "A shuffle's sounds already played this round, one bit each (written by og.audioEvents)." {
    readonly true
  }
  field voices u32 "Its voices playing now (written by og.audioEvents)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 12061</sub>

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

<sub>`api.ogt` line 12166</sub>

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

<sub>`api.ogt` line 12212</sub>

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

<sub>`api.ogt` line 12246</sub>

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

<sub>`api.ogt` line 12254</sub>

<a id="record-SoundEnded"></a>

### `og.SoundEnded` (event)

A sound ended, sent in the tick it happens (systems read it the tick after).

Token `og.SoundEnded`, registered under the schema name `og.SoundEnded`; its value type is `SoundEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose AudioSource ended. |
| `sound` | `Sound` | `0` | — | — | — | The sound. |
| `voice` | `number` | `0` | — | — | — | The voice id in the command stream. |
| `reason` | `number` | `0` | — | — | — | Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3), SOUND_CULLED (4) or SOUND_RETRIGGERED (5). |

```ogt
event SoundEnded {
  doc "A sound ended, sent in the tick it happens (systems read it the tick after)."
  field entity Entity "The entity whose AudioSource ended."
  field sound Sound "The sound."
  field voice u32 "The voice id in the command stream."
  field reason u32 "Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3), SOUND_CULLED (4) or SOUND_RETRIGGERED (5)."
}
```

<sub>`api.ogt` line 12261</sub>

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

<sub>`api.ogt` line 11587</sub>

<a id="type-PanningModel"></a>

### `PanningModel` (enum)

How an AudioListener places spatial sources between the ears, as Web Audio's PannerNode.panningModel.

A string: one of the variants below. The zero value is `"equalpower"`.

| Variant | Doc |
|---|---|
| `"equalpower"` | The stereo panner (Web Audio's equal-power law): for speakers. |
| `"hrtf"` | For headphones: a spherical-head model (Brown and Duda, 1998) that delays the far ear by the interaural time difference (up to 0.66 ms) and shadows its highs (up to -20 dB), so a source moves between the ears; front and back, up and down are not told apart (no pinna). |

```ogt
enum PanningModel {
  doc "How an AudioListener places spatial sources between the ears, as Web Audio's PannerNode.panningModel."
  variant equalpower "The stereo panner (Web Audio's equal-power law): for speakers."
  variant hrtf "For headphones: a spherical-head model (Brown and Duda, 1998) that delays the far ear by the interaural time difference (up to 0.66 ms) and shadows its highs (up to -20 dB), so a source moves between the ears; front and back, up and down are not told apart (no pinna)."
}
```

<sub>`api.ogt` line 11602</sub>

<a id="type-AudioResampler"></a>

### `AudioResampler` (enum)

How the mixer resamples a voice whose pitch or sound rate differs from the output's.

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | Linear interpolation: the engine's mix before round 4, cheaper, with duller highs (-3.4 dB at 15 kHz from 44.1 kHz) and images (-11.5 dB there). |
| `"sinc"` | A Kaiser-windowed sinc (beta 7, 16 taps, 2048 phases; widened up to 64 taps for a voice pitched up to 4x): flat to the highs, images below -70 dB. |

```ogt
enum AudioResampler {
  doc "How the mixer resamples a voice whose pitch or sound rate differs from the output's."
  variant linear "Linear interpolation: the engine's mix before round 4, cheaper, with duller highs (-3.4 dB at 15 kHz from 44.1 kHz) and images (-11.5 dB there)."
  variant sinc "A Kaiser-windowed sinc (beta 7, 16 taps, 2048 phases; widened up to 64 taps for a voice pitched up to 4x): flat to the highs, images below -70 dB."
}
```

<sub>`api.ogt` line 11608</sub>

<a id="type-AudioEventOrder"></a>

### `AudioEventOrder` (enum)

How a AudioEvent chooses its next sound.

A string: one of the variants below. The zero value is `"random"`.

| Variant | Doc |
|---|---|
| `"random"` | A weighted random pick each time (AudioEvent.weights), drawn from the engine's seeded stream. |
| `"roundRobin"` | The sounds in turn, from the first. |
| `"shuffle"` | Random, every sound once before any repeats (and never the last one twice in a row across rounds). |

```ogt
enum AudioEventOrder {
  doc "How a AudioEvent chooses its next sound."
  variant random "A weighted random pick each time (AudioEvent.weights), drawn from the engine's seeded stream."
  variant roundRobin "The sounds in turn, from the first."
  variant shuffle "Random, every sound once before any repeats (and never the last one twice in a row across rounds)."
}
```

<sub>`api.ogt` line 12047</sub>

<a id="type-SoundDesc"></a>

### `SoundDesc` (desc struct)

PCM a game made itself (og.audio.createSound): a tone, a procedural effect, a recording it synthesised.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `Float32Array` | — | — | — | — | The samples, -1 to 1 (clamped), interleaved by channel: frames * channels values; at least one frame. |
| `rate` | `number` | `48000` | 8000 … 192000 | `hz` | — | Samples per second per channel. |
| `channels` | `number` | `1` | 1 … 2 | — | — | Interleaved channels, 1 or 2. |

```ogt
struct SoundDesc {
  doc "PCM a game made itself (og.audio.createSound): a tone, a procedural effect, a recording it synthesised."
  desc true
  field samples Float32Array "The samples, -1 to 1 (clamped), interleaved by channel: frames * channels values; at least one frame."
  field rate u32 "Samples per second per channel." {
    default 48000
    min 8000
    max 192000
    unit hz
  }
  field channels u32 "Interleaved channels, 1 or 2." {
    default 1
    min 1
    max 2
  }
}
```

<sub>`api.ogt` line 12149</sub>

## Column kinds

Each is a factory, `AudioBus(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-AudioBus"></a>`AudioBus` | `AudioBus` | `Uint32Array` | 1 | An AudioBus field: the variant's index. |
| <a id="column-AudioResampler"></a>`AudioResampler` | `AudioResampler` | `Uint32Array` | 1 | An AudioResampler field: the variant's index. |
| <a id="column-PanningModel"></a>`PanningModel` | `PanningModel` | `Uint32Array` | 1 | A PanningModel field: the variant's index. |
| <a id="column-AudioEventOrder"></a>`AudioEventOrder` | `AudioEventOrder` | `Uint32Array` | 1 | A AudioEventOrder field: the variant's index. |

```ogt
column AudioBus {
  doc "An AudioBus field: the variant's index."
  array Uint32Array
  stride 1
  value AudioBus
}
```

<sub>`api.ogt` line 11595</sub>

```ogt
column AudioResampler {
  doc "An AudioResampler field: the variant's index."
  array Uint32Array
  stride 1
  value AudioResampler
}
```

<sub>`api.ogt` line 11614</sub>

```ogt
column PanningModel {
  doc "A PanningModel field: the variant's index."
  array Uint32Array
  stride 1
  value PanningModel
}
```

<sub>`api.ogt` line 11621</sub>

```ogt
column AudioEventOrder {
  doc "A AudioEventOrder field: the variant's index."
  array Uint32Array
  stride 1
  value AudioEventOrder
}
```

<sub>`api.ogt` line 12054</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-SOUND_FINISHED"></a>`SOUND_FINISHED` | `number` | `0` | SoundEnded.reason: a one-shot played to its end. |
| <a id="const-SOUND_STOPPED"></a>`SOUND_STOPPED` | `number` | `1` | SoundEnded.reason: `playing` was set to false. |
| <a id="const-SOUND_REMOVED"></a>`SOUND_REMOVED` | `number` | `2` | SoundEnded.reason: the AudioSource was removed from a living entity. |
| <a id="const-SOUND_REPLACED"></a>`SOUND_REPLACED` | `number` | `3` | SoundEnded.reason: `sound` changed while it played. |
| <a id="const-SOUND_CULLED"></a>`SOUND_CULLED` | `number` | `4` | SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age). |
| <a id="const-SOUND_RETRIGGERED"></a>`SOUND_RETRIGGERED` | `number` | `5` | SoundEnded.reason: AudioSource.retrigger started the sound again from the top; a new voice plays it. |

```ogt
const SOUND_FINISHED u32 0 "SoundEnded.reason: a one-shot played to its end."
```

<sub>`api.ogt` line 11580</sub>

```ogt
const SOUND_STOPPED u32 1 "SoundEnded.reason: `playing` was set to false."
```

<sub>`api.ogt` line 11581</sub>

```ogt
const SOUND_REMOVED u32 2 "SoundEnded.reason: the AudioSource was removed from a living entity."
```

<sub>`api.ogt` line 11582</sub>

```ogt
const SOUND_REPLACED u32 3 "SoundEnded.reason: `sound` changed while it played."
```

<sub>`api.ogt` line 11583</sub>

```ogt
const SOUND_CULLED u32 4 "SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age)."
```

<sub>`api.ogt` line 11584</sub>

```ogt
const SOUND_RETRIGGERED u32 5 "SoundEnded.reason: AudioSource.retrigger started the sound again from the top; a new voice plays it."
```

<sub>`api.ogt` line 11585</sub>
