// The meshlet path's foliage sway (meshlet.rs `MeshFoliage`), after
// grass_common.wgsl and meshlet_foliage.wgsl (the vegetation's
// deformation). No bindings: each pull (meshlet.wgsl, meshlet_depth.wgsl,
// meshlet_shadow.wgsl, meshlet_raster.wgsl) reads the vertex pool's
// foliage region itself and passes the rows here, so every pass moves a
// vertex by the same numbers.
//
// A foliage mesh's cluster rows carry its region in their first row's w
// (meshlet_gpu.rs `pool_foliage`: the region's first vec4 of the vertex
// pool plus one, 0 for none). The region is (canopy centre, top), (flags:
// 1 leaves, its season, its season shift, 0), (autumn colour, 0), (the
// second autumn colour, 0), then per vertex of the mesh (its own index)
// the branch pivot and stiffness, the leaf anchor and flutter. So the
// region's second row is at the cluster row's w itself.

// The draw uniform (meshlet_gpu.rs `DrawParams`): x of flags 1 colours
// each instance by its level; this frame's wind and the last drawn's;
// the leaves' year (meshlet_foliage.wgsl `LeafYear`).
struct MeshletDraw {
  flags: vec4<u32>,
  wind: array<FoliageWind, 2>,
  leaf_year: LeafYear,
};

// meshlet.rs FOLIAGE_HEADER_ROWS.
const SWAY_HEADER_ROWS: u32 = 4u;

// A vertex's leaf in its year (meshlet_foliage.wgsl `season_leaf`) and
// its colour lot: `h1` the region's second row (its season and shift),
// `f1` the vertex's anchor row, `model` its instance now and `origin`
// the render origin, so every pass and both frames of a motion vector
// draw the leaf at one size.
struct MeshletLeaf {
  leaf: SeasonLeaf,
  pick: f32,
};

fn meshlet_leaf(h1: vec4<f32>, f1: vec4<f32>, model: mat4x4<f32>, origin: vec3<f32>, year: LeafYear) -> MeshletLeaf {
  var l = MeshletLeaf(SeasonLeaf(1.0, 0.0, 0.0), 0.0);
  if (h1.y <= 0.0 || year.fall.w < 0.5) { return l; }
  let lot = leaf_order_pick(f1.xyz);
  l.leaf = season_leaf(lot.x, foliage_tree(model[3].xyz + origin), h1.y, h1.z, year);
  l.pick = lot.y;
  return l;
}

// Where the leaf's year puts a vertex resting at `p`: drawn in to its
// anchor as the leaf is less than whole (a bud, a dropped leaf).
fn meshlet_grown(p: vec3<f32>, f1: vec4<f32>, kept: f32) -> vec3<f32> {
  if (kept >= 1.0) { return p; }
  return f1.xyz + (p - f1.xyz) * kept;
}

// The vertex's turned colour (meshlet_foliage.wgsl `season_leaf_color`),
// from the region's third and fourth rows.
fn meshlet_leaf_color(h2: vec4<f32>, h3: vec4<f32>, l: MeshletLeaf) -> vec4<f32> {
  return season_leaf_color(h2.xyz, h3.xyz, l.pick, l.leaf);
}

// The first vec4 of vertex `index`'s two rows in the region `region`
// (the cluster row's w, which is the region plus one).
fn meshlet_sway_row(region: u32, index: u32) -> u32 {
  return region - 1u + SWAY_HEADER_ROWS + index * 2u;
}

// How far `wind` moves the vertex at `p` (the mesh's own space) of an
// instance at `model` swaying by `sway`, in the mesh's own space:
// `foliage_offset`'s world offset carried back through the model's 3x3,
// so the pull adds it to the vertex before the shared vertex code runs.
// `header` is the region's first row (its w the model's top). The trunk
// leans by the wind field at the instance's root (`meshlet_wind_tex`, the
// including module's binding; wind_common.wgsl wind_lean), now or as last
// drawn (`previous`).
fn meshlet_sway(p: vec3<f32>, header: vec4<f32>, f0: vec4<f32>, f1: vec4<f32>,
                model: mat4x4<f32>, wind: FoliageWind, sway: f32, previous: bool) -> vec3<f32> {
  if (sway <= 0.0) { return vec3<f32>(0.0); }
  var w = wind;
  w.gust_top.y = header.w;
  let lean4 = wind_lean(meshlet_wind_tex, wind.field, model[3].xz);
  let lean = select(lean4.xy, lean4.zw, previous) * sway;
  let world = foliage_offset(p, f0, f1, model, w, lean);
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let det = dot(a, cross(b, c));
  if (abs(det) <= 1e-20) { return vec3<f32>(0.0); }
  return vec3<f32>(dot(cross(b, c), world), dot(cross(c, a), world), dot(cross(a, b), world)) / det;
}
