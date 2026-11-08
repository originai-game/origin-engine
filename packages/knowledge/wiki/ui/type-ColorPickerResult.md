<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ColorPickerResult"></a>

### `ColorPickerResult` (struct)

A colour picker's colour after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `PickedColor` | `{}` | — | — | — | The colour, to keep and pass back. |
| `hex` | `string` | `""` | — | — | — | It as `#rrggbb` (`#rrggbbaa` with alpha under 1). |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |
| `committed` | `boolean` | `false` | — | — | — | A drag ended, a swatch was chosen, a hex was entered or a step landed. |

```ogt
struct ColorPickerResult {
  doc "A colour picker's colour after this tick."
  field color PickedColor "The colour, to keep and pass back."
  field hex String "It as `#rrggbb` (`#rrggbbaa` with alpha under 1)."
  field changed bool "It changed this tick."
  field committed bool "A drag ended, a swatch was chosen, a hex was entered or a step landed."
}
```

<sub>Module `ui` line 4077</sub>
