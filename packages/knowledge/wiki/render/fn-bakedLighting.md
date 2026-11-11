<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-bakedLighting"></a>

### `og.render.bakedLighting`

The baked lighting of every scene instantiated in the world, in instantiation order: whether it is drawn, and why not. A scene with no static model and no LightVolume is not listed.

```ts
og.render.bakedLighting(world: World): readonly BakedScene[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `readonly BakedScene[]`.

Binding symbol `og_render_baked_lighting` · publish builds: kept

```ogt
fn bakedLighting {
  doc "The baked lighting of every scene instantiated in the world, in instantiation order: whether it is drawn, and why not. A scene with no static model and no LightVolume is not listed."
  param world World "The world asked."
  returns List.BakedScene
}
```

<sub>Module `render` line 3956</sub>
