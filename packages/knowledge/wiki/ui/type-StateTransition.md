<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StateTransition"></a>

### `StateTransition` (struct)

The transition between two states of a group (`*` matches any state). The exact pair wins, then `* -> to`, `from -> *`, `* -> *`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `string` | `""` | — | — | — | The state left. |
| `to` | `string` | `""` | — | — | — | The state entered. |
| `transition` | `Transition` | — | — | — | — | How the values move. |

```ogt
struct StateTransition {
  doc "The transition between two states of a group (`*` matches any state). The exact pair wins, then `* -> to`, `from -> *`, `* -> *`."
  field from String "The state left."
  field to String "The state entered."
  field transition Transition "How the values move."
}
```

<sub>Module `ui` line 2165</sub>
