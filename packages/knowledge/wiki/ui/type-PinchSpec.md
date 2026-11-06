<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PinchSpec"></a>

### `PinchSpec` (desc struct)

Two touches on the element scale (and turn) it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `minScale` | `number` | `0` | — | — | — | 0: 0.5. |
| `maxScale` | `number` | `0` | — | — | — | 0: 4. |
| `rotate` | `boolean` | `false` | — | — | — | The touches turn it too. |

```ogt
struct PinchSpec {
  doc "Two touches on the element scale (and turn) it."
  desc true
  field minScale f32 "0: 0.5."
  field maxScale f32 "0: 4."
  field rotate bool "The touches turn it too."
}
```

<sub>`api.ogt` line 20522</sub>
