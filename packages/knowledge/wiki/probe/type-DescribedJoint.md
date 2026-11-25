<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DescribedJoint"></a>

### `DescribedJoint` (struct)

One joint of a described skeleton, projected from its posed node through the frame's camera.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its humanoid role when og.anim.humanoid recognises the rig (VRM names: hips, spine, chest, neck, head, leftUpperArm, leftHand, leftUpperLeg, leftFoot, rightFoot ...), else its node's name. |
| `point` | `ScreenPoint` | `{ x: 0, y: 0 }` | — | — | — | Where it lands on the surface; outside the surface when off screen. |
| `depth` | `number` | `0` | — | `m` | — | From the camera along the view; negative behind it. |
| `visible` | `boolean` | `false` | — | — | — | On the surface, in front of the camera, and hidden by nothing else drawn or by opaque UI (its own skeleton's surface never hides it). |

```ogt
struct DescribedJoint {
  doc "One joint of a described skeleton, projected from its posed node through the frame's camera."
  field name String "Its humanoid role when og.anim.humanoid recognises the rig (VRM names: hips, spine, chest, neck, head, leftUpperArm, leftHand, leftUpperLeg, leftFoot, rightFoot ...), else its node's name."
  field point ScreenPoint "Where it lands on the surface; outside the surface when off screen."
  field depth f32 "From the camera along the view; negative behind it." {
    unit m
  }
  field visible bool "On the surface, in front of the camera, and hidden by nothing else drawn or by opaque UI (its own skeleton's surface never hides it)."
}
```

<sub>Module `probe` line 659</sub>
