<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="record-WorldOrigin"></a>

### `og.WorldOrigin` (resource)

Large worlds: naming it keeps every place near the zero. As a tick starts, before any system, a focus over `step` from the zero along a plane axis shifts the world by the multiple of `step` nearest it: every root Transform and every `worldspace` field (bodies with contacts and warm starts, paths, particles, tweens, the camera) moves by the same amount, and nothing sees a change but the numbers.

The shift is world state, decided from world state alone: replays, rollbacks, restores and every og.net peer shift alike, on the same tick. A world that never names it keeps its hash and frames.

Token `og.WorldOrigin`, registered under the schema name `og.WorldOrigin`; its value type is `WorldOrigin`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `number` | `2048` | ≥ 64 | `m` | — | How far the focus may stray from the zero along an axis before the world shifts, and the grid the zero moves on (whole metres). |
| `plane` | `WorldOriginPlane` | `"xz"` | — | — | — | The axes the zero moves along: xz (the default), xyz, or xy for a 2D world. |
| `focus` | `Entity` | `0` | — | — | — | The entity the zero follows (a player, a vehicle; a root). 0: the first og.StreamAnchor by entity id, else og.Camera.position. An og.net session refuses the camera fallback (each peer's camera is its own) and holds the zero until a focus or an anchor exists. |
| `off` | `boolean` | `false` | — | — | — | Hold the zero where it is. |
| `x` | `number` | `0` | — | `m` | read only | Written: where the zero is in the world along x, kept in f64 (a place's absolute position is its own plus this). |
| `y` | `number` | `0` | — | `m` | read only | Written: where the zero is, along y. |
| `z` | `number` | `0` | — | `m` | read only | Written: where the zero is, along z. |
| `shifts` | `number` | `0` | — | — | read only | Written: how many times the world has shifted. |

```ogt
resource WorldOrigin {
  doc "Large worlds: naming it keeps every place near the zero. As a tick starts, before any system, a focus over `step` from the zero along a plane axis shifts the world by the multiple of `step` nearest it: every root Transform and every `worldspace` field (bodies with contacts and warm starts, paths, particles, tweens, the camera) moves by the same amount, and nothing sees a change but the numbers."
  doc "The shift is world state, decided from world state alone: replays, rollbacks, restores and every og.net peer shift alike, on the same tick. A world that never names it keeps its hash and frames."
  field step f32 "How far the focus may stray from the zero along an axis before the world shifts, and the grid the zero moves on (whole metres)." {
    default 2048
    min 64
    unit m
  }
  field plane WorldOriginPlane "The axes the zero moves along: xz (the default), xyz, or xy for a 2D world."
  field focus Entity "The entity the zero follows (a player, a vehicle; a root). 0: the first og.StreamAnchor by entity id, else og.Camera.position. An og.net session refuses the camera fallback (each peer's camera is its own) and holds the zero until a focus or an anchor exists."
  field off bool "Hold the zero where it is."
  field x f64 "Written: where the zero is in the world along x, kept in f64 (a place's absolute position is its own plus this)." {
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

<sub>Module `transform` line 43</sub>
