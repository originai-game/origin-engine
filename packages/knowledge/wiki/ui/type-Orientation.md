<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Orientation"></a>

### `Orientation` (enum)

A screen orientation.

A string: one of the variants below. The zero value is `"portrait"`.

| Variant | Doc |
|---|---|
| `"portrait"` | Taller than wide. |
| `"landscape"` | Wider than tall. |

```ogt
enum Orientation {
  doc "A screen orientation."
  variant portrait "Taller than wide."
  variant landscape "Wider than tall."
}
```

<sub>Module `ui` line 1495</sub>
