<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TelegraphShape"></a>

### `TelegraphShape` (enum)

A telegraph's area.

A string: one of the variants below. The zero value is `"circle"`.

| Variant | Doc |
|---|---|
| `"circle"` | A disc of radius. |
| `"ring"` | A ring between inner and radius (stand inside or outside it). |
| `"cone"` | A cone of angle, radius long, toward direction. |
| `"line"` | A rectangle width wide and radius long toward direction (a charge). |

```ogt
enum TelegraphShape {
  doc "A telegraph's area."
  variant circle "A disc of radius."
  variant ring "A ring between inner and radius (stand inside or outside it)."
  variant cone "A cone of angle, radius long, toward direction."
  variant line "A rectangle width wide and radius long toward direction (a charge)."
}
```

<sub>Module `ui` line 6559</sub>
