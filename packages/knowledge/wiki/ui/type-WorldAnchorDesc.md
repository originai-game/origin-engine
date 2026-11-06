<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WorldAnchorDesc"></a>

### `WorldAnchorDesc` (desc struct)

An element anchored in the world (docs/ui.md §14): laid out at its own size and placed each tick on its target's point projected with the tick's camera. A nameplate, an interaction prompt, a speech bubble, a waypoint.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | What it sits on. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Added to the point in world units (above a head: 0, 2, 0). |
| `screenOffsetX` | `number` | `0` | — | `cssPx` | — | Moves it right on the screen. |
| `screenOffsetY` | `number` | `0` | — | `cssPx` | — | Moves it down on the screen. |
| `pivot` | `Anchor` | `"topLeft"` | — | — | — | Its point that sits on the projected point; the default is its bottom centre. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children; the default is a column. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `scale` | `AnchorScaleMode` | `"constant"` | — | — | — | How its size follows its distance. |
| `reference` | `number` | `10` | ≥ 0 | `m` | — | For scale distance: the distance at which it is its laid-out size. |
| `minScale` | `number` | `0.25` | ≥ 0 | — | — | For scale distance: the smallest it gets. |
| `maxScale` | `number` | `2` | ≥ 0 | — | — | For scale distance: the largest it gets. |
| `occlusion` | `WorldOcclusion` | `"none"` | — | — | — | What the scene in front of it does. |
| `occludedAlpha` | `number` | `0.25` | 0 … 1 | — | — | Its opacity behind the scene, for occlusion fade. |
| `lod` | `readonly number[]` | `[]` | — | — | — | Level-of-detail distances, ascending: level k from lod[k - 1] on (AnchorInfo.lod; build less at a higher level). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | Hidden past this distance, fading over its last tenth; 0 never. |
| `declutter` | `Declutter` | `"none"` | — | — | — | How it resolves against the anchors it overlaps. |
| `priority` | `number` | `0` | — | — | — | Higher keeps its place; equal priorities favour the nearer. |
| `clamp` | `EdgeClampDesc \| null` | `null` | — | — | — | Edge clamping; null lets it leave the screen. |
| `fadeTicks` | `number` | `0` | — | — | — | Ticks a fade (declutter, distance, appearing) takes; 0 is 6. |
| `contain` | `boolean` | `false` | — | — | — | Without clamp: while its point is in the safe area its box is kept inside it (a nameplate near the edge slides in rather than being cut), and it fades out as the point leaves the safe area. |

```ogt
struct WorldAnchorDesc {
  doc "An element anchored in the world (docs/ui.md §14): laid out at its own size and placed each tick on its target's point projected with the tick's camera. A nameplate, an interaction prompt, a speech bubble, a waypoint."
  desc true
  field target WorldTarget "What it sits on."
  field offset Vector3 "Added to the point in world units (above a head: 0, 2, 0)."
  field screenOffsetX f32 "Moves it right on the screen." {
    unit cssPx
  }
  field screenOffsetY f32 "Moves it down on the screen." {
    unit cssPx
  }
  field pivot Anchor "Its point that sits on the projected point; the default is its bottom centre."
  field direction Direction "Which way it lays out its children; the default is a column."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field padding Insets "Space inside its edges."
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field background Option.Paint "Its fill; null draws none."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field scale AnchorScaleMode "How its size follows its distance."
  field reference f32 "For scale distance: the distance at which it is its laid-out size." {
    default 10
    unit m
    min 0
  }
  field minScale f32 "For scale distance: the smallest it gets." {
    default 0.25
    min 0
  }
  field maxScale f32 "For scale distance: the largest it gets." {
    default 2
    min 0
  }
  field occlusion WorldOcclusion "What the scene in front of it does."
  field occludedAlpha f32 "Its opacity behind the scene, for occlusion fade." {
    default 0.25
    min 0
    max 1
  }
  field lod List.f32 "Level-of-detail distances, ascending: level k from lod[k - 1] on (AnchorInfo.lod; build less at a higher level)."
  field maxDistance f32 "Hidden past this distance, fading over its last tenth; 0 never." {
    unit m
    min 0
  }
  field declutter Declutter "How it resolves against the anchors it overlaps."
  field priority i32 "Higher keeps its place; equal priorities favour the nearer."
  field clamp Option.EdgeClampDesc "Edge clamping; null lets it leave the screen."
  field fadeTicks u32 "Ticks a fade (declutter, distance, appearing) takes; 0 is 6."
  field contain bool "Without clamp: while its point is in the safe area its box is kept inside it (a nameplate near the edge slides in rather than being cut), and it fades out as the point leaves the safe area."
}
```

<sub>`api.ogt` line 23098</sub>
