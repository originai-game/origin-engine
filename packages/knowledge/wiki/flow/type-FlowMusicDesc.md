<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowMusicDesc"></a>

### `FlowMusicDesc` (desc struct)

Persistent cross-segment background music (og.flow.setMusic or `og-flow-music` header).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sound` | `string` | `""` | — | — | — | The sound asset id to play across segments; empty stops persistent flow music. |
| `volume` | `number` | `1` | 0 … 2 | — | — | Target linear volume (0 to 2). |
| `fade` | `number` | `0.5` | ≥ 0 | `s` | — | Crossfade duration in seconds. |
| `duck` | `number` | `0.4` | 0 … 1 | — | — | Volume multiplier while a segment plays video audio (0 to 1; 1 disables ducking). |

```ogt
struct FlowMusicDesc {
  doc "Persistent cross-segment background music (og.flow.setMusic or `og-flow-music` header)."
  desc true
  field sound String "The sound asset id to play across segments; empty stops persistent flow music."
  field volume f32 "Target linear volume (0 to 2)." {
    default 1
    min 0
    max 2
  }
  field fade f32 "Crossfade duration in seconds." {
    default 0.5
    min 0
    unit s
  }
  field duck f32 "Volume multiplier while a segment plays video audio (0 to 1; 1 disables ducking)." {
    default 0.4
    min 0
    max 1
  }
}
```

<sub>Module `flow` line 200</sub>
