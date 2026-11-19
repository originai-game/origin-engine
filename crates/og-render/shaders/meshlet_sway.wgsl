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
// second autumn colour, 0), (the crown's half-extents, its extinction),
// the leaf scales of levels 0 to 3 and of 4 to 7, then per vertex of the mesh (its own index) the branch pivot and
// stiffness, the leaf anchor and flutter. So the region's second row is
// at the cluster row's w itself.

// The draw uniform (meshlet_gpu.rs `DrawParams`): x of flags 1 colours
// each instance by its level; this frame's wind and the last drawn's;
// the leaves' year (meshlet_foliage.wgsl `LeafYear`).
struct MeshletDraw {
  flags: vec4<u32>,
  wind: array<FoliageWind, 2>,
  leaf_year: LeafYear,
};

// meshlet.rs FOLIAGE_HEADER_ROWS.
const SWAY_HEADER_ROWS: u32 = 7u;

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

// The header row of the region `region` (the cluster row's w) that holds
// level `level`'s leaf scale (meshlet.rs `MeshFoliage::level_scales`).
fn meshlet_scale_row(region: u32, level: u32) -> u32 {
  return region - 1u + 5u + min(level, 7u) / 4u;
}

// Level `level`'s leaf scale from its header row (`meshlet_scale_row`):
// how much larger a thinned level of leaf cards draws each leaf.
fn meshlet_level_scale(row: vec4<f32>, level: u32) -> f32 {
  return row[min(level, 7u) % 4u];
}

// Where the leaf's year and its level put a vertex resting at `p`: drawn
// in to its anchor as the leaf is less than whole (a bud, a dropped leaf),
// out from it on a thinned level (`grow`: the year's kept share times the
// level's scale).
fn meshlet_grown(p: vec3<f32>, f1: vec4<f32>, grow: f32) -> vec3<f32> {
  if (grow == 1.0) { return p; }
  return f1.xyz + (p - f1.xyz) * grow;
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

// The path inside a crown (an ellipsoid of half-extents `h`, in the
// mesh's own space) from the point `q` (its position over `h`) along the
// unit direction `d`: from inside, to where the ray leaves; from outside,
// the chord ahead of the point.
fn meshlet_crown_path(q: vec3<f32>, d: vec3<f32>, h: vec3<f32>) -> f32 {
  let e = d / h;
  let qa = dot(e, e);
  let qb = dot(q, e);
  let qc = dot(q, q) - 1.0;
  let disc = qb * qb - qa * qc;
  if (disc <= 0.0) { return 0.0; }
  let root = sqrt(disc);
  return select(max(2.0 * root / qa, 0.0) * f32(qb < 0.0), (root - qb) / qa, qc < 0.0);
}

// A world direction `d` in the mesh's own space (normalised: a scaled
// instance turns it as its shape turns).
fn meshlet_model_dir(model: mat4x4<f32>, d: vec3<f32>) -> vec3<f32> {
  let m = transpose(mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz)) * d;
  if (dot(m, m) <= 1e-20) { return vec3<f32>(0.0, 1.0, 0.0); }
  return normalize(m);
}

// What the tree's own crown hides from the point `p` (the mesh's own
// space, at rest): x of the light around it, facing `axis` (its normal in
// that space; a leaf card's, which is lit from both faces, the canopy
// dome's), y of the sun towards `sun` (that space); 0 none, towards 1
// deep in a dense crown. After Hegeman, Premoze, Ashikhmin and Drettakis,
// "Approximate ambient occlusion for trees" (I3D 2006): the crown is an
// ellipsoid of randomly placed leaves, `h0` the region's first row (its
// centre) and `h4` its fifth (half-extents, extinction), and light
// reaches `p` along a direction through the leaves' transmittance,
// exp(-extinction x the path inside the crown), whether it comes from the
// sky or the ground: seven directions of the hemisphere about `axis`,
// cosine-weighted, and the sun's own.
fn meshlet_crown(p: vec3<f32>, axis: vec3<f32>, sun: vec3<f32>, h0: vec4<f32>, h4: vec4<f32>) -> vec2<f32> {
  if (h4.w <= 0.0 || min(h4.x, min(h4.y, h4.z)) <= 0.0 || dot(axis, axis) <= 1e-12) { return vec2<f32>(0.0); }
  let z = normalize(axis);
  let x = normalize(cross(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(z.y) > 0.9), z));
  let y = cross(z, x);
  let q = (p - h0.xyz) / h4.xyz;
  var seen = 0.0;
  for (var k = 0u; k < 7u; k++) {
    // The axis, then six at 50 degrees from it.
    var d = z;
    var w = 0.25;
    if (k > 0u) {
      let a = f32(k) * 1.0471976;
      d = z * 0.643 + (x * cos(a) + y * sin(a)) * 0.766;
      w = 0.125;
    }
    seen += w * exp(-h4.w * meshlet_crown_path(q, d, h4.xyz));
  }
  return vec2<f32>(1.0 - seen, 1.0 - exp(-h4.w * meshlet_crown_path(q, sun, h4.xyz)));
}

// `meshlet_crown`'s axis for a vertex of normal `n` (the mesh's own
// space): a leaf card's, the canopy dome at its anchor `f1` about the
// centre in the region's first row `h0`, with `h1` its second (x: 1 for a
// mesh of leaf cards).
fn meshlet_crown_axis(n: vec3<f32>, f1: vec4<f32>, h0: vec4<f32>, h1: vec4<f32>) -> vec3<f32> {
  if (h1.x > 0.5) { return foliage_normal(n, f1.xyz, h0.xyz, 1.0); }
  return n;
}
