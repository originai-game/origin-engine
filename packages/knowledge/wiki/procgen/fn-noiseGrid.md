<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-noiseGrid"></a>

### `og.procgen.noiseGrid`

Noise over a grid in one call, row-major: `values[j * width + i]` is the sample at (x + i * step, y + j * step).

```ts
og.procgen.noiseGrid(desc?: NoiseGridDesc): Result<readonly number[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `NoiseGridDesc` | yes | The grid and its noise. |

**Returns** `Result<readonly number[]>`: Refused for more than 16,777,216 samples.

Binding symbol `og_procgen_noise_grid` · publish builds: kept

```ogt
fn noiseGrid {
  doc "Noise over a grid in one call, row-major: `values[j * width + i]` is the sample at (x + i * step, y + j * step)."
  param desc NoiseGridDesc "The grid and its noise."
  returns Result.List.f32 "Refused for more than 16,777,216 samples."
}
```

<sub>Module `procgen` line 69</sub>
