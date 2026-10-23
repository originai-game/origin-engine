<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `path` module

Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed however its points are spaced. catmullRom passes through every point (three.js CatmullRomCurve3, tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...` (three.js CubicBezierCurve3 in a CurvePath), linear joins the points with straight segments. A closed path joins its last point to its first and wraps t.

Make one with og.path.create, or declare it in a scene (`path "<name>" { kind catmullRom; closed false; point x y z ... }`) and find it by name with og.path.find; a timeline's camera rides one with `rail "<name>"`. og.PathFollower moves an entity along one, og.CameraRail.path moves the camera along one, og.render.tubeGeometry makes a tube along one (TubeGeometryDesc.path). Paths are assets: they are not world state, and a hot reload registers them again.

- **TypeScript**: functions are the namespace `og.path`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module path {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 13494</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Path`](#type-Path) | handle | A path: its points, kind and arc-length table. |
| [`Path()`](#column-Path) | column kind | A Path handle field (0 is none). |
| [`PathKind`](#type-PathKind) | enum | How the points make the curve. |
| [`PathKind()`](#column-PathKind) | column kind | A PathKind field: the variant's index. |
| [`CurveDesc`](#type-CurveDesc) | desc struct | What makes a path (three.js's Curve; PathDesc is og.nav's path query). |
| [`PathSample`](#type-PathSample) | struct | A point of a path. |
| [`PathNearest`](#type-PathNearest) | struct | The point of a path nearest another point. |
| [`og.PathFollower`](#record-PathFollower) | component | Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there. |
| [`og.path.create`](#fn-create) | function | Make a path from points. |
| [`og.path.find`](#fn-find) | function | A path a loaded scene declares (`path "<name>" { ... }`), by name. |
| [`og.path.sample`](#fn-sample) | function | The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). Equal steps of t are equal lengths along the curve. |
| [`og.path.length`](#fn-length) | function | Its length along the curve. |
| [`og.path.nearest`](#fn-nearest) | function | The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail. |

## Functions

<a id="fn-create"></a>

### `og.path.create`

Make a path from points.

```ts
og.path.create(world: World, desc?: CurveDesc): Result<Path>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `CurveDesc` | yes | The points, kind, closed and tension. |

**Returns** `Result<Path>`: Refused for too few points for the kind (bezier needs 3n + 1 open, 3n closed) or a point that is not finite.

Binding symbol `og_path_create` · publish builds: kept

```ogt
fn create {
  doc "Make a path from points."
  param world World "The world."
  param desc CurveDesc "The points, kind, closed and tension."
  returns Result.Path "Refused for too few points for the kind (bezier needs 3n + 1 open, 3n closed) or a point that is not finite."
}
```

<sub>`api.ogt` line 13583</sub>

<a id="fn-find"></a>

### `og.path.find`

A path a loaded scene declares (`path "<name>" { ... }`), by name.

```ts
og.path.find(world: World, name: string): Result<Path>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The path's name in its scene. |

**Returns** `Result<Path>`: Refused for a name no spawned scene declares; the reason lists the names.

Binding symbol `og_path_find` · publish builds: kept

```ogt
fn find {
  doc "A path a loaded scene declares (`path \"<name>\" { ... }`), by name."
  param world World "The world."
  param name String "The path's name in its scene."
  returns Result.Path "Refused for a name no spawned scene declares; the reason lists the names."
}
```

<sub>`api.ogt` line 13590</sub>

<a id="fn-sample"></a>

### `og.path.sample`

The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). Equal steps of t are equal lengths along the curve.

```ts
og.path.sample(world: World, path: Path, t: number): Result<PathSample>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `Path` | no | The path. |
| `t` | `number` | no | The parameter, 0 to 1. |

**Returns** `Result<PathSample>`: Refused for a dead path.

Binding symbol `og_path_sample` · publish builds: kept

```ogt
fn sample {
  doc "The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). Equal steps of t are equal lengths along the curve."
  param world World "The world."
  param path Path "The path."
  param t f32 "The parameter, 0 to 1."
  returns Result.PathSample "Refused for a dead path."
}
```

<sub>`api.ogt` line 13597</sub>

<a id="fn-length"></a>

### `og.path.length`

Its length along the curve.

```ts
og.path.length(world: World, path: Path): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `Path` | no | The path. |

**Returns** `Result<number>`: Metres; refused for a dead path.

Binding symbol `og_path_length` · publish builds: kept

```ogt
fn length {
  doc "Its length along the curve."
  param world World "The world."
  param path Path "The path."
  returns Result.f32 "Metres; refused for a dead path."
}
```

<sub>`api.ogt` line 13605</sub>

<a id="fn-nearest"></a>

### `og.path.nearest`

The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail.

```ts
og.path.nearest(world: World, path: Path, point: Vector3): Result<PathNearest>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `Path` | no | The path. |
| `point` | `Vector3` | no | The point. |

**Returns** `Result<PathNearest>`: Refused for a dead path.

Binding symbol `og_path_nearest` · publish builds: kept

```ogt
fn nearest {
  doc "The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail."
  param world World "The world."
  param path Path "The path."
  param point Vector3 "The point."
  returns Result.PathNearest "Refused for a dead path."
}
```

<sub>`api.ogt` line 13612</sub>

## Components, resources and events

<a id="record-PathFollower"></a>

### `og.PathFollower` (component)

Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there.

Token `og.PathFollower`, registered under the schema name `og.PathFollower`; its value type is `PathFollower`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `Path` | `0` | — | — | — | The path (og.path.create or og.path.find); 0 does nothing. |
| `speed` | `number` | `1` | — | `mps` | tunable | Metres per second along the path; negative goes backwards. |
| `distance` | `number` | `0` | — | `m` | — | How far along the path it is, from its start; written every tick, set it to place the entity. |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the end (and the start, going backwards); otherwise stop there and set `finished`. |
| `align` | `boolean` | `false` | — | — | — | Turn og.Transform.quaternion to face along the tangent (local -Z forward, +Y the path's up). |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Added to the path's point: along the path's right, up and backward axes with align, else world x, y and z. |
| `playing` | `boolean` | `true` | — | — | — | Moves while true. |
| `finished` | `boolean` | `false` | — | — | read only | Written: a path without loop reached its end. |

```ogt
component PathFollower {
  doc "Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there."
  field path Path "The path (og.path.create or og.path.find); 0 does nothing."
  field speed f32 "Metres per second along the path; negative goes backwards." {
    default 1
    unit mps
    tunable true
  }
  field distance f32 "How far along the path it is, from its start; written every tick, set it to place the entity." {
    unit m
  }
  field loop bool "Wrap at the end (and the start, going backwards); otherwise stop there and set `finished`." {
    default true
  }
  field align bool "Turn og.Transform.quaternion to face along the tangent (local -Z forward, +Y the path's up)."
  field offset Vector3 "Added to the path's point: along the path's right, up and backward axes with align, else world x, y and z." {
    unit m
  }
  field playing bool "Moves while true." {
    default true
  }
  field finished bool "Written: a path without loop reached its end." {
    readonly true
  }
}
```

<sub>`api.ogt` line 13557</sub>

## Types

<a id="type-Path"></a>

### `Path` (handle)

A path: its points, kind and arc-length table.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Path "A path: its points, kind and arc-length table."
```

<sub>`api.ogt` line 13499</sub>

<a id="type-PathKind"></a>

### `PathKind` (enum)

How the points make the curve.

A string: one of the variants below. The zero value is `"catmullRom"`.

| Variant | Doc |
|---|---|
| `"catmullRom"` | Through every point, as three.js CatmullRomCurve3: tension scales the tangents (0.5 is Catmull-Rom). |
| `"bezier"` | Cubic Bezier segments: anchor, two controls, anchor, ... (3n + 1 points open, 3n closed). |
| `"linear"` | Straight segments between the points. |

```ogt
enum PathKind {
  doc "How the points make the curve."
  variant catmullRom "Through every point, as three.js CatmullRomCurve3: tension scales the tangents (0.5 is Catmull-Rom)."
  variant bezier "Cubic Bezier segments: anchor, two controls, anchor, ... (3n + 1 points open, 3n closed)."
  variant linear "Straight segments between the points."
}
```

<sub>`api.ogt` line 13508</sub>

<a id="type-CurveDesc"></a>

### `CurveDesc` (desc struct)

What makes a path (three.js's Curve; PathDesc is og.nav's path query).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | — | — | The points, in order: at least 2 (catmullRom, linear), 3n + 1 open or 3n closed (bezier). |
| `closed` | `boolean` | `false` | — | — | — | Join the last point to the first; t wraps. |
| `kind` | `PathKind` | `"catmullRom"` | — | — | — | How the points make the curve. |
| `tension` | `number` | `0.5` | 0 … 1 | — | — | catmullRom's tangent scale, as three.js's tension. |

```ogt
struct CurveDesc {
  doc "What makes a path (three.js's Curve; PathDesc is og.nav's path query)."
  desc true
  field points List.Vector3 "The points, in order: at least 2 (catmullRom, linear), 3n + 1 open or 3n closed (bezier)."
  field closed bool "Join the last point to the first; t wraps."
  field kind PathKind "How the points make the curve."
  field tension f32 "catmullRom's tangent scale, as three.js's tension." {
    default 0.5
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 13522</sub>

<a id="type-PathSample"></a>

### `PathSample` (struct)

A point of a path.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `tangent` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction of travel. |
| `up` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | A unit up perpendicular to the tangent: world +Y made perpendicular (world +Z where the path runs straight up or down). |

```ogt
struct PathSample {
  doc "A point of a path."
  field position Vector3 "Where it is." {
    unit m
  }
  field tangent Vector3 "The unit direction of travel."
  field up Vector3 "A unit up perpendicular to the tangent: world +Y made perpendicular (world +Z where the path runs straight up or down)."
}
```

<sub>`api.ogt` line 13535</sub>

<a id="type-PathNearest"></a>

### `PathNearest` (struct)

The point of a path nearest another point.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `t` | `number` | `0` | — | `ratio` | — | Its arc-length parameter, 0 to 1. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `distance` | `number` | `0` | — | `m` | — | How far it is from the point asked about. |

```ogt
struct PathNearest {
  doc "The point of a path nearest another point."
  field t f32 "Its arc-length parameter, 0 to 1." {
    unit ratio
  }
  field position Vector3 "Where it is." {
    unit m
  }
  field distance f32 "How far it is from the point asked about." {
    unit m
  }
}
```

<sub>`api.ogt` line 13544</sub>

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

<sub>`api.ogt` line 13501</sub>

```ogt
column PathKind {
  doc "A PathKind field: the variant's index."
  array Uint32Array
  stride 1
  value PathKind
}
```

<sub>`api.ogt` line 13515</sub>
