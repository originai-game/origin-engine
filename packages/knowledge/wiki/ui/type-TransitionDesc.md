<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TransitionDesc"></a>

### `TransitionDesc` (desc struct)

How a transition moves, timed in ticks from the style's motion.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `TransitionKind` | `"fade"` | — | — | — | The movement. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long entering and leaving take; 0 is motion.duration.base (240 ms). |
| `distance` | `number` | `0` | ≥ 0 | `cssPx` | — | How far slides travel; 0 is space.xl. |
| `instant` | `boolean` | `false` | — | — | — | Shown at once the first time it is built (no entrance). |

```ogt
struct TransitionDesc {
  doc "How a transition moves, timed in ticks from the style's motion."
  desc true
  field kind TransitionKind "The movement."
  field durationMs f32 "How long entering and leaving take; 0 is motion.duration.base (240 ms)." {
    unit ms
    min 0
  }
  field distance f32 "How far slides travel; 0 is space.xl." {
    unit cssPx
    min 0
  }
  field instant bool "Shown at once the first time it is built (no entrance)."
}
```

<sub>Module `ui` line 898</sub>
