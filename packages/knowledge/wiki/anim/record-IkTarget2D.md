<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-IkTarget2D"></a>

### `og.IkTarget2D` (component)

Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge, a tail's tip. A constraint is a chain of any length: one bone points at the target, two are solved exactly (bending the way the rig says), three or more by FABRIK from the animated pose, curling the way the rig says. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way.

Token `og.IkTarget2D`, registered under the schema name `og.IkTarget2D`; its value type is `IkTarget2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton2D. |
| `constraint` | `number` | `0` | ≥ 0 | — | — | The constraint: an index into Rig2DInfo.constraints. |
| `target` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `hook` | Where the chain's tip should be, in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent). |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the solve applies (0 is the animated pose, 1 fully solved), times the constraint's own weight (its `weight` line, or its clip's IK keys). |
| `tolerance` | `number` | `0.01` | ≥ 0 | `m` | — | The distance at which the target counts as reached. |
| `reached` | `boolean` | `false` | — | — | read only | Written: the tip is within `tolerance` of the target after solving. |
| `error` | `number` | `0` | — | `m` | read only | Written: the tip's distance from the target after solving (for a one-bone constraint, how far the target lies off the bone's line). |

```ogt
component IkTarget2D {
  doc "Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge, a tail's tip. A constraint is a chain of any length: one bone points at the target, two are solved exactly (bending the way the rig says), three or more by FABRIK from the animated pose, curling the way the rig says. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way."
  field skeleton Entity "The entity with the og.Skeleton2D."
  field constraint u32 "The constraint: an index into Rig2DInfo.constraints." {
    min 0
  }
  field target Vector2 "Where the chain's tip should be, in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent)." {
    worldspace hook
    unit m
  }
  field weight f32 "How much of the solve applies (0 is the animated pose, 1 fully solved), times the constraint's own weight (its `weight` line, or its clip's IK keys)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field tolerance f32 "The distance at which the target counts as reached." {
    default 0.01
    min 0
    unit m
  }
  field reached bool "Written: the tip is within `tolerance` of the target after solving." {
    readonly true
  }
  field error f32 "Written: the tip's distance from the target after solving (for a one-bone constraint, how far the target lies off the bone's line)." {
    unit m
    readonly true
  }
}
```

<sub>`api.ogt` line 12291</sub>
