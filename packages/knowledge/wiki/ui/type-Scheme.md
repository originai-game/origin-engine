<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Scheme"></a>

### `Scheme` (enum)

The palette: games run on a stage, so dark is the default.

A string: one of the variants below. The zero value is `"dark"`.

| Variant | Doc |
|---|---|
| `"dark"` | The dark palette. |
| `"light"` | The light palette. |

```ogt
enum Scheme {
  doc "The palette: games run on a stage, so dark is the default."
  variant dark "The dark palette."
  variant light "The light palette."
}
```

<sub>`api.ogt` line 15707</sub>
