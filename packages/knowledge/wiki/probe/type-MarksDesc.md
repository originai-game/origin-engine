<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-MarksDesc"></a>

### `MarksDesc` (desc struct)

What marks draws: the capture's size and fit, which kinds it marks and how many.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | — | — | Width in pixels; 0 is the render target's. |
| `height` | `number` | `0` | — | — | — | Height in pixels; 0 is the render target's. |
| `fit` | `number` | `1280` | — | — | — | The longest side the PNG may have; 0 keeps the rendered size. |
| `ui` | `boolean` | `true` | — | — | — | Mark the UI elements. |
| `entities` | `boolean` | `true` | — | — | — | Mark the scene entities. |
| `max` | `number` | `64` | — | — | — | The most marks drawn; the rest are counted in omitted. |

```ogt
struct MarksDesc {
  doc "What marks draws: the capture's size and fit, which kinds it marks and how many."
  desc true
  field width u32 "Width in pixels; 0 is the render target's."
  field height u32 "Height in pixels; 0 is the render target's."
  field fit u32 "The longest side the PNG may have; 0 keeps the rendered size." {
    default 1280
  }
  field ui bool "Mark the UI elements." {
    default true
  }
  field entities bool "Mark the scene entities." {
    default true
  }
  field max u32 "The most marks drawn; the rest are counted in omitted." {
    default 64
  }
}
```

<sub>Module `probe` line 221</sub>
