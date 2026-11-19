<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Viewpoint"></a>

### `Viewpoint` (struct)

A camera to capture from (captureViews), in world units.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the eye is. |
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The point it looks at. |
| `up` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | optional | Up; `[0, 1, 0]` when absent. |
| `fov` | `number` | `50` | — | `deg` | — | Vertical field of view in degrees, for a perspective view. |
| `height` | `number \| null` | `null` | — | — | optional | An orthographic view's height in world units; null is perspective. |
| `near` | `number \| null` | `null` | — | — | optional | The near plane; null keeps og.Camera's. |
| `far` | `number \| null` | `null` | — | — | optional | The far plane; null keeps og.Camera's. |

```ogt
struct Viewpoint {
  doc "A camera to capture from (captureViews), in world units."
  field position Vector3 "Where the eye is."
  field target Vector3 "The point it looks at."
  field up Vector3 "Up; `[0, 1, 0]` when absent." {
    optional true
  }
  field fov f32 "Vertical field of view in degrees, for a perspective view." {
    default 50
    unit deg
  }
  field height Option.f32 "An orthographic view's height in world units; null is perspective." {
    optional true
  }
  field near Option.f32 "The near plane; null keeps og.Camera's." {
    optional true
  }
  field far Option.f32 "The far plane; null keeps og.Camera's." {
    optional true
  }
}
```

<sub>Module `probe` line 187</sub>
