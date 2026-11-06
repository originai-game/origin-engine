<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-CameraRail"></a>

### `og.CameraRail` (component)

Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems.

Token `og.CameraRail`, registered under the schema name `og.CameraRail`; its value type is `CameraRail`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `number` | `2` | 2 … 8 | — | — | How many of p0...p7 the rail uses (2 to 8). |
| `p0` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 0. |
| `p1` | `Vector3` | `{ x: 10, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 1. |
| `p2` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 2. |
| `p3` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 3. |
| `p4` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 4. |
| `p5` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 5. |
| `p6` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 6. |
| `p7` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Rail point 7. |
| `smoothTime` | `number` | `0.2` | ≥ 0 | `s` | tunable | Roughly how long the camera takes to catch up along the rail; 0 snaps. |
| `path` | `Path` | `0` | — | — | — | A path (og.path.create, or a scene's `path` by og.path.find) the rail follows instead of p0 to p(points - 1): any number of points, catmullRom, bezier or linear, closed or not. 0 uses the points. |

```ogt
component CameraRail {
  doc "Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems."
  field points u32 "How many of p0...p7 the rail uses (2 to 8)." {
    default 2
    min 2
    max 8
  }
  field p0 Vector3 "Rail point 0." {
    worldspace world
    unit m
  }
  field p1 Vector3 "Rail point 1." {
    worldspace world
    default 10 0 0
    unit m
  }
  field p2 Vector3 "Rail point 2." {
    worldspace world
    unit m
  }
  field p3 Vector3 "Rail point 3." {
    worldspace world
    unit m
  }
  field p4 Vector3 "Rail point 4." {
    worldspace world
    unit m
  }
  field p5 Vector3 "Rail point 5." {
    worldspace world
    unit m
  }
  field p6 Vector3 "Rail point 6." {
    worldspace world
    unit m
  }
  field p7 Vector3 "Rail point 7." {
    worldspace world
    unit m
  }
  field smoothTime f32 "Roughly how long the camera takes to catch up along the rail; 0 snaps." {
    default 0.2
    min 0
    unit s
    tunable true
  }
  field path Path "A path (og.path.create, or a scene's `path` by og.path.find) the rail follows instead of p0 to p(points - 1): any number of points, catmullRom, bezier or linear, closed or not. 0 uses the points."
}
```

<sub>`api.ogt` line 3526</sub>
