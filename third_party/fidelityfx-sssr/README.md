# fidelityfx-sssr

AMD FidelityFX Stochastic Screen-Space Reflections
(https://github.com/GPUOpen-Effects/FidelityFX-SSSR, revision
34dcacd1feefcfab2855b82e76c7d711f2020a75, `ffx-sssr/ffx_sssr.h`) and the
FidelityFX reflection denoiser
(https://github.com/GPUOpen-Effects/FidelityFX-Denoiser, revision
d7dfecbabe7b9523b14e7b067216e06b86e8d189, `ffx-reflection-dnsr/`). MIT,
Copyright (c) 2021 Advanced Micro Devices, Inc.; the licences are in
`LICENSE` and `LICENSE-denoiser`. The denoiser's `ClipAABB` is Playdead's
(MIT, Copyright (c) 2015 Playdead), as quoted in its header.

The engine's screen-space reflections (`crates/og-render/shaders/ssr.wgsl`,
`crates/og-render/src/shading/ssr.rs`) port to WGSL:

- The hierarchical march (`FFX_SSSR_HierarchicalRaymarch`,
  `FFX_SSSR_InitialAdvanceRay`, `FFX_SSSR_AdvanceRay`) over a
  nearest-depth pyramid in `(uv, 1/z)` space, the engine's view depth
  standing in for the depth buffer.
- The hit validation (`FFX_SSSR_ValidateHit`): no hit on the starting
  pixel, on the sky or on a back face; confidence falling with the
  distance behind the surface, and a vignette at the screen's edges.
- The denoiser's reprojection (`FFX_DNSR_Reflections_PickReprojection`):
  the hit point's parallax for mirrors, the surface's motion otherwise,
  and the disocclusion factor with its constants (normal weight 1.4,
  depth weight 1.0, threshold 0.9).
- Its temporal resolve (`FFX_DNSR_Reflections_ResolveTemporal`): the
  history clipped to the neighbourhood's mean and deviation, accumulated
  up to `max(8, 32 * (1 - exp(-100 r)))` samples.

The ray reuse in the resolve pass, where each pixel weighs its neighbours'
rays by its own GGX lobe over each ray's pdf, follows Stachowiak 2015,
"Stochastic Screen-Space Reflections" (Frostbite, SIGGRAPH Advances).
The visible-normal sampling is Dupuy and Benyoub 2023, "Sampling Visible
GGX Normals with Spherical Caps".
