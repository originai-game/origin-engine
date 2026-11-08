<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Crumb"></a>

### `Crumb` (desc struct)

A step of a breadcrumb path.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | What it says. |
| `icon` | `string` | `""` | — | — | — | An icon before it; empty is none. |

```ogt
struct Crumb {
  doc "A step of a breadcrumb path."
  desc true
  field label String "What it says."
  field icon String "An icon before it; empty is none."
}
```

<sub>Module `ui` line 2202</sub>
