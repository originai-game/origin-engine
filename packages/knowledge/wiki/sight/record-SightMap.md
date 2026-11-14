<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="record-SightMap"></a>

### `og.SightMap` (component)

Fog of war over a rectangle of the ground: the og.Volume on its entity gives its extent (its box's x and z, or x and y on the flat plane; none: no map), cut into cells. Each tick every og.Sight sees the cells within its radius that no og.SightBlocker hides, and what a team has seen stays explored. The first og.SightMap in entity order is the world's; changing its cell or extent's size starts it again unexplored. Drawn for `team` over the lit frame: seen ground as it is, explored ground dimmed, the rest covered; nothing outside the map is covered.

See `world §11`.

Also known as: fog of war, FOW, unexplored map, shroud, line of sight map, vision map, reveal the map, RTS fog, roguelike field of view, FOV, minimap fog, dark unexplored area

Token `og.SightMap`, registered under the schema name `og.SightMap`; its value type is `SightMap`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cell` | `number` | `1` | ≥ 0.05 | `m` | — | A cell's side. A map has at most 512 cells a side: a larger extent takes a larger cell. |
| `plane` | `SightPlane` | `"ground"` | — | — | — | The plane it lies in. |
| `team` | `number` | `0` | ≤ 31 | — | — | The team whose sight is drawn (the player's). |
| `color` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | What covers ground the team has never seen (linear, before exposure). |
| `explored` | `number` | `0.35` | 0 … 1 | — | tunable | How much of the scene shows on ground the team has seen before and does not see now: 0 covered like the unseen, 1 as if seen. |
| `drawn` | `boolean` | `true` | — | — | — | Whether the fog is drawn. False keeps the map (og.sight still answers) and shows the whole frame: a spectator, a debug view. |

```ogt
component SightMap {
  aliases "fog of war, FOW, unexplored map, shroud, line of sight map, vision map, reveal the map, RTS fog, roguelike field of view, FOV, minimap fog, dark unexplored area"
  doc "Fog of war over a rectangle of the ground: the og.Volume on its entity gives its extent (its box's x and z, or x and y on the flat plane; none: no map), cut into cells. Each tick every og.Sight sees the cells within its radius that no og.SightBlocker hides, and what a team has seen stays explored. The first og.SightMap in entity order is the world's; changing its cell or extent's size starts it again unexplored. Drawn for `team` over the lit frame: seen ground as it is, explored ground dimmed, the rest covered; nothing outside the map is covered."
  guide "world §11"
  field cell f32 "A cell's side. A map has at most 512 cells a side: a larger extent takes a larger cell." {
    default 1
    min 0.05
    unit m
  }
  field plane SightPlane "The plane it lies in."
  field team u32 "The team whose sight is drawn (the player's)." {
    max 31
  }
  field color Color "What covers ground the team has never seen (linear, before exposure)." {
    default 0 0 0
  }
  field explored f32 "How much of the scene shows on ground the team has seen before and does not see now: 0 covered like the unseen, 1 as if seen." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field drawn bool "Whether the fog is drawn. False keeps the map (og.sight still answers) and shows the whole frame: a spectator, a debug view." {
    default true
  }
}
```

<sub>Module `sight` line 18</sub>
