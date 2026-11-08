<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScrollAreaDesc"></a>

### `ScrollAreaDesc` (desc struct)

A scroll area: og.ui's scroll container whose content fades out at an edge where more lies beyond.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `axes` | `ScrollAxes` | `"vertical"` | — | — | — | Which ways it scrolls. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (it scrolls inside); null fits the content. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space around the content. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Space between children; null is space.sm. |
| `fade` | `number` | `0` | ≥ 0 | `cssPx` | — | How far the edge fades reach; 0 is 24. |
| `fadeInto` | `ThemeColor \| null` | `null` | — | — | — | The surface the content fades into; null is panel. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the region. |

```ogt
struct ScrollAreaDesc {
  doc "A scroll area: og.ui's scroll container whose content fades out at an edge where more lies beyond."
  desc true
  field axes ScrollAxes "Which ways it scrolls."
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height (it scrolls inside); null fits the content."
  field padding Insets "Space around the content."
  field gap Option.f32 "Space between children; null is space.sm." {
    unit cssPx
  }
  field fade f32 "How far the edge fades reach; 0 is 24." {
    unit cssPx
    min 0
  }
  field fadeInto Option.ThemeColor "The surface the content fades into; null is panel."
  field label String "What a screen reader calls the region."
}
```

<sub>Module `ui` line 1990</sub>
