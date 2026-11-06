<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-IkChain"></a>

### `og.IkChain` (component)

An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order.

Also known as: inverse kinematics, IK, reach, two-bone IK, FABRIK, CCD, hand IK, Unity Two Bone IK constraint

Token `og.IkChain`, registered under the schema name `og.IkChain`; its value type is `IkChain`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton this chain bends. |
| `tip` | `number` | `0` | ≥ 0 | — | — | The end joint (an index into the skin's joints): the foot or the hand. |
| `bones` | `number` | `2` | 1 … 32 | — | — | How many bones above the tip turn (two-bone always uses 2). |
| `solver` | `IkSolver` | `"twoBone"` | — | — | — | twoBone (analytic, with a pole) or fabrik (iterative, any length). |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `hook` | Where the tip should be, in `space`: for `world`, the frame of the skeleton entity's og.Transform (its parent's frame when it has a parent, not true world). |
| `pole` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `hook` | A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend. |
| `space` | `IkSpace` | `"world"` | — | — | — | What `target` and `pole` are relative to: world (the frame the skeleton entity's og.Transform is in, so its parent's frame for a child entity, not true world) or model. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved). |
| `iterations` | `number` | `10` | 1 … 64 | — | — | FABRIK iterations at most. |
| `tolerance` | `number` | `0.001` | ≥ 0 | `m` | — | The tip distance (model units) at which the target counts as reached. |
| `reached` | `boolean` | `false` | — | — | read only | Written: the tip is within `tolerance` of the target after solving. |
| `error` | `number` | `0` | — | `m` | read only | Written: the tip's distance from the target after solving, in model units. |

```ogt
component IkChain {
  doc "An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order."
  aliases "inverse kinematics, IK, reach, two-bone IK, FABRIK, CCD, hand IK, Unity Two Bone IK constraint"
  field skeleton Entity "The entity with the og.Skeleton this chain bends."
  field tip u32 "The end joint (an index into the skin's joints): the foot or the hand." {
    min 0
  }
  field bones u32 "How many bones above the tip turn (two-bone always uses 2)." {
    default 2
    min 1
    max 32
  }
  field solver IkSolver "twoBone (analytic, with a pole) or fabrik (iterative, any length)."
  field target Vector3 "Where the tip should be, in `space`: for `world`, the frame of the skeleton entity's og.Transform (its parent's frame when it has a parent, not true world)." {
    worldspace hook
    unit m
  }
  field pole Vector3 "A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend." {
    worldspace hook
    unit m
  }
  field space IkSpace "What `target` and `pole` are relative to: world (the frame the skeleton entity's og.Transform is in, so its parent's frame for a child entity, not true world) or model."
  field weight f32 "How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field iterations u32 "FABRIK iterations at most." {
    default 10
    min 1
    max 64
  }
  field tolerance f32 "The tip distance (model units) at which the target counts as reached." {
    default 0.001
    min 0
    unit m
  }
  field reached bool "Written: the tip is within `tolerance` of the target after solving." {
    readonly true
  }
  field error f32 "Written: the tip's distance from the target after solving, in model units." {
    unit m
    readonly true
  }
}
```

<sub>`api.ogt` line 11733</sub>
