// Anchor-coherent branch/leaf motion adapted from SeedThree's wind.js,
// Copyright (c) 2026 SkyeShark, MIT (third_party/vegetation/SeedThree.LICENSE).
// GPU Gems 3 ch. 16 and Pivot Painter: technique references, no source copied.
// Hierarchy: leaf about its attachment, branch about its pivot, then trunk
// about its root. The cook keeps anchors categorical through every LOD.
// No bindings: callers supply the same current/previous state in every pass.

struct FoliageWind {
  dir_strength: vec4f, // xyz world wind direction, w lean
  gust_top: vec4f,     // gust, model height, two reduced wind_phases
  origin: vec4f,       // xyz current render origin, w wavenumber
  field: WindField,   // this drawn state's shared gust field
};

// Frame and ShadowPass keep this as vec4s so non-foliage shaders need no
// wind includes. The layout is checked against FoliageWind on the CPU.
fn foliage_wind_unpack(v: array<vec4f, 9>) -> FoliageWind {
  return FoliageWind(v[0], v[1], v[2], WindField(v[3], v[4], v[5], v[6], v[7], v[8]));
}

// SeedThree's canopy dome, blended before the inverse-transpose and normal
// mapping. The centre comes from all leaf anchors, not the trunk's bounds.
fn foliage_normal(n: vec3f, anchor: vec3f, center: vec3f, strength: f32) -> vec3f {
  if (strength <= 0.0) { return n; }
  let radial = anchor - center;
  var dome = vec3f(0.0, 1.0, 0.0);
  if (dot(radial, radial) > 1e-12) {
    dome = normalize(normalize(radial) + vec3f(0.0, 0.45, 0.0));
  }
  let blended = mix(n, dome, clamp(strength, 0.0, 1.0));
  if (dot(blended, blended) <= 1e-12) { return dome; }
  return normalize(blended);
}

fn foliage_rotate(p: vec3f, axis: vec3f, angle: f32) -> vec3f {
  let c = cos(angle);
  return p * c + cross(axis, p) * sin(angle) + axis * dot(axis, p) * (1.0 - c);
}

// World-space offset, added to model * p (not transformed a second time).
// f0 = branch pivot/stiffness; f1 = leaf anchor/flutter. Bark has flutter 0.
// `lean` is the tree's lean at its root (world x, z, its sway included):
// wind_common.wgsl wind_lean, the wind field every vegetation tier reads,
// so the trunk bends where and as far as the field blows, gusts and
// motors included; branches and leaves add og.Wind's travelling wave.
fn foliage_offset(p: vec3f, f0: vec4f, f1: vec4f, model: mat4x4f, wind: FoliageWind, lean_xz: vec2f) -> vec3f {
  let lean = length(lean_xz);
  if (lean <= 1e-6 || wind.gust_top.y <= 0.0) { return vec3f(0.0); }
  // Bring the wind into model space before bending: rotated and scaled
  // instances still lean with the world wind, not with their own +x.
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let d = vec3f(lean_xz.x, 0.0, lean_xz.y) / lean;
  let inverse_d = vec3f(dot(cross(b, c), d), dot(cross(c, a), d), dot(cross(a, b), d)) / dot(a, cross(b, c));
  let horizontal = vec3f(inverse_d.x, 0.0, inverse_d.z);
  if (dot(horizontal, horizontal) <= 1e-12) { return vec3f(0.0); }
  let direction = normalize(horizontal);
  let axis = cross(vec3f(0.0, 1.0, 0.0), direction);
  let branch_world = (model * vec4f(f0.xyz, 1.0)).xyz;
  let branch_phase = wind.origin.w * dot(branch_world + wind.origin.xyz, d);
  let branch_wave = (sin(branch_phase - wind.gust_top.z) + 0.5 * sin(2.3 * branch_phase + 1.7 - wind.gust_top.w)) / 1.5;
  let leaf_phase = dot(f1.xyz, vec3f(1.7, 2.3, 1.1)) + branch_phase;
  // Integer harmonics of the reduced phases are continuous at phase wrap.
  let flutter = (sin(3.0 * wind.gust_top.z + leaf_phase) + 0.5 * sin(2.0 * wind.gust_top.w + leaf_phase)) / 1.5;
  let leaf_axis = normalize(direction + vec3f(0.0, 0.6, 0.0));
  var moved = f1.xyz + foliage_rotate(p - f1.xyz, leaf_axis, atan(0.35 * lean * f1.w) * flutter);
  moved = f0.xyz + foliage_rotate(moved - f0.xyz, axis, atan(lean * (1.0 - f0.w)) * branch_wave);
  let height = clamp(moved.y / wind.gust_top.y, 0.0, 1.0);
  moved = foliage_rotate(moved, axis, atan(lean * height * height));
  return (model * vec4f(moved - p, 0.0)).xyz;
}

