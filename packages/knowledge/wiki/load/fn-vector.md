<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-vector"></a>

### `og.load.vector`

A cooked vector from a TS file.vector declaration: source is a module-relative SVG path, or svg supplies inline XML, with optional positive tolerance in view-box units. Paths, basic shapes, groups, transforms, colours and linear/radial gradients become antialiased triangles; unsupported SVG elements are refused by name. Loaded once per path.

```ts
og.load.vector(world: World, path: string): Result<VectorArt>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The canonical asset id, such as `base/ui/emblem`. |

**Returns** `Result<VectorArt>`: Refused for a path not in the bundle or not a vector.

Binding symbol `og_load_vector` · publish builds: kept

```ogt
fn vector {
  doc "A cooked vector from a TS file.vector declaration: source is a module-relative SVG path, or svg supplies inline XML, with optional positive tolerance in view-box units. Paths, basic shapes, groups, transforms, colours and linear/radial gradients become antialiased triangles; unsupported SVG elements are refused by name. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The canonical asset id, such as `base/ui/emblem`."
  returns Result.VectorArt "Refused for a path not in the bundle or not a vector."
}
```

<sub>Module `load` line 405</sub>
