<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-GltfInfo"></a>

### `GltfInfo` (struct)

What a loaded glTF holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scenes` | `number` | `0` | — | — | — | How many scenes it has. |
| `geometries` | `readonly Geometry[]` | `[]` | — | — | — | Mesh-local geometries, without node placement. Use parts to read each placement in model space. |
| `parts` | `readonly GltfPart[]` | `[]` | — | — | — | Placed primitives in the default scene, in draw order. |
| `materials` | `readonly Material[]` | `[]` | — | — | — | Its materials, in glTF order, then the default material. |
| `variants` | `readonly string[]` | `[]` | — | — | — | Its material variants' names (KHR_materials_variants), in glTF order: og.Model.variant draws an instance with one, by its index here. Empty for a model without variants. |
| `lights` | `number` | `0` | — | — | — | How many punctual lights it has (KHR_lights_punctual) in its default scene: world.instantiate and a scene's `model` line spawn each as a child entity of the model's, with an og.PointLight, og.SpotLight or og.DirectionalLight at its node's place (glTF's candela as the radiance at one metre, its range as distance, a light of infinite range ending where its radiance falls to 0.01, a spot's outer cone as angle and 1 - inner / outer as penumbra). |
| `animations` | `readonly string[]` | `[]` | — | — | — | Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name. |

```ogt
struct GltfInfo {
  doc "What a loaded glTF holds."
  field scenes u32 "How many scenes it has."
  field geometries List.Geometry "Mesh-local geometries, without node placement. Use parts to read each placement in model space."
  field parts List.GltfPart "Placed primitives in the default scene, in draw order."
  field materials List.Material "Its materials, in glTF order, then the default material."
  field variants List.String "Its material variants' names (KHR_materials_variants), in glTF order: og.Model.variant draws an instance with one, by its index here. Empty for a model without variants."
  field lights u32 "How many punctual lights it has (KHR_lights_punctual) in its default scene: world.instantiate and a scene's `model` line spawn each as a child entity of the model's, with an og.PointLight, og.SpotLight or og.DirectionalLight at its node's place (glTF's candela as the radiance at one metre, its range as distance, a light of infinite range ending where its radiance falls to 0.01, a spot's outer cone as angle and 1 - inner / outer as penumbra)."
  field animations List.String "Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name."
}
```

<sub>`api.ogt` line 7958</sub>
