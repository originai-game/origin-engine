<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SplitFn"></a>

### `SplitFn` (callback)

Where one saved value goes when its type splits.

Game code supplies this function.

```ts
type SplitFn = (value: unknown, entity: Entity | null) => unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `unknown` | no | The old record. |
| `entity` | `Entity \| null` | no | Its entity, or null for the resource. |

```ogt
callback SplitFn {
  doc "Where one saved value goes when its type splits."
  param value Json "The old record."
  param entity Option.Entity "Its entity, or null for the resource."
  returns Json "An object of the new types' names to their records: `{ Health: { hp: v.hp }, Mana: { mp: v.mp } }`; a type left out or null, and the entity does not get it."
}
```

<sub>Module `world` line 85</sub>
