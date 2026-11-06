<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

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

<sub>`api.ogt` line 26029</sub>
