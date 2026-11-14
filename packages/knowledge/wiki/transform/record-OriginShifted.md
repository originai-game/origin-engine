<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-OriginShifted"></a>

### `og.OriginShifted` (event)

og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. Values a game keeps outside the marked fields (a world position in its own component, a key it set in an og.ai blackboard) are its own: add `delta` to them.

Token `og.OriginShifted`, registered under the schema name `og.OriginShifted`; its value type is `OriginShifted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `delta` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | How far every place moved (a whole multiple of og.WorldOrigin.step along each axis of its plane). |
| `x` | `number` | `0` | — | `m` | — | Where the zero is now in the world, along x (og.WorldOrigin.x). |
| `y` | `number` | `0` | — | `m` | — | Along y. |
| `z` | `number` | `0` | — | `m` | — | Along z. |

```ogt
event OriginShifted {
  doc "og.WorldOrigin shifted the world as this tick started, before any system ran: every marked place moved by `delta`. Values a game keeps outside the marked fields (a world position in its own component, a key it set in an og.ai blackboard) are its own: add `delta` to them."
  field delta Vector3 "How far every place moved (a whole multiple of og.WorldOrigin.step along each axis of its plane)." {
    worldspace none
    unit m
  }
  field x f64 "Where the zero is now in the world, along x (og.WorldOrigin.x)." {
    unit m
  }
  field y f64 "Along y." {
    unit m
  }
  field z f64 "Along z." {
    unit m
  }
}
```

<sub>Module `transform` line 90</sub>
