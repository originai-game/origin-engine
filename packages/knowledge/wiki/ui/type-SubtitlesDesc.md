<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SubtitlesDesc"></a>

### `SubtitlesDesc` (desc struct)

The subtitles showing now.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lines` | `readonly SubtitleLine[]` | `[]` | — | — | — | The lines showing now, oldest first: at most two speakers' lines show (the newest), captions above them. |
| `settings` | `SubtitleSettings` | `{}` | — | — | — | The player's settings. |
| `bottom` | `number` | `0` | ≥ 0 | `cssPx` | — | Extra space above the safe area's bottom (to sit over a HUD bar). |

```ogt
struct SubtitlesDesc {
  doc "The subtitles showing now."
  desc true
  field lines List.SubtitleLine "The lines showing now, oldest first: at most two speakers' lines show (the newest), captions above them."
  field settings SubtitleSettings "The player's settings."
  field bottom f32 "Extra space above the safe area's bottom (to sit over a HUD bar)." {
    unit cssPx
    min 0
  }
}
```

<sub>Module `ui` line 5848</sub>
