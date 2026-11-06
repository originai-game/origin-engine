<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SegmentItem"></a>

### `SegmentItem` (desc struct)

One segment of a segmented control.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its label; empty for an icon alone (name it with ariaLabel). |
| `icon` | `string` | `""` | — | — | — | An icon before it; empty for none. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but cannot be chosen. |
| `ariaLabel` | `string` | `""` | — | — | — | Its accessible name when it has no label. |

```ogt
struct SegmentItem {
  doc "One segment of a segmented control."
  desc true
  field label String "Its label; empty for an icon alone (name it with ariaLabel)."
  field icon String "An icon before it; empty for none."
  field disabled bool "Shown but cannot be chosen."
  field ariaLabel String "Its accessible name when it has no label."
}
```

<sub>`api.ogt` line 19642</sub>
