<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="type-CameraLensStatus"></a>

### `CameraLensStatus` (enum)

The main camera's resolved projection source.

A string: one of the variants below. The zero value is `"artistic"`.

| Variant | Doc |
|---|---|
| `"artistic"` | The existing fov projection. |
| `"physical"` | A valid physical lens or held lens crossfade. |
| `"invalid"` | An incomplete physical lens: projection falls back to fov. |
| `"orthographic"` | Orthographic height owns projection; perspective lens settings are inactive. |

```ogt
enum CameraLensStatus {
  doc "The main camera's resolved projection source."
  variant artistic "The existing fov projection."
  variant physical "A valid physical lens or held lens crossfade."
  variant invalid "An incomplete physical lens: projection falls back to fov."
  variant orthographic "Orthographic height owns projection; perspective lens settings are inactive."
}
```

<sub>Module `camera` line 232</sub>
