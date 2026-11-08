<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-minimapView"></a>

### `og.ui.minimapView`

A minimap to the bar of MOBAs, shooters and open worlds: the map in a round or square frame, centred on the player and zoomed, fixed with north up and the player's arrow turning or turning with the player; markers for allies (their arrows), enemies, loot by rarity, points of interest, objectives and pings, which stay on the frame's edge pointing out when out of range; pings pulse; a north mark on the rim, the zone under it; a press answers clicked (click to move reads the press's position against the map's bounds).

Parts: Minimap.bezel (a dark rounded bezel with a soft drop), Minimap.frame[<shape>] (the map's glass inside it), Minimap.map, Minimap.pitch, Minimap.track, Minimap.start, Minimap.sector, Minimap.marker[<kind>][clamped], Minimap.ring, Minimap.north (straddling the bezel's top edge, over nothing on the map), Minimap.plate (a square map's caption plate at the bezel's foot; a round map's caption is the same plate as a pill centred under the ring), Minimap.ping. Slots: texture (art kind map). Accessibility: image named by the zone; markers images named by their labels. Probe ids: <id>, <id>/Bezel/Map/<key>, <id>/Bezel/Plate/Zone (square) or <id>/ZonePlate/Zone (round).

```ts
og.ui.minimapView(id: string, desc?: MinimapViewDesc): MinimapResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `MinimapViewDesc` | yes | The map and its markers. |

**Returns** `MinimapResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn minimapView {
  doc "A minimap to the bar of MOBAs, shooters and open worlds: the map in a round or square frame, centred on the player and zoomed, fixed with north up and the player's arrow turning or turning with the player; markers for allies (their arrows), enemies, loot by rarity, points of interest, objectives and pings, which stay on the frame's edge pointing out when out of range; pings pulse; a north mark on the rim, the zone under it; a press answers clicked (click to move reads the press's position against the map's bounds)."
  doc "Parts: Minimap.bezel (a dark rounded bezel with a soft drop), Minimap.frame[<shape>] (the map's glass inside it), Minimap.map, Minimap.pitch, Minimap.track, Minimap.start, Minimap.sector, Minimap.marker[<kind>][clamped], Minimap.ring, Minimap.north (straddling the bezel's top edge, over nothing on the map), Minimap.plate (a square map's caption plate at the bezel's foot; a round map's caption is the same plate as a pill centred under the ring), Minimap.ping. Slots: texture (art kind map). Accessibility: image named by the zone; markers images named by their labels. Probe ids: <id>, <id>/Bezel/Map/<key>, <id>/Bezel/Plate/Zone (square) or <id>/ZonePlate/Zone (round)."
  host false
  param id String "Its id."
  param desc MinimapViewDesc "The map and its markers."
  returns MinimapResult
}
```

<sub>Module `ui` line 6164</sub>
