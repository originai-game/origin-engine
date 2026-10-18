# og-nav

Navigation for the engine (`docs/design.md` §2.7): pathfinding on grids
and on a navmesh, flow fields for crowds, steering behaviours, and the
`og.nav` system that moves agents. Pure Rust over og-core and og-ecs;
builds for `wasm32-unknown-unknown`. The voxeliser that turns a level's
triangles into the walkable heightfield is behind the `bake` feature, which
only og-cook enables: the wasm never carries it.

```rust
let mut navs = og_nav::Navs::default();
let grid = navs.insert(og_nav::Nav::Grid(og_nav::Grid::new(256, 256, 1.0, Vec2::ZERO)));
let nav = og_nav::install(&mut world, Arc::new(Mutex::new(navs)), og_nav::NavDesc { plane: og_nav::Plane::Xy, raycast: None, plans_per_tick: og_nav::PLANS_PER_TICK })?;
world.spawn((Transform::default(), og_nav::NavAgent { nav: grid, target, seeking: true, ..Default::default() }))?;
world.step()?;   // the agent plans (jump point search), follows the corners, arrives
```

## API map

| Module | Entry points |
|---|---|
| `grid` | `Grid` (`new`, `from_costs`, `set`, `cost`, `walkable`, `can_step`, `cell_at`, `center`, `nearest_walkable`, `find_path`, `line_of_sight`, `waypoints`), `Algorithm::{AStar, Jps}`, `GridPath`, `Scratch`, `BLOCKED`, `ORTHO`/`DIAG` |
| `flow` | `FlowField::build(&grid, targets)`, `sample`, `cost_at`; `NONE` |
| `mesh` | `NavParams`, `Heightfield` (`flat`, `without`, `column`, `connected`), `NavMesh` (`build`, `from_tiles`, `build_tile`, `find_path`, `funnel`, `nearest`, `move_along`, `set_obstacle`, `rebuild_region`, `poly`, `height_at`, `closest_on`), `Tile`, `Poly`, `PolyRef`, `MeshPath`, `Obstacle` |
| `bake` (feature) | `bake(triangles, &params, bounds) -> Heightfield`, `box_triangles` |
| `steer` | `seek`, `flee`, `arrive`, `predict`, `pursue`, `evade`, `wander` (`WanderCircle`), `separation`, `avoid` (`Feeler`), `accelerate` |
| `system` | `NavAgent`, `Steering` (records), `Transform`/`Velocity` mirrors, `Nav`, `Navs`, `SharedNavs`, `Plane`, `NavDesc`, `Raycaster`, `install`, `plan`, `NavSystem::path_of`, the `STATUS_*` codes, `SYSTEM_NAME` = `og.nav` |

## Grids

- Costs are bytes: 0 blocked, 1 normal, up to 255 for a cell to avoid
  (swamp, fire). Eight-way moves; a diagonal needs both orthogonal cells
  open, so no path squeezes between two walls touching at a corner.
- Integer costs: an orthogonal step is 1000 × the entered cell's cost, a
  diagonal 1414 ×; the heuristic is the octile distance. The open list
  orders by (f, h, insertion), so every host expands the same nodes.
- **Jump point search** (the variant without corner cutting) answers the
  same optimal cost as A* while expanding only jump points; on a grid with
  weighted cells it runs A* (`GridPath.algorithm` says which ran).
- `waypoints` drops collinear cells and, with `smooth`, string-pulls by line
  of sight (an Amanatides–Woo walk that refuses to pass a corner between two
  cells unless both are open), optionally with a clearance (two offset
  lines) so a wide agent does not shave a wall.

## Flow fields

`FlowField::build` runs Dijkstra from every target cell over the reversed
moves: each cell gets its cost to the nearest target and the direction of
its cheapest neighbour. `sample(p)` blends the four nearest cells'
directions bilinearly (unreachable ones left out). One field serves any
number of agents with a lookup each.

## The navmesh

Built from a `Heightfield` (per column, the walkable floors and their
headroom), in tiles of `tile_size × tile_size` cells:

