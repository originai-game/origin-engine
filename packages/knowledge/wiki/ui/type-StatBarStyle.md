<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatBarStyle"></a>

### `StatBarStyle` (enum)

A stat bar's shape.

A string: one of the variants below. The zero value is `"flat"`.

| Variant | Doc |
|---|---|
| `"flat"` | A rounded track (the default). |
| `"slanted"` | Ends cut on a slant (sci-fi, fighting games). |
| `"pill"` | Fully round ends. |
| `"pips"` | Discrete pips with a gap between them (a ship's shield cells, upgrade levels): one per segment (one per unit of max when segment is 0, at most 40), lit whole as the value reaches each, sharing the bar's length equally so bars of one width and count line up row over row. |

```ogt
enum StatBarStyle {
  doc "A stat bar's shape."
  variant flat "A rounded track (the default)."
  variant slanted "Ends cut on a slant (sci-fi, fighting games)."
  variant pill "Fully round ends."
  variant pips "Discrete pips with a gap between them (a ship's shield cells, upgrade levels): one per segment (one per unit of max when segment is 0, at most 40), lit whole as the value reaches each, sharing the bar's length equally so bars of one width and count line up row over row."
}
```

<sub>Module `ui` line 5781</sub>
