<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChipDesc"></a>

### `ChipDesc` (desc struct)

A chip's state.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `boolean` | `false` | — | — | — | Selected: inverted to ink. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |

```ogt
struct ChipDesc {
  doc "A chip's state."
  desc true
  field selected bool "Selected: inverted to ink."
  field disabled bool "Neutral and inert."
}
```

<sub>`api.ogt` line 16746</sub>
