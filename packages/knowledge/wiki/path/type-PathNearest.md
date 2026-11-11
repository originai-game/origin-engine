<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

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

<sub>Module `path` line 51</sub>
