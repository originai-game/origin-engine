<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-UiSurface"></a>

### `UiSurface` (struct)

The surface UI bounds are laid out on.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | `cssPx` | — | Its width. |
| `height` | `number` | `0` | — | `cssPx` | — | Its height. |
| `dpr` | `number` | `1` | — | — | — | Device pixels per CSS pixel: a frame of this surface is width × dpr pixels wide. |

```ogt
struct UiSurface {
  doc "The surface UI bounds are laid out on."
  field width f32 "Its width." {
    unit cssPx
  }
  field height f32 "Its height." {
    unit cssPx
  }
  field dpr f32 "Device pixels per CSS pixel: a frame of this surface is width × dpr pixels wide." {
    default 1
  }
}
```

<sub>Module `probe` line 378</sub>
