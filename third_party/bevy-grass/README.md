# bevy-grass

Gediminas Mikelenas' procedural GPU grass for Bevy
(https://github.com/GEMI/bevy-grass, tag `v0.20.3`, commit
`76a1388ee01e9b0e3305a879b869503c4099d617`), ported to og-render's
grass (`crates/og-render/src/grass.rs` and `shaders/grass*.wgsl`):

- the placement: a progressive blue-noise tile (Mitchell's best
  candidate) copied over 16 m chunks, each copy offset by a hash, blades
  kept by their rank below the density there, so the field thins evenly
  with distance (`shader_model.rs`, `generate.wesl`);
- the distance table (full density to the first level of detail, thinning
  to nothing at the radius), the dithered level-of-detail bands and the
  widening that keeps the coverage as blades thin out (`common.wesl`);
- the packed 16-byte blade, its quadratic Bézier that keeps its length
  as it bends, the turn towards a camera that sees it edge-on, the
  minimum on-screen width, the rounded normals and the flutter
  (`common.wesl`, `blade.wesl`, `wind.wesl`);
- the look: dark roots to bright tips, clump tints, dry clumps and tips,
  the root occlusion, the sheen and the translucency that lights a
  backlit blade as its sunward side tinted by what passed through it
  (`look.wesl`, `grass.wesl`); and the far colour the ground takes past
  the blades (`far.wesl`).

Changed in the port: the per-workgroup atomic append is a fixed slot per
candidate, a prefix sum over the slices and an ordered compaction, so
the same frame lays out the same instances; the chunks are culled on the
CPU; the wind is og.Wind's (its travelling wave, gust patches of
periodic value noise scrolled downwind, a per-blade sway) instead of a
gust texture; the clumps are Voronoi cells computed in place instead of
a table; the lighting is og-render's.

MIT, Copyright (c) 2026 Gediminas Mikelenas, the notice in `LICENSE`,
repeated in the header of every ported file.
