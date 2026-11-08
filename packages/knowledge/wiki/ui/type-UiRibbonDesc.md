<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiRibbonDesc"></a>

### `UiRibbonDesc` (desc struct)

A ribbon on a card or a result.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | Its words (NEW, -50%, New best!). |
| `corner` | `Anchor` | `"topRight"` | — | — | — | Where on the open container: topLeft or topRight wrap the corner diagonally; top or bottom lie across as a banner with notched ends; center slaps it across the middle like a stamp. |
| `tone` | `Tone` | `"primary"` | — | — | — | Its colour; the default is primary. |
| `shownAt` | `number \| null` | `null` | — | — | — | og.ui.tick() when it slaps on (a stamp's motion and a sound); null shows it settled. |
| `name` | `string` | `""` | — | — | — | What a screen reader says; empty is the text. |

```ogt
struct UiRibbonDesc {
  doc "A ribbon on a card or a result."
  desc true
  field text String "Its words (NEW, -50%, New best!)."
  field corner Anchor "Where on the open container: topLeft or topRight wrap the corner diagonally; top or bottom lie across as a banner with notched ends; center slaps it across the middle like a stamp." {
    default topRight
  }
  field tone Tone "Its colour; the default is primary." {
    default primary
  }
  field shownAt Option.u32 "og.ui.tick() when it slaps on (a stamp's motion and a sound); null shows it settled."
  field name String "What a screen reader says; empty is the text."
}
```

<sub>Module `ui` line 5614</sub>
