<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MinimapViewDesc"></a>

### `MinimapViewDesc` (desc struct)

A minimap.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The map; null draws the style's well with a grid. |
| `centerX` | `number` | `0.5` | — | `ratio` | — | The map point at its centre (the player's), 0 to 1. |
| `centerY` | `number` | `0.5` | — | `ratio` | — | The map point at its centre. |
| `zoom` | `number` | `0.3` | — | `ratio` | — | How much of the map it shows across, 0 to 1. |
| `rotate` | `boolean \| null` | `null` | — | — | — | The map turns with heading so up is ahead (player-up); false keeps north up and turns the player's arrow. Null is player-up for a racing track (track given) and north-up otherwise. |
| `heading` | `number` | `0` | — | `rad` | — | The player's heading, clockwise from north. |
| `shape` | `MapShape` | `"circle"` | — | — | — | Its outline. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its side; 0 is 200 (144 on a phone). |
| `markers` | `readonly MapMarkerItem[]` | `[]` | — | — | — | Its markers. |
| `zone` | `string` | `""` | — | — | — | A label under it (the zone's name); empty is none. |
| `track` | `readonly Vector2[]` | `[]` | — | — | — | A racing circuit: its centre line as points in map coordinates (0 to 1, as markers), drawn as a smooth line (through every point) over the style's well in place of the texture and the grid. Give zoom 1 and the centre 0.5, 0.5 to show the whole circuit; three points or more. |
| `trackOpen` | `boolean` | `false` | — | — | — | The track does not close on itself (a point-to-point stage). |
| `sectors` | `readonly number[]` | `[]` | — | `ratio` | — | Sector lines across the track, each a fraction of the way along it (by its points) from the start, which is drawn as a checker line. |
| `pitch` | `boolean` | `false` | — | — | — | A sports radar: a pitch drawn under the markers in place of the grid (with no texture), as hairlines at about 20 % white: the touchlines, the halfway line, the centre circle and both penalty boxes, the field running left to right over the map's coordinates 0.04 to 0.96 (give zoom 1 and the centre 0.5, 0.5 to show it whole). |
| `north` | `string` | `"N"` | — | — | — | The north letter. |
| `anchor` | `Anchor \| null` | `"topRight"` | — | — | — | Opens its own anchored group there; null builds it in place. |

```ogt
struct MinimapViewDesc {
  doc "A minimap."
  desc true
  field texture Option.Texture "The map; null draws the style's well with a grid."
  field centerX f32 "The map point at its centre (the player's), 0 to 1." {
    default 0.5
    unit ratio
  }
  field centerY f32 "The map point at its centre." {
    default 0.5
    unit ratio
  }
  field zoom f32 "How much of the map it shows across, 0 to 1." {
    default 0.3
    unit ratio
  }
  field rotate Option.bool "The map turns with heading so up is ahead (player-up); false keeps north up and turns the player's arrow. Null is player-up for a racing track (track given) and north-up otherwise."
  field heading f32 "The player's heading, clockwise from north." {
    unit rad
  }
  field shape MapShape "Its outline."
  field size f32 "Its side; 0 is 200 (144 on a phone)." {
    min 0
    unit cssPx
  }
  field markers List.MapMarkerItem "Its markers."
  field zone String "A label under it (the zone's name); empty is none."
  field track List.Vector2 "A racing circuit: its centre line as points in map coordinates (0 to 1, as markers), drawn as a smooth line (through every point) over the style's well in place of the texture and the grid. Give zoom 1 and the centre 0.5, 0.5 to show the whole circuit; three points or more."
  field trackOpen bool "The track does not close on itself (a point-to-point stage)."
  field sectors List.f32 "Sector lines across the track, each a fraction of the way along it (by its points) from the start, which is drawn as a checker line." {
    unit ratio
  }
  field pitch bool "A sports radar: a pitch drawn under the markers in place of the grid (with no texture), as hairlines at about 20 % white: the touchlines, the halfway line, the centre circle and both penalty boxes, the field running left to right over the map's coordinates 0.04 to 0.96 (give zoom 1 and the centre 0.5, 0.5 to show it whole)."
  field north String "The north letter." {
    default "N"
  }
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place." {
    default topRight
  }
}
```

<sub>`api.ogt` line 21967</sub>
