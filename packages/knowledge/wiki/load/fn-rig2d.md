<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-rig2d"></a>

### `og.load.rig2d`

Load a cooked file.rig2d TS declaration by canonical content ID and its texture, and register it with og.skeleton2d. Authored fields, DragonBones exchange inputs and layered-art rigs cook directly to native data. Loading the same ID again answers the same handles.

```ts
og.load.rig2d(world: World, path: string): Result<Rig2DInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation and renderer own it. |
| `path` | `string` | no | The rig's canonical content ID, such as `base/rigs/hero`. |

**Returns** `Result<Rig2DInfo>`: Refused for a path not in the bundle (run og cook) or a texture it cannot load.

Binding symbol `og_load_rig2d` · publish builds: kept

```ogt
fn rig2d {
  doc "Load a cooked file.rig2d TS declaration by canonical content ID and its texture, and register it with og.skeleton2d. Authored fields, DragonBones exchange inputs and layered-art rigs cook directly to native data. Loading the same ID again answers the same handles."
  param world World "The world whose animation and renderer own it."
  param path String "The rig's canonical content ID, such as `base/rigs/hero`."
  returns Result.Rig2DInfo "Refused for a path not in the bundle (run og cook) or a texture it cannot load."
}
```

<sub>Module `load` line 538</sub>
