<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadioVariant"></a>

### `RadioVariant` (enum)

How a radio group draws its choices.

A string: one of the variants below. The zero value is `"list"`.

| Variant | Doc |
|---|---|
| `"list"` | A ring and a label per choice (the default). |
| `"cards"` | A framed card per choice, the chosen one ringed (a difficulty, a class). |

```ogt
enum RadioVariant {
  doc "How a radio group draws its choices."
  variant list "A ring and a label per choice (the default)."
  variant cards "A framed card per choice, the chosen one ringed (a difficulty, a class)."
}
```

<sub>`api.ogt` line 19454</sub>