// A stable dither seed. Pass the absolute root, not a camera-relative or
// sorted slot index; rebasing and culling must not reshuffle the dither.
fn foliage_seed(world_root: vec3f) -> u32 {
  var h = 2166136261u;
  for (var i = 0u; i < 3u; i++) {
    h = (h ^ bitcast<u32>(world_root[i])) * 16777619u;
  }
  return h ^ (h >> 16u);
}

// An instance's sway from its record's layer word (instances.rs
// `layer_bits`: the receive-shadows flag folded into the sign; a run
// without array maps carries `Instance::sway` there). 0: still.
fn foliage_sway(layer: f32) -> f32 {
  return max(select(-layer - 1.0, layer, layer >= 0.0), 0.0);
}

// ── The leaves' year (og.Season) ───────────────────────────────────────
// Binding-free, after hash.wgsl. Each leaf of a seasonal species grows
// from its bud, turns and drops on its own day, and the wood it hangs
// from stays: the leaf's lot comes from its anchor (`leaf_order_pick`,
// foliage.rs's twin), its tree's from where the tree stands
// (`foliage_tree`), and the days from og.Season's climate (og-env
// season.rs `LeafYear`, which og.atmosphere.season's leafOut, leafTurn
// and leafFall evaluate for the mean tree). Autumn colour by the
// pigments' order through the year (Braitmaier, Diepstraten and Ertl,
// "Real-time rendering of seasonal influenced trees", TPCG 2004: the
// chlorophyll's green gives way to the carotenoids' yellow and orange
// and the anthocyanins' red, then the dead leaf's tannin brown) and leaf
// by leaf fall (Desbenoit, Galin and Akkouche, "Modeling autumn sceneries",
// Eurographics 2006 short papers: each leaf falls on its own date, the
// crown thinning from the leaves that turned first). No source copied.

// foliage.rs LeafCalendar::gpu: year (the bud's day, the days growing,
// opening and turning), fall (the days after the turn starts that the
// first and the last leaf fall, the date as the place lives it, the kind:
// 0 every leaf as modelled, 1 a seasonal year, 2 bare all year).
struct LeafYear {
  year: vec4f,
  fall: vec4f,
};

// A leaf in its year: how much of it is there (0 fallen or in bud, 1
// whole: a scale about its anchor), how far it has turned to its autumn
// colour and how far it has browned before it drops.
struct SeasonLeaf {
  kept: f32,
  turn: f32,
  brown: f32,
};

// The days the trees of one species bud, turn and drop apart.
const LEAF_TREE_DAYS: f32 = 10.0;
// The share of the turn the first leaves to fall lead the last by.
const LEAF_TURN_LEAD: f32 = 0.6;
// The share of the fall over which one leaf drops (shrinks to its anchor).
const LEAF_DROP: f32 = 0.03;
// The share of the fall before its drop over which a leaf browns.
const LEAF_BROWN: f32 = 0.25;
// A dead leaf's tannin brown (linear).
const LEAF_BROWN_COLOR: vec3f = vec3f(0.19, 0.08, 0.026);

// foliage.rs `leaf_order_pick`, operation for operation: the anchor to
// the millimetre (rounded half away from zero, as Rust's `round`), its
// lattice hash, then pcg twice. x: the leaf's order among its tree's
// (0 turns and falls first), y: where its autumn colour lies.
fn leaf_order_pick(anchor: vec3f) -> vec2f {
  let s = anchor * 1000.0;
  let t = trunc(s);
  let r = t + select(vec3f(0.0), sign(s), abs(s - t) >= vec3f(0.5));
  let q = vec3<i32>(r);
  let h = (bitcast<u32>(q.x) * 73856093u) ^ (bitcast<u32>(q.y) * 19349663u) ^ (bitcast<u32>(q.z) * 83492791u);
  let order = pcg(h);
  return vec2f(hash_to_unit(order), hash_to_unit(pcg(order)));
}

// A tree's lot (0..1) among its species: from where its root stands in
// the world, to the centimetre, so moving the render origin never moves
// a tree's day.
fn foliage_tree(world_root: vec3f) -> f32 {
  let s = world_root * 100.0;
  let q = vec3<i32>(floor(s + 0.5));
  return hash_unit((bitcast<u32>(q.x) * 73856093u) ^ (bitcast<u32>(q.y) * 19349663u) ^ (bitcast<u32>(q.z) * 83492791u) ^ 0x5ea50a1u);
}

