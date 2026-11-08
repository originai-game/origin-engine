<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MeterDesc"></a>

### `MeterDesc` (desc struct)

A meter: a track with a fill that animates its changes by the tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `MeterKind` | `"health"` | — | — | — | What it measures, which sets its colour. |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of your own instead of the kind's. |
| `label` | `string` | `""` | — | — | — | Text inside the bar (such as `75 / 100`); empty is none. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `number` | `0` | ≥ 0 | `cssPx` | — | Its height; 0 is space.md, or the caption's line with a label. |
| `segments` | `number` | `0` | — | — | — | Notches dividing it into this many parts; 0 or 1 is none. |

```ogt
struct MeterDesc {
  doc "A meter: a track with a fill that animates its changes by the tick."
  desc true
  field kind MeterKind "What it measures, which sets its colour."
  field color Option.Paint "A colour of your own instead of the kind's."
  field label String "Text inside the bar (such as `75 / 100`); empty is none."
  field width Option.Size "Its width; null grows."
  field height f32 "Its height; 0 is space.md, or the caption's line with a label." {
    unit cssPx
    min 0
  }
  field segments u32 "Notches dividing it into this many parts; 0 or 1 is none."
}
```

<sub>Module `ui` line 1033</sub>
