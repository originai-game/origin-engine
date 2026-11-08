<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StateEvent"></a>

### `StateEvent` (struct)

Sets a state group of the target.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `string` | `""` | — | — | — | The group. |
| `state` | `string` | `""` | — | — | — | Its new state. |

```ogt
struct StateEvent {
  doc "Sets a state group of the target."
  field group String "The group."
  field state String "Its new state."
}
```

<sub>Module `ui` line 4922</sub>
