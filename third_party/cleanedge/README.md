# cleanedge

torcado's cleanEdge, "a pixel art upscaling algorithm for clean
rotations" (https://torcado.com/cleanEdge, the shader at
https://gist.github.com/torcado194/e2794f5a4b22049ac0a41f972d14c329,
`cleanEdge-shadertoy.glsl`, gist revision
`91c809a60664baddc91abc151f4bf71a82b7891a`), ported from a fragment shader
to a sampler in `crates/og-cook/src/cook/conform/texture2d/cleanedge.rs`:
the cook bakes `ArtPixels.rotation prerotated` sprites with it (SLOPE and
CLEANUP on, highestColor white, similarThreshold 0, lineWidth 1). MIT,
Copyright (c) 2022 torcado, the notice in `LICENSE` (the shader's own
header). The runtime filter of docs/art.md §2.4 (A4) uses the same
algorithm under the same notice.
