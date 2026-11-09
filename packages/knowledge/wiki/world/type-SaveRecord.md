<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SaveRecord"></a>

### `SaveRecord` (struct)

One saved value of a type: a component of an entity, or (entity null) the type's resource.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity \| null` | `null` | — | — | — | The saved entity, or null for the resource. |
| `value` | `unknown` | `null` | — | — | — | The value's fields by name, as saved. |

```ogt
struct SaveRecord {
  doc "One saved value of a type: a component of an entity, or (entity null) the type's resource."
  field entity Option.Entity "The saved entity, or null for the resource."
  field value Json "The value's fields by name, as saved."
}
```

<sub>Module `world` line 54</sub>
