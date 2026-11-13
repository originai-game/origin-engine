<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-Wheel"></a>

### `og.Wheel` (event)

A wheel or trackpad scroll.

Token `og.Wheel`, registered under the schema name `og.Wheel`; its value type is `Wheel`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `deltaX` | `number` | `0` | — | — | — | Horizontal scroll in CSS pixels. |
| `deltaY` | `number` | `0` | — | — | — | Vertical scroll in CSS pixels. |

```ogt
event Wheel {
  doc "A wheel or trackpad scroll."
  field deltaX f32 "Horizontal scroll in CSS pixels."
  field deltaY f32 "Vertical scroll in CSS pixels."
}
```

<sub>Module `input` line 1275</sub>
