<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-PointerMove"></a>

### `og.PointerMove` (event)

A pointer moved.

Token `og.PointerMove`, registered under the schema name `og.PointerMove`; its value type is `PointerMove`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `id` | `number` | `0` | — | — | — | Which pointer: 0 the primary one, 1 to 4 further touches. |
| `movementX` | `number` | `0` | — | — | — | The relative movement this event carried (movementX), CSS pixels; 0 when the host gave none. While the pointer is locked x and y stand still and only this moves. |
| `movementY` | `number` | `0` | — | — | — | The relative vertical movement (movementY, down positive), CSS pixels. |

```ogt
event PointerMove {
  doc "A pointer moved."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches."
  field movementX f32 "The relative movement this event carried (movementX), CSS pixels; 0 when the host gave none. While the pointer is locked x and y stand still and only this moves."
  field movementY f32 "The relative vertical movement (movementY, down positive), CSS pixels."
}
```

<sub>Module `input` line 1256</sub>
