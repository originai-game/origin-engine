<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-GltfDesc"></a>

### `GltfDesc` (desc struct)

How to load a glTF, as three.js GLTFLoader options where they apply.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `number` | `0` | — | — | — | Which glTF scene to instantiate. |
| `castShadow` | `boolean` | `true` | — | — | — | Whether the models instantiated from it cast shadows: world.instantiate of the handle and a scene's `model` line give their og.Model this castShadow. On when omitted, so a model loaded without a desc casts as it always has; false turns it off for the model (og.Model.castShadow turns it off for one instance). The flags belong to the asset: another og.load.gltf of the same path answers the same handle and sets them again, for the models instantiated after it. |
| `receiveShadow` | `boolean` | `true` | — | — | — | Whether the models instantiated from it receive shadows (og.Model.receiveShadow, as castShadow): on when omitted; false lights them as if unshadowed by every light. |

```ogt
struct GltfDesc {
  doc "How to load a glTF, as three.js GLTFLoader options where they apply."
  desc true
  field scene u32 "Which glTF scene to instantiate."
  field castShadow bool "Whether the models instantiated from it cast shadows: world.instantiate of the handle and a scene's `model` line give their og.Model this castShadow. On when omitted, so a model loaded without a desc casts as it always has; false turns it off for the model (og.Model.castShadow turns it off for one instance). The flags belong to the asset: another og.load.gltf of the same path answers the same handle and sets them again, for the models instantiated after it." {
    default true
  }
  field receiveShadow bool "Whether the models instantiated from it receive shadows (og.Model.receiveShadow, as castShadow): on when omitted; false lights them as if unshadowed by every light." {
    default true
  }
}
```

<sub>`api.ogt` line 7966</sub>
