<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-curlNoise"></a>

### `og.procgen.curlNoise`

Divergence-free curl vector field from analytical noise gradients: 2D returns `(dN/dy, -dN/dx, 0)`, 3D returns the full 3D curl of three independent potentials.

```ts
og.procgen.curlNoise(desc: FbmDesc, x: number, y: number, z: number): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `FbmDesc` | no | The seed, frequency and fractal settings. |
| `x` | `number` | no | x. |
| `y` | `number` | no | y. |
| `z` | `number` | no | z (3D only). |

**Returns** `Vector3`.

Binding symbol `og_procgen_curl_noise` · publish builds: kept

```ogt
fn curlNoise {
  doc "Divergence-free curl vector field from analytical noise gradients: 2D returns `(dN/dy, -dN/dx, 0)`, 3D returns the full 3D curl of three independent potentials."
  param desc FbmDesc "The seed, frequency and fractal settings."
  param x f32 "x."
  param y f32 "y."
  param z f32 "z (3D only)."
  returns Vector3
}
```

<sub>Module `procgen` line 103</sub>
