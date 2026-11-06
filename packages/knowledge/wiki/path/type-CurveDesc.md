<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

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

<sub>`api.ogt` line 25916</sub>
