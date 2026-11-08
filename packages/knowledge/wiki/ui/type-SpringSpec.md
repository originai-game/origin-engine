<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SpringSpec"></a>

### `SpringSpec` (desc struct)

A spring by how it feels: about how long it takes to settle and how much it overshoots. Retargeting it keeps its value and velocity, so an interrupted motion never jumps.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `durationMs` | `number` | `350` | — | `ms` | — | About how long it takes to arrive, ms. |
| `bounce` | `number` | `0` | -0.95 … 1 | — | — | 0 arrives without overshoot; 0.3 overshoots a little; up to 1 rings (below 0 is sluggish). The style's overshoot caps it. |

```ogt
struct SpringSpec {
  doc "A spring by how it feels: about how long it takes to settle and how much it overshoots. Retargeting it keeps its value and velocity, so an interrupted motion never jumps."
  desc true
  field durationMs f32 "About how long it takes to arrive, ms." {
    default 350
    unit ms
  }
  field bounce f32 "0 arrives without overshoot; 0.3 overshoots a little; up to 1 rings (below 0 is sluggish). The style's overshoot caps it." {
    min -0.95
    max 1
  }
}
```

<sub>Module `ui` line 2405</sub>
