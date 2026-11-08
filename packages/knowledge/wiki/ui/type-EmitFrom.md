<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-EmitFrom"></a>

### `EmitFrom` (enum)

Where on its element a particle starts.

A string: one of the variants below. The zero value is `"center"`.

| Variant | Doc |
|---|---|
| `"center"` | Its centre. |
| `"area"` | Anywhere inside its box. |
| `"edges"` | On its edges. |

```ogt
enum EmitFrom {
  doc "Where on its element a particle starts."
  variant center "Its centre."
  variant area "Anywhere inside its box."
  variant edges "On its edges."
}
```

<sub>Module `ui` line 7639</sub>
