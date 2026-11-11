<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SaveType"></a>

### `SaveType` (struct)

A type a save holds, as it was saved.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its reflected name (`Wallet`, `og.Transform`). |
| `version` | `number` | `0` | — | — | — | Its schema version when it was saved. |
| `fields` | `readonly string[]` | `[]` | — | — | — | Its fields, in saved order. |

```ogt
struct SaveType {
  doc "A type a save holds, as it was saved."
  field name String "Its reflected name (`Wallet`, `og.Transform`)."
  field version u32 "Its schema version when it was saved."
  field fields List.String "Its fields, in saved order."
}
```

<sub>Module `world` line 54</sub>
