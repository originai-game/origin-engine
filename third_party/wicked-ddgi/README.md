# wicked-ddgi

Wicked Engine's dynamic diffuse global illumination (Turánszki János,
https://github.com/turanszkij/WickedEngine, `WickedEngine/shaders/`
`ddgi_raytraceCS.hlsl`, `ddgi_updateCS.hlsl`, `ddgi_updateCS_depth.hlsl`,
and `ShaderInterop_DDGI.h`). MIT, Copyright (c) 2026
Turánszki János; the licence is in `LICENSE`. Wicked's DDGI is in turn
built on Majercik et al. 2019, "Dynamic Diffuse Global Illumination with
Ray-Traced Irradiance Fields" (JCGT 8(2)).

Ported to WGSL as the probe GI (`crates/og-render/src/gi_probes.rs`,
`shaders/gi_common.wgsl`, `gi_trace.wgsl`, `gi_blend.wgsl`,
`gi_probes.wgsl`). What came from it:

- The spherical Fibonacci ray directions, the first ones fixed for
  relocation and the inside-geometry test.
- The multiscale mean estimator that blends radiance into each probe.
- The octahedral distance moments, blended over time, and the
  Chebyshev visibility test with its wrap backface weight and weight
  crush.
- Probe relocation and the inside-geometry flag.
- The fold at the octahedral map's edges. Wicked stores the borders
  (`DDGI_DEPTH_BORDER_OFFSETS`); here they are folded at the read, which
  a unit test checks against Wicked's table.

Changes:

- Storage buffers instead of textures, because the lit pipelines have
  no texture slot left.
- L1 spherical harmonics, deringed, for the irradiance.
- Cascades that scroll with the camera instead of one volume.
- A software two-level BVH instead of hardware ray tracing.

NVIDIA's RTXGI is cited for the surface bias (DDGIGetSurfaceBias, its
normal and view bias) but none of its code is used. It is under the
NVIDIA RTX SDK licence, not MIT.
