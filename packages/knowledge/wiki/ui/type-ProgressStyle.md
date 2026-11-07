<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ProgressStyle"></a>

### `ProgressStyle` (enum)

How a progress bar shows its value.

A string: one of the variants below. The zero value is `"bar"`.

| Variant | Doc |
|---|---|
| `"bar"` | A track filled from its start. |
| `"steps"` | Equal segments, each filled whole as the value passes it (a level's stages). |
| `"indeterminate"` | No value: a band sweeping along the track (loading of unknown length). |

```ogt
enum ProgressStyle {
  doc "How a progress bar shows its value."
  variant bar "A track filled from its start."
  variant steps "Equal segments, each filled whole as the value passes it (a level's stages)."
  variant indeterminate "No value: a band sweeping along the track (loading of unknown length)."
}
```

<sub>Module `ui` line 5470</sub>
