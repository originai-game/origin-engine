<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-AnimationGraph"></a>

### `og.AnimationGraph` (component)

Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, named in the graph's `param` order (`param speed` is p0).

On an og.Flipbook or og.Skeleton2D entity a GraphDesc.sprite graph drives it instead. A flipbook shows the heaviest clip (an upper override layer at weight 0.5 or more wins) at its rate, cutting on frame ends and keeping the phase within a state or sync group; a 2D skeleton blends the two heaviest clips and cross-fades over a transition's `fade`. Additive layers and masks are skeleton-only.

See `animation §9`.

Token `og.AnimationGraph`, registered under the schema name `og.AnimationGraph`; its value type is `AnimationGraph`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `GraphAsset` | `0` | — | — | — | The graph asset; 0 is none. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the pose the graph decides over the rest pose. |
| `speed` | `number` | `1` | 0 … 10 | — | tunable | A playback-rate multiplier for every layer. |
| `p0` | `number` | `0` | — | — | tunable | Parameter 0 (the graph's first `param`). |
| `p1` | `number` | `0` | — | — | tunable | Parameter 1. |
| `p2` | `number` | `0` | — | — | tunable | Parameter 2. |
| `p3` | `number` | `0` | — | — | tunable | Parameter 3. |
| `p4` | `number` | `0` | — | — | tunable | Parameter 4. |
| `p5` | `number` | `0` | — | — | tunable | Parameter 5. |
| `p6` | `number` | `0` | — | — | tunable | Parameter 6. |
| `p7` | `number` | `0` | — | — | tunable | Parameter 7. |
| `state` | `number` | `0` | — | — | read only | Written: the base layer's current state (its index in the graph file). |
| `stateTime` | `number` | `0` | — | `s` | read only | Written: seconds since the base layer entered `state`. |

```ogt
component AnimationGraph {
  doc "Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, named in the graph's `param` order (`param speed` is p0)."
  doc "On an og.Flipbook or og.Skeleton2D entity a GraphDesc.sprite graph drives it instead. A flipbook shows the heaviest clip (an upper override layer at weight 0.5 or more wins) at its rate, cutting on frame ends and keeping the phase within a state or sync group; a 2D skeleton blends the two heaviest clips and cross-fades over a transition's `fade`. Additive layers and masks are skeleton-only."
  guide "animation §9"
  field asset GraphAsset "The graph asset; 0 is none."
  field weight f32 "How much of the pose the graph decides over the rest pose." {
    default 1
    min 0
    max 1
    tunable true
  }
  field speed f32 "A playback-rate multiplier for every layer." {
    default 1
    min 0
    max 10
    tunable true
  }
  field p0 f32 "Parameter 0 (the graph's first `param`)." {
    tunable true
  }
  field p1 f32 "Parameter 1." {
    tunable true
  }
  field p2 f32 "Parameter 2." {
    tunable true
  }
  field p3 f32 "Parameter 3." {
    tunable true
  }
  field p4 f32 "Parameter 4." {
    tunable true
  }
  field p5 f32 "Parameter 5." {
    tunable true
  }
  field p6 f32 "Parameter 6." {
    tunable true
  }
  field p7 f32 "Parameter 7." {
    tunable true
  }
  field state u32 "Written: the base layer's current state (its index in the graph file)." {
    readonly true
  }
  field stateTime f32 "Written: seconds since the base layer entered `state`." {
    unit s
    readonly true
  }
}
```

<sub>Module `anim` line 170</sub>
