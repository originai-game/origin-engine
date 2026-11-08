<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChartDesc"></a>

### `ChartDesc` (desc struct)

A chart: line, area, bar, radar, pie or donut, with axes, a legend, a tooltip and its states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ChartKind` | `"line"` | — | — | — | What it draws. |
| `series` | `readonly ChartSeries[]` | `[]` | — | — | — | The series. |
| `categories` | `readonly string[]` | `[]` | — | — | — | The categories' names: the x axis, a radar's axes, a pie's wedges. |
| `title` | `string` | `""` | — | — | — | A title over it; empty is none. |
| `subtitle` | `string` | `""` | — | — | — | A line under the title. |
| `stacked` | `boolean` | `false` | — | — | — | Bars and areas stack instead of standing side by side. |
| `smooth` | `boolean` | `true` | — | — | — | Lines curve through the values (monotone: never past them). |
| `dots` | `boolean \| null` | `null` | — | — | — | A dot on every value; null is on short series. |
| `min` | `number \| null` | `null` | — | — | — | The value axis's start; null fits the data (with zero for bars and areas). |
| `max` | `number \| null` | `null` | — | — | — | Its end; null fits the data. |
| `format` | `NumberFormat` | `"grouped"` | — | — | — | How values read. |
| `digits` | `number` | `0` | — | — | — | Decimals. |
| `unit` | `string` | `""` | — | — | — | A unit after values in the tooltip. |
| `axis` | `boolean` | `true` | — | — | — | The value and category axes. |
| `grid` | `boolean` | `true` | — | — | — | Grid lines at the value ticks. |
| `legend` | `boolean \| null` | `null` | — | — | — | A legend; null is when there is more than one series (a pie lists its wedges always). |
| `hole` | `number` | `0.62` | 0.2 … 0.9 | `ratio` | — | A donut's hole, as a fraction of its radius. |
| `totalLabel` | `string` | `""` | — | — | — | What a donut's centre says under the total; empty is Total. |
| `selected` | `number \| null` | `null` | — | — | — | The category marked as selected. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `number \| null` | `null` | — | `cssPx` | — | The plot's height; null is 240 (200 on a phone). |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it; its description is made from the data. |
| `loading` | `boolean` | `false` | — | — | — | Loading. |
| `error` | `string` | `""` | — | — | — | An error to show with Retry; empty is none. |
| `emptyLabel` | `string` | `""` | — | — | — | What it says with no data; empty is No data yet. |
| `emptyHint` | `string` | `""` | — | — | — | A line under it. |
| `retry` | `boolean` | `true` | — | — | — | The error has a Retry button. |
| `retryLabel` | `string` | `""` | — | — | — | Retry's label. |
| `loadingLabel` | `string` | `""` | — | — | — | What a screen reader hears while loading. |

```ogt
struct ChartDesc {
  doc "A chart: line, area, bar, radar, pie or donut, with axes, a legend, a tooltip and its states."
  desc true
  field kind ChartKind "What it draws."
  field series List.ChartSeries "The series."
  field categories List.String "The categories' names: the x axis, a radar's axes, a pie's wedges."
  field title String "A title over it; empty is none."
  field subtitle String "A line under the title."
  field stacked bool "Bars and areas stack instead of standing side by side."
  field smooth bool "Lines curve through the values (monotone: never past them)." {
    default true
  }
  field dots Option.bool "A dot on every value; null is on short series."
  field min Option.f32 "The value axis's start; null fits the data (with zero for bars and areas)."
  field max Option.f32 "Its end; null fits the data."
  field format NumberFormat "How values read."
  field digits u32 "Decimals."
  field unit String "A unit after values in the tooltip."
  field axis bool "The value and category axes." {
    default true
  }
  field grid bool "Grid lines at the value ticks." {
    default true
  }
  field legend Option.bool "A legend; null is when there is more than one series (a pie lists its wedges always)."
  field hole f32 "A donut's hole, as a fraction of its radius." {
    default 0.62
    min 0.2
    max 0.9
    unit ratio
  }
  field totalLabel String "What a donut's centre says under the total; empty is Total."
  field selected Option.u32 "The category marked as selected."
  field width Option.Size "Its width; null grows."
  field height Option.f32 "The plot's height; null is 240 (200 on a phone)." {
    unit cssPx
  }
  field label String "What a screen reader calls it; its description is made from the data."
  field loading bool "Loading."
  field error String "An error to show with Retry; empty is none."
  field emptyLabel String "What it says with no data; empty is No data yet."
  field emptyHint String "A line under it."
  field retry bool "The error has a Retry button." {
    default true
  }
  field retryLabel String "Retry's label."
  field loadingLabel String "What a screen reader hears while loading."
}
```

<sub>Module `ui` line 2761</sub>
