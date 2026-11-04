// The terrain's clipmap mesh with its displacement, for the lit and
// shadow modules (after terrain.wgsl and terrain_layers.wgsl).
//
// The clipmap's mesh (terrain.rs ring_indices): each level's (n + 1)²
// grid corners, then one vertex at each of its n² cells' centres; each
// cell is four triangles fanned around its centre, so the centre's
// height chooses the cell's diagonal and can move between the two
// diagonals smoothly. A corner keeps its place and moves only in height:
// near the level's outer edge (the morph) a corner goes from its own
// height to the coarser level's surface under it (CDLOD's geomorph on the
// coarse edge's midpoint, Strugar 2009; Losasso & Hoppe 2004: an odd
// corner to the coarser edge or diagonal it lies on, an even one to the
// coarser corner as that level reads it, which differs only displaced), and
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

// The field's height (the terrain's space) at field coordinates `s`: a
// sample's own, and between samples on the triangle of its cell's split
// (og_core's heightfield_zigzag, as collision and og.terrain.sample
// answer), so levels finer than the field draw the ground the bodies on
// it stand on.
fn field_height(s: vec2<f32>) -> f32 {
  let i = floor(s);
  let f = s - i;
  if (all(f < vec2<f32>(1e-4))) { return terrain_height_at(0u, s); }
  let h = vec4<f32>(
    terrain_height_at(0u, i),
    terrain_height_at(0u, i + vec2<f32>(1.0, 0.0)),
    terrain_height_at(0u, i + vec2<f32>(0.0, 1.0)),
    terrain_height_at(0u, i + vec2<f32>(1.0, 1.0)),
  );
  return split_height(h, f);
}

// The height over a cell of corner heights `h` (-x -z, +x -z, -x +z,
// +x +z) at `f` (0 to 1 along x and z) on the triangle of its split.
fn split_height(h: vec4<f32>, f: vec2<f32>) -> f32 {
  if (zigzag(h.x, h.y, h.z, h.w)) {
    if (f.x + f.y <= 1.0) { return h.x + f.x * (h.y - h.x) + f.y * (h.z - h.x); }
    return h.w + (1.0 - f.x) * (h.z - h.w) + (1.0 - f.y) * (h.y - h.w);
  }
  if (f.x >= f.y) { return h.x + f.x * (h.y - h.x) + f.y * (h.w - h.y); }
  return h.x + f.y * (h.z - h.x) + f.x * (h.w - h.z);
}

// The height (the terrain's space) a level of `step` grid units a quad
// draws at clamped grid point `c`: the field's, displaced near the camera
// by its layers' heights. The coarser level's corners are read at its own
// step, so both levels draw a corner they share alike.
fn grid_height(c: vec2<f32>, step: f32) -> f32 {
  let s = c * level.camera.z / terrain_cell();
  let h = field_height(s);
  return h + terrain_displacement(c, s, h, step);
}

// ── Displacement ────────────────────────────────────────────────────────
// Near the camera the drawn surface takes the relief its layers' height
// maps (their albedo's alpha) describe: each grid point moves up or down
// by its blended layers' heights about their middle times each layer's
// `displacement` (world metres), the layers and their weights the ones
// the lit pass shades it with, the maps read over the level's spacing (so
// a coarser level carries a smoother relief and nothing aliases). It
// fades out towards `displace.x` from the camera and on steep ground (it
// moves the ground up; a cliff's face keeps its place). Clipmap levels
// finer than the field (terrain.rs Terrain::clipmap) carry it near the
// eye: the tessellation and displacement of Unreal's Nanite landscapes
// and Frostbite's terrain, built from the clipmap's own levels instead of
// a tessellator (WebGPU has none). Only drawn: collision,
// og.terrain.sample and the id pass keep the field.

// The displacement at clamped grid point `c`, field coordinates `s`,
// over field height `h`, read for a level of `step` grid units a quad
// (`terrain_relief`'s `d`).
fn terrain_displacement(c: vec2<f32>, s: vec2<f32>, h: f32, step: f32) -> f32 {
  return terrain_relief(c, s, h, step, false).d;
}

// The displaced surface at a grid point (the terrain's space): `d` the
// height it adds, `slope` d's change along x and z (per metre; with
// `gradient`, else 0) and `bare` the part of `slope` from layers without
// a normal map, whose relief the shading then has to draw itself. The
// slope is the layers' heights differenced over the level's spacing (the
// footprint their maps are read over), their weights held.
struct Relief {
  d: f32,
  slope: vec2<f32>,
  bare: vec2<f32>,
};

