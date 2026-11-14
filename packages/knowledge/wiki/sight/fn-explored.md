<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="fn-explored"></a>

### `og.sight.explored`

Whether a team has ever seen a point (or had it revealed): its cell of the og.SightMap is explored.

Also known as: has seen, was explored, discovered area, map revealed at

```ts
og.sight.explored(world: World, team: number, point: Vector3): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `team` | `number` | no | The team (0 to 31). |
| `point` | `Vector3` | no | The point, in world space. |

**Returns** `Result<boolean>`: False outside the map, and in a world without an og.SightMap. Refused for a point that is not finite.

Binding symbol `og_sight_explored` · publish builds: kept

```ogt
fn explored {
  doc "Whether a team has ever seen a point (or had it revealed): its cell of the og.SightMap is explored."
  aliases "has seen, was explored, discovered area, map revealed at"
  param world World "The world."
  param team u32 "The team (0 to 31)."
  param point Vector3 "The point, in world space."
  returns Result.bool "False outside the map, and in a world without an og.SightMap. Refused for a point that is not finite."
}
```

<sub>Module `sight` line 79</sub>
