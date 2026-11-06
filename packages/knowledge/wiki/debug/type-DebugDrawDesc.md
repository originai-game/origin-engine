<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`debug`](../debug.md) · [index](../index.md)

<a id="type-DebugDrawDesc"></a>

### `DebugDrawDesc` (desc struct)

How a debug shape looks and how long it lasts.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 0 }` | — | — | — | The linear colour. |
| `width` | `number` | `2` | ≥ 0 | — | — | Line width in CSS pixels. |
| `duration` | `number` | `0` | — | `ticks` | — | How many ticks after this one it stays; 0 is this frame only (draw it every tick it should show). |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The shape's rotation: a box's or a sphere's circles', the axes' frame. |

```ogt
struct DebugDrawDesc {
  doc "How a debug shape looks and how long it lasts."
  desc true
  field color Color "The linear colour." {
    default 1 1 0
  }
  field width f32 "Line width in CSS pixels." {
    default 2
    min 0
  }
  field duration u32 "How many ticks after this one it stays; 0 is this frame only (draw it every tick it should show)." {
    unit ticks
  }
  field rotation Quaternion "The shape's rotation: a box's or a sphere's circles', the axes' frame."
}
```

<sub>`api.ogt` line 25720</sub>
