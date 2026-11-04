// The terrain's clipmap mesh, for the lit and shadow modules (after
// terrain.wgsl).
//
// The clipmap's mesh (terrain.rs ring_indices): each level's (n + 1)²
// grid corners, then one vertex at each of its n² cells' centres; each
// cell is four triangles fanned around its centre, so the centre's
// height chooses the cell's diagonal and can move between the two
// diagonals smoothly. A corner keeps its place and moves only in height:
// near the level's outer edge (the morph) an odd corner goes from its
// own height to the coarser level's surface under it (CDLOD's geomorph
// on the coarse edge's midpoint, Strugar 2009; Losasso & Hoppe 2004), and
// a centre from its own diagonal to its coarser cell's. At the edge the
// level's triangles are the coarser level's: no cracks and no pops, and no
// diagonal flips mid-morph. Zero-area triangles along each morphing
// level's outer edge (its skirt) close the T-junctions where its odd
// corners meet the coarser level's edges.

// The level's grid units (its corners' positions are whole numbers of
// them from the terrain's -x, -z corner) to field coordinates, clamped to
// the terrain.
fn grid_clamp(c: vec2<f32>) -> vec2<f32> {
  return clamp(c, vec2<f32>(0.0), terrain.extent.xy / level.camera.z);
}

// The height (the terrain's space) the level draws at a clamped grid
// point: the field's.
fn grid_height(c: vec2<f32>) -> f32 {
  return terrain_height_at(0u, c * level.camera.z / terrain_cell());
}

// How far a point (grid units, unclamped) has morphed: 0 inside the
// band, 1 at its outer edge and past it, 0 always for the coarsest level.
fn morph_alpha(c: vec2<f32>) -> f32 {
  if (level.grid.z <= 0.0) { return 0.0; }
  let d = max(abs(c.x - level.camera.x), abs(c.y - level.camera.y)) / level.center.z;
  return clamp((d - level.grid.y) / level.grid.z, 0.0, 1.0);
}

// The diagonal a cell is split along from its corners' heights: true for
// (+x, -z) to (-x, +z), the one whose ends differ less (og_core's
// heightfield_zigzag on the CPU, ties included).
fn zigzag(h00: f32, h10: f32, h01: f32, h11: f32) -> bool {
  return abs(h00 - h11) > abs(h10 - h01);
}

// A coarser cell: its clamped corners and their heights.
struct Coarse {
  lo: vec2<f32>,
  hi: vec2<f32>,
  h: vec4<f32>,  // (-x -z, +x -z, -x +z, +x +z)
};

// The coarser level's cell at grid units `c` (unclamped): its corners are
// at multiples of twice this level's quad.
fn coarse_cell(c: vec2<f32>) -> Coarse {
  let s2 = 2.0 * level.center.z;
  let p = floor(c / s2) * s2;
  var o: Coarse;
  o.lo = grid_clamp(p);
  o.hi = grid_clamp(p + s2);
  o.h = vec4<f32>(
    grid_height(o.lo),
    grid_height(vec2<f32>(o.hi.x, o.lo.y)),
    grid_height(vec2<f32>(o.lo.x, o.hi.y)),
    grid_height(o.hi),
  );
  return o;
}

// The coarser level's surface over clamped grid point `m` in its cell:
// on the triangle of the cell's split under it.
fn coarse_height(k: Coarse, m: vec2<f32>) -> f32 {
  let span = k.hi - k.lo;
  let f = select(vec2<f32>(0.0), (m - k.lo) / max(span, vec2<f32>(1e-6)), span > vec2<f32>(0.0));
  let h = k.h;
  if (zigzag(h.x, h.y, h.z, h.w)) {
    if (f.x + f.y <= 1.0) { return h.x + f.x * (h.y - h.x) + f.y * (h.z - h.x); }
    return h.w + (1.0 - f.x) * (h.z - h.w) + (1.0 - f.y) * (h.y - h.w);
  }
  if (f.x >= f.y) { return h.x + f.x * (h.y - h.x) + f.y * (h.w - h.y); }
  return h.x + f.y * (h.z - h.x) + f.x * (h.w - h.z);
}

