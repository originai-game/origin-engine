<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GraphLink"></a>

### `GraphLink` (desc struct)

A link between two nodes.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `string` | `""` | — | — | — | The node it starts at. |
| `to` | `string` | `""` | — | — | — | The node it ends at. |
| `active` | `boolean` | `false` | — | — | — | Drawn solid in the primary ink (a path taken). |

```ogt
struct GraphLink {
  doc "A link between two nodes."
  desc true
  field from String "The node it starts at."
  field to String "The node it ends at."
  field active bool "Drawn solid in the primary ink (a path taken)."
}
```

<sub>`api.ogt` line 18435</sub>
