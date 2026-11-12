<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-TubeGeometryDesc"></a>

### `TubeGeometryDesc` (desc struct)

A tube along a curve, as three.js TubeGeometry(curve, tubularSegments, radius, radialSegments, closed): its rings are evenly spaced along the curve's length (three.js getPointAt), u runs along it, v around. The curve is `path` when set (an og.path, whatever its kind, followed exactly: a scene's or a timeline's path becomes a tube), otherwise new CatmullRomCurve3(points, closed, 'centripetal') through `points`, which may come from anywhere: a rope's points, a spline a tool made.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | `m` | — | The points the curve passes through, in order (at least two), when `path` is 0. |
| `path` | `Path` | `0` | — | — | — | A path (og.path.create, or a scene's `path` by og.path.find) the tube follows instead of the points: its own curve, kind and closed flag (`closed` is the path's then). 0 uses the points. |
| `tubularSegments` | `number` | `64` | ≥ 1 | — | — | Segments along the tube. |
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The tube's radius. |
| `radialSegments` | `number` | `8` | ≥ 3 | — | — | Segments around it. |
| `closed` | `boolean` | `false` | — | — | — | Join the last point back to the first. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |

```ogt
struct TubeGeometryDesc {
  doc "A tube along a curve, as three.js TubeGeometry(curve, tubularSegments, radius, radialSegments, closed): its rings are evenly spaced along the curve's length (three.js getPointAt), u runs along it, v around. The curve is `path` when set (an og.path, whatever its kind, followed exactly: a scene's or a timeline's path becomes a tube), otherwise new CatmullRomCurve3(points, closed, 'centripetal') through `points`, which may come from anywhere: a rope's points, a spline a tool made."
  desc true
  field points List.Vector3 "The points the curve passes through, in order (at least two), when `path` is 0." {
    unit m
  }
  field path Path "A path (og.path.create, or a scene's `path` by og.path.find) the tube follows instead of the points: its own curve, kind and closed flag (`closed` is the path's then). 0 uses the points."
  field tubularSegments u32 "Segments along the tube." {
    default 64
    min 1
  }
  field radius f32 "The tube's radius." {
    default 1
    min 0
    unit m
  }
  field radialSegments u32 "Segments around it." {
    default 8
    min 3
  }
  field closed bool "Join the last point back to the first."
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
}
```

<sub>Module `render` line 3320</sub>
