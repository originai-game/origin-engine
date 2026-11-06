<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-Grass"></a>

### `og.Grass` (component)

A field of grass over a loaded terrain where one of its layers shows, generated on the GPU every frame out to `distance` (a meadow with every other field at its default). Blades are curved strips placed on a blue-noise pattern fixed to the terrain, so each keeps its place as the camera moves; they grow where the layer's weight is (its control-map channel times its slope and altitude rules), never over holes, in clumps that share their tint and height. With distance they thin out, the ones left widening to keep the ground covered, and take fewer vertices (15, 7, then 3); under and past them the terrain takes the field's average colour (`farTint`), so the far field reads as grass to the horizon with no seam. og.Wind bends them in gusts that roll across the field, each blade swaying on its own; lit like the terrain under them, with light through blades lit from behind (`translucency`), a sheen on the upper blade and the roots shaded by their neighbours. Visual only: drawn from the tick, nothing in the simulation reads it, the same tick draws the same blades on every run. It grows over the og.Terrain drawing the same data (a visible one).

Token `og.Grass`, registered under the schema name `og.Grass`; its value type is `Grass`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `data` | `TerrainData` | `0` | — | — | — | The terrain (og.terrain.load's TerrainInfo.data). |
| `layer` | `number` | `0` | ≤ 15 | — | — | The terrain layer that places it, 0 to 15. |
| `density` | `number` | `300` | 0 … 4000 | — | tunable | Blades per square metre near the camera where the layer's weight is 1. |
| `distance` | `number` | `150` | 0 … 1000 | `m` | tunable | How far from the camera blades are drawn; they thin out to nothing there. |
| `farTint` | `boolean` | `true` | — | — | — | The terrain takes the field's average colour under its blades, where they thin out and past them, so the gaps and the far field read as the same grass. |
| `height` | `number` | `0.45` | 0 … 4 | `m` | tunable | The tallest blade. |
| `heightVariation` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | How much shorter blades are than the tallest: 0 all alike, 1 down to nothing. |
| `width` | `number` | `0.012` | 0 … 0.5 | `m` | tunable | A blade's width at its root. |
| `curve` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How far blades droop along their length: 0 straight, 1 arching over. |
| `lean` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How far blades lean at rest, as a share of their height. |
| `clumpSize` | `number` | `1.2` | 0.05 … 20 | `m` | tunable | The size of the clumps blades grow in, sharing their tint and height. |
| `clumpLean` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | How far blades lean out from their clump's centre, as a share of their height. |
| `tufts` | `number` | `0` | 0 … 1 | `ratio` | tunable | How far blades are pulled in towards their clump's centre: 0 an even lawn, 1 tight tufts. |
| `baseColor` | `Color` | `{ r: 0.04, g: 0.085, b: 0.024 }` | — | — | — | The roots' colour (linear). |
| `tipColor` | `Color` | `{ r: 0.235, g: 0.39, b: 0.11 }` | — | — | — | The tips' colour (linear). |
| `dryColor` | `Color` | `{ r: 0.45, g: 0.38, b: 0.17 }` | — | — | — | Dry grass's colour (linear): dry clumps and drying tips. |
| `colorVariation` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How much blades vary in brightness by clump, blade and patch. |
| `dryness` | `number` | `0.15` | 0 … 1 | `ratio` | tunable | The share of clumps that are dry, and how far tips dry. |
| `translucency` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | How much sunlight passes through a blade lit from behind, tinted by it. |
| `sheen` | `number` | `0.4` | 0 … 1 | `ratio` | tunable | The upper blade's gloss: gusts show as a moving sheen. |
| `roughness` | `number` | `0.6` | 0.04 … 1 | — | tunable | The blade's roughness. |
| `sway` | `number` | `1` | 0 … 2 | — | tunable | How much og.Wind bends the blades: 1 as it blows, 0 still. |
| `stiffness` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How stiff the blades are: 0 bends furthest, 1 least, and sways faster. |
| `flutter` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How much the tips of near blades shiver in the wind. |
| `castShadows` | `boolean` | `true` | — | — | — | Whether the blades cast the sun's shadows (near the camera, into the first two cascades). Either way a blade is shaded by the field's own canopy between it and the sun, darker towards the roots the denser the field and the lower the sun. |
| `seed` | `number` | `1` | — | — | — | Another seed places the blades elsewhere. |
| `recovery` | `number` | `4` | 0 … 120 | `s` | tunable | How long blades pressed by an og.GrassPusher take to stand again after it leaves. |
| `types` | `readonly Entity[]` | `[]` | — | — | a list of at most 3 `Entity` (`List.Entity.3`): reads a frozen copy, writes a whole array | Up to three more kinds of blade growing in the field among its own (a meadow of grass, taller reeds and flowers): entities carrying an og.GrassType. The field's own height, width, curve and colours are its first kind. Each clump grows one kind, chosen by `typeMap` or, without one, by the kinds' `share`; 0s and entities without an og.GrassType are skipped. |
| `typeMap` | `Texture` | `0` | — | — | — | Where each of `types` grows, over the terrain's extent (its top row along the terrain's -z edge, as the control map; loaded linear): red, green and blue the share of the first, second and third of `types` at each point, the rest the field's own kind. 0: every kind everywhere, by its share. |
| `densityMap` | `Texture` | `0` | — | — | — | Multiplies the blades' density over the terrain's extent (its red channel, laid out as typeMap; loaded linear): black grows none, a path through the meadow. 0: none. |
| `heightMap` | `Texture` | `0` | — | — | — | Multiplies the blades' height over the terrain's extent (its red channel, laid out as typeMap; loaded linear): short grass by a path, tall in the hollows. 0: none. |

