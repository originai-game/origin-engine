<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-UiDump"></a>

### `UiDump` (struct)

The UI tree, flattened in draw order.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `surface` | `UiSurface \| null` | `null` | — | — | — | The surface the bounds are on; null without a UI module. |
| `elements` | `readonly UiElement[]` | `[]` | — | — | — | The elements. |

```ogt
struct UiDump {
  doc "The UI tree, flattened in draw order."
  field tick u32 "The tick."
  field surface Option.UiSurface "The surface the bounds are on; null without a UI module."
  field elements List.UiElement "The elements."
}
```

<sub>`api.ogt` line 24392</sub>
