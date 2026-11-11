<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-rig2d"></a>

### `og.load.rig2d`

Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. Loading the same path again answers the same handles.

```ts
og.load.rig2d(world: World, path: string): Result<Rig2DInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation and renderer own it. |
| `path` | `string` | no | The rig's asset id, such as `rigs/hero.ogt`. |

**Returns** `Result<Rig2DInfo>`: Refused for a path not in the bundle (run og cook) or a texture it cannot load.

Binding symbol `og_load_rig2d` · publish builds: kept

```ogt
fn rig2d {
  doc "Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. Loading the same path again answers the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The rig's asset id, such as `rigs/hero.ogt`."
  returns Result.Rig2DInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>Module `load` line 465</sub>
