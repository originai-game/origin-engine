<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-vector"></a>

### `og.load.vector`

A cooked vector for og.ui's VectorFill, from an `.ogt` of `type vector`: `source` an SVG (or `svg` inline) of paths, basic shapes, groups and transforms, filled and stroked with colours or linear and radial gradients, and `tolerance` (curves flattened to within it, view-box units). The cook cuts it into triangles whose edges the UI pass antialiases at any size; anything outside the subset is refused by name. Loaded once per path.

```ts
og.load.vector(world: World, path: string): Result<VectorArt>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `ui/emblem.ogt`. |

**Returns** `Result<VectorArt>`: Refused for a path not in the bundle or not a vector.

Binding symbol `og_load_vector` · publish builds: kept

```ogt
fn vector {
  doc "A cooked vector for og.ui's VectorFill, from an `.ogt` of `type vector`: `source` an SVG (or `svg` inline) of paths, basic shapes, groups and transforms, filled and stroked with colours or linear and radial gradients, and `tolerance` (curves flattened to within it, view-box units). The cook cuts it into triangles whose edges the UI pass antialiases at any size; anything outside the subset is refused by name. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `ui/emblem.ogt`."
  returns Result.VectorArt "Refused for a path not in the bundle or not a vector."
}
```

<sub>Module `load` line 286</sub>
