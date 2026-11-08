<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CalendarMark"></a>

### `CalendarMark` (desc struct)

Something on a day: a dot in a tone, or a claimed tick for a login calendar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `date` | `string` | `""` | — | — | — | The day, `YYYY-MM-DD`. |
| `kind` | `string` | `""` | — | — | — | `claimed` draws a tick; anything else a dot. |
| `tone` | `string` | `""` | — | — | — | The dot's tone: primary, success, warn, danger, info, gold, neutral. |
| `label` | `string` | `""` | — | — | — | What it is, for a screen reader (`Double rewards`). |

```ogt
struct CalendarMark {
  doc "Something on a day: a dot in a tone, or a claimed tick for a login calendar."
  desc true
  field date String "The day, `YYYY-MM-DD`."
  field kind String "`claimed` draws a tick; anything else a dot."
  field tone String "The dot's tone: primary, success, warn, danger, info, gold, neutral."
  field label String "What it is, for a screen reader (`Double rewards`)."
}
```

<sub>Module `ui` line 2341</sub>
