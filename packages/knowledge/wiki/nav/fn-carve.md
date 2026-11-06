<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-carve"></a>

### `og.nav.carve`

og.nav.setObstacle for any shape: a turned box (a door ajar, a fallen log), a cylinder (a pillar, a barrel) or a convex prism. The cells whose centres the shape covers, grown by the agent radius, are blocked and only the tiles it touches, and their neighbours, are rebuilt. Shares og.nav.setObstacle's ids. World state.

```ts
og.nav.carve(world: World, mesh: Navigation, id: number, shape?: NavShape): Result<NavRebuild>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The obstacle's id. |
| `shape` | `NavShape` | yes | The shape; omitted removes the obstacle. |

**Returns** `Result<NavRebuild>`: Refused for an unknown navmesh or a convex shape of fewer than three points.

Binding symbol `og_nav_carve` · publish builds: kept

```ogt
fn carve {
  doc "og.nav.setObstacle for any shape: a turned box (a door ajar, a fallen log), a cylinder (a pillar, a barrel) or a convex prism. The cells whose centres the shape covers, grown by the agent radius, are blocked and only the tiles it touches, and their neighbours, are rebuilt. Shares og.nav.setObstacle's ids. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The obstacle's id."
  param shape NavShape "The shape; omitted removes the obstacle." {
    optional true
  }
  returns Result.NavRebuild "Refused for an unknown navmesh or a convex shape of fewer than three points."
}
```

<sub>Module `nav` line 594</sub>
