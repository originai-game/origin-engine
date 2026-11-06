<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `audio` module

Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).

The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.

Spatial gain and pan are computed in the tick from the source's world transform relative to the first AudioListener's (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z). A source, listener or zone on a child entity is placed at its og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a child is heard where its parent carries it, a listener under a camera rig hears from the rig.

Propagation, all in the tick and so in the stream: a spatial source or listener that moves shifts the pitch (Doppler, from their tick velocities: AudioSource.dopplerFactor, AudioListener.dopplerFactor, Audio.speedOfSound), a source with a cone is turned down away from its +Z (coneInnerAngle, coneOuterAngle, coneOuterGain, as three.js's setDirectionalCone), and a listener whose panningModel is hrtf places sources for headphones (interaural delay and head shadow) instead of the stereo panner.

- **TypeScript**: functions are the namespace `og.audio`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module audio {
  doc "Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively)."
  doc "The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends."
  doc "Spatial gain and pan are computed in the tick from the source's world transform relative to the first AudioListener's (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z). A source, listener or zone on a child entity is placed at its og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a child is heard where its parent carries it, a listener under a camera rig hears from the rig."
  doc "Propagation, all in the tick and so in the stream: a spatial source or listener that moves shifts the pitch (Doppler, from their tick velocities: AudioSource.dopplerFactor, AudioListener.dopplerFactor, Audio.speedOfSound), a source with a cone is turned down away from its +Z (coneInnerAngle, coneOuterAngle, coneOuterGain, as three.js's setDirectionalCone), and a listener whose panningModel is hrtf places sources for headphones (interaural delay and head shadow) instead of the stereo panner."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 12762</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`SOUND_FINISHED`](#const-SOUND_FINISHED) | constant |  | SoundEnded.reason: a one-shot played to its end. |
| [`SOUND_STOPPED`](#const-SOUND_STOPPED) | constant |  | SoundEnded.reason: `playing` was set to false. |
| [`SOUND_REMOVED`](#const-SOUND_REMOVED) | constant |  | SoundEnded.reason: the AudioSource was removed from a living entity. |
| [`SOUND_REPLACED`](#const-SOUND_REPLACED) | constant |  | SoundEnded.reason: `sound` changed while it played. |
| [`SOUND_CULLED`](#const-SOUND_CULLED) | constant |  | SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age). |
| [`SOUND_RETRIGGERED`](#const-SOUND_RETRIGGERED) | constant |  | SoundEnded.reason: AudioSource.retrigger started the sound again from the top; a new voice plays it. |
| [`AudioBus`](audio/type-AudioBus.md) | enum |  | The bus a source plays on; every bus passes through master. |
| [`AudioBus()`](#column-AudioBus) | column kind |  | An AudioBus field: the variant's index. |
| [`PanningModel`](audio/type-PanningModel.md) | enum |  | How an AudioListener places spatial sources between the ears, as Web Audio's PannerNode.panningModel. |
| [`AudioResampler`](audio/type-AudioResampler.md) | enum |  | How the mixer resamples a voice whose pitch or sound rate differs from the output's. |
| [`AudioResampler()`](#column-AudioResampler) | column kind |  | An AudioResampler field: the variant's index. |
| [`PanningModel()`](#column-PanningModel) | column kind |  | A PanningModel field: the variant's index. |
| [`og.AudioSource`](audio/record-AudioSource.md) | component |  | A sound an entity plays. |
| [`og.AudioListener`](audio/record-AudioListener.md) | component |  | Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. |
| [`og.Audio`](audio/record-Audio.md) | resource |  | Bus volumes; master scales every bus. |
| [`og.AudioBusFx`](audio/record-AudioBusFx.md) | component |  | A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. |
| [`og.AudioBusNode`](audio/record-AudioBusNode.md) | component |  | A user bus of the bus graph: bus `index` (4 to 15) sums the sources whose AudioSource.userBus (or AudioEvent.userBus) names it, runs its og.AudioBusFx (the one whose userBus is the index), and plays … |
| [`og.AudioSnapshot`](audio/record-AudioSnapshot.md) | component |  | A mixer snapshot (combat, underwater, paused): per-bus volumes and filters the mix blends towards while `active`, and back from when not, over `fade` seconds. |
| [`AudioEventOrder`](audio/type-AudioEventOrder.md) | enum |  | How a AudioEvent chooses its next sound. |
| [`AudioEventOrder()`](#column-AudioEventOrder) | column kind |  | A AudioEventOrder field: the variant's index. |
| [`og.AudioEvent`](audio/record-AudioEvent.md) | component |  | An audio event: one of a set of sounds played on each trigger, with pitch and volume jitter, a cooldown and a voice limit (footsteps, impacts, gunshots). |
| [`SoundDesc`](audio/type-SoundDesc.md) | desc struct |  | PCM a game made itself (og.audio.createSound): a tone, a procedural effect, a recording it synthesised. |
| [`og.ReverbZone`](audio/record-ReverbZone.md) | component |  | A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. |
| [`og.Music`](audio/record-Music.md) | component |  | A musical clock: tempo, metre and where bar 1 starts. |
| [`og.MusicBeat`](audio/record-MusicBeat.md) | event |  | A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after). |
| [`og.SoundStarted`](audio/record-SoundStarted.md) | event |  | A source started, sent in the tick it happens (systems read it the tick after). |
| [`og.SoundEnded`](audio/record-SoundEnded.md) | event |  | A sound ended, sent in the tick it happens (systems read it the tick after). |
| [`og.audio.createSound`](audio/fn-createSound.md) | function | `(world: World, desc?: SoundDesc): Result<Sound>` | Make a sound from PCM the game computed: it plays, loops and pitches as a loaded one (AudioSource.sound, AudioEvent.sounds), always resident, stored as 16-bit PCM, and og.load.soundInfo reports it. |
| [`og.audio.outputLatency`](audio/fn-outputLatency.md) | function | `(world: World): number` | How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's lat… |

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

<sub>`api.ogt` line 12783</sub>

```ogt
column AudioResampler {
  doc "An AudioResampler field: the variant's index."
  array Uint32Array
  stride 1
  value AudioResampler
}
```

<sub>`api.ogt` line 12802</sub>

```ogt
column PanningModel {
  doc "A PanningModel field: the variant's index."
  array Uint32Array
  stride 1
  value PanningModel
}
```

<sub>`api.ogt` line 12809</sub>

```ogt
column AudioEventOrder {
  doc "A AudioEventOrder field: the variant's index."
  array Uint32Array
  stride 1
  value AudioEventOrder
}
```

<sub>`api.ogt` line 13242</sub>

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

<sub>`api.ogt` line 12768</sub>

```ogt
const SOUND_STOPPED u32 1 "SoundEnded.reason: `playing` was set to false."
```

<sub>`api.ogt` line 12769</sub>

```ogt
const SOUND_REMOVED u32 2 "SoundEnded.reason: the AudioSource was removed from a living entity."
```

<sub>`api.ogt` line 12770</sub>

```ogt
const SOUND_REPLACED u32 3 "SoundEnded.reason: `sound` changed while it played."
```

<sub>`api.ogt` line 12771</sub>

```ogt
const SOUND_CULLED u32 4 "SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age)."
```

<sub>`api.ogt` line 12772</sub>

```ogt
const SOUND_RETRIGGERED u32 5 "SoundEnded.reason: AudioSource.retrigger started the sound again from the top; a new voice plays it."
```

<sub>`api.ogt` line 12773</sub>
