<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TweenSpec"></a>

### `TweenSpec` (desc struct)

A duration on an ease: a retarget starts a new ease from where it is.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `durationMs` | `number` | `200` | — | `ms` | — | Its length, ms. |
| `ease` | `Ease` | `"power1.out"` | — | — | — | A GSAP-named curve shared with tweens. Omitted: the UI arrival cubic Bézier, not a GSAP approximation. |
| `bezier` | `readonly number[]` | `[]` | — | — | — | A cubic Bézier (x1, y1, x2, y2) in place of the named ease (empty: the named one). |

```ogt
struct TweenSpec {
  doc "A duration on an ease: a retarget starts a new ease from where it is."
  desc true
  field durationMs f32 "Its length, ms." {
    default 200
    unit ms
  }
  field ease Ease "A GSAP-named curve shared with tweens. Omitted: the UI arrival cubic Bézier, not a GSAP approximation."
  field bezier List.f32 "A cubic Bézier (x1, y1, x2, y2) in place of the named ease (empty: the named one)."
}
```

<sub>Module `ui` line 4453</sub>
