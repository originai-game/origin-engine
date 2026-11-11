<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-HumanoidDesc"></a>

### `HumanoidDesc` (desc struct)

A skeleton and, optionally, the rig that says how its bones are named.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton (og.load.animation's `skeleton`). |
| `rig` | `Rig` | `0` | — | — | — | A humanoid map from og.load.rig; 0 detects the bones from the node names (Mixamo, VRM, Unity, Rigify, Unreal, Blender or SOMA names). |

```ogt
struct HumanoidDesc {
  doc "A skeleton and, optionally, the rig that says how its bones are named."
  desc true
  field skeleton SkeletonAsset "The skeleton (og.load.animation's `skeleton`)."
  field rig Rig "A humanoid map from og.load.rig; 0 detects the bones from the node names (Mixamo, VRM, Unity, Rigify, Unreal, Blender or SOMA names)."
}
```

<sub>Module `anim` line 611</sub>
