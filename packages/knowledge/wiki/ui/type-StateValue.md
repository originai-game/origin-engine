<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StateValue"></a>

### `StateValue` (struct)

The current state of a group.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `string` | `""` | — | — | — | The group. |
| `state` | `string` | `""` | — | — | — | Its state. |

```ogt
struct StateValue {
  doc "The current state of a group."
  field group String "The group."
  field state String "Its state."
}
```

<sub>Module `ui` line 2533</sub>
