<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WaypointDesc"></a>

### `WaypointDesc` (desc struct)

A waypoint or a ping in the world.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | What it marks. |
| `height` | `number` | `0.5` | — | `m` | — | World units above the point. |
| `kind` | `WaypointKind` | `"objective"` | — | — | — | What it is. |
| `icon` | `string` | `""` | — | — | — | Its icon; empty is the kind's. |
| `label` | `string` | `""` | — | — | — | A word under it (Defend, Enemy spotted); empty is none. |
| `by` | `string` | `""` | — | — | — | Who pinged it (shown with the label). |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of its own. |
| `rarity` | `Rarity` | `"none"` | — | — | — | Loot's tier. |
| `at` | `number` | `0` | — | — | — | og.ui.tick() when it was placed: a ping drops in and pulses from then. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | A ping leaves after this; 0 is 8000 for a ping, never for the rest. |
| `units` | `string` | `"m"` | — | — | — | The distance's unit. |
| `clampMargin` | `number` | `0` | ≥ 0 | `cssPx` | — | Inset from the safe area's edges when clamped; 0 is 32. |

```ogt
struct WaypointDesc {
  doc "A waypoint or a ping in the world."
  desc true
  field target WorldTarget "What it marks."
  field height f32 "World units above the point." {
    default 0.5
    unit m
  }
  field kind WaypointKind "What it is."
  field icon String "Its icon; empty is the kind's."
  field label String "A word under it (Defend, Enemy spotted); empty is none."
  field by String "Who pinged it (shown with the label)."
  field color Option.Paint "A colour of its own."
  field rarity Rarity "Loot's tier."
  field at u32 "og.ui.tick() when it was placed: a ping drops in and pulses from then."
  field durationMs f32 "A ping leaves after this; 0 is 8000 for a ping, never for the rest." {
    min 0
    unit ms
  }
  field units String "The distance's unit." {
    default "m"
  }
  field clampMargin f32 "Inset from the safe area's edges when clamped; 0 is 32." {
    min 0
    unit cssPx
  }
}
```

<sub>Module `ui` line 6727</sub>
