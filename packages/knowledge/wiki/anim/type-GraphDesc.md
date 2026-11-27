<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphDesc"></a>

### `GraphDesc` (desc struct)

A blend-tree graph to register.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `params` | `readonly string[]` | `[]` | — | — | — | At most eight unique parameter names, in AnimationGraph p0..p7 order. |
| `motions` | `Readonly<Record<string, GraphMotion>>` | `{}` | — | — | — | Named clip and blend motions; cycles are rejected. |
| `layers` | `readonly GraphLayer[]` | `[]` | — | — | — | At least one layer, in playback order. |
| `clips` | `readonly ClipInfo[]` | `[]` | — | — | — | Clip leaves resolve by ClipInfo.name, usually og.load.animation's clips. |
| `sprite` | `boolean` | `false` | — | — | — | Resolve clip names on an entity's flipbook or 2D rig at playback instead; clips is not read. |

```ogt
struct GraphDesc {
  doc "A blend-tree graph to register."
  desc true
  field params List.String "At most eight unique parameter names, in AnimationGraph p0..p7 order."
  field motions Map.GraphMotion "Named clip and blend motions; cycles are rejected."
  field layers List.GraphLayer "At least one layer, in playback order."
  field clips List.ClipInfo "Clip leaves resolve by ClipInfo.name, usually og.load.animation's clips."
  field sprite bool "Resolve clip names on an entity's flipbook or 2D rig at playback instead; clips is not read."
}
```

<sub>Module `anim` line 790</sub>
