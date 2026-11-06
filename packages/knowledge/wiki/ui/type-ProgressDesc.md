<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ProgressDesc"></a>

### `ProgressDesc` (desc struct)

A progress bar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows into the free space. |

```ogt
struct ProgressDesc {
  doc "A progress bar."
  desc true
  field width Option.Size "Its width; null grows into the free space."
}
```

<sub>`api.ogt` line 15979</sub>
