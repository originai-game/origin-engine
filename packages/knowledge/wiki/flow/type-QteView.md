<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-QteView"></a>

### `QteView` (struct)

A choice taken by a quick-time event (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>`, `#qte:timing:<action>:<start>-<end>`, `#qte:sequence:<a1>+<a2>+...`, `#qte:swipe:<dir>`, `#qte:drag:<dir>`).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `choice` | `number` | `0` | — | — | — | The ChoiceView.index it takes. |
| `kind` | `string` | `""` | — | — | — | press, hold, mash, timing, sequence, swipe or drag. |
| `action` | `string` | `""` | — | — | — | The og.input action it reads (or the current step's action for `sequence`; empty for `swipe`/`drag`). |
| `direction` | `string` | `""` | — | — | — | Direction for `swipe` or `drag` (`up`, `down`, `left`, `right`); empty otherwise. |
| `progress` | `number` | `0` | — | `ratio` | — | 0 to 1: the hold's time, presses, timing cursor, sequence steps or drag progress. |
| `zoneStart` | `number` | `0` | — | `ratio` | — | For `timing`: start of the target zone (0 to 1). |
| `zoneEnd` | `number` | `0` | — | `ratio` | — | For `timing`: end of the target zone (0 to 1). |
| `grade` | `string` | `""` | — | — | — | Resolved result grade: `perfect`, `good`, `miss`, or empty while active. |
| `done` | `boolean` | `false` | — | — | — | Reached: the choice is pending (or resolved with a grade). |

```ogt
struct QteView {
  doc "A choice taken by a quick-time event (`#qte:press:<action>`, `#qte:hold:<action>:<ms>`, `#qte:mash:<action>:<presses>`, `#qte:timing:<action>:<start>-<end>`, `#qte:sequence:<a1>+<a2>+...`, `#qte:swipe:<dir>`, `#qte:drag:<dir>`)."
  field choice u32 "The ChoiceView.index it takes."
  field kind String "press, hold, mash, timing, sequence, swipe or drag."
  field action String "The og.input action it reads (or the current step's action for `sequence`; empty for `swipe`/`drag`)."
  field direction String "Direction for `swipe` or `drag` (`up`, `down`, `left`, `right`); empty otherwise."
  field progress f32 "0 to 1: the hold's time, presses, timing cursor, sequence steps or drag progress." {
    unit ratio
  }
  field zoneStart f32 "For `timing`: start of the target zone (0 to 1)." {
    unit ratio
  }
  field zoneEnd f32 "For `timing`: end of the target zone (0 to 1)." {
    unit ratio
  }
  field grade String "Resolved result grade: `perfect`, `good`, `miss`, or empty while active."
  field done bool "Reached: the choice is pending (or resolved with a grade)."
}
```

<sub>Module `flow` line 126</sub>
