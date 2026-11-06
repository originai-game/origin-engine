<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `path` module

Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed however its points are spaced. catmullRom passes through every point (three.js CatmullRomCurve3, tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...` (three.js CubicBezierCurve3 in a CurvePath), linear joins the points with straight segments. A closed path joins its last point to its first and wraps t.

Make one with og.path.create, or declare it in a scene (`path "<name>" { kind catmullRom; closed false; point x y z ... }`) and find it by name with og.path.find; a timeline's camera rides one with `rail "<name>"`. og.PathFollower moves an entity along one, og.CameraRail.path moves the camera along one, og.render.tubeGeometry makes a tube along one (TubeGeometryDesc.path). Paths are assets: they are not world state, and a hot reload registers them again.

- **TypeScript**: functions are the namespace `og.path`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module path {
  doc "Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed however its points are spaced. catmullRom passes through every point (three.js CatmullRomCurve3, tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...` (three.js CubicBezierCurve3 in a CurvePath), linear joins the points with straight segments. A closed path joins its last point to its first and wraps t."
  doc "Make one with og.path.create, or declare it in a scene (`path \"<name>\" { kind catmullRom; closed false; point x y z ... }`) and find it by name with og.path.find; a timeline's camera rides one with `rail \"<name>\"`. og.PathFollower moves an entity along one, og.CameraRail.path moves the camera along one, og.render.tubeGeometry makes a tube along one (TubeGeometryDesc.path). Paths are assets: they are not world state, and a hot reload registers them again."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 26016</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Path`](path/type-Path.md) | handle |  | A path: its points, kind and arc-length table. |
| [`Path()`](#column-Path) | column kind |  | A Path handle field (0 is none). |
| [`PathKind`](path/type-PathKind.md) | enum |  | How the points make the curve. |
| [`PathKind()`](#column-PathKind) | column kind |  | A PathKind field: the variant's index. |
| [`CurveDesc`](path/type-CurveDesc.md) | desc struct |  | What makes a path (three.js's Curve; PathDesc is og.nav's path query). |
| [`PathSample`](path/type-PathSample.md) | struct |  | A point of a path. |
| [`PathNearest`](path/type-PathNearest.md) | struct |  | The point of a path nearest another point. |
| [`og.PathFollower`](path/record-PathFollower.md) | component |  | Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the pat… |
| [`og.path.create`](path/fn-create.md) | function | `(world: World, desc?: CurveDesc): Result<Path>` | Make a path from points. |
| [`og.path.find`](path/fn-find.md) | function | `(world: World, name: string): Result<Path>` | A path a loaded scene declares (`path "<name>" { ... |
| [`og.path.sample`](path/fn-sample.md) | function | `(world: World, path: Path, t: number): Result<PathSample>` | The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). |
| [`og.path.length`](path/fn-length.md) | function | `(world: World, path: Path): Result<number>` | Its length along the curve. |
| [`og.path.nearest`](path/fn-nearest.md) | function | `(world: World, path: Path, point: Vector3): Result<PathNearest>` | The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail. |

## Column kinds

Each is a factory, `Path(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Path"></a>`Path` | `Path` | `Uint32Array` | 2 | A Path handle field (0 is none). |
| <a id="column-PathKind"></a>`PathKind` | `PathKind` | `Uint32Array` | 1 | A PathKind field: the variant's index. |

```ogt
column Path {
  doc "A Path handle field (0 is none)."
  array Uint32Array
  stride 2
  value Path
}
```

<sub>`api.ogt` line 26022</sub>

```ogt
column PathKind {
  doc "A PathKind field: the variant's index."
  array Uint32Array
  stride 1
  value PathKind
}
```

<sub>`api.ogt` line 26036</sub>
