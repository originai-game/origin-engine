# bend-sss

Bend Studio's screen-space shadows, as shipped in Days Gone (Graham
Aldridge, "Screen Space Shadows", SIGGRAPH 2023,
https://www.bendstudio.com/blog/inside-bend-screen-space-shadows; the code
at https://www.bendstudio.com/assets/cms/downloads/code_final_candidate.zip,
sha256 `75707a8e287d485c0f71d04fb0ede245bb9a7e9569f1492b1c4d1f6ab943de83`, files `bend_sss_gpu.h` and
`bend_sss_cpu.h`). Apache-2.0, Copyright 2023 Sony Interactive
Entertainment, the licence in `LICENSE`.

Ported to WGSL and Rust as the contact-shadow pass
(`crates/og-render/shaders/contact.wgsl` `cs_contact`,
`crates/og-render/src/post/contact.rs`): `BuildDispatchList` and
`LightCoordinate` from `bend_sss_cpu.h` (the dispatches that tile the
screen in 64-pixel runs of the lines through the light), and
`ComputeWavefrontExtents` and `WriteScreenSpaceShadow` from
`bend_sss_gpu.h` (one 64-thread workgroup per run, its depths read once
into workgroup memory and walked for every pixel of the run), with Bend's
recommended defaults (SurfaceThickness 0.005, BilinearThreshold 0.02,
ShadowContrast 4, 60 samples, 4 hard, 8 fading; `render.rs`
`CONTACT_PARAMS`; READ_COUNT 2, so at most 64 samples). Changes: depth as
1 / view depth (the prepass stores view depth; reversed-Z device depth to
a scale) or an orthographic camera's affine reversed depth, one mask
texture per light (up to three, composed into the AO texture's g, b and
a by `cs_contact_compose`), a sample that is an edge also against the
previous sample along its line (the rim speckled where a line grazed a
caster's silhouette), and an early out for surfaces facing away from the
light or in the sky.
