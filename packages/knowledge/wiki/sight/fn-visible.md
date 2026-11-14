<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="fn-visible"></a>

### `og.sight.visible`

Whether a team sees a point this tick: the point's cell of the og.SightMap is within one of the team's og.Sight and not hidden. Use it to hide what stands in the fog (an enemy unit is drawn while `visible` holds for the player's team) and for what an AI may know.

See `world §11`.

Also known as: is visible to team, can see position, in fog of war, is in line of sight, unit visible, is revealed

```ts
og.sight.visible(world: World, team: number, point: Vector3): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `team` | `number` | no | The team (0 to 31). |
| `point` | `Vector3` | no | The point, in world space (its x and z on the ground plane, x and y on the flat plane). |

**Returns** `Result<boolean>`: False outside the map, and in a world without an og.SightMap. Refused for a point that is not finite.

Binding symbol `og_sight_visible` · publish builds: kept

```ogt
fn visible {
  doc "Whether a team sees a point this tick: the point's cell of the og.SightMap is within one of the team's og.Sight and not hidden. Use it to hide what stands in the fog (an enemy unit is drawn while `visible` holds for the player's team) and for what an AI may know."
  aliases "is visible to team, can see position, in fog of war, is in line of sight, unit visible, is revealed"
  guide "world §11"
  param world World "The world."
  param team u32 "The team (0 to 31)."
  param point Vector3 "The point, in world space (its x and z on the ground plane, x and y on the flat plane)."
  returns Result.bool "False outside the map, and in a world without an og.SightMap. Refused for a point that is not finite."
}
```

<sub>Module `sight` line 69</sub>
