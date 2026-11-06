<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ListRange"></a>

### `ListRange` (struct)

The rows (or cells) to build this tick: first up to, not including, last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `first` | `number` | `0` | — | — | — | The first index. |
| `last` | `number` | `0` | — | — | — | One past the last index. |

```ogt
struct ListRange {
  doc "The rows (or cells) to build this tick: first up to, not including, last."
  field first u32 "The first index."
  field last u32 "One past the last index."
}
```

<sub>Module `ui` line 1006</sub>
