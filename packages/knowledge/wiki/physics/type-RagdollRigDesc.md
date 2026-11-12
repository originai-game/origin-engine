<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RagdollRigDesc"></a>

### `RagdollRigDesc` (desc struct)

The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bones` | `readonly RagdollBoneDesc[]` | `[]` | — | — | — | The bones. |
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton the bones' joints belong to (0: none): the entity's posed model is drawn blended towards the bones by og.Ragdoll.blend. |

```ogt
struct RagdollRigDesc {
  doc "The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton."
  desc true
  field bones List.RagdollBoneDesc "The bones."
  field skeleton SkeletonAsset "The skeleton the bones' joints belong to (0: none): the entity's posed model is drawn blended towards the bones by og.Ragdoll.blend."
}
```

<sub>Module `physics` line 1995</sub>
