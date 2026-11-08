<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-chart"></a>

### `og.ui.chart`

A line, area, bar, radar, pie or donut chart.

```ts
og.ui.chart(id: string, desc?: ChartDesc): ChartResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ChartDesc` | yes | The kind, the series and the categories. |

**Returns** `ChartResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn chart {
  doc "A line, area, bar, radar, pie or donut chart."
  host false
  param id String "Its id."
  param desc ChartDesc "The kind, the series and the categories."
  returns ChartResult
}
```

<sub>Module `ui` line 2540</sub>
