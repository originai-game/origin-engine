<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-AudioFeedbackDesc"></a>

### `AudioFeedbackDesc` (desc struct)

Presentation-only one-shot parameters.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `volume` | `number` | `1` | ≥ 0 | — | — | Linear gain. |
| `playbackRate` | `number` | `1` | — | — | — | Playback rate, strictly greater than zero. |

```ogt
struct AudioFeedbackDesc {
  doc "Presentation-only one-shot parameters."
  desc true
  field volume f32 "Linear gain." {
    default 1
    min 0
  }
  field playbackRate f32 "Playback rate, strictly greater than zero." {
    default 1
  }
}
```

<sub>Module `audio` line 709</sub>
