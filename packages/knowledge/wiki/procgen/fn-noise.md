<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-noise"></a>

### `og.procgen.noise`

Noise at one point, about -1 to 1: a pure function of the desc and the point.

```ts
og.procgen.noise(desc: FbmDesc, x: number, y: number, z: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `FbmDesc` | no | The seed, kind and fractal settings. |
| `x` | `number` | no | x. |
| `y` | `number` | no | y. |
| `z` | `number` | no | z (3D only). |

**Returns** `number`.

Binding symbol `og_procgen_noise` · publish builds: kept

```ogt
fn noise {
  doc "Noise at one point, about -1 to 1: a pure function of the desc and the point."
  param desc FbmDesc "The seed, kind and fractal settings."
  param x f32 "x."
  param y f32 "y."
  param z f32 "z (3D only)."
  returns f32
}
```

<sub>`api.ogt` line 1838</sub>
