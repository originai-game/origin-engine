<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

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

<sub>`api.ogt` line 13311</sub>
