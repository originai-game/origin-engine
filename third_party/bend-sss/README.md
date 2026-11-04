# bend-sss

Bend Studio's screen-space shadows, as shipped in Days Gone (Graham
Aldridge, "Screen Space Shadows", SIGGRAPH 2023,
https://www.bendstudio.com/blog/inside-bend-screen-space-shadows; the code
at https://www.bendstudio.com/assets/cms/downloads/code_final_candidate.zip,
sha256 `75707a8e287d485c0f71d04fb0ede245bb9a7e9569f1492b1c4d1f6ab943de83`, files `bend_sss_gpu.h` and
`bend_sss_cpu.h`). Apache-2.0, Copyright 2023 Sony Interactive
Entertainment, the licence in `LICENSE`.

Ported to WGSL as the contact-shadow pass
(`crates/og-render/shaders/fxpost.wgsl` `fs_contact`): the light's
coordinate from `BuildDispatchList`, and `WriteScreenSpaceShadow`'s
samples, slab thickness, edge-detected minor-axis bias, hard, averaged and
fading samples and contrast, with Bend's recommended defaults
(SurfaceThickness 0.005, BilinearThreshold 0.02, ShadowContrast 4, 60
samples, 4 hard, 8 fading; `render.rs` `CONTACT_PARAMS`). Changes:
one fragment per pixel instead of one compute wave per line of pixels
(the same samples, not shared through groupshared memory), depth as
1 / view depth (the prepass stores view depth; reversed-Z device depth to
a scale) or an orthographic camera's affine reversed depth, up to three
lights per pass (one mask channel each), and an early out for surfaces
facing away from the light.
