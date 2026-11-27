<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="type-SightPlane"></a>

### `SightPlane` (enum)

The plane an og.SightMap lies in.

A string: one of the variants below. The zero value is `"ground"`.

| Variant | Doc |
|---|---|
| `"ground"` | The ground of a 3D world: x and z. The fog is drawn on each pixel's surface, whatever its height. |
| `"flat"` | A 2D world's plane: x and y. |

```ogt
enum SightPlane {
  doc "The plane an og.SightMap lies in."
  variant ground "The ground of a 3D world: x and z. The fog is drawn on each pixel's surface, whatever its height."
  variant flat "A 2D world's plane: x and y."
}
```

<sub>Module `sight` line 5</sub>
