<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BreadcrumbsResult"></a>

### `BreadcrumbsResult` (struct)

What breadcrumbs answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `number \| null` | `null` | — | — | — | The crumb pressed (go back to it). |

```ogt
struct BreadcrumbsResult {
  doc "What breadcrumbs answered this tick."
  field chosen Option.u32 "The crumb pressed (go back to it)."
}
```

<sub>Module `ui` line 2248</sub>
