<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-FrameCap"></a>

### `FrameCap` (enum)

How many frames a second the display draws at most (DisplaySettings.frameCap). The simulation's fixed tick is the same at every cap: a capped frame runs the ticks its time owes, and input is read on every display frame, drawn or not.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | The device class's default: 60 on a phone or tablet (a coarse pointer), which halves the GPU's work on a 120 Hz ProMotion display for a game whose tick is 60 Hz or less; the display's own rate on a desktop. 30 with batterySaver. |
| `"display"` | Every frame the display shows (requestAnimationFrame): 120 on ProMotion, 144 and more on gaming monitors. |
| `"fps60"` | At most 60 a second. |
| `"fps30"` | At most 30 a second: the battery's choice, and a steady rate for a GPU that cannot hold 60. |
| `"fps120"` | At most 120 a second: a 144 Hz or faster monitor held to a rate its GPU keeps. |
| `"fps144"` | At most 144 a second: a 240 Hz monitor held to the gaming rate. |

```ogt
enum FrameCap {
  doc "How many frames a second the display draws at most (DisplaySettings.frameCap). The simulation's fixed tick is the same at every cap: a capped frame runs the ticks its time owes, and input is read on every display frame, drawn or not."
  variant auto "The device class's default: 60 on a phone or tablet (a coarse pointer), which halves the GPU's work on a 120 Hz ProMotion display for a game whose tick is 60 Hz or less; the display's own rate on a desktop. 30 with batterySaver."
  variant display "Every frame the display shows (requestAnimationFrame): 120 on ProMotion, 144 and more on gaming monitors."
  variant fps60 "At most 60 a second."
  variant fps30 "At most 30 a second: the battery's choice, and a steady rate for a GPU that cannot hold 60."
  variant fps120 "At most 120 a second: a 144 Hz or faster monitor held to a rate its GPU keeps."
  variant fps144 "At most 144 a second: a 240 Hz monitor held to the gaming rate."
}
```

<sub>Module `render` line 2745</sub>
