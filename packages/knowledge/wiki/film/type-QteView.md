<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-QteView"></a>

### `QteView` (struct)

A choice taken by a quick-time event (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>`): the og.input action's presses or hold while the window is open.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `choice` | `number` | `0` | — | — | — | The ChoiceView.index it takes. |
| `kind` | `string` | `""` | — | — | — | press, hold or mash. |
| `action` | `string` | `""` | — | — | — | The og.input action it reads. |
| `progress` | `number` | `0` | — | `ratio` | — | 0 to 1: the hold's time or the presses so far. |
| `done` | `boolean` | `false` | — | — | — | Reached: the choice is pending. |

```ogt
struct QteView {
  doc "A choice taken by a quick-time event (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>`): the og.input action's presses or hold while the window is open."
  field choice u32 "The ChoiceView.index it takes."
  field kind String "press, hold or mash."
  field action String "The og.input action it reads."
  field progress f32 "0 to 1: the hold's time or the presses so far." {
    unit ratio
  }
  field done bool "Reached: the choice is pending."
}
```

<sub>Module `film` line 107</sub>
