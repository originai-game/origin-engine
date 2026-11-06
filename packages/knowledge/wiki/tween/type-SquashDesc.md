<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-SquashDesc"></a>

### `SquashDesc` (desc struct)

Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `impact` | `Partial<Vector3>` | `{}` | — | — | — | The x, y and z factors at the impact: (1.25, 0.8) squashes (a landing, a hit), (0.8, 1.25) stretches (a jump). z is for 3D models (a squash bulges in x and z: (1.15, 0.85, 1.15)); left out or 0 it stays at rest, so a sprite's { x, y } is unchanged. A component left out takes its default: x 1.25, y 0.8, z 0; under a project's art contract, ArtMotion.squash scales it (0.5 the same, 1 rubber hose: x 1.5, y 0.6). |
| `rest` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The scale it recovers to. |
| `duration` | `number` | `0.18` | ≥ 0 | `s` | — | Seconds to recover. |
| `ease` | `Ease` | `"backOut"` | — | — | — | The recovery curve; backOut overshoots a little, elasticOut wobbles. |

```ogt
struct SquashDesc {
  doc "Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest."
  desc true
  field entity Entity "The entity."
  field impact Partial.Vector3 "The x, y and z factors at the impact: (1.25, 0.8) squashes (a landing, a hit), (0.8, 1.25) stretches (a jump). z is for 3D models (a squash bulges in x and z: (1.15, 0.85, 1.15)); left out or 0 it stays at rest, so a sprite's { x, y } is unchanged. A component left out takes its default: x 1.25, y 0.8, z 0; under a project's art contract, ArtMotion.squash scales it (0.5 the same, 1 rubber hose: x 1.5, y 0.6)."
  field rest Vector3 "The scale it recovers to." {
    default 1 1 1
  }
  field duration f32 "Seconds to recover." {
    default 0.18
    min 0
    unit s
  }
  field ease Ease "The recovery curve; backOut overshoots a little, elasticOut wobbles." {
    default backOut
  }
}
```

<sub>`api.ogt` line 12452</sub>
