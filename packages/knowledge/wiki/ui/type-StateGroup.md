<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StateGroup"></a>

### `StateGroup` (struct)

A visual state group (docs/ui.md §3.1): one of its states is current, and moving between them animates every value that changes. A group named interaction is derived from input: normal, hover, pressed, focused and disabled (a state the group lacks falls back: pressed to hover to normal, focused to hover). Other groups take their state from ElementProps.state, a timeline's state event or og.ui.forceState; the first state is the default.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `states` | `readonly VisualState[]` | `[]` | — | — | — | Its states. |
| `transitions` | `readonly StateTransition[]` | `[]` | — | — | — | Its transitions; a pair without one moves on the element's motion, else the style's ui spring. |

```ogt
struct StateGroup {
  doc "A visual state group (docs/ui.md §3.1): one of its states is current, and moving between them animates every value that changes. A group named interaction is derived from input: normal, hover, pressed, focused and disabled (a state the group lacks falls back: pressed to hover to normal, focused to hover). Other groups take their state from ElementProps.state, a timeline's state event or og.ui.forceState; the first state is the default."
  field name String "Its name."
  field states List.VisualState "Its states."
  field transitions List.StateTransition "Its transitions; a pair without one moves on the element's motion, else the style's ui spring."
}
```

<sub>Module `ui` line 1745</sub>
