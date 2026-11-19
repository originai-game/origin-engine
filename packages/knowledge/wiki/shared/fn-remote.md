<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-remote"></a>

### `og.shared.remote`

Who an og.RemoteAvatar is: its player id, name and kind. Null for an entity that is not one.

```ts
og.shared.remote(world: World, entity: Entity): SharedRemote | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The og.RemoteAvatar entity. |

**Returns** `SharedRemote | null`.

Binding symbol `og_shared_remote` · publish builds: kept

```ogt
fn remote {
  doc "Who an og.RemoteAvatar is: its player id, name and kind. Null for an entity that is not one."
  param world World "The world."
  param entity Entity "The og.RemoteAvatar entity."
  returns Option.SharedRemote
}
```

<sub>Module `shared` line 284</sub>
