<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SubtitleSettings"></a>

### `SubtitleSettings` (desc struct)

The player's subtitle settings (XAG 104), from a settings screen.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `true` | — | — | — | Subtitles show. |
| `captions` | `boolean` | `false` | — | — | — | Sound captions show ([explosion], [footsteps]). |
| `size` | `SubtitleSize` | `"medium"` | — | — | — | The text size. |
| `background` | `number` | `0.75` | 0 … 1 | `ratio` | — | The box behind the text: 0 none (the text keeps its outline), 1 opaque. |
| `speakerNames` | `boolean` | `true` | — | — | — | Each line begins with its speaker's name. |
| `speakerColors` | `boolean` | `true` | — | — | — | Each speaker's name in its colour. |
| `directions` | `boolean` | `true` | — | — | — | An arrow beside a line whose speaker or sound is off screen, toward it. |

```ogt
struct SubtitleSettings {
  doc "The player's subtitle settings (XAG 104), from a settings screen."
  desc true
  field enabled bool "Subtitles show." {
    default true
  }
  field captions bool "Sound captions show ([explosion], [footsteps])."
  field size SubtitleSize "The text size." {
    default medium
  }
  field background f32 "The box behind the text: 0 none (the text keeps its outline), 1 opaque." {
    default 0.75
    min 0
    max 1
    unit ratio
  }
  field speakerNames bool "Each line begins with its speaker's name." {
    default true
  }
  field speakerColors bool "Each speaker's name in its colour." {
    default true
  }
  field directions bool "An arrow beside a line whose speaker or sound is off screen, toward it." {
    default true
  }
}
```

<sub>Module `ui` line 4862</sub>
