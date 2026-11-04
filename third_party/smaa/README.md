# SMAA 1x

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
to sRGB, and zero subsample indices (1x). No temporal resolve or jitter.
