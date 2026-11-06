<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PointShadows"></a>

### `PointShadows` (enum)

How a point light's shadow map is stored, as og-render's PointShadows. RenderCapabilities.pointShadows reports what the device resolved.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Cube faces. |
| `"cube"` | Six 90 degree faces: six shadow passes per light. |
| `"paraboloid"` | Two paraboloid hemispheres (below and above the light): two passes per light instead of six, coarser, and large casters' silhouettes approximate (their edges stay straight in the curved projection). |

```ogt
enum PointShadows {
  doc "How a point light's shadow map is stored, as og-render's PointShadows. RenderCapabilities.pointShadows reports what the device resolved."
  variant auto "Cube faces."
  variant cube "Six 90 degree faces: six shadow passes per light."
  variant paraboloid "Two paraboloid hemispheres (below and above the light): two passes per light instead of six, coarser, and large casters' silhouettes approximate (their edges stay straight in the curved projection)."
}
```

<sub>`api.ogt` line 3842</sub>
