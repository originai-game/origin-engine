<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FixedSize"></a>

### `FixedSize` (struct)

A fixed size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `px` | `number` | `0` | — | `cssPx` | — | The size. |

```ogt
struct FixedSize {
  doc "A fixed size."
  field px f32 "The size." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 155</sub>
