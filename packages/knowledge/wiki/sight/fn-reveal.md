<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="fn-reveal"></a>

### `og.sight.reveal`

Mark the ground within a radius of a point as explored by a team without anyone seeing it: a map bought, a scout's report, the start area. Lasts until og.sight.forget.

Also known as: reveal map, uncover fog, explore area, show map, remove fog of war

```ts
og.sight.reveal(world: World, team: number, point: Vector3, radius: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `team` | `number` | no | The team (0 to 31). |
| `point` | `Vector3` | no | The centre, in world space. |
| `radius` | `number` | no | How far around it, in metres. |

**Returns** `Result<void>`: Refused for a world without an og.SightMap, a team over 31, or a point or radius that is not finite.

Binding symbol `og_sight_reveal` · publish builds: kept

```ogt
fn reveal {
  doc "Mark the ground within a radius of a point as explored by a team without anyone seeing it: a map bought, a scout's report, the start area. Lasts until og.sight.forget."
  aliases "reveal map, uncover fog, explore area, show map, remove fog of war"
  param world World "The world."
  param team u32 "The team (0 to 31)."
  param point Vector3 "The centre, in world space."
  param radius f32 "How far around it, in metres."
  returns Result.Void "Refused for a world without an og.SightMap, a team over 31, or a point or radius that is not finite."
}
```

<sub>Module `sight` line 88</sub>
