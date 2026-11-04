# SMAA 1x and T2x

Jimenez et al., *SMAA: Enhanced Subpixel Morphological Antialiasing* (2012).
Reference: https://github.com/iryoku/smaa at
`71c806a838bdd7d517df19192a20f0c61b3ca29d` (MIT, `LICENSE.txt`).

`area.bin` and `search.bin` are the byte arrays in `Textures/AreaTex.h`
and `Textures/SearchTex.h`, respectively, without transformation: RG8
160×560 and R8 64×16. Their copyright and permission notice is
`LICENSE.txt`; it applies to both lookup textures as well as the reference.

The WGSL chunks are from https://github.com/dmnsgn/shaders-smaa at
`1d73155c07e1bd68a53536c5b517ece08c2cae11`, MIT (`LICENSE-wgsl.txt`),
retaining diagonal search, corner detection and the reference lookup layout.
Engine entry points live in `crates/og-render/shaders/smaa_*.wgsl`.
The engine uses the high preset, luma edges with linear samples re-encoded
to sRGB, and zero subsample indices for 1x (no temporal resolve or jitter).
For HDR, edge samples are clamped to the SDR paper-white range before
encoding; the blend pass still samples the original float image. This
engine adaptation keeps highlights from inflating perceptual edge thresholds.

T2x follows `Demo/DX10/Code/SMAA.cpp`'s `getJitter`, `JitteredMatrix`
and `getSubsampleIndex`: image displacement in y-down pixels is
(-0.25, -0.25) / (0.25, 0.25), area layers (1, 1, 1, 0) / (2, 2, 2, 0).
The HLSL `@SUBSAMPLE_INDICES` table describes sample positions, despite
calling them camera jitter: samples move opposite to the image translation.
The two raw spatially filtered frames are averaged in linear display colour,
before stylisation and UI. Its adaptation in `taa.wgsl` shares the engine's
camera/object reprojection, bounds rejection and YCoCg neighbourhood clamp,
instead of overwriting alpha with velocity magnitude as the reference's
optional reprojection does. Alpha remains opacity. The resolved result is
never stored back into history, so this is two-frame supersampling rather
than recursive TAA. No additional licence is needed for the reference's
two-frame mean; the same MIT notice above applies.
S2x is not implemented: it requires genuine 2x MSAA and known sample order,
not the engine's 4x MSAA mode.
