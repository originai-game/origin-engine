<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SceneViewportInfo"></a>

### `SceneViewportInfo` (struct)

What the pointer does in a viewport this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | `cssPx` | — | Its laid-out width, last tick. |
| `height` | `number` | `0` | — | `cssPx` | — | Its laid-out height, last tick. |
| `pixelWidth` | `number` | `0` | — | — | — | The target width that fills it at the device's pixel ratio. |
| `pixelHeight` | `number` | `0` | — | — | — | The target height that fills it. |
| `hovered` | `boolean` | `false` | — | — | — | The pointer is over it. |
| `pressed` | `boolean` | `false` | — | — | — | A press that started in it is held. |
| `clicked` | `boolean` | `false` | — | — | — | Pressed and released in it. |
| `drag` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | The pointer's move while held this tick (orbit, turn). |
| `wheel` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The wheel over it (zoom). |
| `at` | `Vector2 \| null` | `null` | — | — | — | The pointer in it, fractions of its box from the top left. |
| `ray` | `Ray \| null` | `null` | — | — | — | The ray into its scene through the pointer. |

```ogt
struct SceneViewportInfo {
  doc "What the pointer does in a viewport this tick."
  field width f32 "Its laid-out width, last tick." {
    unit cssPx
  }
  field height f32 "Its laid-out height, last tick." {
    unit cssPx
  }
  field pixelWidth u32 "The target width that fills it at the device's pixel ratio."
  field pixelHeight u32 "The target height that fills it."
  field hovered bool "The pointer is over it."
  field pressed bool "A press that started in it is held."
  field clicked bool "Pressed and released in it."
  field drag Vector2 "The pointer's move while held this tick (orbit, turn)." {
    unit cssPx
  }
  field wheel Vector2 "The wheel over it (zoom)."
  field at Option.Vector2 "The pointer in it, fractions of its box from the top left."
  field ray Option.Ray "The ray into its scene through the pointer."
}
```

<sub>Module `ui` line 7889</sub>
