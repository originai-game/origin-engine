<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavShapeKind"></a>

### `NavShapeKind` (enum)

What a NavShape is.

A string: one of the variants below. The zero value is `"box"`.

| Variant | Doc |
|---|---|
| `"box"` | An axis-aligned box from `min` to `max`. |
| `"orientedBox"` | A box of size `size` round `center`, turned `yaw` radians about +y. |
| `"cylinder"` | An upright cylinder of `radius` and `height` standing on `center`. |
| `"convex"` | A convex prism: `points` (x and z, either winding) from `min.y` to `max.y`. |

```ogt
enum NavShapeKind {
  doc "What a NavShape is."
  variant box "An axis-aligned box from `min` to `max`."
  variant orientedBox "A box of size `size` round `center`, turned `yaw` radians about +y."
  variant cylinder "An upright cylinder of `radius` and `height` standing on `center`."
  variant convex "A convex prism: `points` (x and z, either winding) from `min.y` to `max.y`."
}
```

<sub>`api.ogt` line 13836</sub>
