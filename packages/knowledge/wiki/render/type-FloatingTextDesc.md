<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-FloatingTextDesc"></a>

### `FloatingTextDesc` (desc struct)

A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). Deterministic: its drift comes from the tick and its order, and it is world state (the og.sprite state block).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text, such as `-12` or `+1 key`. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it appears (above a hit, usually). |
| `font` | `Font \| null` | `null` | — | — | — | The font; null uses the UI's body font. |
| `size` | `number` | `0.5` | ≥ 0 | `m` | — | The em size in world units. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear fill colour. |
| `outline` | `number` | `0.6` | 0 … 1 | — | — | The outline's width as a fraction of the font's distance range (0 none). |
| `outlineColor` | `Color` | `{ r: 0.02, g: 0.02, b: 0.03 }` | — | — | — | The outline's linear colour. |
| `rise` | `number` | `1` | — | `m` | — | How far it climbs over its life. |
| `spread` | `number` | `0.3` | ≥ 0 | `m` | — | The largest sideways drift, left or right. |
| `lifetime` | `number` | `0.8` | ≥ 0.05 | `s` | — | How long it shows. |
| `fade` | `number` | `0.4` | 0 … 1 | `ratio` | — | The last part of its life it fades over. |
| `pop` | `number` | `0.4` | ≥ 0 | — | — | How much bigger it starts, settling to its size in the first fifth of its life. |
| `crit` | `boolean` | `false` | — | — | — | A critical hit: critScale times bigger, a stronger pop. |
| `critScale` | `number` | `1.6` | ≥ 1 | — | — | The size of a crit. |
| `layer` | `number` | `100` | — | — | — | The sprite layer (above the action). |
| `facing` | `SpriteFacing` | `"none"` | — | — | — | A billboard (TextDesc.facing): none keeps the text in the world's xy plane, as for a 2D game; camera or cameraY turns it towards each camera that draws it, for damage numbers over a 3D scene. |

```ogt
struct FloatingTextDesc {
  doc "A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). Deterministic: its drift comes from the tick and its order, and it is world state (the og.sprite state block)."
  desc true
  field text String "The text, such as `-12` or `+1 key`."
  field position Vector3 "Where it appears (above a hit, usually)." {
    unit m
  }
  field font Option.Font "The font; null uses the UI's body font."
  field size f32 "The em size in world units." {
    default 0.5
    min 0
    unit m
  }
  field color Color "The linear fill colour." {
    default 1 1 1
  }
  field outline f32 "The outline's width as a fraction of the font's distance range (0 none)." {
    default 0.6
    min 0
    max 1
  }
  field outlineColor Color "The outline's linear colour." {
    default 0.02 0.02 0.03
  }
  field rise f32 "How far it climbs over its life." {
    default 1
    unit m
  }
  field spread f32 "The largest sideways drift, left or right." {
    default 0.3
    min 0
    unit m
  }
  field lifetime f32 "How long it shows." {
    default 0.8
    min 0.05
    unit s
  }
  field fade f32 "The last part of its life it fades over." {
    default 0.4
    min 0
    max 1
    unit ratio
  }
  field pop f32 "How much bigger it starts, settling to its size in the first fifth of its life." {
    default 0.4
    min 0
  }
  field crit bool "A critical hit: critScale times bigger, a stronger pop."
  field critScale f32 "The size of a crit." {
    default 1.6
    min 1
  }
  field layer i32 "The sprite layer (above the action)." {
    default 100
  }
  field facing SpriteFacing "A billboard (TextDesc.facing): none keeps the text in the world's xy plane, as for a 2D game; camera or cameraY turns it towards each camera that draws it, for damage numbers over a 3D scene."
}
```

<sub>Module `render` line 2410</sub>
