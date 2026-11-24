<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="fn-createSynthSound"></a>

### `og.audio.createSynthSound`

Synthesise a deterministic procedural sound effect and register it directly into the world's sound bank, answering a playable Sound handle.

```ts
og.audio.createSynthSound(world: World, desc?: SfxDesc): Result<Sound>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the sound. |
| `desc` | `SfxDesc` | yes | The preset or custom oscillator layers and variation settings. |

**Returns** `Result<Sound>`: Refused when synthesis fails or the engine has no audio bank.

Binding symbol `og_audio_create_synth_sound` · publish builds: kept

```ogt
fn createSynthSound {
  doc "Synthesise a deterministic procedural sound effect and register it directly into the world's sound bank, answering a playable Sound handle."
  param world World "The world that owns the sound."
  param desc SfxDesc "The preset or custom oscillator layers and variation settings."
  returns Result.Sound "Refused when synthesis fails or the engine has no audio bank."
}
```

<sub>Module `audio` line 914</sub>
