# hextile

Morten S. Mikkelsen's hex-tiling demo
(https://github.com/mmikk/hextile-demo, commit
`43c3ed7e18e1e4539fa9323f72d5e2a65147ebb3`), the code of "Practical
Real-Time Hex-Tiling" (Journal of Computer Graphics Techniques 11(2),
2022), ported to og-render's terrain layers (`crates/og-render/shaders/terrain_lit.wgsl`,
`plane_taps`, `taps_albedo`, `taps_normal`):

- the triangle grid in skewed space and its three hexagon vertices with
  their barycentric weights (`TriangleGrid`), the hexagon centres
  (`MakeCenST`);
- each hexagon's copy of the texture turned about its centre and moved,
  its sample gradients turned with it, and the normal map's slopes turned
  back (`hex2colTex`, `bumphex2derivNMap`);
- the weights: barycentric to the seventh power, scaled towards the
  brighter tap by 0.6 (`g_fallOffContrast`, `g_exp`).

Changed: the per-cell hash is pcg2d on integer ids (Jarzynski and Olano,
JCGT 2020) and the turn comes from it, not from `sin` and the ids'
products; the lattice is placed from the world's zero with values the
CPU computes in f64 per frame (`hex_lattice` in `crates/og-render/src/terrain.rs`),
so it stays on the ground when the render origin moves; the normal
taps reuse the colour taps' weights.

MIT licence: `LICENSE`.
