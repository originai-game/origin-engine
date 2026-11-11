<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Antialias"></a>

### `Antialias` (enum)

Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved.

See `render §2`.

A string: one of the variants below. The zero value is `"tier"`.

| Variant | Doc |
|---|---|
| `"tier"` | The engine's default method, on every backend (WebGPU, Metal, Vulkan, DX12): 4x MSAA in a 2D project (project.ogt's profile 2d), where TAA's jitter and history blend would soften nearest-sampled pixel art; TAA in a 3D one. |
| `"off"` | One sample per pixel. |
| `"msaa4"` | 4x MSAA (what tier resolves to in a 2D project): sharp sprites and text, no temporal blur; FXAA where the HDR format cannot be multisampled. |
| `"taa"` | Temporal antialiasing (what tier resolves to in a 3D project): a sub-pixel jitter per frame and a reprojected history; converges over about eight frames. RenderOptions.taaHistory and taaSharpen refine it. |
| `"fxaa"` | FXAA: one full-screen pass over the tonemapped image (the overlay stays sharp), no extra samples and nothing to converge; softer than MSAA on fine texture. |
| `"smaa"` | SMAA 1x high preset: perceptual luma edges, area/search weights and neighbourhood blending over the tonemapped image; the overlay stays sharp. No jitter or history; an alternative to TAA, not combined with TAAU. |
| `"smaaT2x"` | SMAA temporal 2x: two quarter-pixel-jittered SMAA frames, reprojected with the engine's camera and object velocity and neighbourhood rejection. Exclusive of TAA, TAAU and MSAA; render scale uses spatial upscaling, and taaHistory and taaSharpen do not apply. |

```ogt
enum Antialias {
  doc "Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved."
  guide "render §2"
  variant tier "The engine's default method, on every backend (WebGPU, Metal, Vulkan, DX12): 4x MSAA in a 2D project (project.ogt's profile 2d), where TAA's jitter and history blend would soften nearest-sampled pixel art; TAA in a 3D one."
  variant off "One sample per pixel."
  variant msaa4 "4x MSAA (what tier resolves to in a 2D project): sharp sprites and text, no temporal blur; FXAA where the HDR format cannot be multisampled."
  variant taa "Temporal antialiasing (what tier resolves to in a 3D project): a sub-pixel jitter per frame and a reprojected history; converges over about eight frames. RenderOptions.taaHistory and taaSharpen refine it."
  variant fxaa "FXAA: one full-screen pass over the tonemapped image (the overlay stays sharp), no extra samples and nothing to converge; softer than MSAA on fine texture."
  variant smaa "SMAA 1x high preset: perceptual luma edges, area/search weights and neighbourhood blending over the tonemapped image; the overlay stays sharp. No jitter or history; an alternative to TAA, not combined with TAAU."
  variant smaaT2x "SMAA temporal 2x: two quarter-pixel-jittered SMAA frames, reprojected with the engine's camera and object velocity and neighbourhood rejection. Exclusive of TAA, TAAU and MSAA; render scale uses spatial upscaling, and taaHistory and taaSharpen do not apply."
}
```

<sub>Module `render` line 38</sub>
