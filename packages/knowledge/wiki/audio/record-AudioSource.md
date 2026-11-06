<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-AudioSource"></a>

### `og.AudioSource` (component)

A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded.

A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event.

Also known as: sound emitter, AudioSource, AudioStreamPlayer, play a sound

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
  aliases "sound emitter, AudioSource, AudioStreamPlayer, play a sound"
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

<sub>Module `audio` line 55</sub>
