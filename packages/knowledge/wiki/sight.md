<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `sight` module

Fog of war: which ground each team sees now and has seen before. An og.SightMap lays a grid over the og.Volume on its entity; every og.Sight is a team's eyes, and og.SightBlocker regions stop them. The map is world state, cast again as each tick starts: saved, restored and hashed with the world, the same on every host. The game asks it about a point (og.sight.visible, og.sight.explored), and the renderer draws the same cells over the frame.

See `world §11`.

- **TypeScript**: functions are the namespace `og.sight`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module sight {
  doc "Fog of war: which ground each team sees now and has seen before. An og.SightMap lays a grid over the og.Volume on its entity; every og.Sight is a team's eyes, and og.SightBlocker regions stop them. The map is world state, cast again as each tick starts: saved, restored and hashed with the world, the same on every host. The game asks it about a point (og.sight.visible, og.sight.explored), and the renderer draws the same cells over the frame."
  # declarations follow, each on its own page
```

<sub>Module `sight` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`SightPlane`](sight/type-SightPlane.md) | enum |  | The plane an og.SightMap lies in. |
| [`SightPlane()`](#column-SightPlane) | column kind |  | A SightPlane field: the variant's index. |
| [`og.SightMap`](sight/record-SightMap.md) | component |  | Fog of war over a rectangle of the ground: the og.Volume on its entity gives its extent (its box's x and z, or x and y on the flat plane; none: no map), cut into cells. |
| [`og.Sight`](sight/record-Sight.md) | component |  | A team's eyes on the og.SightMap: from its entity's world position it sees every cell within `radius` that no og.SightBlocker hides (symmetric: between two open cells, one sees the other exactly when… |
| [`og.SightBlocker`](sight/record-SightBlocker.md) | component |  | Stops sight on the og.SightMap: the cells whose centre lies inside the og.Volume on its entity (a box or a sphere, at its entity's pose, cut at its own height) hide what is behind them. |
| [`og.sight.visible`](sight/fn-visible.md) | function | `(world: World, team: number, point: Vector3): Result<boolean>` | Whether a team sees a point this tick: the point's cell of the og.SightMap is within one of the team's og.Sight and not hidden. |
| [`og.sight.explored`](sight/fn-explored.md) | function | `(world: World, team: number, point: Vector3): Result<boolean>` | Whether a team has ever seen a point (or had it revealed): its cell of the og.SightMap is explored. |
| [`og.sight.reveal`](sight/fn-reveal.md) | function | `(world: World, team: number, point: Vector3, radius: number): Result<void>` | Mark the ground within a radius of a point as explored by a team without anyone seeing it: a map bought, a scout's report, the start area. |
| [`og.sight.forget`](sight/fn-forget.md) | function | `(world: World, team: number): Result<void>` | Forget what a team explored: its map is unexplored again but for what it sees this tick (a new round, a memory wipe). |

## Column kinds

Each is a factory, `SightPlane(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-SightPlane"></a>`SightPlane` | `SightPlane` | `Uint32Array` | 1 | A SightPlane field: the variant's index. |

```ogt
column SightPlane {
  doc "A SightPlane field: the variant's index."
  array Uint32Array
  stride 1
  value SightPlane
}
```

<sub>Module `sight` line 11</sub>
