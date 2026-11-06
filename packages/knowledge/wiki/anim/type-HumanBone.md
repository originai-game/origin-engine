<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-HumanBone"></a>

### `HumanBone` (struct)

One standard humanoid bone of a skeleton.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bone` | `string` | `""` | — | — | — | The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg, then each hand's fingers (VRM 1.0 names): leftThumbMetacarpal, leftThumbProximal, leftThumbDistal, leftIndexProximal, leftIndexIntermediate, leftIndexDistal, the same for Middle, Ring and Little, and the right hand's. |
| `node` | `string` | `""` | — | — | — | The skeleton node that is that bone. |

```ogt
struct HumanBone {
  doc "One standard humanoid bone of a skeleton."
  field bone String "The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg, then each hand's fingers (VRM 1.0 names): leftThumbMetacarpal, leftThumbProximal, leftThumbDistal, leftIndexProximal, leftIndexIntermediate, leftIndexDistal, the same for Middle, Ring and Little, and the right hand's."
  field node String "The skeleton node that is that bone."
}
```

<sub>`api.ogt` line 12060</sub>
