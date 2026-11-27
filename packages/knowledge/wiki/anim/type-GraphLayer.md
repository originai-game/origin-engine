<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphLayer"></a>

### `GraphLayer` (struct)

A named state machine. The first layer supplies root motion and AnimationGraph.state.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Unique layer name. |
| `mode` | `GraphLayerMode` | `"override"` | — | — | optional | Pose combination; omitted is override. |
| `weight` | `number` | `0` | — | — | optional | Fixed weight in 0..1; omitted is one. Exclusive with weightParam. |
| `weightParam` | `string` | `""` | — | — | optional | Parameter-driven weight, clamped to 0..1 at playback. |
| `states` | `readonly GraphState[]` | `[]` | — | — | — | At least one state, in state-index order. |
| `entry` | `string` | `""` | — | — | — | Initial state name. |
| `transitions` | `readonly GraphTransition[]` | `[]` | — | — | optional | Priority order; first eligible transition fires. |
| `mask` | `Readonly<Record<string, number>>` | `{}` | — | — | optional | Skeleton node names to subtree weights in 0..1; omitted means all joints. |

```ogt
struct GraphLayer {
  doc "A named state machine. The first layer supplies root motion and AnimationGraph.state."
  field name String "Unique layer name."
  field mode GraphLayerMode "Pose combination; omitted is override." {
    optional true
  }
  field weight f32 "Fixed weight in 0..1; omitted is one. Exclusive with weightParam." {
    optional true
  }
  field weightParam String "Parameter-driven weight, clamped to 0..1 at playback." {
    optional true
  }
  field states List.GraphState "At least one state, in state-index order."
  field entry String "Initial state name."
  field transitions List.GraphTransition "Priority order; first eligible transition fires." {
    optional true
  }
  field mask Map.f32 "Skeleton node names to subtree weights in 0..1; omitted means all joints." {
    optional true
  }
}
```

<sub>Module `anim` line 780</sub>
