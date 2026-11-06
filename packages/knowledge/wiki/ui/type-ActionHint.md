<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ActionHint"></a>

### `ActionHint` (desc struct)

A hint of the action bar: an action, its glyph on the device in use and what it does.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action's name (answered when the hint is pressed). |
| `hint` | `string` | `""` | — | — | — | What it does (`Buy`, `Back`). |
| `icon` | `string` | `""` | — | — | — | The glyph's icon, for an action with no glyph on the device in use (a UI action's own glyph, og.ui.frameGlyph, follows the device live and wins). |
| `text` | `string` | `""` | — | — | — | The glyph as text (`Enter`, `A`, `LB`), with the same fallback as icon. |
| `hold` | `boolean` | `false` | — | — | — | It fires on a hold: a ring fills. |
| `progress` | `number` | `0` | 0 … 1 | `ratio` | — | How far the hold has gone. |

```ogt
struct ActionHint {
  doc "A hint of the action bar: an action, its glyph on the device in use and what it does."
  desc true
  field action String "The action's name (answered when the hint is pressed)."
  field hint String "What it does (`Buy`, `Back`)."
  field icon String "The glyph's icon, for an action with no glyph on the device in use (a UI action's own glyph, og.ui.frameGlyph, follows the device live and wins)."
  field text String "The glyph as text (`Enter`, `A`, `LB`), with the same fallback as icon."
  field hold bool "It fires on a hold: a ring fills."
  field progress f32 "How far the hold has gone." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>`api.ogt` line 18117</sub>
