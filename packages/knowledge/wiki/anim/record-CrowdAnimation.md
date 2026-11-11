<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-CrowdAnimation"></a>

### `og.CrowdAnimation` (component)

Plays a clip on a model on the GPU, for crowds: on an entity with og.Model and no og.Skeleton (which og.anim would pose on the CPU), the model is drawn skinned by `clip` baked into joint palettes, shared by every entity playing that clip on that skeleton. Nothing reads the pose (no markers, root motion, IK, blendshapes or joint attachments) and a change of `clip` cuts without a fade.

The clip's time is `offset` plus `speed` times the world's time. Drawn only, never fed back; TAA takes its motion vectors from the frame before.

See `animation §7`.

Token `og.CrowdAnimation`, registered under the schema name `og.CrowdAnimation`; its value type is `CrowdAnimation`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton asset of the model (og.load.animation's `skeleton`); 0 draws nothing posed. |
| `skin` | `number` | `0` | ≥ 0 | — | — | Which of the skeleton's skins poses the model (glTF skin index). |
| `clip` | `ClipAsset` | `0` | — | — | — | The clip played (og.load.animation's `clips`); 0 holds the rest pose. |
| `offset` | `number` | `0` | — | `s` | tunable | Seconds into the clip at the world's time 0. |
| `speed` | `number` | `1` | -10 … 10 | — | tunable | Playback rate (1 = as authored, negative plays backwards). |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the end of the clip; otherwise hold its last frame. |
| `rate` | `number` | `30` | 1 … 120 | — | — | Frames per second the clip is baked at (the GPU blends between them): more is smoother and costs more memory, `rate` × the clip's seconds × its joints × 64 bytes, once per clip. |

```ogt
component CrowdAnimation {
  doc "Plays a clip on a model on the GPU, for crowds: on an entity with og.Model and no og.Skeleton (which og.anim would pose on the CPU), the model is drawn skinned by `clip` baked into joint palettes, shared by every entity playing that clip on that skeleton. Nothing reads the pose (no markers, root motion, IK, blendshapes or joint attachments) and a change of `clip` cuts without a fade."
  doc "The clip's time is `offset` plus `speed` times the world's time. Drawn only, never fed back; TAA takes its motion vectors from the frame before."
  guide "animation §7"
  field skeleton SkeletonAsset "The skeleton asset of the model (og.load.animation's `skeleton`); 0 draws nothing posed."
  field skin u32 "Which of the skeleton's skins poses the model (glTF skin index)." {
    min 0
  }
  field clip ClipAsset "The clip played (og.load.animation's `clips`); 0 holds the rest pose."
  field offset f32 "Seconds into the clip at the world's time 0." {
    unit s
    tunable true
  }
  field speed f32 "Playback rate (1 = as authored, negative plays backwards)." {
    default 1
    min -10
    max 10
    tunable true
  }
  field loop bool "Wrap at the end of the clip; otherwise hold its last frame." {
    default true
  }
  field rate f32 "Frames per second the clip is baked at (the GPU blends between them): more is smoother and costs more memory, `rate` × the clip's seconds × its joints × 64 bytes, once per clip." {
    default 30
    min 1
    max 120
  }
}
```

<sub>Module `anim` line 101</sub>
