<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-OverlayBarResult"></a>

### `OverlayBarResult` (struct)

What the player asked for this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `string \| null` | `null` | — | — | — | An overlay was pressed: show it (the empty string: the shown one was pressed again, turn it off). |

```ogt
struct OverlayBarResult {
  doc "What the player asked for this tick."
  field selected Option.String "An overlay was pressed: show it (the empty string: the shown one was pressed again, turn it off)."
}
```

<sub>Module `ui` line 4166</sub>
