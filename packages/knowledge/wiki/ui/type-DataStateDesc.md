<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DataStateDesc"></a>

### `DataStateDesc` (desc struct)

What a data view shows instead of its rows: loading, an error, or nothing.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `loading` | `boolean` | `false` | — | — | — | Loading: skeleton rows. |
| `error` | `string` | `""` | — | — | — | An error to show with Retry; empty is none. |
| `emptyLabel` | `string` | `""` | — | — | — | What it says when there is nothing; empty is Nothing here yet. |
| `emptyHint` | `string` | `""` | — | — | — | A line under it (what to do). |
| `retry` | `boolean` | `true` | — | — | — | The error has a Retry button. |
| `retryLabel` | `string` | `""` | — | — | — | Retry's label; empty is Retry. |
| `loadingLabel` | `string` | `""` | — | — | — | What a screen reader hears while loading; empty is Loading. |

```ogt
struct DataStateDesc {
  doc "What a data view shows instead of its rows: loading, an error, or nothing."
  desc true
  field loading bool "Loading: skeleton rows."
  field error String "An error to show with Retry; empty is none."
  field emptyLabel String "What it says when there is nothing; empty is Nothing here yet."
  field emptyHint String "A line under it (what to do)."
  field retry bool "The error has a Retry button." {
    default true
  }
  field retryLabel String "Retry's label; empty is Retry."
  field loadingLabel String "What a screen reader hears while loading; empty is Loading."
}
```

<sub>Module `ui` line 1557</sub>
