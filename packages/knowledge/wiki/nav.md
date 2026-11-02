<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `nav` module

Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch.

Agents are entities with an og.NavAgent (and an og.Transform): the og.nav system, after the game's systems and before physics, plans their paths, follows them and adds the og.Steering behaviours; an entity with an og.Velocity (a physics body) gets its velocity written, any other its Transform moved. Everything is deterministic and in snapshots: paths, grid edits, obstacles and flow targets.

Agents move on the profile's plane: x and y in 2D (a grid's rows run up), x and z in 3D (y kept on grids, from the surface on a navmesh).

- **TypeScript**: functions are the namespace `og.nav`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-ai`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module nav {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 11981</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Navigation`](#type-Navigation) | handle | A grid, a navmesh or a flow field agents plan on. |
| [`Navigation()`](#column-Navigation) | column kind | A Navigation handle field. |
| [`NAV_IDLE`](#const-NAV_IDLE) | constant | NavAgent.status: not seeking. |
| [`NAV_MOVING`](#const-NAV_MOVING) | constant | NavAgent.status: following a path or a flow field. |
| [`NAV_ARRIVED`](#const-NAV_ARRIVED) | constant | NavAgent.status: within stopDistance of the target. |
| [`NAV_UNREACHABLE`](#const-NAV_UNREACHABLE) | constant | NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh. |
| [`NAV_PARTIAL`](#const-NAV_PARTIAL) | constant | NavAgent.status: the target is cut off; moving to the reachable point nearest it. |
| [`NAV_ON_LINK`](#const-NAV_ON_LINK) | constant | NavAgent.status: crossing an off-mesh link (a jump, a ladder, a door, a teleport). |
| [`NavAlgorithm`](#type-NavAlgorithm) | enum | The grid search a path query runs. |
| [`og.NavAgent`](#record-NavAgent) | component | An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents. |
| [`og.Steering`](#record-Steering) | component | Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays). |
| [`og.NavFilter`](#record-NavFilter) | component | Which navmesh areas an agent crosses and what a metre in each costs: beside an og.NavAgent, its plans weigh every polygon's length by its area's cost and never enter an excluded area. Areas are marked at cook time (`area` in a navmesh asset) or at run time (og.nav.setArea); 0 is ground. Without one every area costs 1. |
| [`og.Crowd`](#record-Crowd) | component | Reciprocal crowd avoidance (ORCA) for an agent: beside an og.NavAgent, each tick its velocity is the one nearest what its path and steering want that keeps it clear of its nearest agents for `horizon` seconds and of the navmesh's walls for `obstacleHorizon`, each pair of crowd agents sharing the avoidance. Every agent solves from the same start-of-tick state in a fixed order, so crowds replay identically on every host; agents meeting perfectly symmetrically (a ring swapping places) are kept from locking by a margin of up to 5% of the radius picked from the entity, so a game adds no jitter of its own. Without one an agent moves as before (og.Steering's separation only). |
| [`NavLinkKind`](#type-NavLinkKind) | enum | How an agent crosses an off-mesh link. |
| [`NavLinkDesc`](#type-NavLinkDesc) | desc struct | An off-mesh link: a way between two points of a navmesh its surface does not join (a gap to jump, a ladder, a door, a teleporter). Each end snaps to the polygon under it within 1 m. |
| [`og.NavLinkReached`](#record-NavLinkReached) | event | An agent reached an off-mesh link and started across it (systems read it the tick after): play the jump or the climb here. With `game`, move it and call og.nav.finishLink when it is over. |
| [`NavShapeKind`](#type-NavShapeKind) | enum | What a NavShape is. |
| [`NavShape`](#type-NavShape) | desc struct | A volume over a navmesh's floor: an obstacle (og.nav.carve) or an area (og.nav.setArea). |
| [`NavAreaDesc`](#type-NavAreaDesc) | desc struct | A run-time area: the spans under `shape` (their floors from the agent's climb below its bottom to its top) take area `area`, over the baked areas and over run-time areas of lower ids. |
| [`NavGridDesc`](#type-NavGridDesc) | desc struct | A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up. |
| [`TilemapGridDesc`](#type-TilemapGridDesc) | desc struct | A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile. |
| [`NavCell`](#type-NavCell) | struct | A grid cell. |
| [`PathDesc`](#type-PathDesc) | desc struct | A path query. |
| [`NavPath`](#type-NavPath) | struct | A path. |
| [`FlowDesc`](#type-FlowDesc) | desc struct | A flow field over a grid. |
| [`NavBox`](#type-NavBox) | desc struct | An axis-aligned box. |
| [`NavMeshDesc`](#type-NavMeshDesc) | desc struct | A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for. |
| [`NavMeshInfo`](#type-NavMeshInfo) | struct | A navmesh's shape. |
| [`NavRebuild`](#type-NavRebuild) | struct | What an obstacle change rebuilt. |
| [`og.nav.grid`](#fn-grid) | function | A grid agents and path queries plan on. Runtime edits (og.nav.setCost) are world state. |
| [`og.nav.gridFromTilemap`](#fn-gridFromTilemap) | function | A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another. |
| [`og.nav.setCost`](#fn-setCost) | function | Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild. |
| [`og.nav.cellAt`](#fn-cellAt) | function | The grid cell under a world point. |
| [`og.nav.findPath`](#fn-findPath) | function | A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end. |
| [`og.nav.flowField`](#fn-flowField) | function | A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. It rebuilds when the grid changes. |
| [`og.nav.setFlowTargets`](#fn-setFlowTargets) | function | Move a flow field's targets (rebuilds it). |
| [`og.nav.flowAt`](#fn-flowAt) | function | The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable). |
| [`og.nav.load`](#fn-load) | function | A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes; og-ai §1 has every line) baked by og cook. |
| [`og.nav.build`](#fn-build) | function | A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load). |
| [`og.nav.setObstacle`](#fn-setObstacle) | function | Set, move or (with no box) remove an obstacle on a navmesh: the spans under the box (grown by the agent radius) are blocked and only the tiles it touches, and their neighbours, are rebuilt. Agents on the mesh re-plan. World state. |
| [`og.nav.carve`](#fn-carve) | function | og.nav.setObstacle for any shape: a turned box (a door ajar, a fallen log), a cylinder (a pillar, a barrel) or a convex prism. The cells whose centres the shape covers, grown by the agent radius, are blocked and only the tiles it touches, and their neighbours, are rebuilt. Shares og.nav.setObstacle's ids. World state. |
| [`og.nav.setArea`](#fn-setArea) | function | Set, move or (with no desc) remove a run-time area on a navmesh (water, mud, a road): its tiles rebuild so no polygon mixes areas, and agents re-plan, weighing it by their og.NavFilter. World state. |
| [`og.nav.setLink`](#fn-setLink) | function | Set, move or (with no desc) remove an off-mesh link on a navmesh. Paths cross it where it is cheaper than walking; an agent that reaches its start crosses it (NAV_ON_LINK, og.NavLinkReached) and lands at its end. Links baked in the navmesh asset (`link` lines) have the ids they were given. World state. |
| [`og.nav.setLinkOpen`](#fn-setLinkOpen) | function | Open or close an off-mesh link (a door): a closed one is never planned across, and agents on the mesh re-plan. World state. |
| [`og.nav.finishLink`](#fn-finishLink) | function | End a crossing the game moves (NavLinkDesc.game): the agent lands at the link's end as og.nav next runs and goes on along its path. |
| [`og.nav.nearest`](#fn-nearest) | function | The point on a navmesh nearest a world point, within `extent` (half sizes). |
| [`og.nav.pathOf`](#fn-pathOf) | function | The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same. |

## Functions

<a id="fn-grid"></a>

### `og.nav.grid`

A grid agents and path queries plan on. Runtime edits (og.nav.setCost) are world state.

```ts
og.nav.grid(world: World, desc?: NavGridDesc): Result<Navigation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NavGridDesc` | yes | Its size, cell size, origin and costs. |

**Returns** `Result<Navigation>`: Refused for costs of the wrong length.

Binding symbol `og_nav_grid` · publish builds: kept

```ogt
fn grid {
  doc "A grid agents and path queries plan on. Runtime edits (og.nav.setCost) are world state."
  param world World "The world."
  param desc NavGridDesc "Its size, cell size, origin and costs."
  returns Result.Navigation "Refused for costs of the wrong length."
}
```

<sub>`api.ogt` line 12475</sub>

<a id="fn-gridFromTilemap"></a>

### `og.nav.gridFromTilemap`

A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another.

```ts
og.nav.gridFromTilemap(world: World, desc?: TilemapGridDesc): Result<Navigation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TilemapGridDesc` | yes | The map's entity and the blocking layers. |

**Returns** `Result<Navigation>`: Refused for an entity without an og.Tilemap or a map not loaded.

Binding symbol `og_nav_grid_from_tilemap` · publish builds: kept

```ogt
fn gridFromTilemap {
  doc "A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another."
  param world World "The world."
  param desc TilemapGridDesc "The map's entity and the blocking layers."
  returns Result.Navigation "Refused for an entity without an og.Tilemap or a map not loaded."
}
```

<sub>`api.ogt` line 12482</sub>

<a id="fn-setCost"></a>

### `og.nav.setCost`

Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild.

```ts
og.nav.setCost(world: World, grid: Navigation, x: number, y: number, cost: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `grid` | `Navigation` | no | The grid. |
| `x` | `number` | no | The column. |
| `y` | `number` | no | The row (0 is the bottom). |
| `cost` | `number` | no | 0 blocked, 1 normal, up to 255. |

**Returns** `Result<void>`: Refused for an unknown grid or a cell outside it.

Binding symbol `og_nav_set_cost` · publish builds: kept

```ogt
fn setCost {
  doc "Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild."
  param world World "The world."
  param grid Navigation "The grid."
  param x u32 "The column."
  param y u32 "The row (0 is the bottom)."
  param cost u32 "0 blocked, 1 normal, up to 255."
  returns Result.Void "Refused for an unknown grid or a cell outside it."
}
```

<sub>`api.ogt` line 12489</sub>

<a id="fn-cellAt"></a>

### `og.nav.cellAt`

The grid cell under a world point.

```ts
og.nav.cellAt(world: World, grid: Navigation, point: Vector3): NavCell
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `grid` | `Navigation` | no | The grid. |
| `point` | `Vector3` | no | A world point. |

**Returns** `NavCell`.

Binding symbol `og_nav_cell_at` · publish builds: kept

```ogt
fn cellAt {
  doc "The grid cell under a world point."
  param world World "The world."
  param grid Navigation "The grid."
  param point Vector3 "A world point."
  returns NavCell
}
```

<sub>`api.ogt` line 12499</sub>

<a id="fn-findPath"></a>

### `og.nav.findPath`

A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end.

```ts
og.nav.findPath(world: World, desc?: PathDesc): Result<NavPath>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `PathDesc` | yes | The grid or navmesh and the two points. |

**Returns** `Result<NavPath>`: Refused for an unknown grid or navmesh, or when no path starts (an end off the grid or mesh).

Binding symbol `og_nav_find_path` · publish builds: kept

```ogt
fn findPath {
  doc "A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end."
  param world World "The world."
  param desc PathDesc "The grid or navmesh and the two points."
  returns Result.NavPath "Refused for an unknown grid or navmesh, or when no path starts (an end off the grid or mesh)."
}
```

<sub>`api.ogt` line 12507</sub>

<a id="fn-flowField"></a>

### `og.nav.flowField`

A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. It rebuilds when the grid changes.

```ts
og.nav.flowField(world: World, desc?: FlowDesc): Result<Navigation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FlowDesc` | yes | The grid and the targets. |

**Returns** `Result<Navigation>`: Refused for an unknown grid.

Binding symbol `og_nav_flow_field` · publish builds: kept

```ogt
fn flowField {
  doc "A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. It rebuilds when the grid changes."
  param world World "The world."
  param desc FlowDesc "The grid and the targets."
  returns Result.Navigation "Refused for an unknown grid."
}
```

<sub>`api.ogt` line 12514</sub>

<a id="fn-setFlowTargets"></a>

### `og.nav.setFlowTargets`

Move a flow field's targets (rebuilds it).

```ts
og.nav.setFlowTargets(world: World, field: Navigation, targets: readonly Vector3[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `field` | `Navigation` | no | The flow field. |
| `targets` | `readonly Vector3[]` | no | The new targets. |

**Returns** `Result<void>`: Refused for an unknown flow field.

Binding symbol `og_nav_set_flow_targets` · publish builds: kept

```ogt
fn setFlowTargets {
  doc "Move a flow field's targets (rebuilds it)."
  param world World "The world."
  param field Navigation "The flow field."
  param targets List.Vector3 "The new targets."
  returns Result.Void "Refused for an unknown flow field."
}
```

<sub>`api.ogt` line 12521</sub>

<a id="fn-flowAt"></a>

### `og.nav.flowAt`

The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable).

```ts
og.nav.flowAt(world: World, field: Navigation, point: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `field` | `Navigation` | no | The flow field. |
| `point` | `Vector3` | no | A world point. |

**Returns** `Vector3`.

Binding symbol `og_nav_flow_at` · publish builds: kept

```ogt
fn flowAt {
  doc "The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable)."
  param world World "The world."
  param field Navigation "The flow field."
  param point Vector3 "A world point."
  returns Vector3
}
```

<sub>`api.ogt` line 12529</sub>

<a id="fn-load"></a>

### `og.nav.load`

A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes; og-ai §1 has every line) baked by og cook.

```ts
og.nav.load(world: World, path: string, profile?: string): Result<NavMeshInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The navmesh asset, such as `levels/arena.nav.ogt`. |
| `profile` | `string` | yes | The agent profile, an `agent <name>` block's name: one mesh per body size (a rat under the table, an ogre round it). Omitted: the asset's first agent block (its unnamed one, if any). |

**Returns** `Result<NavMeshInfo>`: Refused for a path that is not a cooked navmesh, or a profile it has no agent block for (naming the ones it has).

Binding symbol `og_nav_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes; og-ai §1 has every line) baked by og cook."
  param world World "The world."
  param path String "The navmesh asset, such as `levels/arena.nav.ogt`."
  param profile String "The agent profile, an `agent <name>` block's name: one mesh per body size (a rat under the table, an ogre round it). Omitted: the asset's first agent block (its unnamed one, if any)." {
    optional true
  }
  returns Result.NavMeshInfo "Refused for a path that is not a cooked navmesh, or a profile it has no agent block for (naming the ones it has)."
}
```

<sub>`api.ogt` line 12537</sub>

<a id="fn-build"></a>

### `og.nav.build`

A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load).

```ts
og.nav.build(world: World, desc?: NavMeshDesc): Result<NavMeshInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NavMeshDesc` | yes | The floor, the boxes and the agent. |

**Returns** `Result<NavMeshInfo>`: Refused for an empty floor.

Binding symbol `og_nav_build` · publish builds: kept

```ogt
fn build {
  doc "A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load)."
  param world World "The world."
  param desc NavMeshDesc "The floor, the boxes and the agent."
  returns Result.NavMeshInfo "Refused for an empty floor."
}
```

<sub>`api.ogt` line 12547</sub>

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

<sub>`api.ogt` line 12554</sub>

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

<sub>`api.ogt` line 12565</sub>

<a id="fn-setArea"></a>

### `og.nav.setArea`

Set, move or (with no desc) remove a run-time area on a navmesh (water, mud, a road): its tiles rebuild so no polygon mixes areas, and agents re-plan, weighing it by their og.NavFilter. World state.

```ts
og.nav.setArea(world: World, mesh: Navigation, id: number, desc?: NavAreaDesc): Result<NavRebuild>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The area's id, the game's own. |
| `desc` | `NavAreaDesc` | yes | The shape and the area; omitted removes it. |

**Returns** `Result<NavRebuild>`: Refused for an unknown navmesh or an area past 63.

Binding symbol `og_nav_set_area` · publish builds: kept

```ogt
fn setArea {
  doc "Set, move or (with no desc) remove a run-time area on a navmesh (water, mud, a road): its tiles rebuild so no polygon mixes areas, and agents re-plan, weighing it by their og.NavFilter. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The area's id, the game's own."
  param desc NavAreaDesc "The shape and the area; omitted removes it." {
    optional true
  }
  returns Result.NavRebuild "Refused for an unknown navmesh or an area past 63."
}
```

<sub>`api.ogt` line 12576</sub>

<a id="fn-setLink"></a>

### `og.nav.setLink`

Set, move or (with no desc) remove an off-mesh link on a navmesh. Paths cross it where it is cheaper than walking; an agent that reaches its start crosses it (NAV_ON_LINK, og.NavLinkReached) and lands at its end. Links baked in the navmesh asset (`link` lines) have the ids they were given. World state.

```ts
og.nav.setLink(world: World, mesh: Navigation, id: number, desc?: NavLinkDesc): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The link's id, the game's own. |
| `desc` | `NavLinkDesc` | yes | The link; omitted removes it. |

**Returns** `Result<boolean>`: Whether both ends are on the mesh (a link with an end off it is kept but never crossed). Refused for an unknown navmesh.

Binding symbol `og_nav_set_link` · publish builds: kept

```ogt
fn setLink {
  doc "Set, move or (with no desc) remove an off-mesh link on a navmesh. Paths cross it where it is cheaper than walking; an agent that reaches its start crosses it (NAV_ON_LINK, og.NavLinkReached) and lands at its end. Links baked in the navmesh asset (`link` lines) have the ids they were given. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The link's id, the game's own."
  param desc NavLinkDesc "The link; omitted removes it." {
    optional true
  }
  returns Result.bool "Whether both ends are on the mesh (a link with an end off it is kept but never crossed). Refused for an unknown navmesh."
}
```

<sub>`api.ogt` line 12587</sub>

<a id="fn-setLinkOpen"></a>

### `og.nav.setLinkOpen`

Open or close an off-mesh link (a door): a closed one is never planned across, and agents on the mesh re-plan. World state.

```ts
og.nav.setLinkOpen(world: World, mesh: Navigation, id: number, open: boolean): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The link's id. |
| `open` | `boolean` | no | Open. |

**Returns** `Result<void>`: Refused for an unknown navmesh or link.

Binding symbol `og_nav_set_link_open` · publish builds: kept

```ogt
fn setLinkOpen {
  doc "Open or close an off-mesh link (a door): a closed one is never planned across, and agents on the mesh re-plan. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The link's id."
  param open bool "Open."
  returns Result.Void "Refused for an unknown navmesh or link."
}
```

<sub>`api.ogt` line 12598</sub>

<a id="fn-finishLink"></a>

### `og.nav.finishLink`

End a crossing the game moves (NavLinkDesc.game): the agent lands at the link's end as og.nav next runs and goes on along its path.

```ts
og.nav.finishLink(world: World, entity: Entity): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The agent. |

**Returns** `boolean`: False when it is not crossing such a link.

Binding symbol `og_nav_finish_link` · publish builds: kept

```ogt
fn finishLink {
  doc "End a crossing the game moves (NavLinkDesc.game): the agent lands at the link's end as og.nav next runs and goes on along its path."
  param world World "The world."
  param entity Entity "The agent."
  returns bool "False when it is not crossing such a link."
}
```

<sub>`api.ogt` line 12607</sub>

<a id="fn-nearest"></a>

### `og.nav.nearest`

The point on a navmesh nearest a world point, within `extent` (half sizes).

```ts
og.nav.nearest(world: World, mesh: Navigation, point: Vector3, extent?: Vector3): Vector3 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `point` | `Vector3` | no | A world point. |
| `extent` | `Vector3` | yes | The search box's half sizes; zero: 2 2 2. |

**Returns** `Vector3 | null`: Null when no polygon is within the box.

Binding symbol `og_nav_nearest` · publish builds: kept

```ogt
fn nearest {
  doc "The point on a navmesh nearest a world point, within `extent` (half sizes)."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param point Vector3 "A world point."
  param extent Vector3 "The search box's half sizes; zero: 2 2 2." {
    optional true
  }
  returns Option.Vector3 "Null when no polygon is within the box."
}
```

<sub>`api.ogt` line 12614</sub>

<a id="fn-pathOf"></a>

### `og.nav.pathOf`

The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same.

```ts
og.nav.pathOf(world: World, entity: Entity): NavPath | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.NavAgent. |

**Returns** `NavPath | null`: Null for an entity that has not planned.

Binding symbol `og_nav_path_of` · publish builds: kept

```ogt
fn pathOf {
  doc "The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same."
  param world World "The world."
  param entity Entity "An entity with an og.NavAgent."
  returns Option.NavPath "Null for an entity that has not planned."
}
```

<sub>`api.ogt` line 12625</sub>

## Components, resources and events

<a id="record-NavAgent"></a>

### `og.NavAgent` (component)

An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents.

Token `og.NavAgent`, registered under the schema name `og.NavAgent`; its value type is `NavAgent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nav` | `Navigation` | `0` | — | — | — | The grid, navmesh or flow field it plans on (og.nav.grid, og.nav.load, og.nav.flowField); 0 moves straight at the target. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where it goes while seeking. |
| `seeking` | `boolean` | `false` | — | — | — | Moves towards the target while true. |
| `speed` | `number` | `3.5` | 0 … 100 | `mps` | tunable | Top speed. |
| `acceleration` | `number` | `20` | 0 … 1000 | `mps2` | tunable | How fast it changes velocity. |
| `radius` | `number` | `0.4` | 0 … 10 | `m` | tunable | Its radius: grid paths keep this clear of walls; separation uses it. |
| `stopDistance` | `number` | `0.1` | 0 … 10 | `m` | tunable | It has arrived within this distance of the target. |
| `slowRadius` | `number` | `1` | 0 … 20 | `m` | tunable | It slows down inside this distance of the target. |
| `repathTicks` | `number` | `30` | 0 … 3600 | `ticks` | — | Re-plans this often while seeking (0: only when the target moves). |
| `repathDistance` | `number` | `0.5` | 0 … 100 | `m` | — | Re-plans when the target moved farther than this from the last plan. |
| `smooth` | `boolean` | `true` | — | — | — | Pulls grid paths straight where nothing is in the way. |
| `faceMovement` | `boolean` | `false` | — | — | — | Turns the entity to face where it moves (3D: about +Y, its +Z forward; 2D: about +Z, its +X forward). |
| `turnSpeed` | `number` | `10` | 0 … 100 | `radps` | — | How fast it turns when facing its movement. |
| `drive` | `boolean` | `true` | — | — | — | Moves the entity (its og.Velocity, or its og.Transform) while true; false keeps planning but leaves the motion to the game (a knock-back, a stun). |
| `status` | `number` | `0` | — | — | read only | NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE or NAV_PARTIAL (written by og.nav). |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | read only | Its velocity (written by og.nav). |
| `waypoint` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only | The path corner it heads for (written by og.nav). |
| `remaining` | `number` | `0` | — | `m` | read only | The path length left (written by og.nav). |
| `plans` | `number` | `0` | — | — | read only | Paths planned so far (written by og.nav). |

```ogt
component NavAgent {
  doc "An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents."
  field nav Navigation "The grid, navmesh or flow field it plans on (og.nav.grid, og.nav.load, og.nav.flowField); 0 moves straight at the target."
  field target Vector3 "Where it goes while seeking."
  field seeking bool "Moves towards the target while true."
  field speed f32 "Top speed." {
    default 3.5
    min 0
    max 100
    unit mps
    tunable true
  }
  field acceleration f32 "How fast it changes velocity." {
    default 20
    min 0
    max 1000
    unit mps2
    tunable true
  }
  field radius f32 "Its radius: grid paths keep this clear of walls; separation uses it." {
    default 0.4
    min 0
    max 10
    unit m
    tunable true
  }
  field stopDistance f32 "It has arrived within this distance of the target." {
    default 0.1
    min 0
    max 10
    unit m
    tunable true
  }
  field slowRadius f32 "It slows down inside this distance of the target." {
    default 1
    min 0
    max 20
    unit m
    tunable true
  }
  field repathTicks u32 "Re-plans this often while seeking (0: only when the target moves)." {
    default 30
    min 0
    max 3600
    unit ticks
  }
  field repathDistance f32 "Re-plans when the target moved farther than this from the last plan." {
    default 0.5
    min 0
    max 100
    unit m
  }
  field smooth bool "Pulls grid paths straight where nothing is in the way." {
    default true
  }
  field faceMovement bool "Turns the entity to face where it moves (3D: about +Y, its +Z forward; 2D: about +Z, its +X forward)."
  field turnSpeed f32 "How fast it turns when facing its movement." {
    default 10
    min 0
    max 100
    unit radps
  }
  field drive bool "Moves the entity (its og.Velocity, or its og.Transform) while true; false keeps planning but leaves the motion to the game (a knock-back, a stun)." {
    default true
  }
  field status u32 "NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE or NAV_PARTIAL (written by og.nav)." {
    readonly true
  }
  field velocity Vector3 "Its velocity (written by og.nav)." {
    unit mps
    readonly true
  }
  field waypoint Vector3 "The path corner it heads for (written by og.nav)." {
    unit m
    readonly true
  }
  field remaining f32 "The path length left (written by og.nav)." {
    unit m
    readonly true
  }
  field plans u32 "Paths planned so far (written by og.nav)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 12009</sub>

<a id="record-Steering"></a>

### `og.Steering` (component)

Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays).

Token `og.Steering`, registered under the schema name `og.Steering`; its value type is `Steering`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seek` | `number` | `1` | 0 … 10 | — | tunable | Weight of following the path to the target (seek, then arrive at the end). |
| `flee` | `number` | `0` | 0 … 10 | — | tunable | Weight of fleeing fleeFrom. |
| `fleeFrom` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | What it flees. |
| `fleeRadius` | `number` | `6` | 0 … 100 | `m` | tunable | It flees (and evades) only inside this distance (0: always). |
| `wander` | `number` | `0` | 0 … 10 | — | tunable | Weight of wandering. |
| `wanderRadius` | `number` | `1` | 0 … 20 | `m` | tunable | The wander circle's radius. |
| `wanderDistance` | `number` | `2` | 0 … 20 | `m` | tunable | How far ahead the wander circle is. |
| `wanderJitter` | `number` | `3` | 0 … 50 | `radps` | tunable | How fast the wander point moves round the circle. |
| `pursue` | `Entity` | `0` | — | — | — | An entity to pursue (seek where it will be, from its NavAgent or og.Velocity); 0: none. |
| `pursueWeight` | `number` | `1` | 0 … 10 | — | tunable | Weight of pursuing. |
| `evade` | `Entity` | `0` | — | — | — | An entity to evade (flee where it will be); 0: none. |
| `evadeWeight` | `number` | `1` | 0 … 10 | — | tunable | Weight of evading. |
| `separation` | `number` | `0` | 0 … 10 | — | tunable | Weight of keeping apart from other agents. |
| `separationRadius` | `number` | `1` | 0 … 20 | `m` | tunable | Other agents closer than this push it away. |
| `avoidance` | `number` | `0` | 0 … 10 | — | tunable | Weight of avoiding colliders ahead. |
| `avoidDistance` | `number` | `2` | 0 … 50 | `m` | tunable | How far ahead the avoidance rays reach. |
| `avoidMask` | `number` | `0` | — | — | — | Collision layers the avoidance rays hit (0: every layer). |
| `wanderAngle` | `number` | `0` | — | `rad` | read only | The wander angle (written by og.nav). |

```ogt
component Steering {
  doc "Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays)."
  field seek f32 "Weight of following the path to the target (seek, then arrive at the end)." {
    default 1
    min 0
    max 10
    tunable true
  }
  field flee f32 "Weight of fleeing fleeFrom." {
    min 0
    max 10
    tunable true
  }
  field fleeFrom Vector3 "What it flees." {
    unit m
  }
  field fleeRadius f32 "It flees (and evades) only inside this distance (0: always)." {
    default 6
    min 0
    max 100
    unit m
    tunable true
  }
  field wander f32 "Weight of wandering." {
    min 0
    max 10
    tunable true
  }
  field wanderRadius f32 "The wander circle's radius." {
    default 1
    min 0
    max 20
    unit m
    tunable true
  }
  field wanderDistance f32 "How far ahead the wander circle is." {
    default 2
    min 0
    max 20
    unit m
    tunable true
  }
  field wanderJitter f32 "How fast the wander point moves round the circle." {
    default 3
    min 0
    max 50
    unit radps
    tunable true
  }
  field pursue Entity "An entity to pursue (seek where it will be, from its NavAgent or og.Velocity); 0: none."
  field pursueWeight f32 "Weight of pursuing." {
    default 1
    min 0
    max 10
    tunable true
  }
  field evade Entity "An entity to evade (flee where it will be); 0: none."
  field evadeWeight f32 "Weight of evading." {
    default 1
    min 0
    max 10
    tunable true
  }
  field separation f32 "Weight of keeping apart from other agents." {
    min 0
    max 10
    tunable true
  }
  field separationRadius f32 "Other agents closer than this push it away." {
    default 1
    min 0
    max 20
    unit m
    tunable true
  }
  field avoidance f32 "Weight of avoiding colliders ahead." {
    min 0
    max 10
    tunable true
  }
  field avoidDistance f32 "How far ahead the avoidance rays reach." {
    default 2
    min 0
    max 50
    unit m
    tunable true
  }
  field avoidMask u32 "Collision layers the avoidance rays hit (0: every layer)."
  field wanderAngle f32 "The wander angle (written by og.nav)." {
    unit rad
    readonly true
  }
}
```

<sub>`api.ogt` line 12094</sub>

<a id="record-NavFilter"></a>

### `og.NavFilter` (component)

Which navmesh areas an agent crosses and what a metre in each costs: beside an og.NavAgent, its plans weigh every polygon's length by its area's cost and never enter an excluded area. Areas are marked at cook time (`area` in a navmesh asset) or at run time (og.nav.setArea); 0 is ground. Without one every area costs 1.

Token `og.NavFilter`, registered under the schema name `og.NavFilter`; its value type is `NavFilter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `areaCosts` | `readonly number[]` | `[]` | — | — | a list of at most 64 `f32` (`List.f32.64`): reads a frozen copy, writes a whole array | Per area id, the cost of a metre in it (past the list's length 1, and at least 0.01): `{ areaCosts: [1, 1, 1, 8] }` makes area 3 (water) eight times dearer, so a path wades only when going round is longer than that. |
| `excluded` | `readonly number[]` | `[]` | — | — | a list of at most 64 `u32` (`List.u32.64`): reads a frozen copy, writes a whole array | Area ids the agent never enters (0 to 63): `{ excluded: [4] }` keeps it out of area 4 (lava). |

```ogt
component NavFilter {
  doc "Which navmesh areas an agent crosses and what a metre in each costs: beside an og.NavAgent, its plans weigh every polygon's length by its area's cost and never enter an excluded area. Areas are marked at cook time (`area` in a navmesh asset) or at run time (og.nav.setArea); 0 is ground. Without one every area costs 1."
  field areaCosts List.f32.64 "Per area id, the cost of a metre in it (past the list's length 1, and at least 0.01): `{ areaCosts: [1, 1, 1, 8] }` makes area 3 (water) eight times dearer, so a path wades only when going round is longer than that."
  field excluded List.u32.64 "Area ids the agent never enters (0 to 63): `{ excluded: [4] }` keeps it out of area 4 (lava)."
}
```

<sub>`api.ogt` line 12188</sub>

<a id="record-Crowd"></a>

### `og.Crowd` (component)

Reciprocal crowd avoidance (ORCA) for an agent: beside an og.NavAgent, each tick its velocity is the one nearest what its path and steering want that keeps it clear of its nearest agents for `horizon` seconds and of the navmesh's walls for `obstacleHorizon`, each pair of crowd agents sharing the avoidance. Every agent solves from the same start-of-tick state in a fixed order, so crowds replay identically on every host; agents meeting perfectly symmetrically (a ring swapping places) are kept from locking by a margin of up to 5% of the radius picked from the entity, so a game adds no jitter of its own. Without one an agent moves as before (og.Steering's separation only).

Token `og.Crowd`, registered under the schema name `og.Crowd`; its value type is `Crowd`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `neighbours` | `number` | `8` | 0 … 32 | — | tunable | How many nearest agents it avoids (at most 32). |
| `horizon` | `number` | `2` | 0 … 10 | `s` | tunable | Seconds ahead it keeps clear of other agents: longer turns aside earlier and more gently. |
| `obstacleHorizon` | `number` | `0.5` | 0 … 10 | `s` | tunable | Seconds ahead it keeps clear of walls (a navmesh's edges). |
| `weight` | `number` | `0.5` | 0 … 1 | — | tunable | Its share of each avoidance with another crowd agent (0.5: half each); an agent without og.Crowd is avoided wholly by it. |

```ogt
component Crowd {
  doc "Reciprocal crowd avoidance (ORCA) for an agent: beside an og.NavAgent, each tick its velocity is the one nearest what its path and steering want that keeps it clear of its nearest agents for `horizon` seconds and of the navmesh's walls for `obstacleHorizon`, each pair of crowd agents sharing the avoidance. Every agent solves from the same start-of-tick state in a fixed order, so crowds replay identically on every host; agents meeting perfectly symmetrically (a ring swapping places) are kept from locking by a margin of up to 5% of the radius picked from the entity, so a game adds no jitter of its own. Without one an agent moves as before (og.Steering's separation only)."
  field neighbours u32 "How many nearest agents it avoids (at most 32)." {
    default 8
    min 0
    max 32
    tunable true
  }
  field horizon f32 "Seconds ahead it keeps clear of other agents: longer turns aside earlier and more gently." {
    default 2
    min 0
    max 10
    unit s
    tunable true
  }
  field obstacleHorizon f32 "Seconds ahead it keeps clear of walls (a navmesh's edges)." {
    default 0.5
    min 0
    max 10
    unit s
    tunable true
  }
  field weight f32 "Its share of each avoidance with another crowd agent (0.5: half each); an agent without og.Crowd is avoided wholly by it." {
    default 0.5
    min 0
    max 1
    tunable true
  }
}
```

<sub>`api.ogt` line 12194</sub>

<a id="record-NavLinkReached"></a>

### `og.NavLinkReached` (event)

An agent reached an off-mesh link and started across it (systems read it the tick after): play the jump or the climb here. With `game`, move it and call og.nav.finishLink when it is over.

Token `og.NavLinkReached`, registered under the schema name `og.NavLinkReached`; its value type is `NavLinkReached`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `agent` | `Entity` | `0` | — | — | — | The agent. |
| `mesh` | `Navigation` | `0` | — | — | — | The navmesh. |
| `link` | `number` | `0` | — | — | — | The link's id. |
| `kind` | `number` | `0` | — | — | — | Its NavLinkKind, by index: 0 jump, 1 ladder, 2 door, 3 teleport. |
| `start` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the agent leaves the surface. |
| `end` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it lands. |
| `game` | `boolean` | `false` | — | — | — | The game moves it across. |

```ogt
event NavLinkReached {
  doc "An agent reached an off-mesh link and started across it (systems read it the tick after): play the jump or the climb here. With `game`, move it and call og.nav.finishLink when it is over."
  field agent Entity "The agent."
  field mesh Navigation "The navmesh."
  field link u32 "The link's id."
  field kind u32 "Its NavLinkKind, by index: 0 jump, 1 ladder, 2 door, 3 teleport."
  field start Vector3 "Where the agent leaves the surface." {
    unit m
  }
  field end Vector3 "Where it lands." {
    unit m
  }
  field game bool "The game moves it across."
}
```

<sub>`api.ogt` line 12259</sub>

## Types

<a id="type-Navigation"></a>

### `Navigation` (handle)

A grid, a navmesh or a flow field agents plan on.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Navigation "A grid, a navmesh or a flow field agents plan on."
```

<sub>`api.ogt` line 11987</sub>

<a id="type-NavAlgorithm"></a>

### `NavAlgorithm` (enum)

The grid search a path query runs.

A string: one of the variants below. The zero value is `"jps"`.

| Variant | Doc |
|---|---|
| `"jps"` | Jump point search: the same optimal paths as A* with fewer nodes; on a grid with weighted cells it runs A*. |
| `"astar"` | A* over every cell. |

```ogt
enum NavAlgorithm {
  doc "The grid search a path query runs."
  variant jps "Jump point search: the same optimal paths as A* with fewer nodes; on a grid with weighted cells it runs A*."
  variant astar "A* over every cell."
}
```

<sub>`api.ogt` line 12003</sub>

<a id="type-NavLinkKind"></a>

### `NavLinkKind` (enum)

How an agent crosses an off-mesh link.

A string: one of the variants below. The zero value is `"jump"`.

| Variant | Doc |
|---|---|
| `"jump"` | Through the air on an arc, in its length over the agent's speed (at least 0.3 s). |
| `"ladder"` | Climbing, at half the agent's speed. |
| `"door"` | Walking through at the agent's speed, while the link is open (og.nav.setLinkOpen). |
| `"teleport"` | At once: on the far side the tick after it is reached. |

```ogt
enum NavLinkKind {
  doc "How an agent crosses an off-mesh link."
  variant jump "Through the air on an arc, in its length over the agent's speed (at least 0.3 s)."
  variant ladder "Climbing, at half the agent's speed."
  variant door "Walking through at the agent's speed, while the link is open (og.nav.setLinkOpen)."
  variant teleport "At once: on the far side the tick after it is reached."
}
```

<sub>`api.ogt` line 12224</sub>

<a id="type-NavLinkDesc"></a>

### `NavLinkDesc` (desc struct)

An off-mesh link: a way between two points of a navmesh its surface does not join (a gap to jump, a ladder, a door, a teleporter). Each end snaps to the polygon under it within 1 m.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `start` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the agent leaves the surface. |
| `end` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it lands. |
| `kind` | `NavLinkKind` | `"jump"` | — | — | — | How it crosses. |
| `bidirectional` | `boolean` | `true` | — | — | — | Crossed both ways; false: from start to end only. |
| `cost` | `number` | `0` | ≥ 0 | `m` | — | What crossing costs a path, in metres of walking; 0 is the link's length. A dear link loses to a shorter walk round. |
| `open` | `boolean` | `true` | — | — | — | A closed link is never planned across (a shut door); agents re-plan when it opens or closes. |
| `duration` | `number` | `0` | ≥ 0 | `s` | — | Seconds a crossing takes; 0 is the kind's own. |
| `game` | `boolean` | `false` | — | — | — | The game moves the agent across (an animation, a cutscene): og.nav holds it at the start until og.nav.finishLink, then lands it at the end. |

```ogt
struct NavLinkDesc {
  doc "An off-mesh link: a way between two points of a navmesh its surface does not join (a gap to jump, a ladder, a door, a teleporter). Each end snaps to the polygon under it within 1 m."
  desc true
  field start Vector3 "Where the agent leaves the surface." {
    unit m
  }
  field end Vector3 "Where it lands." {
    unit m
  }
  field kind NavLinkKind "How it crosses."
  field bidirectional bool "Crossed both ways; false: from start to end only." {
    default true
  }
  field cost f32 "What crossing costs a path, in metres of walking; 0 is the link's length. A dear link loses to a shorter walk round." {
    min 0
    unit m
  }
  field open bool "A closed link is never planned across (a shut door); agents re-plan when it opens or closes." {
    default true
  }
  field duration f32 "Seconds a crossing takes; 0 is the kind's own." {
    min 0
    unit s
  }
  field game bool "The game moves the agent across (an animation, a cutscene): og.nav holds it at the start until og.nav.finishLink, then lands it at the end."
}
```

<sub>`api.ogt` line 12232</sub>

<a id="type-NavShapeKind"></a>

### `NavShapeKind` (enum)

What a NavShape is.

A string: one of the variants below. The zero value is `"box"`.

| Variant | Doc |
|---|---|
| `"box"` | An axis-aligned box from `min` to `max`. |
| `"orientedBox"` | A box of size `size` round `center`, turned `yaw` radians about +y. |
| `"cylinder"` | An upright cylinder of `radius` and `height` standing on `center`. |
| `"convex"` | A convex prism: `points` (x and z, either winding) from `min.y` to `max.y`. |

```ogt
enum NavShapeKind {
  doc "What a NavShape is."
  variant box "An axis-aligned box from `min` to `max`."
  variant orientedBox "A box of size `size` round `center`, turned `yaw` radians about +y."
  variant cylinder "An upright cylinder of `radius` and `height` standing on `center`."
  variant convex "A convex prism: `points` (x and z, either winding) from `min.y` to `max.y`."
}
```

<sub>`api.ogt` line 12274</sub>

<a id="type-NavShape"></a>

### `NavShape` (desc struct)

A volume over a navmesh's floor: an obstacle (og.nav.carve) or an area (og.nav.setArea).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `NavShapeKind` | `"box"` | — | — | — | Which shape. |
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | box: its lowest corner; convex: its bottom (min.y). |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | box: its highest corner; convex: its top (max.y). |
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | orientedBox: its centre; cylinder: the middle of its base. |
| `size` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | orientedBox: its full size before the turn. |
| `yaw` | `number` | `0` | — | `rad` | — | orientedBox: its turn about +y. |
| `radius` | `number` | `0` | ≥ 0 | `m` | — | cylinder: its radius. |
| `height` | `number` | `0` | ≥ 0 | `m` | — | cylinder: its height. |
| `points` | `readonly Vector3[]` | `[]` | — | — | — | convex: its outline's corners (their y is ignored). |

```ogt
struct NavShape {
  doc "A volume over a navmesh's floor: an obstacle (og.nav.carve) or an area (og.nav.setArea)."
  desc true
  field kind NavShapeKind "Which shape."
  field min Vector3 "box: its lowest corner; convex: its bottom (min.y)." {
    unit m
  }
  field max Vector3 "box: its highest corner; convex: its top (max.y)." {
    unit m
  }
  field center Vector3 "orientedBox: its centre; cylinder: the middle of its base." {
    unit m
  }
  field size Vector3 "orientedBox: its full size before the turn." {
    unit m
  }
  field yaw f32 "orientedBox: its turn about +y." {
    unit rad
  }
  field radius f32 "cylinder: its radius." {
    min 0
    unit m
  }
  field height f32 "cylinder: its height." {
    min 0
    unit m
  }
  field points List.Vector3 "convex: its outline's corners (their y is ignored)."
}
```

<sub>`api.ogt` line 12282</sub>

<a id="type-NavAreaDesc"></a>

### `NavAreaDesc` (desc struct)

A run-time area: the spans under `shape` (their floors from the agent's climb below its bottom to its top) take area `area`, over the baked areas and over run-time areas of lower ids.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `NavShape` | `{}` | — | — | — | Where. |
| `area` | `number` | `0` | ≤ 63 | — | — | The area id, 0 to 63 (0 is ground). |

```ogt
struct NavAreaDesc {
  doc "A run-time area: the spans under `shape` (their floors from the agent's climb below its bottom to its top) take area `area`, over the baked areas and over run-time areas of lower ids."
  desc true
  field shape NavShape "Where."
  field area u32 "The area id, 0 to 63 (0 is ground)." {
    max 63
  }
}
```

<sub>`api.ogt` line 12312</sub>

<a id="type-NavGridDesc"></a>

### `NavGridDesc` (desc struct)

A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `32` | 1 … 4096 | — | — | Cells across. |
| `height` | `number` | `32` | 1 … 4096 | — | — | Cells up. |
| `cellSize` | `number` | `1` | 0.01 … 100 | `m` | — | A cell's side. |
| `origin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Where cell (0, 0)'s lower corner is (x, y in 2D; x, z in 3D). |
| `costs` | `readonly number[]` | `[]` | — | — | — | Row-major cell costs from row 0 (0 blocked, 1 normal, up to 255 to avoid); empty: every cell 1. |

```ogt
struct NavGridDesc {
  doc "A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up."
  desc true
  field width u32 "Cells across." {
    default 32
    min 1
    max 4096
  }
  field height u32 "Cells up." {
    default 32
    min 1
    max 4096
  }
  field cellSize f32 "A cell's side." {
    default 1
    min 0.01
    max 100
    unit m
  }
  field origin Vector2 "Where cell (0, 0)'s lower corner is (x, y in 2D; x, z in 3D)." {
    unit m
  }
  field costs List.u32 "Row-major cell costs from row 0 (0 blocked, 1 normal, up to 255 to avoid); empty: every cell 1."
}
```

<sub>`api.ogt` line 12321</sub>

<a id="type-TilemapGridDesc"></a>

### `TilemapGridDesc` (desc struct)

A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Tilemap. |
| `layers` | `readonly string[]` | `[]` | — | — | — | The layers whose solid tiles block; empty: every collision layer. |
| `inflate` | `number` | `0` | ≤ 8 | — | — | Also block this many cells around each blocked one (wide agents). |

```ogt
struct TilemapGridDesc {
  doc "A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile."
  desc true
  field entity Entity "The entity with the og.Tilemap."
  field layers List.String "The layers whose solid tiles block; empty: every collision layer."
  field inflate u32 "Also block this many cells around each blocked one (wide agents)." {
    max 8
  }
}
```

<sub>`api.ogt` line 12346</sub>

<a id="type-NavCell"></a>

### `NavCell` (struct)

A grid cell.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | The column. |
| `y` | `number` | `0` | — | — | — | The row (0 is the bottom). |
| `inside` | `boolean` | `false` | — | — | — | Whether the point was on the grid. |

```ogt
struct NavCell {
  doc "A grid cell."
  field x u32 "The column."
  field y u32 "The row (0 is the bottom)."
  field inside bool "Whether the point was on the grid."
}
```

<sub>`api.ogt` line 12356</sub>

<a id="type-PathDesc"></a>

### `PathDesc` (desc struct)

A path query.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nav` | `Navigation` | `0` | — | — | — | The grid or navmesh. |
| `from` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The start. |
| `to` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The end. |
| `algorithm` | `NavAlgorithm` | `"jps"` | — | — | — | The grid search. |
| `smooth` | `boolean` | `true` | — | — | — | Pull a grid path straight where nothing is in the way. |
| `radius` | `number` | `0` | 0 … 10 | `m` | — | Keep this clear of grid walls when smoothing. |
| `areaCosts` | `readonly number[]` | `[]` | — | — | — | On a navmesh: per area id, the cost of a metre in it, as og.NavFilter's. |
| `excluded` | `readonly number[]` | `[]` | — | — | — | On a navmesh: area ids the path never enters. |

```ogt
struct PathDesc {
  doc "A path query."
  desc true
  field nav Navigation "The grid or navmesh."
  field from Vector3 "The start." {
    unit m
  }
  field to Vector3 "The end." {
    unit m
  }
  field algorithm NavAlgorithm "The grid search."
  field smooth bool "Pull a grid path straight where nothing is in the way." {
    default true
  }
  field radius f32 "Keep this clear of grid walls when smoothing." {
    min 0
    max 10
    unit m
  }
  field areaCosts List.f32 "On a navmesh: per area id, the cost of a metre in it, as og.NavFilter's."
  field excluded List.u32 "On a navmesh: area ids the path never enters."
}
```

<sub>`api.ogt` line 12363</sub>

<a id="type-NavPath"></a>

### `NavPath` (struct)

A path.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | — | — | Its corners, start and end included. |
| `length` | `number` | `0` | — | `m` | — | Its length. |
| `complete` | `boolean` | `false` | — | — | — | Whether it reaches the end (false: it stops at the reachable point nearest it). |
| `expanded` | `number` | `0` | — | — | — | Cells or polygons the search took off its open list. |
| `algorithm` | `string` | `""` | — | — | — | What ran: jps, astar or navmesh. |
| `links` | `readonly number[]` | `[]` | — | — | — | Where it crosses off-mesh links: each an index into points, the link's start (the next point is where it lands). |

```ogt
struct NavPath {
  doc "A path."
  field points List.Vector3 "Its corners, start and end included."
  field length f32 "Its length." {
    unit m
  }
  field complete bool "Whether it reaches the end (false: it stops at the reachable point nearest it)."
  field expanded u32 "Cells or polygons the search took off its open list."
  field algorithm String "What ran: jps, astar or navmesh."
  field links List.u32 "Where it crosses off-mesh links: each an index into points, the link's start (the next point is where it lands)."
}
```

<sub>`api.ogt` line 12386</sub>

<a id="type-FlowDesc"></a>

### `FlowDesc` (desc struct)

A flow field over a grid.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `grid` | `Navigation` | `0` | — | — | — | The grid. |
| `targets` | `readonly Vector3[]` | `[]` | — | — | — | Where agents go: each point's cell is a target. |

```ogt
struct FlowDesc {
  doc "A flow field over a grid."
  desc true
  field grid Navigation "The grid."
  field targets List.Vector3 "Where agents go: each point's cell is a target."
}
```

<sub>`api.ogt` line 12398</sub>

<a id="type-NavBox"></a>

### `NavBox` (desc struct)

An axis-aligned box.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its lowest corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its highest corner. |

```ogt
struct NavBox {
  doc "An axis-aligned box."
  desc true
  field min Vector3 "Its lowest corner." {
    unit m
  }
  field max Vector3 "Its highest corner." {
    unit m
  }
}
```

<sub>`api.ogt` line 12405</sub>

<a id="type-NavMeshDesc"></a>

### `NavMeshDesc` (desc struct)

A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The floor's lowest corner (its y is the floor's height). |
| `max` | `Vector3` | `{ x: 16, y: 1, z: 16 }` | — | `m` | — | The floor's highest corner. |
| `blocked` | `readonly NavBox[]` | `[]` | — | — | — | Boxes standing on the floor (walls, crates). |
| `cellSize` | `number` | `0.25` | 0.02 … 4 | `m` | — | The voxel width. |
| `agentRadius` | `number` | `0.4` | 0 … 10 | `m` | — | Walls are kept this far away. |
| `agentHeight` | `number` | `1.8` | 0.1 … 20 | `m` | — | The agent's height. |
| `tileSize` | `number` | `32` | 8 … 256 | — | — | Cells per tile side: an obstacle rebuilds the tiles it touches. |

```ogt
struct NavMeshDesc {
  doc "A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for."
  desc true
  field min Vector3 "The floor's lowest corner (its y is the floor's height)." {
    unit m
  }
  field max Vector3 "The floor's highest corner." {
    default 16 1 16
    unit m
  }
  field blocked List.NavBox "Boxes standing on the floor (walls, crates)."
  field cellSize f32 "The voxel width." {
    default 0.25
    min 0.02
    max 4
    unit m
  }
  field agentRadius f32 "Walls are kept this far away." {
    default 0.4
    min 0
    max 10
    unit m
  }
  field agentHeight f32 "The agent's height." {
    default 1.8
    min 0.1
    max 20
    unit m
  }
  field tileSize u32 "Cells per tile side: an obstacle rebuilds the tiles it touches." {
    default 32
    min 8
    max 256
  }
}
```

<sub>`api.ogt` line 12416</sub>

<a id="type-NavMeshInfo"></a>

### `NavMeshInfo` (struct)

A navmesh's shape.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mesh` | `Navigation` | `0` | — | — | — | What NavAgent.nav and the queries take. |
| `polygons` | `number` | `0` | — | — | — | Its convex polygons. |
| `tiles` | `number` | `0` | — | — | — | Its tiles. |
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The lowest corner of the box around its polygons. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The highest corner of the box around its polygons (its highest walkable floor's y). |
| `cellSize` | `number` | `0` | — | `m` | — | Its voxel width. |
| `profile` | `string` | `""` | — | — | — | The agent profile it was baked for (og.nav.load's profile); empty for a navmesh of one unnamed agent and for og.nav.build. |

```ogt
struct NavMeshInfo {
  doc "A navmesh's shape."
  field mesh Navigation "What NavAgent.nav and the queries take."
  field polygons u32 "Its convex polygons."
  field tiles u32 "Its tiles."
  field min Vector3 "The lowest corner of the box around its polygons." {
    unit m
  }
  field max Vector3 "The highest corner of the box around its polygons (its highest walkable floor's y)." {
    unit m
  }
  field cellSize f32 "Its voxel width." {
    unit m
  }
  field profile String "The agent profile it was baked for (og.nav.load's profile); empty for a navmesh of one unnamed agent and for og.nav.build."
}
```

<sub>`api.ogt` line 12452</sub>

<a id="type-NavRebuild"></a>

### `NavRebuild` (struct)

What an obstacle change rebuilt.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tiles` | `readonly number[]` | `[]` | — | — | — | The tiles rebuilt, by index. |
| `polygons` | `number` | `0` | — | — | — | The navmesh's polygons now. |

```ogt
struct NavRebuild {
  doc "What an obstacle change rebuilt."
  field tiles List.u32 "The tiles rebuilt, by index."
  field polygons u32 "The navmesh's polygons now."
}
```

<sub>`api.ogt` line 12469</sub>

## Column kinds

Each is a factory, `Navigation(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Navigation"></a>`Navigation` | `Navigation` | `Uint32Array` | 2 | A Navigation handle field. |

```ogt
column Navigation {
  doc "A Navigation handle field."
  array Uint32Array
  stride 2
  value Navigation
}
```

<sub>`api.ogt` line 11989</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-NAV_IDLE"></a>`NAV_IDLE` | `number` | `0` | NavAgent.status: not seeking. |
| <a id="const-NAV_MOVING"></a>`NAV_MOVING` | `number` | `1` | NavAgent.status: following a path or a flow field. |
| <a id="const-NAV_ARRIVED"></a>`NAV_ARRIVED` | `number` | `2` | NavAgent.status: within stopDistance of the target. |
| <a id="const-NAV_UNREACHABLE"></a>`NAV_UNREACHABLE` | `number` | `3` | NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh. |
| <a id="const-NAV_PARTIAL"></a>`NAV_PARTIAL` | `number` | `4` | NavAgent.status: the target is cut off; moving to the reachable point nearest it. |
| <a id="const-NAV_ON_LINK"></a>`NAV_ON_LINK` | `number` | `5` | NavAgent.status: crossing an off-mesh link (a jump, a ladder, a door, a teleport). |

```ogt
const NAV_IDLE u32 0 "NavAgent.status: not seeking."
```

<sub>`api.ogt` line 11996</sub>

```ogt
const NAV_MOVING u32 1 "NavAgent.status: following a path or a flow field."
```

<sub>`api.ogt` line 11997</sub>

```ogt
const NAV_ARRIVED u32 2 "NavAgent.status: within stopDistance of the target."
```

<sub>`api.ogt` line 11998</sub>

```ogt
const NAV_UNREACHABLE u32 3 "NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh."
```

<sub>`api.ogt` line 11999</sub>

```ogt
const NAV_PARTIAL u32 4 "NavAgent.status: the target is cut off; moving to the reachable point nearest it."
```

<sub>`api.ogt` line 12000</sub>

```ogt
const NAV_ON_LINK u32 5 "NavAgent.status: crossing an off-mesh link (a jump, a ladder, a door, a teleport)."
```

<sub>`api.ogt` line 12001</sub>
