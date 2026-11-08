<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-VisualState"></a>

### `VisualState` (struct)

One state of a group: the values it sets, and timelines it starts.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `values` | `readonly PropValue[]` | `[]` | — | — | — | The values it sets. |
| `onEnter` | `string` | `""` | — | — | — | A timeline (og.ui.defineTimeline) played on the element when the state is entered (empty: none). |
| `onExit` | `string` | `""` | — | — | — | One played when it is left. |

```ogt
struct VisualState {
  doc "One state of a group: the values it sets, and timelines it starts."
  field name String "Its name."
  field values List.PropValue "The values it sets."
  field onEnter String "A timeline (og.ui.defineTimeline) played on the element when the state is entered (empty: none)."
  field onExit String "One played when it is left."
}
```

<sub>Module `ui` line 2511</sub>
