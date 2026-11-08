<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TweenDefaults"></a>

### `TweenDefaults` (desc struct)

What a timeline's tweens take when they leave it out (GSAP's `defaults`).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `component` | `AnyComponent \| null` | `null` | — | — | — | The component; null is og.Transform. |
| `duration` | `number` | `0.3` | ≥ 0 | `s` | — | Seconds per cycle. |
| `ease` | `Ease` | `"power1.out"` | — | — | — | The curve. |

```ogt
struct TweenDefaults {
  doc "What a timeline's tweens take when they leave it out (GSAP's `defaults`)."
  desc true
  field component Option.AnyComponent "The component; null is og.Transform."
  field duration f32 "Seconds per cycle." {
    default 0.3
    min 0
    unit s
  }
  field ease Ease "The curve."
}
```

<sub>Module `tween` line 132</sub>
