<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GridFlow"></a>

### `GridFlow` (enum)

Which way auto-placed items fill a grid.

A string: one of the variants below. The zero value is `"row"`.

| Variant | Doc |
|---|---|
| `"row"` | Along rows, adding rows (the default). |
| `"column"` | Down columns, adding columns. |

```ogt
enum GridFlow {
  doc "Which way auto-placed items fill a grid."
  variant row "Along rows, adding rows (the default)."
  variant column "Down columns, adding columns."
}
```

<sub>Module `ui` line 4232</sub>
