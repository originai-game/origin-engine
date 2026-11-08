<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SubtitleSize"></a>

### `SubtitleSize` (enum)

The subtitle text size (XAG 104: the player chooses).

A string: one of the variants below. The zero value is `"small"`.

| Variant | Doc |
|---|---|
| `"small"` | Small: the caption step, never under 18 px at 1080p. |
| `"medium"` | Medium (the default). |
| `"large"` | Large. |
| `"huge"` | Huge. |

```ogt
enum SubtitleSize {
  doc "The subtitle text size (XAG 104: the player chooses)."
  variant small "Small: the caption step, never under 18 px at 1080p."
  variant medium "Medium (the default)."
  variant large "Large."
  variant huge "Huge."
}
```

<sub>Module `ui` line 5590</sub>
