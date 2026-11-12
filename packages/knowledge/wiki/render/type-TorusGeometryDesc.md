<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-TorusGeometryDesc"></a>

### `TorusGeometryDesc` (desc struct)

A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | From the centre to the tube's centre. |
| `tube` | `number` | `0.4` | ≥ 0 | `m` | — | The tube's radius. |
| `radialSegments` | `number` | `12` | ≥ 2 | — | — | Segments around the tube. |
| `tubularSegments` | `number` | `48` | ≥ 3 | — | — | Segments around the ring. |
| `arc` | `number` | `6.283185307179586` | — | `rad` | — | How far around the ring goes. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |

```ogt
struct TorusGeometryDesc {
  doc "A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc)."
  desc true
  field radius f32 "From the centre to the tube's centre." {
    default 1
    min 0
    unit m
  }
  field tube f32 "The tube's radius." {
    default 0.4
    min 0
    unit m
  }
  field radialSegments u32 "Segments around the tube." {
    default 12
    min 2
  }
  field tubularSegments u32 "Segments around the ring." {
    default 48
    min 3
  }
  field arc f32 "How far around the ring goes." {
    default 6.283185307179586
    unit rad
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
}
```

<sub>Module `render` line 3233</sub>
