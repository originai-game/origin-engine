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
