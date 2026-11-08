<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-VectorFill"></a>

### `VectorFill` (desc struct)

A cooked vector contained and centred in the box.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vector` | `VectorArt` | `0` | — | — | — | The vector (og.load.vector). |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies its paints; null is white. |

```ogt
struct VectorFill {
  doc "A cooked vector contained and centred in the box."
  desc true
  field vector VectorArt "The vector (og.load.vector)."
  field tint Option.Paint "Multiplies its paints; null is white."
}
```

<sub>Module `ui` line 7018</sub>
