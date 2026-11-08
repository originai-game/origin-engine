<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TooltipCardResult"></a>

### `TooltipCardResult` (struct)

What a tooltip card did this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean` | `false` | — | — | — | It is on screen (or leaving). |
| `pinned` | `boolean` | `false` | — | — | — | It is pinned. |
| `link` | `string \| null` | `null` | — | — | — | The link pressed in it (or in a nested card) this tick: its key. |
| `depth` | `number` | `0` | — | — | — | How many nested cards are open beneath it. |

```ogt
struct TooltipCardResult {
  doc "What a tooltip card did this tick."
  field visible bool "It is on screen (or leaving)."
  field pinned bool "It is pinned."
  field link Option.String "The link pressed in it (or in a nested card) this tick: its key."
  field depth u32 "How many nested cards are open beneath it."
}
```

<sub>Module `ui` line 5369</sub>