// og-env season.rs `smooth`.
fn leaf_smooth(a: f32, b: f32, x: f32) -> f32 {
  if (b <= a) { return select(1.0, 0.0, x < a); }
  let t = clamp((x - a) / (b - a), 0.0, 1.0);
  return t * t * (3.0 - 2.0 * t);
}

// The leaf of lot `order` on a tree of lot `tree`, of a species that
// follows the year by `season` (0 evergreen, 1 deciduous) and runs
// `shift` days late, in `y`. Over a tree's leaves the mean `kept` is
// og-env's canopy share (out times what has not fallen) at the tree's
// day: what og.atmosphere.season answers is what is drawn.
fn season_leaf(order: f32, tree: f32, season: f32, shift: f32, y: LeafYear) -> SeasonLeaf {
  var s = SeasonLeaf(1.0, 0.0, 0.0);
  let follow = clamp(season, 0.0, 1.0);
  if (y.fall.w < 0.5 || follow <= 0.0) { return s; }
  if (y.fall.w > 1.5) {
    s.kept = 1.0 - follow;
    return s;
  }
  let day = y.fall.z - shift - LEAF_TREE_DAYS * (tree - 0.5) - y.year.x;
  let x = day - 365.0 * floor(day / 365.0);
  if (x < y.year.y) {
    s.kept = leaf_smooth(0.0, y.year.z, x);
  } else {
    let after = x - y.year.y;
    let fallen = leaf_smooth(y.fall.x, y.fall.y, after);
    let turned = leaf_smooth(0.0, y.year.w, after);
    // The leaf drops while the fallen share passes its order.
    let drop_at = order * (1.0 - LEAF_DROP);
    s.kept = clamp(1.0 - (fallen - drop_at) / LEAF_DROP, 0.0, 1.0);
    s.turn = clamp((turned - LEAF_TURN_LEAD * order) / (1.0 - LEAF_TURN_LEAD), 0.0, 1.0);
    s.brown = clamp(1.0 - (drop_at - fallen) / LEAF_BROWN, 0.0, 1.0);
  }
  s.kept = mix(1.0, s.kept, follow);
  s.turn *= follow;
  s.brown *= follow;
  return s;
}

// The colour the leaf of colour lot `pick` has turned to, between its
// species' `autumn` and `autumn2` and then brown (rgb, linear), and how
// far it replaces the leaf's own colour (a: 0 its own). A caller mixes
// its own colour (the base colour times the vertex colour, before the
// texture multiplies it) towards rgb by a.
fn season_leaf_color(autumn: vec3f, autumn2: vec3f, pick: f32, s: SeasonLeaf) -> vec4f {
  let a = max(s.turn, s.brown);
  if (a <= 0.0) { return vec4f(0.0); }
  return vec4f(mix(mix(autumn, autumn2, pick), LEAF_BROWN_COLOR, s.brown / a), a);
}

// A vertex's leaf in its year (foliage.wgsl `season_leaf`) and
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

// The leaves a far copy's card draws at `p` (the model's space about its
// centre, at its scale; an impostor's leaf texel): the crown's cell of
// `CROWN_CELL` metres about `p` drawn as one leaf in its year
// (`meshlet_leaf`), its lot from the cell, on the tree standing at
// `world_root`, so the card's crown thins and turns leaf by leaf as its
// mesh's does, in clumps a leaf's size that stay where they are as the
// eye moves.
const CROWN_CELL: f32 = 0.1;

fn crown_cell_leaf(h1: vec4<f32>, p: vec3<f32>, world_root: vec3<f32>, year: LeafYear) -> MeshletLeaf {
  var l = MeshletLeaf(SeasonLeaf(1.0, 0.0, 0.0), 0.0);
  if (h1.y <= 0.0 || year.fall.w < 0.5) { return l; }
  let lot = leaf_order_pick(floor(p / CROWN_CELL) * CROWN_CELL);
  l.leaf = season_leaf(lot.x, foliage_tree(world_root), h1.y, h1.z, year);
  l.pick = lot.y;
  return l;
}

// Where the leaf's year and its level put a vertex resting at `p`: drawn
// in to its anchor as the leaf is less than whole (a bud, a dropped leaf),
// out from it on a thinned level (`grow`: the year's kept share times the
// level's scale).
fn meshlet_grown(p: vec3<f32>, f1: vec4<f32>, grow: f32) -> vec3<f32> {
  if (grow == 1.0) { return p; }
  return f1.xyz + (p - f1.xyz) * grow;
}

// The vertex's turned colour (foliage.wgsl `season_leaf_color`),
// from the region's third and fourth rows.
fn meshlet_leaf_color(h2: vec4<f32>, h3: vec4<f32>, l: MeshletLeaf) -> vec4<f32> {
  return season_leaf_color(h2.xyz, h3.xyz, l.pick, l.leaf);
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
