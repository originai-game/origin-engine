<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MapOverlay"></a>

### `MapOverlay` (desc struct)

One of the map's data overlays (Traffic, Power, Land value), with its legend's ramp.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key: its button's element id. |
| `name` | `string` | `""` | — | — | — | Its name, under its art and to a screen reader. |
| `image` | `Texture \| null` | `null` | — | — | — | Its art (art kind itemIcon) on the button; null draws the style's placeholder mark. |
| `ramp` | `readonly string[]` | `[]` | — | — | — | The legend's colours from low to high (`#rrggbb`), as the game draws the overlay on its map. |
| `low` | `string` | `""` | — | — | — | The legend's word at the low end (Free flowing). |
| `high` | `string` | `""` | — | — | — | The legend's word at the high end (Gridlock). |

```ogt
struct MapOverlay {
  doc "One of the map's data overlays (Traffic, Power, Land value), with its legend's ramp."
  desc true
  field key String "Its key: its button's element id."
  field name String "Its name, under its art and to a screen reader."
  field image Option.Texture "Its art (art kind itemIcon) on the button; null draws the style's placeholder mark."
  field ramp List.String "The legend's colours from low to high (`#rrggbb`), as the game draws the overlay on its map."
  field low String "The legend's word at the low end (Free flowing)."
  field high String "The legend's word at the high end (Gridlock)."
}
```

<sub>Module `ui` line 3472</sub>
