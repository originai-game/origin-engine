<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-noiseGrad"></a>

### `og.procgen.noiseGrad`

Analytical value and gradient of Perlin FBM at one point (exact chain-rule derivatives, no finite-difference jitter).

```ts
og.procgen.noiseGrad(desc: FbmDesc, x: number, y: number, z: number): NoiseGrad
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `FbmDesc` | no | The seed and fractal settings. |
| `x` | `number` | no | x. |
| `y` | `number` | no | y. |
| `z` | `number` | no | z (3D only). |

**Returns** `NoiseGrad`.

Binding symbol `og_procgen_noise_grad` · publish builds: kept

```ogt
fn noiseGrad {
  doc "Analytical value and gradient of Perlin FBM at one point (exact chain-rule derivatives, no finite-difference jitter)."
  param desc FbmDesc "The seed and fractal settings."
  param x f32 "x."
  param y f32 "y."
  param z f32 "z (3D only)."
  returns NoiseGrad
}
```

<sub>Module `procgen` line 94</sub>
