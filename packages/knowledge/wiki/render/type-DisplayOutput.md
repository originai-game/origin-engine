<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-DisplayOutput"></a>

### `DisplayOutput` (enum)

What the play surface presents, as og-render's DisplayOutput (RenderOptions.output). Offscreen and headless renderers, and so every capture and reference frame, are sRGB at standard range whatever it says.

See `render §5`.

A string: one of the variants below. The zero value is `"best"`.

| Variant | Doc |
|---|---|
| `"best"` | The widest the surface and display offer: extended range (HDR) with the Display P3 gamut, else Display P3 at standard range, else sRGB. A display that reports no headroom (an SDR monitor on a Mac) takes Display P3 at standard range. |
| `"wide"` | Display P3 at standard range where the surface offers it, never extended range. |
| `"srgb"` | sRGB at standard range on every surface, the output before wide gamut and HDR. |

```ogt
enum DisplayOutput {
  doc "What the play surface presents, as og-render's DisplayOutput (RenderOptions.output). Offscreen and headless renderers, and so every capture and reference frame, are sRGB at standard range whatever it says."
  guide "render §5"
  variant best "The widest the surface and display offer: extended range (HDR) with the Display P3 gamut, else Display P3 at standard range, else sRGB. A display that reports no headroom (an SDR monitor on a Mac) takes Display P3 at standard range."
  variant wide "Display P3 at standard range where the surface offers it, never extended range."
  variant srgb "sRGB at standard range on every surface, the output before wide gamut and HDR."
}
```

<sub>Module `render` line 82</sub>
