<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SegmentedDesc"></a>

### `SegmentedDesc` (desc struct)

A segmented control: its segments and layout.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `items` | `readonly SegmentItem[]` | `[]` | — | — | — | The segments. |
| `selected` | `number` | `0` | — | — | — | The chosen one from game state. |
| `size` | `ControlSize` | `"medium"` | — | — | — | Its height. |
| `equal` | `boolean` | `true` | — | — | — | Every segment as wide as the widest. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `label` | `string` | `""` | — | — | — | Its accessible name. |
| `disabled` | `boolean` | `false` | — | — | — | Every segment inert. |

```ogt
struct SegmentedDesc {
  doc "A segmented control: its segments and layout."
  desc true
  field items List.SegmentItem "The segments."
  field selected i32 "The chosen one from game state."
  field size ControlSize "Its height."
  field equal bool "Every segment as wide as the widest." {
    default true
  }
  field width Option.Size "Its width; null fits."
  field label String "Its accessible name."
  field disabled bool "Every segment inert."
}
```

<sub>Module `ui` line 3184</sub>
