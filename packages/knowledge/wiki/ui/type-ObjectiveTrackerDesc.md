<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ObjectiveTrackerDesc"></a>

### `ObjectiveTrackerDesc` (desc struct)

The objective tracker.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `quests` | `readonly ObjectiveQuest[]` | `[]` | — | — | — | The tracked objectives. |
| `max` | `number` | `0` | — | — | — | The most shown; 0 is 3 (2 on a phone, 1 on a phone at a large text size). |
| `maxSteps` | `number` | `0` | — | — | — | The most steps a quest lists, its open ones first, the rest counted on a last line (+2 more); 0 is every step, or 2 on a phone at a large text size. When its side's stack outgrows the room the side leaves between its top and bottom pieces it folds itself: one quest, then two open steps, then one. |
| `width` | `number` | `0` | ≥ 0 | `cssPx` | — | Its width; 0 is 300. |
| `units` | `string` | `"m"` | — | — | — | The distance's unit. |
| `anchor` | `Anchor \| null` | `"right"` | — | — | — | Opens its own anchored group there; null builds it in place. |

```ogt
struct ObjectiveTrackerDesc {
  doc "The objective tracker."
  desc true
  field quests List.ObjectiveQuest "The tracked objectives."
  field max u32 "The most shown; 0 is 3 (2 on a phone, 1 on a phone at a large text size)."
  field maxSteps u32 "The most steps a quest lists, its open ones first, the rest counted on a last line (+2 more); 0 is every step, or 2 on a phone at a large text size. When its side's stack outgrows the room the side leaves between its top and bottom pieces it folds itself: one quest, then two open steps, then one."
  field width f32 "Its width; 0 is 300." {
    min 0
    unit cssPx
  }
  field units String "The distance's unit." {
    default "m"
  }
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place." {
    default right
  }
}
```

<sub>Module `ui` line 6320</sub>
