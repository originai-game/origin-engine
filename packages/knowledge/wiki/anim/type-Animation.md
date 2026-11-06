<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-Animation"></a>

### `Animation` (struct)

What og.load.animation registered from a model's `#anim` blob.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The rig, what Skeleton.asset takes. |
| `clips` | `readonly ClipInfo[]` | `[]` | — | — | — | Its clips, in glTF order. |
| `nodes` | `readonly string[]` | `[]` | — | — | — | The rig's node names, in topological order. |
| `skins` | `readonly number[]` | `[]` | — | — | — | Each skin's joint count, in glTF order. |
| `morphTargets` | `number` | `0` | — | — | — | Morph targets across the model's meshes. |

```ogt
struct Animation {
  doc "What og.load.animation registered from a model's `#anim` blob."
  field skeleton SkeletonAsset "The rig, what Skeleton.asset takes."
  field clips List.ClipInfo "Its clips, in glTF order."
  field nodes List.String "The rig's node names, in topological order."
  field skins List.u32 "Each skin's joint count, in glTF order."
  field morphTargets u32 "Morph targets across the model's meshes."
}
```

<sub>`api.ogt` line 12027</sub>
