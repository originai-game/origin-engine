<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MapMarkerItem"></a>

### `MapMarkerItem` (desc struct)

A marker on a minimap.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key (its probe id). |
| `kind` | `MarkerKind` | `"player"` | — | — | — | What it is. |
| `x` | `number` | `0` | — | `ratio` | — | Where across the map, 0 to 1. |
| `y` | `number` | `0` | — | `ratio` | — | Where down the map, 0 to 1. |
| `heading` | `number` | `0` | — | `rad` | — | Its facing, clockwise from north (players and allies). |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas (a poi's). |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the kind's. |
| `rarity` | `Rarity` | `"none"` | — | — | — | Loot's tier. |
| `label` | `string` | `""` | — | — | — | What a screen reader says of it. |
| `ringed` | `boolean` | `false` | — | — | — | A ring round it in its colour: the rivals just ahead of and behind the player. |

```ogt
struct MapMarkerItem {
  doc "A marker on a minimap."
  desc true
  field key String "Its key (its probe id)."
  field kind MarkerKind "What it is."
  field x f32 "Where across the map, 0 to 1." {
    unit ratio
  }
  field y f32 "Where down the map, 0 to 1." {
    unit ratio
  }
  field heading f32 "Its facing, clockwise from north (players and allies)." {
    unit rad
  }
  field icon String "An icon of the atlas (a poi's)."
  field color Option.Paint "Its colour; null is the kind's."
  field rarity Rarity "Loot's tier."
  field label String "What a screen reader says of it."
  field ringed bool "A ring round it in its colour: the rivals just ahead of and behind the player."
}
```

<sub>Module `ui` line 6091</sub>
