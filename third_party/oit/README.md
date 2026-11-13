# Order-independent transparency

Weighted blending follows McGuire and Bavoil, *Weighted Blended
Order-Independent Transparency*, JCGT 2(2), 2013, equations 9–10:
https://jcgt.org/published/0002/02/09/ . The implementation is written
for the engine; no code is copied from the paper or its blog examples.

The weighted mode's exact front layer follows Maule, Comba, Torchelsen
and Bastos, *Hybrid Transparency*, I3D 2013,
https://doi.org/10.1145/2448196.2448212 , with k = 1 so the nearest
layer is found by a 32-bit `atomicMax` (WebGPU has no 64-bit atomics or
ordered pixel access); written for the engine from the paper.

The linked-list construction and resolve are adapted from:

- Bevy, revision `4c344352e063923257e50efd65f265fea76bb4ff`,
  `crates/bevy_core_pipeline/src/oit/`: MIT or Apache-2.0, notices in
  `LICENSE-bevy-MIT` and `LICENSE-bevy-APACHE`.
- WebGPU Samples, revision `e040ec1a20dbbe01afed30c812e825b2639cfe5b`,
  `sample/a-buffer/`: BSD-3-Clause, `LICENSE-webgpu-samples`.

Engine adaptations (crates/og-render/src/shading/oit.rs, shaders/oit*.wgsl): one
packed atomic-u32 arena at group 0 binding 12 (a header, a fragment count
per pixel, then each pixel's own fixed run of seven-word nodes) rather
than separate bindings and a global node pool; every fragment also
accumulated weighted; a pixel past its node budget resolved whole from the
weighted targets and counted in the header, so allocation order never
changes which pixels lose exact sorting and no fragment is dropped; nodes
tested per MSAA sample against the opaque depth in the resolve; equal
depths ordered by a stable key of the draw's transform and the primitive
(FNV-1a), never by arrival or colour.
