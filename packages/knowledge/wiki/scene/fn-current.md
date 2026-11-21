<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`scene`](../scene.md) · [index](../index.md)

<a id="fn-current"></a>

### `og.scene.current`

The asset id of the scene in place: the last og.scene.change's, else GameDesc.scene's; null when the world started without a scene.

```ts
og.scene.current(world: World): string | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string | null`.

Binding symbol `og_scene_current` · publish builds: kept

```ogt
fn current {
  doc "The asset id of the scene in place: the last og.scene.change's, else GameDesc.scene's; null when the world started without a scene."
  param world World "The world."
  returns Option.String
}
```

<sub>Module `scene` line 84</sub>
