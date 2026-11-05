# WebGPU-Ocean (MLS-MPM)

- Upstream: <https://github.com/matsuoka-601/WebGPU-Ocean>
- Revision: `3bd932778650b5e756ba2590969ed618313843ad` (2025-06-09)
- Licence: MIT, Copyright (c) 2025 matsuoka-601 (`LICENSE` here, verbatim)
- Files ported: `mls-mpm/clearGrid.wgsl`, `p2g_1.wgsl`, `p2g_2.wgsl`,
  `updateGrid.wgsl`, `g2p.wgsl` (the parameters of `mls-mpm.ts`)
- Into: `crates/og-render/shaders/visual_fluid.wgsl`, driven by
  `crates/og-render/src/visual_fluid.rs`

The upstream credits, carried here: the method is Hu et al., *A Moving
Least Squares Material Point Method with Displacement Discontinuity and
Two-Way Rigid Body Coupling* (SIGGRAPH 2018), as nialltl's *MPM guide*
(<https://nialltl.neocities.org/articles/mpm_guide>) lays it out for
fluids (the volume recomputed every step); its fixed-point `atomicAdd`
scatter comes from EA SEED's pbmpm (BSD-3-Clause, the idea only, no code).
The upstream repository carries no NOTICE file and no per-file headers.

Changes from upstream, all in the engine's file: one module of entry
points over a six-buffer layout instead of six pipelines with overrides;
the grid placed in the world (cells of the fluid's radius, origin and
dimensions from its container) with gravity and time in the solver's
units derived from metres and the tick; a pool of particles with a
spawn lattice, an emitter and a sink; the scene's colliders
(`scene_colliders.wgsl`) pushed out in the particle step; the velocity
clamped for the fixed-point range; and the particles written straight
into the screen-space fluid's instances. The SPH half and the renderer
of upstream are not used (the engine's `fluid3d.wgsl` draws).

## Splash (the surface's depth filter)

- Upstream: <https://github.com/matsuoka-601/Splash>
- Revision: `3df5d621e6f153d7269037cbd8606bd83796f3fd`
- Licence: MIT, Copyright (c) 2025 matsuoka-601, word for word the
  `LICENSE` here (checked identical)
- File ported: `render/narrowRangeFilter.wgsl` (with the pass order of
  `render/fluidRender.ts`: one-dimensional twice over x and y, then the
  two-dimensional square)
- Into: `crates/og-render/shaders/fluid3d_blur.wgsl`, driven by
  `crates/og-render/src/fluid3d.rs`. Changes: the range constants are
  given in the fluid's drawn particle radius instead of upstream's fixed
  0.6, the filter size follows the fluid's smoothing, and an empty tap
  counts as a farther one.

The method is Truong and Yuksel, *A Narrow-Range Filter for Screen-Space
Fluid Rendering* (I3D 2018).
