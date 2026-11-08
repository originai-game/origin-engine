<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PickedColor"></a>

### `PickedColor` (desc struct)

A colour as a picker answers it: sRGB components 0 to 1 and alpha.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `r` | `number` | `0` | — | — | — | Red. |
| `g` | `number` | `0` | — | — | — | Green. |
| `b` | `number` | `0` | — | — | — | Blue. |
| `a` | `number` | `1` | — | — | — | Alpha. |

```ogt
struct PickedColor {
  doc "A colour as a picker answers it: sRGB components 0 to 1 and alpha."
  desc true
  field r f32 "Red."
  field g f32 "Green."
  field b f32 "Blue."
  field a f32 "Alpha." {
    default 1
  }
}
```

<sub>Module `ui` line 3950</sub>
