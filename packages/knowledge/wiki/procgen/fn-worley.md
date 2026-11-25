<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-worley"></a>

### `og.procgen.worley`

Cellular Worley noise distances at a point: `x` is F1 (nearest feature distance) and `y` is F2 (second-nearest feature distance).

```ts
og.procgen.worley(seed: number, x: number, y: number, z: number, threeD: boolean): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `seed` | `number` | no | Seed. |
| `x` | `number` | no | x. |
| `y` | `number` | no | y. |
| `z` | `number` | no | z (3D only). |
| `threeD` | `boolean` | no | Sample 3D cellular lattice when true, 2D when false. |

**Returns** `Vector2`.

Binding symbol `og_procgen_worley` · publish builds: kept

```ogt
fn worley {
  doc "Cellular Worley noise distances at a point: `x` is F1 (nearest feature distance) and `y` is F2 (second-nearest feature distance)."
  param seed u32 "Seed."
  param x f32 "x."
  param y f32 "y."
  param z f32 "z (3D only)."
  param threeD bool "Sample 3D cellular lattice when true, 2D when false."
  returns Vector2
}
```

<sub>Module `procgen` line 113</sub>
