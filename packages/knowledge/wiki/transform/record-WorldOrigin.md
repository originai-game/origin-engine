<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-WorldOrigin"></a>

### `og.WorldOrigin` (resource)

Large worlds (R12): naming it keeps every place in the world near its zero. As each tick starts, before any system runs, when the focus is farther from the zero than `step` along one of the plane's axes, the whole world shifts by the multiple of `step` nearest the focus, so the focus is within half a step of the zero again: every Transform of an entity without a parent, and every field the API marks `worldspace`, moves by the same amount (a body, its contacts and warm starts, a path, a particle, a tween to a place, the camera), and nothing in the world sees a change but the numbers. Precision then holds however far the world goes, while the zero's own place is kept here, in f64.

The shift is world state, decided from world state alone (the focus's Transform, else the first og.StreamAnchor, else og.Camera.position), so a replay, a rollback, a restore and every peer of an og.net session shift on the same tick by the same amount; a session refuses the og.Camera fallback (each peer's camera is its own) and holds the zero until a focus or an anchor exists. A world that never names it keeps its hash and frames. Values a game keeps outside the marked fields (a world position in its own component, a key it set in an og.ai blackboard) are its own: OriginShifted says when to add its `delta` to them. 2D worlds shift too, on `xy`.

Token `og.WorldOrigin`, registered under the schema name `og.WorldOrigin`; its value type is `WorldOrigin`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `number` | `2048` | ≥ 64 | `m` | — | How far the focus may stray from the zero along an axis before the world shifts, and the grid the zero moves on (whole metres). |
| `plane` | `WorldOriginPlane` | `"xz"` | — | — | — | The axes the zero moves along: xz (the default), xyz, or xy for a 2D world. |
| `focus` | `Entity` | `0` | — | — | — | The entity the zero follows (a player, a vehicle; a root). 0: the first og.StreamAnchor by entity id, else og.Camera.position. |
| `off` | `boolean` | `false` | — | — | — | Hold the zero where it is. |
| `x` | `number` | `0` | — | `m` | read only | Written: where the zero is in the world, along x (a place's absolute position is its own plus this). |
| `y` | `number` | `0` | — | `m` | read only | Written: where the zero is, along y. |
| `z` | `number` | `0` | — | `m` | read only | Written: where the zero is, along z. |
| `shifts` | `number` | `0` | — | — | read only | Written: how many times the world has shifted. |

```ogt
resource WorldOrigin {
  doc "Large worlds (R12): naming it keeps every place in the world near its zero. As each tick starts, before any system runs, when the focus is farther from the zero than `step` along one of the plane's axes, the whole world shifts by the multiple of `step` nearest the focus, so the focus is within half a step of the zero again: every Transform of an entity without a parent, and every field the API marks `worldspace`, moves by the same amount (a body, its contacts and warm starts, a path, a particle, a tween to a place, the camera), and nothing in the world sees a change but the numbers. Precision then holds however far the world goes, while the zero's own place is kept here, in f64."
  doc "The shift is world state, decided from world state alone (the focus's Transform, else the first og.StreamAnchor, else og.Camera.position), so a replay, a rollback, a restore and every peer of an og.net session shift on the same tick by the same amount; a session refuses the og.Camera fallback (each peer's camera is its own) and holds the zero until a focus or an anchor exists. A world that never names it keeps its hash and frames. Values a game keeps outside the marked fields (a world position in its own component, a key it set in an og.ai blackboard) are its own: OriginShifted says when to add its `delta` to them. 2D worlds shift too, on `xy`."
  field step f32 "How far the focus may stray from the zero along an axis before the world shifts, and the grid the zero moves on (whole metres)." {
    default 2048
    min 64
    unit m
  }
  field plane WorldOriginPlane "The axes the zero moves along: xz (the default), xyz, or xy for a 2D world."
  field focus Entity "The entity the zero follows (a player, a vehicle; a root). 0: the first og.StreamAnchor by entity id, else og.Camera.position."
  field off bool "Hold the zero where it is."
  field x f64 "Written: where the zero is in the world, along x (a place's absolute position is its own plus this)." {
    readonly true
    unit m
  }
  field y f64 "Written: where the zero is, along y." {
    readonly true
    unit m
  }
  field z f64 "Written: where the zero is, along z." {
    readonly true
    unit m
  }
  field shifts u32 "Written: how many times the world has shifted." {
    readonly true
  }
}
```

<sub>`api.ogt` line 1525</sub>
