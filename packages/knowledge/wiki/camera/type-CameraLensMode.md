<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="type-CameraLensMode"></a>

### `CameraLensMode` (enum)

The source of a perspective camera's projection and optical effects.

A string: one of the variants below. The zero value is `"artistic"`.

| Variant | Doc |
|---|---|
| `"artistic"` | Use the existing independent artistic parameters. |
| `"physical"` | Use the authored physical lens; incomplete channels keep their artistic projection or disable that physical effect. |

```ogt
enum CameraLensMode {
  doc "The source of a perspective camera's projection and optical effects."
  variant artistic "Use the existing independent artistic parameters."
  variant physical "Use the authored physical lens; incomplete channels keep their artistic projection or disable that physical effect."
}
```

<sub>Module `camera` line 196</sub>
