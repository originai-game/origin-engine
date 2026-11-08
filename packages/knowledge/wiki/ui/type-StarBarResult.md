<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StarBarResult"></a>

### `StarBarResult` (struct)

What the bar showed this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `earned` | `number` | `0` | — | — | — | Stars earned at this score. |
| `crossed` | `number` | `0` | — | — | — | Stars earned this tick (a threshold crossed): play the game's own fanfare if it has one. |

```ogt
struct StarBarResult {
  doc "What the bar showed this tick."
  field earned u32 "Stars earned at this score."
  field crossed u32 "Stars earned this tick (a threshold crossed): play the game's own fanfare if it has one."
}
```

<sub>Module `ui` line 4114</sub>