// Corner `g` of the level's grid (whole numbers, 0 to n) in grid units.
fn grid_corner(g: vec2<f32>) -> vec2<f32> {
  return level.center.xy + (g - level.grid.x * 0.5) * level.center.z;
}

// Whether corner `g` lies between the coarser level's corners (the level
// has a multiple of 4 quads a side, centred on a coarser corner).
fn grid_odd(g: vec2<f32>) -> bool {
  return any(g - 2.0 * floor(g * 0.5) > vec2<f32>(0.5));
}

// A corner's height after the morph, its own height `h` given.
fn morphed(g: vec2<f32>, c: vec2<f32>, m: vec2<f32>, h: f32, k: Coarse) -> f32 {
  let a = morph_alpha(c);
  if (a <= 0.0 || !grid_odd(g)) { return h; }
  return mix(h, coarse_height(k, m), a);
}

// Grid corner `g` in the terrain's space.
fn terrain_grid_corner(g: vec2<f32>) -> vec3<f32> {
  let c = grid_corner(g);
  let m = grid_clamp(c);
  var h = grid_height(m);
  if (morph_alpha(c) > 0.0 && grid_odd(g)) {
    h = morphed(g, c, m, h, coarse_cell(c));
  }
  let xz = terrain_min() + m * level.camera.z;
  return vec3<f32>(xz.x, h, xz.y);
}

// Cell `g`'s centre in the terrain's space: on its own diagonal's middle
// (its corners as morphed), moving to its coarser cell's diagonal over
// the morph. Its own diagonal is chosen from its corners' own heights and
// the coarser one from the coarser cell's, so neither changes mid-morph.
fn terrain_cell_centre(g: vec2<f32>) -> vec3<f32> {
  let s = level.center.z;
  let c00 = grid_corner(g);
  let k = coarse_cell(c00 + 0.5 * s);
  let cs = array<vec2<f32>, 4>(c00, c00 + vec2<f32>(s, 0.0), c00 + vec2<f32>(0.0, s), c00 + vec2<f32>(s, s));
  let gs = array<vec2<f32>, 4>(g, g + vec2<f32>(1.0, 0.0), g + vec2<f32>(0.0, 1.0), g + vec2<f32>(1.0, 1.0));
  var own: vec4<f32>;
  var mh: vec4<f32>;
  for (var i = 0u; i < 4u; i++) {
    let m = grid_clamp(cs[i]);
    own[i] = grid_height(m);
    mh[i] = morphed(gs[i], cs[i], m, own[i], k);
  }
  let main_mid = (mh.x + mh.w) * 0.5;
  let cross_mid = (mh.y + mh.z) * 0.5;
  let mine = select(main_mid, cross_mid, zigzag(own.x, own.y, own.z, own.w));
  let theirs = select(main_mid, cross_mid, zigzag(k.h.x, k.h.y, k.h.z, k.h.w));
  let h = mix(mine, theirs, morph_alpha(c00 + 0.5 * s));
  let m = (grid_clamp(cs[0]) + grid_clamp(cs[3])) * 0.5;
  let xz = terrain_min() + m * level.camera.z;
  return vec3<f32>(xz.x, h, xz.y);
}

// Vertex `vi` of the level's mesh in the terrain's space: a corner, then
// the cells' centres.
fn terrain_local_vertex(vi: u32) -> vec3<f32> {
  let n = u32(level.grid.x);
  let corners = (n + 1u) * (n + 1u);
  if (vi < corners) {
    return terrain_grid_corner(vec2<f32>(f32(vi % (n + 1u)), f32(vi / (n + 1u))));
  }
  let k = vi - corners;
  return terrain_cell_centre(vec2<f32>(f32(k % n), f32(k / n)));
}

