<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-waypoint"></a>

### `og.ui.waypoint`

A waypoint or ping to the bar of Apex Legends and Destiny: a diamond with its kind's icon in its colour, the distance under it, a label and who pinged it; off screen or behind it stays on the safe area's edge with an arrow toward its point; a ping drops in with rings rippling from it and leaves after its time; it shrinks a little far away and dims when the crosshair is not near.

Parts: Waypoint.marker[<kind>][clamped], Waypoint.icon, Waypoint.distance, Waypoint.label, Waypoint.ripple. States: group place (onScreen, clamped). Sounds: notify when a ping lands. Accessibility: image named the label, value the distance. Probe ids: <id>, <id>/Marker, <id>/Distance, <id>/Label.

```ts
og.ui.waypoint(id: string, desc?: WaypointDesc): AnchorInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `WaypointDesc` | yes | What it marks. |

**Returns** `AnchorInfo`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn waypoint {
  doc "A waypoint or ping to the bar of Apex Legends and Destiny: a diamond with its kind's icon in its colour, the distance under it, a label and who pinged it; off screen or behind it stays on the safe area's edge with an arrow toward its point; a ping drops in with rings rippling from it and leaves after its time; it shrinks a little far away and dims when the crosshair is not near."
  doc "Parts: Waypoint.marker[<kind>][clamped], Waypoint.icon, Waypoint.distance, Waypoint.label, Waypoint.ripple. States: group place (onScreen, clamped). Sounds: notify when a ping lands. Accessibility: image named the label, value the distance. Probe ids: <id>, <id>/Marker, <id>/Distance, <id>/Label."
  host false
  param id String "Its id."
  param desc WaypointDesc "What it marks."
  returns AnchorInfo
}
```

<sub>Module `ui` line 6632</sub>
