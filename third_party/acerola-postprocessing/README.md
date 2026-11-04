# acerola-postprocessing

Garrett Gunnell's (Acerola) post-processing collection,
https://github.com/GarrettGunnell/Post-Processing,
`Assets/Kuwahara Filter/AnisotropicKuwahara.shader` (main branch,
2026-11-04): the anisotropic Kuwahara filter with eight sectors and
polynomial weights over a smoothed structure tensor (Kyprianidis, Kang and
Döllner 2009; Kyprianidis 2010). Ported from HLSL to WGSL in
`crates/og-render/shaders/stylise.wgsl` (`paint_frame`, `kuwahara`) and to
Rust in `crates/og-cook/src/cook/conform/texture2d/paint.rs`, with its
constants (sharpness 8, hardness 8, zero crossing 0.58, the tensor's
Gaussian sigma 2); the engine adds the object-edge and detail awareness.
MIT, Copyright (c) 2022 Garrett Gunnell, the notice in `LICENSE`.
