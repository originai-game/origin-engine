<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

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

<sub>`api.ogt` line 13456</sub>
