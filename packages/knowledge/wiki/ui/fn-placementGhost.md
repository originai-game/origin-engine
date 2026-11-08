<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-placementGhost"></a>

### `og.ui.placementGhost`

A placement ghost to the bar of Age of Empires IV, Anno and Cities: Skylines: the footprint on the ground (draped on the surface with desc.ground) in success or danger (hatched when invalid, so it is not told by colour alone), its grid cells, a range ring for a tower's reach, and its cost or the reason it cannot go there; it breathes while held.

Parts: PlacementGhost.footprint[valid|invalid], PlacementGhost.cells, PlacementGhost.range, PlacementGhost.label. States: group place (valid, invalid). Accessibility: status named the label. Probe ids: <id>, <id>Label (the label, a tag of its own: it declutters with the other tags and stays in the safe area).

```ts
og.ui.placementGhost(world: World, id: string, desc?: PlacementGhostDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads. |
| `id` | `string` | no | Its id. |
| `desc` | `PlacementGhostDesc` | yes | The footprint. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn placementGhost {
  doc "A placement ghost to the bar of Age of Empires IV, Anno and Cities: Skylines: the footprint on the ground (draped on the surface with desc.ground) in success or danger (hatched when invalid, so it is not told by colour alone), its grid cells, a range ring for a tower's reach, and its cost or the reason it cannot go there; it breathes while held."
  doc "Parts: PlacementGhost.footprint[valid|invalid], PlacementGhost.cells, PlacementGhost.range, PlacementGhost.label. States: group place (valid, invalid). Accessibility: status named the label. Probe ids: <id>, <id>Label (the label, a tag of its own: it declutters with the other tags and stays in the safe area)."
  host false
  param world World "The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads."
  param id String "Its id."
  param desc PlacementGhostDesc "The footprint."
  returns Void
}
```

<sub>Module `ui` line 6723</sub>
