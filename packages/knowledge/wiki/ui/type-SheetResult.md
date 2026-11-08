<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SheetResult"></a>

### `SheetResult` (struct)

What a sheet answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `detent` | `number` | `0` | — | — | — | The detent it rests at after this tick: keep it. |
| `dismissed` | `boolean` | `false` | — | — | — | It asked to close: set open false. |
| `visible` | `boolean` | `false` | — | — | — | It is on screen (open, or leaving). |

```ogt
struct SheetResult {
  doc "What a sheet answered this tick."
  field detent u32 "The detent it rests at after this tick: keep it."
  field dismissed bool "It asked to close: set open false."
  field visible bool "It is on screen (open, or leaving)."
}
```

<sub>Module `ui` line 1825</sub>
