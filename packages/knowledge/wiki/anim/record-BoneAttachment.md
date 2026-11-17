<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-BoneAttachment"></a>

### `og.BoneAttachment` (component)

An entity parented to an animated bone, as a child of a three.js Bone. Its og.Parent must name an entity with og.Skeleton. Its og.Transform is the offset in that bone's frame; the engine composes MatrixWorld as the parent's world matrix times the posed node's model matrix times that offset. Children use the normal parent chain. Dynamic rigid bodies cannot own an attached transform; use a kinematic body instead.

Also known as: attach to bone, bone socket, Unreal socket, Unity parent to bone, three.js bone.add

Token `og.BoneAttachment`, registered under the schema name `og.BoneAttachment`; its value type is `BoneAttachment`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `node` | `number` | `0` | ≥ 0 | — | — | A node index in Animation.nodes, not a skin.joints index used by FootLock, IkChain, LookAt, SpringBone and SpringCollider. Non-joint glTF socket helpers work here. Use Animation.nodes.indexOf(name); 0 selects the first node. |

```ogt
component BoneAttachment {
  aliases "attach to bone, bone socket, Unreal socket, Unity parent to bone, three.js bone.add"
  doc "An entity parented to an animated bone, as a child of a three.js Bone. Its og.Parent must name an entity with og.Skeleton. Its og.Transform is the offset in that bone's frame; the engine composes MatrixWorld as the parent's world matrix times the posed node's model matrix times that offset. Children use the normal parent chain. Dynamic rigid bodies cannot own an attached transform; use a kinematic body instead."
  field node u32 "A node index in Animation.nodes, not a skin.joints index used by FootLock, IkChain, LookAt, SpringBone and SpringCollider. Non-joint glTF socket helpers work here. Use Animation.nodes.indexOf(name); 0 selects the first node." {
    min 0
  }
}
```

<sub>Module `anim` line 101</sub>