1. **Regions** — a row sweep gives every span a region; a run of connected
   spans inherits the region below it only when it is the only run touching
   it (Recast's monotone partition), so every region is one run per row and
   its outline a simple polygon without holes.
2. **Contours** — each region's outline on the cell lattice, every edge
   tagged with what lies across it (a wall, or a region of this or the
   neighbouring tile). Vertices stay where the outline turns or the tag
   changes; runs of wall edges are simplified by Douglas–Peucker
   (`max_error` cells) when the outline stays simple. A border vertex is
   kept by both tiles exactly when the region across changes, so the two
   sides share their vertices.
3. **Polygons** — ear clipping in integer lattice coordinates, then the
   triangles merged into convex polygons of at most 12 vertices (collinear
   vertices allowed), longest shared edge first.
4. **Links** — polygons sharing an edge, by lattice coordinates and within
   the climb in height, are neighbours, inside and across tiles.

**Queries.** `find_path` takes the nearest polygons to both ends, runs A*
over polygons through edge midpoints (a partial path to the reachable
polygon nearest the end when the end is cut off) and pulls the corridor
taut with the simple stupid funnel algorithm (collinear portal points count
as inside, so a straight line through tile corners stays straight).
`move_along` walks a segment across linked edges and slides along walls; a
crossing on collinear edges takes the edge that holds the point.

**Runtime rebuilds.** `set_obstacle(id, Some(box))` blocks the spans under
the box (grown by the agent radius), recomputes the regions of the tiles it
touches and rebuilds those tiles and their four neighbours (whose border
vertices follow the regions across); `None` removes it, and the mesh is
then byte-for-byte what it was. `rebuild_region` rebuilds the tiles over a
box after the level changed there.

**The bake** (`bake`, cook time only): every triangle is rasterised into the
columns it covers as a solid span (Sutherland–Hodgman clipping per row and
column), walkable when no steeper than `max_slope`; overlapping spans merge
with the walkable flag kept when their tops are within the climb. Then the
filters: low-hanging obstacles (a kerb, a step) become walkable, ledges and
spans whose reachable neighbours differ by more than the climb do not, nor
spans with less headroom than the agent; the walkable tops become floors,
eroded by the agent radius (a two-pass chamfer distance).

## Steering and agents

`og.NavAgent` (component): `nav` (a grid, navmesh or flow field handle; 0
moves straight), `target`, `seeking`, `speed`, `acceleration`, `radius`,
`stopDistance`, `slowRadius`, `repathTicks`, `repathDistance`, `smooth`,
`faceMovement`, `turnSpeed`; written by og.nav: `status` (`STATUS_IDLE`,
`MOVING`, `ARRIVED`, `UNREACHABLE`, `PARTIAL`), `velocity`, `waypoint`,
`remaining`, `plans`.

`og.Steering` (component): weights and settings of `seek` (the path),
`flee` (`fleeFrom`, `fleeRadius`), `wander` (`wanderRadius`,
`wanderDistance`, `wanderJitter`; `wanderAngle` written), `pursue` and
`evade` (entities, their predicted positions from their NavAgent or
og.Velocity), `separation` (`separationRadius`) and `avoidance`
(`avoidDistance`, `avoidMask`: three physics rays through the host's
`Raycaster`).

Per tick, in query order: re-plan when the target moved more than
`repathDistance`, every `repathTicks`, or when the grid or mesh changed (at
most `plans_per_tick` plans a tick across agents, the rest the next ticks);
follow the corners (a corner is passed within reach, or once the agent is
past it towards the next one), arriving at the last; add the steering
behaviours by weight; turn the velocity towards the result by at most
`acceleration · dt`, cap it at `speed`. An entity with an `og.Velocity`
gets its velocity written (physics moves it, so walls hold it); any other
moves its `og.Transform`, along the surface on a navmesh. The time is the
scaled tick: a hit stop holds agents. Wander noise is a hash of the entity
and the tick.

The plane is the profile's: x and y in 2D, x and z in 3D (y kept on grids,
from the surface on a navmesh).

**State.** The `og.nav` state block holds every agent's path (corners, the
next one, what it was planned for, its age, its navmesh polygon), every
grid's costs, every navmesh's obstacles and every flow field's targets, so
snapshots restore edits made at runtime and the world hash covers them;
loaded grids and meshes are assets, not state.

## Costs

`cargo run --release -p og-nav --example nav_costs` (Apple M4 Pro, one
thread; `tests/evidence/E6/nav-costs.md` has the full table):

| | |
|---|---|
| 256 × 256 grid, 12 % walls: A* / JPS per random query | 0.31 ms / 0.19 ms (1,884 / 977 nodes) |
| flow field over the 256 × 256 grid | 3.5 ms |
| 200 agents on the grid, a tick (separation on) | 0.25 ms; the tick they all plan 4.9 ms (32 plans) |
| 200 agents on a flow field, a tick | 0.18 ms |
| 64 × 64 m arena (1,512 triangles): bake / build | 12.4 ms / 5.6 ms, 401 polygons in 64 tiles |
| navmesh query | 0.023 ms (82 polygons expanded, 7 corners) |
| an obstacle's rebuild | 0.84 ms (12 tiles) |
| 200 agents on the navmesh, a tick | 0.22 ms |

## Tests

`cargo test -p og-nav` (`tests/nav.rs`, through the public API; each was
broken once and failed, `tests/evidence/E6/break-and-restore.md`):

| Test | Asserts |
|---|---|
| `jump_point_search_finds_paths_as_short_as_astar_and_never_cuts_a_corner` | 79 random mazes: equal costs, JPS expands fewer nodes, every step legal; no squeezing between diagonal walls |
| `weighted_cells_are_avoided_when_a_detour_is_cheaper_and_jps_falls_back_to_astar` | a costly strip is skirted, crossed when there is no way round; JPS on a weighted grid runs A* |
| `smoothed_grid_paths_keep_line_of_sight_and_open_rooms_are_one_segment` | an empty room is one segment; around a wall every segment has line of sight and no sample is in a wall |
| `a_flow_field_leads_every_reachable_cell_downhill_to_a_target` | following directions from every reachable cell lowers the cost each step and reaches a target |
| `a_baked_navmesh_routes_around_walls_up_ramps_and_stays_on_the_surface` | convex counter-clockwise polygons; a path through the wall's gap, on the surface, never inside the wall; a platform reached by its ramp at its height; the same heightfield builds the same tiles |
| `an_obstacle_rebuilds_only_its_tiles_and_removing_it_restores_them` | closing a gap makes the far side unreachable (a partial path), rebuilds only tiles near the box, leaves the others' polygons alone; removal restores the mesh exactly |
| `moving_along_the_surface_slides_on_walls_and_never_leaves_the_mesh` | a move into a wall stops before it and keeps the motion along it |
| `two_hundred_agents_cross_a_256_grid_and_replay_identically_after_a_snapshot` | 200 agents cross a maze; two runs hash alike; a world restored at tick 301 ends identical |
| `separation_spreads_a_pile_of_agents_and_wander_is_deterministic` | 20 agents spawned on one point spread apart; two runs hash alike; 3D agents keep their height |
| `an_agent_walks_a_navmesh_around_the_wall_and_arrives_on_the_surface` | an agent goes round through the gap, never inside the wall or off the surface, and arrives |