```ogt
component Grass {
  doc "A field of grass over a loaded terrain where one of its layers shows, generated on the GPU every frame out to `distance` (a meadow with every other field at its default). Blades are curved strips placed on a blue-noise pattern fixed to the terrain, so each keeps its place as the camera moves; they grow where the layer's weight is (its control-map channel times its slope and altitude rules), never over holes, in clumps that share their tint and height. With distance they thin out, the ones left widening to keep the ground covered, and take fewer vertices (15, 7, then 3); under and past them the terrain takes the field's average colour (`farTint`), so the far field reads as grass to the horizon with no seam. og.Wind bends them in gusts that roll across the field, each blade swaying on its own; lit like the terrain under them, with light through blades lit from behind (`translucency`), a sheen on the upper blade and the roots shaded by their neighbours. Visual only: drawn from the tick, nothing in the simulation reads it, the same tick draws the same blades on every run. It grows over the og.Terrain drawing the same data (a visible one)."
  field data TerrainData "The terrain (og.terrain.load's TerrainInfo.data)."
  field layer u32 "The terrain layer that places it, 0 to 15." {
    max 15
  }
  field density f32 "Blades per square metre near the camera where the layer's weight is 1." {
    default 300
    min 0
    max 4000
    tunable true
  }
  field distance f32 "How far from the camera blades are drawn; they thin out to nothing there." {
    default 150
    min 0
    max 1000
    unit m
    tunable true
  }
  field farTint bool "The terrain takes the field's average colour under its blades, where they thin out and past them, so the gaps and the far field read as the same grass." {
    default true
  }
  field height f32 "The tallest blade." {
    default 0.45
    min 0
    max 4
    unit m
    tunable true
  }
  field heightVariation f32 "How much shorter blades are than the tallest: 0 all alike, 1 down to nothing." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field width f32 "A blade's width at its root." {
    default 0.012
    min 0
    max 0.5
    unit m
    tunable true
  }
  field curve f32 "How far blades droop along their length: 0 straight, 1 arching over." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field lean f32 "How far blades lean at rest, as a share of their height." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field clumpSize f32 "The size of the clumps blades grow in, sharing their tint and height." {
    default 1.2
    min 0.05
    max 20
    unit m
    tunable true
  }
  field clumpLean f32 "How far blades lean out from their clump's centre, as a share of their height." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field tufts f32 "How far blades are pulled in towards their clump's centre: 0 an even lawn, 1 tight tufts." {
    min 0
    max 1
    unit ratio
    tunable true
  }
  field baseColor Color "The roots' colour (linear)." {
    default 0.04 0.085 0.024
  }
  field tipColor Color "The tips' colour (linear)." {
    default 0.235 0.39 0.11
  }
  field dryColor Color "Dry grass's colour (linear): dry clumps and drying tips." {
    default 0.45 0.38 0.17
  }
  field colorVariation f32 "How much blades vary in brightness by clump, blade and patch." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field dryness f32 "The share of clumps that are dry, and how far tips dry." {
    default 0.15
    min 0
    max 1
    unit ratio
    tunable true
  }
  field translucency f32 "How much sunlight passes through a blade lit from behind, tinted by it." {
    default 0.6
    min 0
    max 1
    unit ratio
    tunable true
  }
  field sheen f32 "The upper blade's gloss: gusts show as a moving sheen." {
    default 0.4
    min 0
    max 1
    unit ratio
    tunable true
  }
  field roughness f32 "The blade's roughness." {
    default 0.6
    min 0.04
    max 1
    tunable true
  }
  field sway f32 "How much og.Wind bends the blades: 1 as it blows, 0 still." {
    default 1
    min 0
    max 2
    tunable true
  }
  field stiffness f32 "How stiff the blades are: 0 bends furthest, 1 least, and sways faster." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field flutter f32 "How much the tips of near blades shiver in the wind." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field castShadows bool "Whether the blades cast the sun's shadows (near the camera, into the first two cascades). Either way a blade is shaded by the field's own canopy between it and the sun, darker towards the roots the denser the field and the lower the sun." {
    default true
  }
  field seed u32 "Another seed places the blades elsewhere." {
    default 1
  }
  field recovery f32 "How long blades pressed by an og.GrassPusher take to stand again after it leaves." {
    default 4
    min 0
    max 120
    unit s
    tunable true
  }
  field types List.Entity.3 "Up to three more kinds of blade growing in the field among its own (a meadow of grass, taller reeds and flowers): entities carrying an og.GrassType. The field's own height, width, curve and colours are its first kind. Each clump grows one kind, chosen by `typeMap` or, without one, by the kinds' `share`; 0s and entities without an og.GrassType are skipped."
  field typeMap Texture "Where each of `types` grows, over the terrain's extent (its top row along the terrain's -z edge, as the control map; loaded linear): red, green and blue the share of the first, second and third of `types` at each point, the rest the field's own kind. 0: every kind everywhere, by its share."
  field densityMap Texture "Multiplies the blades' density over the terrain's extent (its red channel, laid out as typeMap; loaded linear): black grows none, a path through the meadow. 0: none."
  field heightMap Texture "Multiplies the blades' height over the terrain's extent (its red channel, laid out as typeMap; loaded linear): short grass by a path, tall in the hollows. 0: none."
}
```

<sub>`api.ogt` line 24973</sub>
