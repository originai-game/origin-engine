<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CarouselDesc"></a>

### `CarouselDesc` (desc struct)

Pages side by side that a drag or a flick moves, snapping to a page.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many pages. |
| `index` | `number` | `0` | — | — | — | The current page (keep CarouselResult.index). |
| `peek` | `number \| null` | `null` | — | `cssPx` | — | How much of the pages beside the current one shows; null is 24 on a phone, else 0. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Space between pages; null is space.md. |
| `arrows` | `boolean \| null` | `null` | — | — | — | Previous and next buttons; null is all but a phone. |
| `dots` | `boolean` | `true` | — | — | — | Page dots under it. |
| `loop` | `boolean` | `false` | — | — | — | The arrows wrap around. |
| `autoplayMs` | `number` | `0` | ≥ 0 | `ms` | — | Move on by itself after this long untouched; 0 never (and never under reduced motion). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | The pages' height; null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |

```ogt
struct CarouselDesc {
  doc "Pages side by side that a drag or a flick moves, snapping to a page."
  desc true
  field count u32 "How many pages."
  field index u32 "The current page (keep CarouselResult.index)."
  field peek Option.f32 "How much of the pages beside the current one shows; null is 24 on a phone, else 0." {
    unit cssPx
  }
  field gap Option.f32 "Space between pages; null is space.md." {
    unit cssPx
  }
  field arrows Option.bool "Previous and next buttons; null is all but a phone."
  field dots bool "Page dots under it." {
    default true
  }
  field loop bool "The arrows wrap around."
  field autoplayMs f32 "Move on by itself after this long untouched; 0 never (and never under reduced motion)." {
    unit ms
    min 0
  }
  field width Option.Size "Its width; null grows."
  field height Option.Size "The pages' height; null fits."
  field label String "What a screen reader calls it."
}
```

<sub>Module `ui` line 2302</sub>
