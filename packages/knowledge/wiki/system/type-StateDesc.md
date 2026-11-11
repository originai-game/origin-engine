<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-StateDesc"></a>

### `StateDesc` (struct)

A state machine (GameDesc.states): a name and the values it moves between; it holds one at a time, and world.setState moves it as the next tick begins. Its value is world state: saves, the hash and replays carry it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its unique name. |
| `values` | `readonly string[]` | `[]` | — | — | — | Its values, at least one, each unique. |
| `initial` | `string` | `""` | — | — | optional | The value it starts in; empty or omitted: the first. |

```ogt
struct StateDesc {
  doc "A state machine (GameDesc.states): a name and the values it moves between; it holds one at a time, and world.setState moves it as the next tick begins. Its value is world state: saves, the hash and replays carry it."
  field name String "Its unique name."
  field values List.String "Its values, at least one, each unique."
  field initial String "The value it starts in; empty or omitted: the first." {
    optional true
  }
}
```

<sub>Module `system` line 13</sub>
