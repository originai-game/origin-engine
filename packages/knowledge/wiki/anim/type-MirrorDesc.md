<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-MirrorDesc"></a>

### `MirrorDesc` (desc struct)

A clip to play on the other side of the skeleton it was made for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | The clip, made for `skeleton`. |
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton the clip was made for (og.load.animation's `skeleton`); its left and right nodes are paired by name, the side tokens every rig naming scheme writes swapped (LeftArm and RightArm, thigh.L and thigh.R, J_Bip_L_ and J_Bip_R_, calf_l and calf_r). |

```ogt
struct MirrorDesc {
  doc "A clip to play on the other side of the skeleton it was made for."
  desc true
  field clip ClipAsset "The clip, made for `skeleton`."
  field skeleton SkeletonAsset "The skeleton the clip was made for (og.load.animation's `skeleton`); its left and right nodes are paired by name, the side tokens every rig naming scheme writes swapped (LeftArm and RightArm, thigh.L and thigh.R, J_Bip_L_ and J_Bip_R_, calf_l and calf_r)."
}
```

<sub>Module `anim` line 646</sub>
