<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SpeechBubbleDesc"></a>

### `SpeechBubbleDesc` (desc struct)

A speech bubble over a speaker.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | The speaker. |
| `height` | `number` | `2.2` | — | `m` | — | World units above the speaker's point. |
| `text` | `string` | `""` | — | — | — | What is said. |
| `speaker` | `string` | `""` | — | — | — | Who says it, over the text; empty is none. |
| `at` | `number` | `0` | — | — | — | og.ui.tick() when it was said: it types out from then and leaves after its time. |
| `charsPerSecond` | `number` | `40` | ≥ 0 | — | — | How fast it types; 0 shows it at once. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it stays after it has typed; 0 is 1500 plus 60 per character. |
| `shout` | `boolean` | `false` | — | — | — | A shout: bold, a jagged edge. |
| `thought` | `boolean` | `false` | — | — | — | A thought: rounder, the tail as bubbles. |
| `maxWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | Its widest; 0 is 260. |
| `clamp` | `boolean` | `false` | — | — | — | Off screen, it stays on the edge so its words are not missed. |

```ogt
struct SpeechBubbleDesc {
  doc "A speech bubble over a speaker."
  desc true
  field target WorldTarget "The speaker."
  field height f32 "World units above the speaker's point." {
    default 2.2
    unit m
  }
  field text String "What is said."
  field speaker String "Who says it, over the text; empty is none."
  field at u32 "og.ui.tick() when it was said: it types out from then and leaves after its time."
  field charsPerSecond f32 "How fast it types; 0 shows it at once." {
    default 40
    min 0
  }
  field durationMs f32 "How long it stays after it has typed; 0 is 1500 plus 60 per character." {
    min 0
    unit ms
  }
  field shout bool "A shout: bold, a jagged edge."
  field thought bool "A thought: rounder, the tail as bubbles."
  field maxWidth f32 "Its widest; 0 is 260." {
    min 0
    unit cssPx
  }
  field clamp bool "Off screen, it stays on the edge so its words are not missed."
}
```

<sub>Module `ui` line 6854</sub>
