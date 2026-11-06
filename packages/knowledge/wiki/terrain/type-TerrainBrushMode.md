<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainBrushMode"></a>

### `TerrainBrushMode` (enum)

What og.terrain.edit does under its brush.

A string: one of the variants below. The zero value is `"raise"`.

| Variant | Doc |
|---|---|
| `"raise"` | Raise the heights by `strength` metres at the centre. |
| `"lower"` | Lower them by `strength` metres at the centre. |
| `"flatten"` | Pull them towards the brush centre's y, by `strength` (0 to 1) at the centre. |
| `"smooth"` | Pull each towards its neighbours' mean, by `strength` (0 to 1) at the centre. |
| `"paint"` | Paint `layer` into the control map, by `strength` (0 to 1) at the centre. |
| `"hole"` | Cut holes: every cell whose centre the brush weighs at least half (its falloff times `strength`) is cut out of the surface, neither drawn nor collided with. |
| `"fill"` | Fill holes in again where `hole` would cut them. |

```ogt
enum TerrainBrushMode {
  doc "What og.terrain.edit does under its brush."
  variant raise "Raise the heights by `strength` metres at the centre."
  variant lower "Lower them by `strength` metres at the centre."
  variant flatten "Pull them towards the brush centre's y, by `strength` (0 to 1) at the centre."
  variant smooth "Pull each towards its neighbours' mean, by `strength` (0 to 1) at the centre."
  variant paint "Paint `layer` into the control map, by `strength` (0 to 1) at the centre."
  variant hole "Cut holes: every cell whose centre the brush weighs at least half (its falloff times `strength`) is cut out of the surface, neither drawn nor collided with."
  variant fill "Fill holes in again where `hole` would cut them."
}
```

<sub>`api.ogt` line 24806</sub>
