<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-RecordFn"></a>

### `RecordFn` (callback)

A value's new form: called once per saved value of the type, in storage order.

Game code supplies this function.

```ts
type RecordFn = (value: unknown, entity: Entity | null) => unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `unknown` | no | The old record: its fields by name, as saved. |
| `entity` | `Entity \| null` | no | Its entity, or null for the type's resource. |

```ogt
callback RecordFn {
  doc "A value's new form: called once per saved value of the type, in storage order."
  param value Json "The old record: its fields by name, as saved."
  param entity Option.Entity "Its entity, or null for the type's resource."
  returns Json "The new record: fields by name of the type it is written as; one left out takes its default."
}
```

<sub>Module `world` line 87</sub>
