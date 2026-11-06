<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `nav` module

Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch.

Agents are entities with an og.NavAgent (and an og.Transform): the og.nav system, after the game's systems and before physics, plans their paths, follows them and adds the og.Steering behaviours; an entity with an og.Velocity (a physics body) gets its velocity written, any other its Transform moved. Everything is deterministic and in snapshots: paths, grid edits, obstacles and flow targets.

Agents move on the profile's plane: x and y in 2D (a grid's rows run up), x and z in 3D (y kept on grids, from the surface on a navmesh).

- **TypeScript**: functions are the namespace `og.nav`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module nav {
  doc "Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch."
  doc "Agents are entities with an og.NavAgent (and an og.Transform): the og.nav system, after the game's systems and before physics, plans their paths, follows them and adds the og.Steering behaviours; an entity with an og.Velocity (a physics body) gets its velocity written, any other its Transform moved. Everything is deterministic and in snapshots: paths, grid edits, obstacles and flow targets."
  doc "Agents move on the profile's plane: x and y in 2D (a grid's rows run up), x and z in 3D (y kept on grids, from the surface on a navmesh)."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 13543</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Navigation`](nav/type-Navigation.md) | handle |  | A grid, a navmesh or a flow field agents plan on. |
| [`Navigation()`](#column-Navigation) | column kind |  | A Navigation handle field. |
| [`NAV_IDLE`](#const-NAV_IDLE) | constant |  | NavAgent.status: not seeking. |
| [`NAV_MOVING`](#const-NAV_MOVING) | constant |  | NavAgent.status: following a path or a flow field. |
| [`NAV_ARRIVED`](#const-NAV_ARRIVED) | constant |  | NavAgent.status: within stopDistance of the target. |
| [`NAV_UNREACHABLE`](#const-NAV_UNREACHABLE) | constant |  | NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh. |
| [`NAV_PARTIAL`](#const-NAV_PARTIAL) | constant |  | NavAgent.status: the target is cut off; moving to the reachable point nearest it. |
| [`NAV_ON_LINK`](#const-NAV_ON_LINK) | constant |  | NavAgent.status: crossing an off-mesh link (a jump, a ladder, a door, a teleport). |
| [`NavAlgorithm`](nav/type-NavAlgorithm.md) | enum |  | The grid search a path query runs. |
| [`og.NavAgent`](nav/record-NavAgent.md) | component |  | An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). |
| [`og.Steering`](nav/record-Steering.md) | component |  | Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics ray… |
| [`og.NavFilter`](nav/record-NavFilter.md) | component |  | Which navmesh areas an agent crosses and what a metre in each costs: beside an og.NavAgent, its plans weigh every polygon's length by its area's cost and never enter an excluded area. |
| [`og.Crowd`](nav/record-Crowd.md) | component |  | Reciprocal crowd avoidance (ORCA) for an agent: beside an og.NavAgent, each tick its velocity is the one nearest what its path and steering want that keeps it clear of its nearest agents for `horizon… |
| [`NavLinkKind`](nav/type-NavLinkKind.md) | enum |  | How an agent crosses an off-mesh link. |
| [`NavLinkDesc`](nav/type-NavLinkDesc.md) | desc struct |  | An off-mesh link: a way between two points of a navmesh its surface does not join (a gap to jump, a ladder, a door, a teleporter). |
| [`og.NavLinkReached`](nav/record-NavLinkReached.md) | event |  | An agent reached an off-mesh link and started across it (systems read it the tick after): play the jump or the climb here. |
| [`NavShapeKind`](nav/type-NavShapeKind.md) | enum |  | What a NavShape is. |
| [`NavShape`](nav/type-NavShape.md) | desc struct |  | A volume over a navmesh's floor: an obstacle (og.nav.carve) or an area (og.nav.setArea). |
| [`NavAreaDesc`](nav/type-NavAreaDesc.md) | desc struct |  | A run-time area: the spans under `shape` (their floors from the agent's climb below its bottom to its top) take area `area`, over the baked areas and over run-time areas of lower ids. |
| [`NavGridDesc`](nav/type-NavGridDesc.md) | desc struct |  | A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up. |
| [`TilemapGridDesc`](nav/type-TilemapGridDesc.md) | desc struct |  | A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile. |
| [`NavCell`](nav/type-NavCell.md) | struct |  | A grid cell. |
| [`PathDesc`](nav/type-PathDesc.md) | desc struct |  | A path query. |
| [`NavPath`](nav/type-NavPath.md) | struct |  | A path. |
| [`FlowDesc`](nav/type-FlowDesc.md) | desc struct |  | A flow field over a grid. |
| [`NavBox`](nav/type-NavBox.md) | desc struct |  | An axis-aligned box. |
| [`NavMeshDesc`](nav/type-NavMeshDesc.md) | desc struct |  | A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for. |
| [`NavMeshInfo`](nav/type-NavMeshInfo.md) | struct |  | A navmesh's shape. |
| [`NavRebuild`](nav/type-NavRebuild.md) | struct |  | What an obstacle change rebuilt. |
| [`og.nav.grid`](nav/fn-grid.md) | function | `(world: World, desc?: NavGridDesc): Result<Navigation>` | A grid agents and path queries plan on. |
| [`og.nav.gridFromTilemap`](nav/fn-gridFromTilemap.md) | function | `(world: World, desc?: TilemapGridDesc): Result<Navigation>` | A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. |
| [`og.nav.setCost`](nav/fn-setCost.md) | function | `(world: World, grid: Navigation, x: number, y: number, cost: number): Result<void>` | Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild. |
| [`og.nav.cellAt`](nav/fn-cellAt.md) | function | `(world: World, grid: Navigation, point: Vector3): NavCell` | The grid cell under a world point. |
| [`og.nav.findPath`](nav/fn-findPath.md) | function | `(world: World, desc?: PathDesc): Result<NavPath>` | A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end. |
| [`og.nav.flowField`](nav/fn-flowField.md) | function | `(world: World, desc?: FlowDesc): Result<Navigation>` | A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. |
| [`og.nav.setFlowTargets`](nav/fn-setFlowTargets.md) | function | `(world: World, field: Navigation, targets: readonly Vector3[]): Result<void>` | Move a flow field's targets (rebuilds it). |
| [`og.nav.flowAt`](nav/fn-flowAt.md) | function | `(world: World, field: Navigation, point: Vector3): Vector3` | The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable). |
| [`og.nav.load`](nav/fn-load.md) | function | `(world: World, path: string, profile?: string): Result<NavMeshInfo>` | A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes) baked by og cook. |
| [`og.nav.build`](nav/fn-build.md) | function | `(world: World, desc?: NavMeshDesc): Result<NavMeshInfo>` | A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load). |
| [`og.nav.setObstacle`](nav/fn-setObstacle.md) | function | `(world: World, mesh: Navigation, id: number, obstacle?: NavBox): Result<NavRebuild>` | Set, move or (with no box) remove an obstacle on a navmesh: the spans under the box (grown by the agent radius) are blocked and only the tiles it touches, and their neighbours, are rebuilt. |
| [`og.nav.carve`](nav/fn-carve.md) | function | `(world: World, mesh: Navigation, id: number, shape?: NavShape): Result<NavRebuild>` | og.nav.setObstacle for any shape: a turned box (a door ajar, a fallen log), a cylinder (a pillar, a barrel) or a convex prism. |
| [`og.nav.setArea`](nav/fn-setArea.md) | function | `(world: World, mesh: Navigation, id: number, desc?: NavAreaDesc): Result<NavRebuild>` | Set, move or (with no desc) remove a run-time area on a navmesh (water, mud, a road): its tiles rebuild so no polygon mixes areas, and agents re-plan, weighing it by their og.NavFilter. |
| [`og.nav.setLink`](nav/fn-setLink.md) | function | `(world: World, mesh: Navigation, id: number, desc?: NavLinkDesc): Result<boolean>` | Set, move or (with no desc) remove an off-mesh link on a navmesh. |
| [`og.nav.setLinkOpen`](nav/fn-setLinkOpen.md) | function | `(world: World, mesh: Navigation, id: number, open: boolean): Result<void>` | Open or close an off-mesh link (a door): a closed one is never planned across, and agents on the mesh re-plan. |
| [`og.nav.finishLink`](nav/fn-finishLink.md) | function | `(world: World, entity: Entity): boolean` | End a crossing the game moves (NavLinkDesc.game): the agent lands at the link's end as og.nav next runs and goes on along its path. |
| [`og.nav.nearest`](nav/fn-nearest.md) | function | `(world: World, mesh: Navigation, point: Vector3, extent?: Vector3): Vector3 \| null` | The point on a navmesh nearest a world point, within `extent` (half sizes). |
| [`og.nav.pathOf`](nav/fn-pathOf.md) | function | `(world: World, entity: Entity): NavPath \| null` | The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same. |

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

<sub>`api.ogt` line 13550</sub>

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

<sub>`api.ogt` line 13557</sub>

```ogt
const NAV_MOVING u32 1 "NavAgent.status: following a path or a flow field."
```

<sub>`api.ogt` line 13558</sub>

```ogt
const NAV_ARRIVED u32 2 "NavAgent.status: within stopDistance of the target."
```

<sub>`api.ogt` line 13559</sub>

```ogt
const NAV_UNREACHABLE u32 3 "NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh."
```

<sub>`api.ogt` line 13560</sub>

```ogt
const NAV_PARTIAL u32 4 "NavAgent.status: the target is cut off; moving to the reachable point nearest it."
```

<sub>`api.ogt` line 13561</sub>

```ogt
const NAV_ON_LINK u32 5 "NavAgent.status: crossing an off-mesh link (a jump, a ladder, a door, a teleport)."
```

<sub>`api.ogt` line 13562</sub>
