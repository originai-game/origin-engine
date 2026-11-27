<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphClip"></a>

### `GraphClip` (struct)

A named clip leaf.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `string` | `""` | — | — | — | ClipInfo.name, or a sheet/2D rig clip name for sprite graphs. |
| `speed` | `number` | `0` | — | — | optional | Positive playback rate; omitted is one. |
| `looping` | `boolean` | `false` | — | — | optional | Loop playback; omitted is true. |
| `mirror` | `boolean` | `false` | — | — | optional | Mirror a skeletal pose; omitted is false. |

```ogt
struct GraphClip {
  doc "A named clip leaf."
  field clip String "ClipInfo.name, or a sheet/2D rig clip name for sprite graphs."
  field speed f32 "Positive playback rate; omitted is one." {
    optional true
  }
  field looping bool "Loop playback; omitted is true." {
    optional true
  }
  field mirror bool "Mirror a skeletal pose; omitted is false." {
    optional true
  }
}
```

<sub>Module `anim` line 710</sub>
