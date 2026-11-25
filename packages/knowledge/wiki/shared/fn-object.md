<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-object"></a>

### `og.shared.object`

Which shared object an og.SharedObject entity is: its key and version. Null for an entity that is not one.

```ts
og.shared.object(world: World, entity: Entity): SharedObjectInfo | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The og.SharedObject entity. |

**Returns** `SharedObjectInfo | null`.

Binding symbol `og_shared_object` · publish builds: kept

```ogt
fn object {
  doc "Which shared object an og.SharedObject entity is: its key and version. Null for an entity that is not one."
  param world World "The world."
  param entity Entity "The og.SharedObject entity."
  returns Option.SharedObjectInfo
}
```

<sub>Module `shared` line 278</sub>
