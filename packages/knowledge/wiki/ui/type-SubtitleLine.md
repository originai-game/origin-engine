<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SubtitleLine"></a>

### `SubtitleLine` (desc struct)

A line of dialogue or a sound caption showing now.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key (its probe id), stable while it shows. |
| `text` | `string` | `""` | — | — | — | What is said, or the sound ([door creaks]). |
| `speaker` | `string` | `""` | — | — | — | Who says it; empty for a sound caption or an unseen narrator. |
| `color` | `Paint \| null` | `null` | — | — | — | The speaker's colour; null is the highlight. |
| `caption` | `boolean` | `false` | — | — | — | A sound caption, not speech: shown in brackets, only with captions on. |
| `direction` | `number \| null` | `null` | — | `rad` | — | Where its source is, clockwise from straight ahead (0 ahead, π/2 right); null on screen. With directions on, an arrow points there. |

```ogt
struct SubtitleLine {
  doc "A line of dialogue or a sound caption showing now."
  desc true
  field key String "Its key (its probe id), stable while it shows."
  field text String "What is said, or the sound ([door creaks])."
  field speaker String "Who says it; empty for a sound caption or an unseen narrator."
  field color Option.Paint "The speaker's colour; null is the highlight."
  field caption bool "A sound caption, not speech: shown in brackets, only with captions on."
  field direction Option.f32 "Where its source is, clockwise from straight ahead (0 ahead, π/2 right); null on screen. With directions on, an arrow points there." {
    unit rad
  }
}
```

<sub>Module `ui` line 5836</sub>
