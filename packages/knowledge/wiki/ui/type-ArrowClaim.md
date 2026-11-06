<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ArrowClaim"></a>

### `ArrowClaim` (desc struct)

Which arrows a focused element claims (ElementDesc.arrows); none by default.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `left` | `boolean` | `false` | — | — | — | Left steps it. |
| `right` | `boolean` | `false` | — | — | — | Right steps it. |
| `up` | `boolean` | `false` | — | — | — | Up steps it. |
| `down` | `boolean` | `false` | — | — | — | Down steps it. |

```ogt
struct ArrowClaim {
  doc "Which arrows a focused element claims (ElementDesc.arrows); none by default."
  desc true
  field left bool "Left steps it."
  field right bool "Right steps it."
  field up bool "Up steps it."
  field down bool "Down steps it."
}
```

<sub>`api.ogt` line 15832</sub>
