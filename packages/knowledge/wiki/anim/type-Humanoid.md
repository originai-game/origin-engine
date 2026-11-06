<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-Humanoid"></a>

### `Humanoid` (struct)

Which node of a skeleton is each standard humanoid bone (what retargeting maps).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `string` | `""` | — | — | — | The naming scheme the bones were read in: mixamo, vrm, unity, rigify, ue, blender or soma; custom when a rig named them, none when nothing matched. |
| `bones` | `readonly HumanBone[]` | `[]` | — | — | — | The bones found, in the standard order (hips first). |

```ogt
struct Humanoid {
  doc "Which node of a skeleton is each standard humanoid bone (what retargeting maps)."
  field scheme String "The naming scheme the bones were read in: mixamo, vrm, unity, rigify, ue, blender or soma; custom when a rig named them, none when nothing matched."
  field bones List.HumanBone "The bones found, in the standard order (hips first)."
}
```

<sub>`api.ogt` line 12073</sub>
