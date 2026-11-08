<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CalendarDesc"></a>

### `CalendarDesc` (desc struct)

A month grid of days: today, a selected day, a range, marks, and days that cannot be chosen.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `year` | `number` | `0` | — | — | — | The year shown; 0 is today's (keep CalendarResult.year). |
| `month` | `number` | `0` | — | — | — | The month shown, 1 to 12; 0 is today's (keep CalendarResult.month). |
| `today` | `string` | `""` | — | — | — | Today, `YYYY-MM-DD` (the game's clock, never read inside the tick). |
| `selected` | `string` | `""` | — | — | — | The selected day. |
| `rangeStart` | `string` | `""` | — | — | — | A range's first day: shaded to rangeEnd. |
| `rangeEnd` | `string` | `""` | — | — | — | A range's last day. |
| `min` | `string` | `""` | — | — | — | The first day that can be chosen; empty is none. |
| `max` | `string` | `""` | — | — | — | The last; empty is none. |
| `disabled` | `readonly string[]` | `[]` | — | — | — | Days that cannot be chosen. |
| `marks` | `readonly CalendarMark[]` | `[]` | — | — | — | Marks on days. |
| `weekStart` | `number` | `1` | ≤ 6 | — | — | The first day of the week, 0 Sunday to 6 Saturday. |
| `monthNames` | `readonly string[]` | `[]` | — | — | — | The months' names, January first; empty is English. |
| `weekdayNames` | `readonly string[]` | `[]` | — | — | — | The weekdays' names, Sunday first; empty is English. |
| `weekdayShort` | `readonly string[]` | `[]` | — | — | — | The weekday heads, Sunday first; empty is the names' first letters. |
| `showOutside` | `boolean` | `true` | — | — | — | The days of the months before and after, faint. |
| `bumpers` | `boolean` | `false` | — | — | — | The tabPrev and tabNext actions turn the month. |
| `cellSize` | `number \| null` | `null` | — | `cssPx` | — | A day's height; null is 44. |
| `todayLabel` | `string` | `""` | — | — | — | What a screen reader adds for today; empty is today. |
| `claimedLabel` | `string` | `""` | — | — | — | What it adds for a claimed day; empty is claimed. |
| `prevLabel` | `string` | `""` | — | — | — | Previous month's accessible name. |
| `nextLabel` | `string` | `""` | — | — | — | Next month's accessible name. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits seven days. |

```ogt
struct CalendarDesc {
  doc "A month grid of days: today, a selected day, a range, marks, and days that cannot be chosen."
  desc true
  field year u32 "The year shown; 0 is today's (keep CalendarResult.year)."
  field month u32 "The month shown, 1 to 12; 0 is today's (keep CalendarResult.month)."
  field today String "Today, `YYYY-MM-DD` (the game's clock, never read inside the tick)."
  field selected String "The selected day."
  field rangeStart String "A range's first day: shaded to rangeEnd."
  field rangeEnd String "A range's last day."
  field min String "The first day that can be chosen; empty is none."
  field max String "The last; empty is none."
  field disabled List.String "Days that cannot be chosen."
  field marks List.CalendarMark "Marks on days."
  field weekStart u32 "The first day of the week, 0 Sunday to 6 Saturday." {
    default 1
    max 6
  }
  field monthNames List.String "The months' names, January first; empty is English."
  field weekdayNames List.String "The weekdays' names, Sunday first; empty is English."
  field weekdayShort List.String "The weekday heads, Sunday first; empty is the names' first letters."
  field showOutside bool "The days of the months before and after, faint." {
    default true
  }
  field bumpers bool "The tabPrev and tabNext actions turn the month."
  field cellSize Option.f32 "A day's height; null is 44." {
    unit cssPx
  }
  field todayLabel String "What a screen reader adds for today; empty is today."
  field claimedLabel String "What it adds for a claimed day; empty is claimed."
  field prevLabel String "Previous month's accessible name."
  field nextLabel String "Next month's accessible name."
  field width Option.Size "Its width; null fits seven days."
}
```

<sub>Module `ui` line 2654</sub>
