<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GridSpec"></a>

### `GridSpec` (desc struct)

A grid's tracks and areas.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `columns` | `readonly Track[]` | `[]` | — | — | — | The column tracks. |
| `rows` | `readonly Track[]` | `[]` | — | — | — | The row tracks; items past them add autoTrack rows. |
| `autoTrack` | `Track \| null` | `null` | — | — | — | The track added for items past the declared ones; null is auto. |
| `columnGap` | `number \| null` | `null` | — | `cssPx` | — | Space between columns; absent is the element's gap. |
| `rowGap` | `number \| null` | `null` | — | `cssPx` | — | Space between rows; absent is the element's gap. |
| `areas` | `readonly string[]` | `[]` | — | — | — | Named areas, one string per row of space-separated cell names, as CSS grid-template-areas (`.` is an unnamed cell). |
| `flow` | `GridFlow` | `"row"` | — | — | — | Which way auto-placed items fill it. |
| `dense` | `boolean` | `false` | — | — | — | Auto-placement back-fills holes left by spanning items. |

```ogt
struct GridSpec {
  doc "A grid's tracks and areas."
  desc true
  field columns List.Track "The column tracks."
  field rows List.Track "The row tracks; items past them add autoTrack rows."
  field autoTrack Option.Track "The track added for items past the declared ones; null is auto."
  field columnGap Option.f32 "Space between columns; absent is the element's gap." {
    unit cssPx
  }
  field rowGap Option.f32 "Space between rows; absent is the element's gap." {
    unit cssPx
  }
  field areas List.String "Named areas, one string per row of space-separated cell names, as CSS grid-template-areas (`.` is an unnamed cell)."
  field flow GridFlow "Which way auto-placed items fill it."
  field dense bool "Auto-placement back-fills holes left by spanning items."
}
```

<sub>Module `ui` line 4496</sub>
