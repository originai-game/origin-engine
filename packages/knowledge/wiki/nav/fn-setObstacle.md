<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-setObstacle"></a>

### `og.nav.setObstacle`

Set, move or (with no box) remove an obstacle on a navmesh: the spans under the box (grown by the agent radius) are blocked and only the tiles it touches, and their neighbours, are rebuilt. Agents on the mesh re-plan. World state.

```ts
og.nav.setObstacle(world: World, mesh: Navigation, id: number, obstacle?: NavBox): Result<NavRebuild>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The obstacle's id, the game's own (a door's number). |
| `obstacle` | `NavBox` | yes | The box; omitted removes the obstacle. |

**Returns** `Result<NavRebuild>`: Refused for an unknown navmesh.

Binding symbol `og_nav_set_obstacle` · publish builds: kept

```ogt
fn setObstacle {
  doc "Set, move or (with no box) remove an obstacle on a navmesh: the spans under the box (grown by the agent radius) are blocked and only the tiles it touches, and their neighbours, are rebuilt. Agents on the mesh re-plan. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The obstacle's id, the game's own (a door's number)."
  param obstacle NavBox "The box; omitted removes the obstacle." {
    optional true
  }
  returns Result.NavRebuild "Refused for an unknown navmesh."
}
```

<sub>`api.ogt` line 14118</sub>
