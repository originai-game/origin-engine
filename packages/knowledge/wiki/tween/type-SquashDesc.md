<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-SquashDesc"></a>

### `SquashDesc` (desc struct)

Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `impact` | `Partial<Vector3>` | `{}` | — | — | — | The x, y and z factors at the impact; z 0 (or left out) stays at rest. A component left out takes its default: x 1.25, y 0.8, z 0. ArtMotion.squash scales it under a project's art contract. See `animation §12`. |
| `rest` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The scale it recovers to. |
| `duration` | `number` | `0.18` | ≥ 0 | `s` | — | Seconds to recover. |
| `ease` | `Ease` | `"back.out"` | — | — | — | The recovery curve; back.out overshoots a little, elastic.out wobbles. |

```ogt
struct SquashDesc {
  doc "Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest."
  desc true
  field entity Entity "The entity."
  field impact Partial.Vector3 "The x, y and z factors at the impact; z 0 (or left out) stays at rest. A component left out takes its default: x 1.25, y 0.8, z 0. ArtMotion.squash scales it under a project's art contract." {
    guide "animation §12"
  }
  field rest Vector3 "The scale it recovers to." {
    default 1 1 1
  }
  field duration f32 "Seconds to recover." {
    default 0.18
    min 0
    unit s
  }
  field ease Ease "The recovery curve; back.out overshoots a little, elastic.out wobbles." {
    default back.out
  }
}
```

<sub>Module `tween` line 360</sub>
