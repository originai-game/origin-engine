<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DialogueDesc"></a>

### `DialogueDesc` (desc struct)

A dialogue box: a portrait slot, the speaker and the line typed out by the tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `speaker` | `string` | `""` | — | — | — | Who speaks (primary ink). |
| `text` | `string` | `""` | — | — | — | The line. A new text starts typing again. |
| `portrait` | `Texture \| null` | `null` | — | — | — | A portrait texture; null is the speaker's initial. |
| `portraitUv` | `[number, number, number, number] \| null` | `null` | — | — | — | The portrait's region of the texture; null is the whole texture. |
| `portraitIcon` | `string` | `""` | — | — | — | A portrait from the icon atlas by name, when no texture is given; empty is the speaker's initial. |
| `charsPerSecond` | `number` | `0` | ≥ 0 | — | — | Typing speed; 0 is 40. |
| `portraitRight` | `boolean` | `false` | — | — | — | The portrait on the right. |

```ogt
struct DialogueDesc {
  doc "A dialogue box: a portrait slot, the speaker and the line typed out by the tick."
  desc true
  field speaker String "Who speaks (primary ink)."
  field text String "The line. A new text starts typing again."
  field portrait Option.Texture "A portrait texture; null is the speaker's initial."
  field portraitUv Option.Array.f32.4 "The portrait's region of the texture; null is the whole texture."
  field portraitIcon String "A portrait from the icon atlas by name, when no texture is given; empty is the speaker's initial."
  field charsPerSecond f32 "Typing speed; 0 is 40." {
    min 0
  }
  field portraitRight bool "The portrait on the right."
}
```

<sub>Module `ui` line 1060</sub>
