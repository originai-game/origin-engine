<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`transform`](../transform.md) · [index](../index.md)

<a id="fn-decompose"></a>

### `og.transform.decompose`

Position, rotation and scale from a matrix, as three.js Matrix4.decompose.

```ts
og.transform.decompose(matrix: Matrix4): Transform
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `matrix` | `Matrix4` | no | An affine matrix. |

**Returns** `Transform`.

Binding symbol `og_transform_decompose` · publish builds: kept

```ogt
fn decompose {
  doc "Position, rotation and scale from a matrix, as three.js Matrix4.decompose."
  param matrix Matrix4 "An affine matrix."
  returns Transform
}
```

<sub>`api.ogt` line 1573</sub>
