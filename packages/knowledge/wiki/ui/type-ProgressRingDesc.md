<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ProgressRingDesc"></a>

### `ProgressRingDesc` (desc struct)

A progress ring or spinner.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number \| null` | `null` | — | `ratio` | — | Done, 0 to 1; null spins (indeterminate). |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its diameter; 0 is 48. |
| `thickness` | `number` | `0` | ≥ 0 | `cssPx` | — | The ring's width; 0 is a tenth of the size, at least 3. |
| `tone` | `Tone` | `"neutral"` | — | — | — | The fill's tone; neutral is the primary. |
| `label` | `string` | `""` | — | — | — | Text in the middle; empty is the percent for a value (none for a spinner), and none when the ring is under 40 px. |
| `name` | `string` | `""` | — | — | — | What a screen reader calls it (Loading); empty is Progress. |

```ogt
struct ProgressRingDesc {
  doc "A progress ring or spinner."
  desc true
  field value Option.f32 "Done, 0 to 1; null spins (indeterminate)." {
    unit ratio
  }
  field size f32 "Its diameter; 0 is 48." {
    unit cssPx
    min 0
  }
  field thickness f32 "The ring's width; 0 is a tenth of the size, at least 3." {
    unit cssPx
    min 0
  }
  field tone Tone "The fill's tone; neutral is the primary."
  field label String "Text in the middle; empty is the percent for a value (none for a spinner), and none when the ring is under 40 px."
  field name String "What a screen reader calls it (Loading); empty is Progress."
}
```

<sub>Module `ui` line 5455</sub>
