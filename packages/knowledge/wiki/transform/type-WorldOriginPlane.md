<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="type-WorldOriginPlane"></a>

### `WorldOriginPlane` (enum)

The axes og.WorldOrigin moves the world's zero along.

A string: one of the variants below. The zero value is `"xz"`.

| Variant | Doc |
|---|---|
| `"xz"` | x and z: the ground plane of a 3D world (heights stay absolute). |
| `"xyz"` | x, y and z: a world that is also deep or high (space, flight). |
| `"xy"` | x and y: a 2D world (its z, the drawing order, stays). |

```ogt
enum WorldOriginPlane {
  doc "The axes og.WorldOrigin moves the world's zero along."
  variant xz "x and z: the ground plane of a 3D world (heights stay absolute)."
  variant xyz "x, y and z: a world that is also deep or high (space, flight)."
  variant xy "x and y: a 2D world (its z, the drawing order, stays)."
}
```

<sub>Module `transform` line 48</sub>
