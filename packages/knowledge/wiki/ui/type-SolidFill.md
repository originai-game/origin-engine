<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SolidFill"></a>

### `SolidFill` (desc struct)

A colour.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Paint` | `{}` | — | — | — | The colour: a theme token, resolved when the frame is drawn, or a literal. |

```ogt
struct SolidFill {
  doc "A colour."
  desc true
  field color Paint "The colour: a theme token, resolved when the frame is drawn, or a literal."
}
```

<sub>Module `ui` line 3001</sub>
