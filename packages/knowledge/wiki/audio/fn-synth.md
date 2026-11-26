<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="fn-synth"></a>

### `og.audio.synth`

Synthesise a deterministic procedural sound effect into a Float32Array (48 kHz mono PCM), without mutating world state.

Also known as: procedural sound, sfx synthesizer, generate audio, chiptune, sound effect

```ts
og.audio.synth(desc?: SfxDesc): Result<SfxSamples>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SfxDesc` | yes | The preset or custom oscillator layers and variation settings. |

**Returns** `Result<SfxSamples>`: Refused when neither a known preset nor any layer is provided, or when a parameter is not finite.

Binding symbol `og_audio_synth` · publish builds: kept

```ogt
fn synth {
  doc "Synthesise a deterministic procedural sound effect into a Float32Array (48 kHz mono PCM), without mutating world state."
  aliases "procedural sound, sfx synthesizer, generate audio, chiptune, sound effect"
  param desc SfxDesc "The preset or custom oscillator layers and variation settings."
  returns Result.SfxSamples "Refused when neither a known preset nor any layer is provided, or when a parameter is not finite."
}
```

<sub>Module `audio` line 921</sub>