fn terrain_relief(c: vec2<f32>, s: vec2<f32>, h: f32, step: f32, gradient: bool) -> Relief {
  var out: Relief;
  let reach = terrain.displace.x;
  if (reach <= 0.0) { return out; }
  let unit = level.camera.z;
  let dist = length(vec3<f32>((c.x - level.camera.x) * unit, (h - level.camera.w) * terrain.displace.z, (c.y - level.camera.y) * unit));
  let fade = 1.0 - smoothstep(0.6 * reach, reach, dist);
  if (fade <= 0.0) { return out; }
  // The level's spacing along x and z, in field coordinates and in metres.
  let spacing = step * unit;
  let gx = vec2<f32>(spacing / terrain_cell().x, 0.0);
  let gy = vec2<f32>(0.0, spacing / terrain_cell().y);
  let maps = terrain_maps_at(s, gx, gy);
  let n = normalize(terrain.normal * maps.normal);
  let steep = smoothstep(0.5, 0.8, n.y);
  if (steep <= 0.0) { return out; }
  let local = vec3<f32>(terrain_min().x + c.x * unit, h, terrain_min().y + c.y * unit);
  let p = terrain_world(local);
  let pick = pick_layers(p, n, maps.control);
  var any = false;
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && pick.w[k] > 0.0 && terrain.layers[pick.ids[k]].offset.w != 0.0) { any = true; }
  }
  if (!any) { return out; }
  let dpx = (terrain.model * vec4<f32>(spacing, 0.0, 0.0, 0.0)).xyz;
  let dpy = (terrain.model * vec4<f32>(0.0, 0.0, spacing, 0.0)).xyz;
  // As the lit pass weighs them (a layer without a map stands at its
  // full height there), each moving the ground by its own height about
  // its middle (one without a map, not at all).
  var heights = vec4<f32>(1.0);
  var relief = vec4<f32>(0.0);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && (pick.w[k] > 0.0 || k == 0u)) {
      let lh = layer_height(pick.ids[k], p, n, dpx, dpy);
      heights[k] = select(1.0, lh, lh >= 0.0);
      relief[k] = select(0.0, lh - 0.5, lh >= 0.0);
    }
  }
  let w = blend_weights(pick, heights);
  let scale = fade * steep / max(terrain.displace.y, 1e-6);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      let l = terrain.layers[pick.ids[k]];
      let amount = l.offset.w * w[k] * scale;
      out.d += relief[k] * amount;
      if (gradient && amount != 0.0 && (u32(l.mapping.w) & 1u) != 0u) {
        let sx = (layer_height(pick.ids[k], p + dpx, n, dpx, dpy) - 0.5 - relief[k]) * amount / spacing;
        let sz = (layer_height(pick.ids[k], p + dpy, n, dpx, dpy) - 0.5 - relief[k]) * amount / spacing;
        out.slope += vec2<f32>(sx, sz);
        if ((u32(l.mapping.w) & 2u) == 0u) { out.bare += vec2<f32>(sx, sz); }
      }
    }
  }
  return out;
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
    grid_height(o.lo, s2),
    grid_height(vec2<f32>(o.hi.x, o.lo.y), s2),
    grid_height(vec2<f32>(o.lo.x, o.hi.y), s2),
    grid_height(o.hi, s2),
  );
  return o;
}

// The coarser level's surface over clamped grid point `m` in its cell:
// on the triangle of the cell's split under it.
fn coarse_height(k: Coarse, m: vec2<f32>) -> f32 {
  let span = k.hi - k.lo;
  let f = select(vec2<f32>(0.0), (m - k.lo) / max(span, vec2<f32>(1e-6)), span > vec2<f32>(0.0));
  return split_height(k.h, f);
}

// Corner `g` of the level's grid (whole numbers, 0 to n) in grid units.
fn grid_corner(g: vec2<f32>) -> vec2<f32> {
  return level.center.xy + (g - level.grid.x * 0.5) * level.center.z;
}

// A corner's height after the morph, its own height `h` given: towards
// the coarser level's surface under it (an even corner, the coarser
// level's own corner as that level reads it).
fn morphed(c: vec2<f32>, m: vec2<f32>, h: f32, k: Coarse) -> f32 {
  let a = morph_alpha(c);
  if (a <= 0.0) { return h; }
  return mix(h, coarse_height(k, m), a);
}

// Grid corner `g` in the terrain's space.
fn terrain_grid_corner(g: vec2<f32>) -> vec3<f32> {
  let c = grid_corner(g);
  let m = grid_clamp(c);
  var h = grid_height(m, level.center.z);
  if (morph_alpha(c) > 0.0) {
    h = morphed(c, m, h, coarse_cell(c));
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
  let cs = array<vec2<f32>, 4>(c00, c00 + vec2<f32>(s, 0.0), c00 + vec2<f32>(0.0, s), c00 + vec2<f32>(s, s));
  var own: vec4<f32>;
  var morphing = morph_alpha(c00 + 0.5 * s) > 0.0;
  for (var i = 0u; i < 4u; i++) {
    own[i] = grid_height(grid_clamp(cs[i]), s);
    morphing = morphing || morph_alpha(cs[i]) > 0.0;
  }
  let flip = zigzag(own.x, own.y, own.z, own.w);
  var h = select((own.x + own.w) * 0.5, (own.y + own.z) * 0.5, flip);
  // In the morph band (the coarser cell read only there: displaced, its
  // corners are the costliest heights a vertex reads).
  if (morphing) {
    let k = coarse_cell(c00 + 0.5 * s);
    var mh: vec4<f32>;
    for (var i = 0u; i < 4u; i++) {
      mh[i] = morphed(cs[i], grid_clamp(cs[i]), own[i], k);
    }
    let main_mid = (mh.x + mh.w) * 0.5;
    let cross_mid = (mh.y + mh.z) * 0.5;
    let mine = select(main_mid, cross_mid, flip);
    let theirs = select(main_mid, cross_mid, zigzag(k.h.x, k.h.y, k.h.z, k.h.w));
    h = mix(mine, theirs, morph_alpha(c00 + 0.5 * s));
  }
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

