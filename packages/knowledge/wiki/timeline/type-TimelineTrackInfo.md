<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="type-TimelineTrackInfo"></a>

### `TimelineTrackInfo` (struct)

One track of a loaded timeline.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | camera, transform, animation, audio, music, caption, event, transition, marker, property, light, visibility, post, blendshape, material, timeline or shot. |
| `target` | `string` | `""` | — | — | — | The entity a transform, animation, property, light, visibility, blendshape or material track drives, the path a rail rides, the entity a camera follows; empty otherwise. |
| `keys` | `number` | `0` | — | — | — | Its keys. |

```ogt
struct TimelineTrackInfo {
  doc "One track of a loaded timeline."
  field kind String "camera, transform, animation, audio, music, caption, event, transition, marker, property, light, visibility, post, blendshape, material, timeline or shot."
  field target String "The entity a transform, animation, property, light, visibility, blendshape or material track drives, the path a rail rides, the entity a camera follows; empty otherwise."
  field keys u32 "Its keys."
}
```

<sub>`api.ogt` line 26155</sub>
