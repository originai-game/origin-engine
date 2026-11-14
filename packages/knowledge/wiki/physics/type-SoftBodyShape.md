<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-SoftBodyShape"></a>

### `SoftBodyShape` (enum)

What a soft body is built as: SoftBody.shape.

A string: one of the variants below. The zero value is `"sphere"`.

| Variant | Doc |
|---|---|
| `"sphere"` | A sphere of `radius` (an icosphere; 2D a ring). |
| `"box"` | A box of `size` (each face a grid of `detail` + 1 cells; 2D a rectangle). |
| `"mesh"` | The closed surface of geometry `mesh` (3D only). |

```ogt
enum SoftBodyShape {
  doc "What a soft body is built as: SoftBody.shape."
  variant sphere "A sphere of `radius` (an icosphere; 2D a ring)."
  variant box "A box of `size` (each face a grid of `detail` + 1 cells; 2D a rectangle)."
  variant mesh "The closed surface of geometry `mesh` (3D only)."
}
```

<sub>Module `physics` line 1256</sub>
