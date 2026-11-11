<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="fn-noise"></a>

### `og.ai.noise`

Make a noise agents may hear next tick (a gunshot, a door, a thrown stone).

```ts
og.ai.noise(world: World, desc?: NoiseDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NoiseDesc` | yes | Where and how loud. |

**Returns** `Result<void>`: Refused for a world without og.ai.

Binding symbol `og_ai_noise` · publish builds: kept

```ogt
fn noise {
  doc "Make a noise agents may hear next tick (a gunshot, a door, a thrown stone)."
  param world World "The world."
  param desc NoiseDesc "Where and how loud."
  returns Result.Void "Refused for a world without og.ai."
}
```

<sub>Module `ai` line 243</sub>
