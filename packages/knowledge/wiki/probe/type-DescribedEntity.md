<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DescribedEntity"></a>

### `DescribedEntity` (struct)

One entity the frame shows, from the entity-id pass (a CPU rasterisation of the frame's meshes, skinned meshes and world sprites at a quarter of its pixels).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its probe id. |
| `name` | `string \| null` | `null` | — | — | — | Its name. |
| `order` | `number` | `0` | — | — | — | Its depth order: 0 is the nearest. |
| `archetype` | `readonly string[]` | `[]` | — | — | — | Its components, by name. |
| `labels` | `readonly string[]` | `[]` | — | — | — | What draws it: `model` with its asset path, `mesh`, `terrain`, `terrainStamp`, `grass`, `skinned`, `sprite`. A `grass` field is seen at field fidelity, not per blade: its coverage is the ground it grows on lifted half its tallest blade, so a pixel between blades answers the field and a tip above that height answers what is behind it. |
| `box` | `ScreenBox` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | The box of its visible pixels. |
| `drawn` | `ScreenBox` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | The box of what it covers with nothing in front of it. |
| `point` | `ScreenPoint` | `{ x: 0, y: 0 }` | — | — | — | A visible point nearest the box's centre: pick answers this entity there. |
| `depth` | `number` | `0` | — | `m` | — | From the camera to its nearest visible point, along the view. |
| `coverage` | `number` | `0` | — | `ratio` | — | Its visible share of the surface. |
| `occlusion` | `number` | `0` | — | `ratio` | — | The share of what it covers alone that something else hides: another object in front, or a UI element drawn opaque over it (DescribedUi.opaque); on a secondary camera's image, objects only. |
| `views` | `readonly string[]` | `[]` | — | — | — | The probe ids of the og.CameraView and og.Mirror entities whose image also shows it, from their own id passes on their own layers; empty when only the main camera does (under DescribedView.entities it is empty). |

```ogt
struct DescribedEntity {
  doc "One entity the frame shows, from the entity-id pass (a CPU rasterisation of the frame's meshes, skinned meshes and world sprites at a quarter of its pixels)."
  field id String "Its probe id."
  field name Option.String "Its name."
  field order u32 "Its depth order: 0 is the nearest."
  field archetype List.String "Its components, by name."
  field labels List.String "What draws it: `model` with its asset path, `mesh`, `terrain`, `terrainStamp`, `grass`, `skinned`, `sprite`. A `grass` field is seen at field fidelity, not per blade: its coverage is the ground it grows on lifted half its tallest blade, so a pixel between blades answers the field and a tip above that height answers what is behind it."
  field box ScreenBox "The box of its visible pixels."
  field drawn ScreenBox "The box of what it covers with nothing in front of it."
  field point ScreenPoint "A visible point nearest the box's centre: pick answers this entity there."
  field depth f32 "From the camera to its nearest visible point, along the view." {
    unit m
  }
  field coverage f32 "Its visible share of the surface." {
    unit ratio
  }
  field occlusion f32 "The share of what it covers alone that something else hides: another object in front, or a UI element drawn opaque over it (DescribedUi.opaque); on a secondary camera's image, objects only." {
    unit ratio
  }
  field views List.String "The probe ids of the og.CameraView and og.Mirror entities whose image also shows it, from their own id passes on their own layers; empty when only the main camera does (under DescribedView.entities it is empty)."
}
```

<sub>`api.ogt` line 24364</sub>
