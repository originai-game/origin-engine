<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-compass"></a>

### `og.ui.compass`

A compass strip to the bar of Skyrim and Apex: degree ticks every 15° and the bearing under a centre notch, the cardinal and intercardinal letters, points of interest at their bearings with their distance under them, objectives (a diamond, a waypoint's mark) and pings held at the strip's ends pointing round when behind; the ends fade.

Parts: Compass.strip, Compass.tick[major], Compass.point[cardinal], Compass.notch, Compass.bearing, Compass.poi[<kind>][clamped], Compass.distance. Accessibility: status named the heading in degrees and its point. Probe ids: <id>, <id>/Bearing, <id>/Strip/<key>.

```ts
og.ui.compass(id: string, desc?: CompassDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CompassDesc` | yes | The heading and its points. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn compass {
  doc "A compass strip to the bar of Skyrim and Apex: degree ticks every 15° and the bearing under a centre notch, the cardinal and intercardinal letters, points of interest at their bearings with their distance under them, objectives (a diamond, a waypoint's mark) and pings held at the strip's ends pointing round when behind; the ends fade."
  doc "Parts: Compass.strip, Compass.tick[major], Compass.point[cardinal], Compass.notch, Compass.bearing, Compass.poi[<kind>][clamped], Compass.distance. Accessibility: status named the heading in degrees and its point. Probe ids: <id>, <id>/Bearing, <id>/Strip/<key>."
  host false
  param id String "Its id."
  param desc CompassDesc "The heading and its points."
  returns Void
}
```

<sub>Module `ui` line 6162</sub>
