<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

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

<sub>`api.ogt` line 13843</sub>
