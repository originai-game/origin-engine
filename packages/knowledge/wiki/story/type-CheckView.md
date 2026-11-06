<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-CheckView"></a>

### `CheckView` (struct)

A choice's check.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stat` | `string` | `""` | — | — | — | The variable read as the stat. |
| `target` | `number` | `0` | — | — | — | What the stat (passive) or a d20 plus the stat (active) must reach. |
| `chance` | `number` | `0` | — | — | — | The chance it passes, 0 to 1: an active check's share of the d20's faces that pass; a passive one's 1 or 0. |
| `kind` | `string` | `""` | — | — | — | passive (a threshold that enables the choice) or active (rolled when chosen). |

```ogt
struct CheckView {
  doc "A choice's check."
  field stat String "The variable read as the stat."
  field target f32 "What the stat (passive) or a d20 plus the stat (active) must reach."
  field chance f32 "The chance it passes, 0 to 1: an active check's share of the d20's faces that pass; a passive one's 1 or 0."
  field kind String "passive (a threshold that enables the choice) or active (rolled when chosen)."
}
```

<sub>`api.ogt` line 14553</sub>
