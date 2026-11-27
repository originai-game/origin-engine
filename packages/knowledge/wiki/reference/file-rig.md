<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-rig"></a>
### `file:rig`

Checked `file["rig"](input)` declaration from `@origingame/engine/author`.

```ts
export type Input = ({
/** Humanoid bone name to skeleton node name; null explicitly unmaps it. */
readonly "bones"?: ({
readonly "chest"?: ((string) | (null));
readonly "head"?: ((string) | (null));
readonly "hips"?: ((string) | (null));
readonly "leftFoot"?: ((string) | (null));
readonly "leftHand"?: ((string) | (null));
readonly "leftIndexDistal"?: ((string) | (null));
readonly "leftIndexIntermediate"?: ((string) | (null));
readonly "leftIndexProximal"?: ((string) | (null));
readonly "leftLittleDistal"?: ((string) | (null));
readonly "leftLittleIntermediate"?: ((string) | (null));
readonly "leftLittleProximal"?: ((string) | (null));
readonly "leftLowerArm"?: ((string) | (null));
readonly "leftLowerLeg"?: ((string) | (null));
readonly "leftMiddleDistal"?: ((string) | (null));
readonly "leftMiddleIntermediate"?: ((string) | (null));
readonly "leftMiddleProximal"?: ((string) | (null));
readonly "leftRingDistal"?: ((string) | (null));
readonly "leftRingIntermediate"?: ((string) | (null));
readonly "leftRingProximal"?: ((string) | (null));
readonly "leftShoulder"?: ((string) | (null));
readonly "leftThumbDistal"?: ((string) | (null));
readonly "leftThumbMetacarpal"?: ((string) | (null));
readonly "leftThumbProximal"?: ((string) | (null));
readonly "leftToes"?: ((string) | (null));
readonly "leftUpperArm"?: ((string) | (null));
readonly "leftUpperLeg"?: ((string) | (null));
readonly "neck"?: ((string) | (null));
readonly "rightFoot"?: ((string) | (null));
readonly "rightHand"?: ((string) | (null));
readonly "rightIndexDistal"?: ((string) | (null));
readonly "rightIndexIntermediate"?: ((string) | (null));
readonly "rightIndexProximal"?: ((string) | (null));
readonly "rightLittleDistal"?: ((string) | (null));
readonly "rightLittleIntermediate"?: ((string) | (null));
readonly "rightLittleProximal"?: ((string) | (null));
readonly "rightLowerArm"?: ((string) | (null));
readonly "rightLowerLeg"?: ((string) | (null));
readonly "rightMiddleDistal"?: ((string) | (null));
readonly "rightMiddleIntermediate"?: ((string) | (null));
readonly "rightMiddleProximal"?: ((string) | (null));
readonly "rightRingDistal"?: ((string) | (null));
readonly "rightRingIntermediate"?: ((string) | (null));
readonly "rightRingProximal"?: ((string) | (null));
readonly "rightShoulder"?: ((string) | (null));
readonly "rightThumbDistal"?: ((string) | (null));
readonly "rightThumbMetacarpal"?: ((string) | (null));
readonly "rightThumbProximal"?: ((string) | (null));
readonly "rightToes"?: ((string) | (null));
readonly "rightUpperArm"?: ((string) | (null));
readonly "rightUpperLeg"?: ((string) | (null));
readonly "spine"?: ((string) | (null));
readonly "upperChest"?: ((string) | (null));
});
/** Left and right sole heights. */
readonly "footHeight"?: ((ReadonlyArray<(number)> & { readonly length: 2 }) | (null));
/** Naming scheme understood by the humanoid mapper. */
readonly "scheme"?: ("mixamo" | "vrm" | "unity" | "rigify" | "ue" | "blender" | "soma" | "auto" | null) & ((string) | (null));
});
```
