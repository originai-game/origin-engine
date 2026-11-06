<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Box3"></a>

### `Box3` (struct)

An axis-aligned box, as three.js Box3. A box whose min exceeds its max on an axis is empty (box3.empty() makes one: min +Infinity, max -Infinity).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The lower corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The upper corner. |

```ogt
struct Box3 {
  doc "An axis-aligned box, as three.js Box3. A box whose min exceeds its max on an axis is empty (box3.empty() makes one: min +Infinity, max -Infinity)."
  field min Vector3 "The lower corner."
  field max Vector3 "The upper corner."
}
```

<sub>`api.ogt` line 89</sub>
