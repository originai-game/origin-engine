<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-RetargetDesc"></a>

### `RetargetDesc` (desc struct)

A clip made for one skeleton, to play on another (other proportions, rest pose or bone names).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | The clip, made for `source`. |
| `source` | `SkeletonAsset` | `0` | — | — | — | The skeleton the clip was made for (og.load.animation's `skeleton`: a character's, or an animation library's). |
| `target` | `SkeletonAsset` | `0` | — | — | — | The skeleton that will play it. |
| `sourceRig` | `Rig` | `0` | — | — | — | How the source's bones are named (og.load.rig); 0 detects them. |
| `targetRig` | `Rig` | `0` | — | — | — | How the target's bones are named; 0 detects them. |
| `footIk` | `boolean` | `true` | — | — | — | Solve the legs so a planted foot stays planted on the target's ground (and the hips lower when a leg cannot reach). |

```ogt
struct RetargetDesc {
  doc "A clip made for one skeleton, to play on another (other proportions, rest pose or bone names)."
  desc true
  field clip ClipAsset "The clip, made for `source`."
  field source SkeletonAsset "The skeleton the clip was made for (og.load.animation's `skeleton`: a character's, or an animation library's)."
  field target SkeletonAsset "The skeleton that will play it."
  field sourceRig Rig "How the source's bones are named (og.load.rig); 0 detects them."
  field targetRig Rig "How the target's bones are named; 0 detects them."
  field footIk bool "Solve the legs so a planted foot stays planted on the target's ground (and the hips lower when a leg cannot reach)." {
    default true
  }
}
```

<sub>`api.ogt` line 12093</sub>
