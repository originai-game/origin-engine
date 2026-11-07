<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CardVariant"></a>

### `CardVariant` (enum)

How a card stands off what it sits on.

A string: one of the variants below. The zero value is `"raised"`.

| Variant | Doc |
|---|---|
| `"raised"` | A raised panel with the card class's elevation (the default). |
| `"flat"` | A flat panel, no shadow. |
| `"outlined"` | A panel with an outline and no shadow. |
| `"inset"` | A well sunk into the surface. |

```ogt
enum CardVariant {
  doc "How a card stands off what it sits on."
  variant raised "A raised panel with the card class's elevation (the default)."
  variant flat "A flat panel, no shadow."
  variant outlined "A panel with an outline and no shadow."
  variant inset "A well sunk into the surface."
}
```

<sub>Module `ui` line 1914</sub>
