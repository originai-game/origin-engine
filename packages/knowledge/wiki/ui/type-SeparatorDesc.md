<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SeparatorDesc"></a>

### `SeparatorDesc` (desc struct)

A separator: a hairline, upright or across, with an optional word in its middle.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vertical` | `boolean` | `false` | — | — | — | Upright (between items of a row). |
| `label` | `string` | `""` | — | — | — | A word in its middle (`OR`, a date); empty for a plain line. |
| `inset` | `number` | `0` | ≥ 0 | `cssPx` | — | Space kept at each end. |
| `strong` | `boolean` | `false` | — | — | — | The style's strong line (between sections) rather than its hairline. |

```ogt
struct SeparatorDesc {
  doc "A separator: a hairline, upright or across, with an optional word in its middle."
  desc true
  field vertical bool "Upright (between items of a row)."
  field label String "A word in its middle (`OR`, a date); empty for a plain line."
  field inset f32 "Space kept at each end." {
    unit cssPx
    min 0
  }
  field strong bool "The style's strong line (between sections) rather than its hairline."
}
```

<sub>`api.ogt` line 19117</sub>
