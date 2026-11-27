<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-rig"></a>

### `og.load.rig`

Load a checked file.rig humanoid map for og.anim.retarget and og.anim.humanoid. Its native fields name the scheme, explicit bone overrides and optional ankle heights. Loading the same ID again answers the same handle.

```ts
og.load.rig(world: World, path: string): Result<Rig>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation uses it. |
| `path` | `string` | no | The canonical rig content ID, such as base/rigs/strider. |

**Returns** `Result<Rig>`: Refused for a path not in the bundle (run og cook) or a cook stamp mismatch.

Binding symbol `og_load_rig` · publish builds: kept

```ogt
fn rig {
  doc "Load a checked file.rig humanoid map for og.anim.retarget and og.anim.humanoid. Its native fields name the scheme, explicit bone overrides and optional ankle heights. Loading the same ID again answers the same handle."
  param world World "The world whose animation uses it."
  param path String "The canonical rig content ID, such as base/rigs/strider."
  returns Result.Rig "Refused for a path not in the bundle (run og cook) or a cook stamp mismatch."
}
```

<sub>Module `load` line 488</sub>
