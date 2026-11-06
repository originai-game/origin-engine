<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SearchResult"></a>

### `SearchResult` (struct)

A search box's state after this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `query` | `string` | `""` | — | — | — | The query, to keep and pass back. |
| `changed` | `boolean` | `false` | — | — | — | The query changed this tick. |
| `submitted` | `boolean` | `false` | — | — | — | Enter was pressed with the query (search for it). |
| `chosen` | `string \| null` | `null` | — | — | — | The suggestion chosen this tick (the query becomes it). |
| `cleared` | `boolean` | `false` | — | — | — | The clear button emptied it. |
| `focused` | `boolean` | `false` | — | — | — | The field or a suggestion has the focus. |

```ogt
struct SearchResult {
  doc "A search box's state after this tick."
  field query String "The query, to keep and pass back."
  field changed bool "The query changed this tick."
  field submitted bool "Enter was pressed with the query (search for it)."
  field chosen Option.String "The suggestion chosen this tick (the query becomes it)."
  field cleared bool "The clear button emptied it."
  field focused bool "The field or a suggestion has the focus."
}
```

<sub>`api.ogt` line 19789</sub>
