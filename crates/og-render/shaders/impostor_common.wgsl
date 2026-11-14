// Octahedral impostors (impostor.rs): the card, the frames it blends and
// their samples. Shared by the lit, prepass and shadow passes; follows
// scatter_common.wgsl (group 2: the impostor's sphere and grid) and
// material.wgsl (group 1: the atlas as base_color_tex and normal_tex).

// Whether the atlases hold the lower hemisphere's frames too, in their
// right half (impostor.rs `Impostor::below`).
fn imp_below() -> bool { return scatter.impostor_light.w > 0.5; }

// The direction the card is drawn for when the eye (or the light) stands
// along `v`: `v` itself where the atlases hold frames around it; without
// the lower hemisphere's a view from below is the view from the horizon
// above it, on a card standing upright. (Facing the eye there, the card
// showed the horizon's frames reprojected 20 degrees and more along
// their depth, which tore the model apart: its trunk's foot and the
// crown's edge drew as loose pieces.)
fn imp_view(v: vec3<f32>) -> vec3<f32> {
  if (imp_below() || v.y >= 0.0) { return v; }
  let level = vec3<f32>(v.x, 0.0, v.z);
  if (dot(level, level) < 1e-8) { return vec3<f32>(0.0, 1.0, 0.0); }
  return normalize(level);
}

// impostor.rs `hemi_oct_encode`: +y at the centre, the horizon on the
// border; below the horizon, the horizon above it.
fn imp_oct_encode(d: vec3<f32>) -> vec2<f32> {
  var h = vec3<f32>(d.x, max(d.y, 0.0), d.z);
  if (dot(h, h) < 1e-12) { h = vec3<f32>(0.0, 1.0, 0.0); }
  let n = h / (abs(h.x) + h.y + abs(h.z));
  return vec2<f32>(n.x + n.z, n.x - n.z);
}

fn imp_oct_decode(p: vec2<f32>) -> vec3<f32> {
  let x = (p.x + p.y) * 0.5;
  let z = (p.x - p.y) * 0.5;
  return normalize(vec3<f32>(x, max(1.0 - abs(x) - abs(z), 0.0), z));
}

fn imp_grid() -> u32 { return max(u32(scatter.impostor_grid.x), 2u); }

// Frame `f` (column + row × grid; from grid × grid on, the lower
// hemisphere's, each the frame of the upper one mirrored in y) looks from
// this direction.
fn imp_frame_dir(f: u32) -> vec3<f32> {
  let g = imp_grid();
  let n = f32(g - 1u);
  let k = f % (g * g);
  let d = imp_oct_decode(vec2<f32>(f32(k % g), f32(k / g)) / n * 2.0 - 1.0);
  return vec3<f32>(d.x, select(d.y, -d.y, f >= g * g), d.z);
}

// The grid cell around the direction `d` (impostor.rs `blend_frames`): its
// first frame, and `d`'s place in it. A direction below the horizon is in
// the lower hemisphere's frames where the atlases hold them.
struct ImpCell {
  base: u32,
  at: vec2<f32>,
};

fn imp_cell(d: vec3<f32>) -> ImpCell {
  let g = imp_grid();
  let n = f32(g - 1u);
  let lower = imp_below() && d.y < 0.0;
  let at = clamp((imp_oct_encode(vec3<f32>(d.x, abs(d.y), d.z)) * 0.5 + 0.5) * n, vec2<f32>(0.0), vec2<f32>(n));
  let cell = min(floor(at), vec2<f32>(n - 1.0));
  var o: ImpCell;
  o.base = u32(cell.x) + u32(cell.y) * g + select(0u, g * g, lower);
  o.at = at - cell;
  return o;
}

// impostor.rs `frame_basis`: the right of a view from `d` (up = cross(d, right)).
fn imp_right(d: vec3<f32>) -> vec3<f32> {
  let reference = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(d.y) > 0.999);
  return normalize(cross(reference, d));
}

struct ImpCard {
  world: vec4<f32>,
  // The card's vertex in the model's space (its sway's `local`).
  local: vec3<f32>,
  // Each frame's image of the card's vertex ([-1, 1]², y up): where the
  // frame's own orthographic view along its direction sees that point.
  uv01: vec4<f32>,
  uv2: vec2<f32>,
  // Per frame, how far its image moves per unit of depth along the eye
  // (the parallax step of `imp_parallax`).
  par01: vec4<f32>,
  par2: vec2<f32>,
  frames: vec3<u32>,
  weights: vec3<f32>,
  // The model's sphere radius in the world.
  extent: f32,
  // Per frame, the card's point's depth along it (imp_depths' `pd`, linear
  // over the card) and the eye's (`vd`).
  pd: vec3<f32>,
  vd: vec3<f32>,
};

// The card's point `pr` (about the centre, the model's space, in radii)
// against frames `frames`, seen along `v`: per frame, the point's own depth
// along the frame's direction (`pd`) and how much of a unit step along `v`
// is depth along it (`vd`, kept from 0.2 as imp_frame_par). A frame's
// stored depth is along its own direction, so the step to the surface
// along the eye is (depth - pd) / vd: the card's plane faces the eye, not
// the frame, and reading the depth as the step itself tilted every frame's
// sample by the angle between them (the far side's shading and rim).
struct ImpDepths {
  pd: vec3<f32>,
  vd: vec3<f32>,
};

fn imp_depths(pr: vec3<f32>, v: vec3<f32>, frames: vec3<u32>) -> ImpDepths {
  let d0 = imp_frame_dir(frames.x);
  let d1 = imp_frame_dir(frames.y);
  let d2 = imp_frame_dir(frames.z);
  var o: ImpDepths;
  o.pd = vec3<f32>(dot(pr, d0), dot(pr, d1), dot(pr, d2));
  o.vd = max(vec3<f32>(dot(v, d0), dot(v, d1), dot(v, d2)), vec3<f32>(0.2));
  return o;
}

// Frame `f`'s image of the point `x` (about the centre, the model's
// space, in radii): its orthographic projection along the frame's
// direction, in the frame's square. (Projecting along the eye instead
// threw the rim outside the frames around it and shrank the silhouette.)
fn imp_frame_uv(x: vec3<f32>, f: u32) -> vec2<f32> {
  let d = imp_frame_dir(f);
  let r = imp_right(d);
  return vec2<f32>(dot(x, r), dot(x, cross(d, r)));
}

// How far frame `f`'s image moves per radius of depth the surface stands
// towards the frame's viewer: the eye's ray `v`, rescaled from the
// frame's depth axis to the eye's.
fn imp_frame_par(v: vec3<f32>, f: u32) -> vec2<f32> {
  return imp_frame_uv(v, f) / max(dot(v, imp_frame_dir(f)), 0.2);
}

// The card's strips up its height (scatter.rs IMPOSTOR_CARD_VERTICES: six
// vertices each). The wind leans a model by its height squared
// (scatter_sway); one quad's two heights drew that as a straight line, so
// a trunk's lower half stood twice and more too far downwind and its foot
// slid with the gusts, a second image beside its mesh through the
// crossfade.
const IMP_ROWS: u32 = 4u;

// The card's corner `vertex` (0..6 × IMP_ROWS, two triangles a strip) of
// the copy `model` facing `toward` (a point, or a direction when `is_dir`).
fn imp_card(model: mat4x4<f32>, vertex: u32, toward: vec3<f32>, is_dir: bool) -> ImpCard {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
  );
  let corner = corners[vertex % 6u];
  let strip = f32(min(vertex / 6u, IMP_ROWS - 1u));
  let q = vec2<f32>(corner.x, (strip + corner.y * 0.5 + 0.5) / f32(IMP_ROWS) * 2.0 - 1.0);
  let m3 = mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz);
  let c = scatter.impostor.xyz;
  let radius = scatter.impostor.w;
  let centre = mul_point(model, vec4<f32>(c, 1.0)).xyz;
  let to_eye = select(toward - centre, toward, is_dir);
  // The eye's direction in the model's space (its scale divides out).
  let v = imp_view(normalize(transpose(m3) * to_eye));
  let r = imp_right(v);
  let u = cross(v, r);
  let p = (r * q.x + u * q.y) * radius;
  var o: ImpCard;
  o.world = mul_point(model, vec4<f32>(c + p, 1.0));
  o.local = c + p;
  // The three frames around v (impostor.rs `blend_frames`).
  let g = imp_grid();
  let cell = imp_cell(v);
  let f = cell.at;
  let base = cell.base;
  if (f.x + f.y <= 1.0) {
    o.frames = vec3<u32>(base, base + 1u, base + g);
    o.weights = vec3<f32>(1.0 - f.x - f.y, f.x, f.y);
  } else {
    o.frames = vec3<u32>(base + g + 1u, base + g, base + 1u);
    o.weights = vec3<f32>(f.x + f.y - 1.0, 1.0 - f.x, 1.0 - f.y);
  }
  let pr = p / radius;
  o.uv01 = vec4<f32>(imp_frame_uv(pr, o.frames.x), imp_frame_uv(pr, o.frames.y));
  o.uv2 = imp_frame_uv(pr, o.frames.z);
  o.par01 = vec4<f32>(imp_frame_par(v, o.frames.x), imp_frame_par(v, o.frames.y));
  o.par2 = imp_frame_par(v, o.frames.z);
  let dd = imp_depths(pr, v, o.frames);
  o.pd = dd.pd;
  o.vd = dd.vd;
  o.extent = radius * max(length(model[0].xyz), max(length(model[1].xyz), length(model[2].xyz)));
  return o;
}

// Where frame `f`'s point `uv` is in the atlas, kept half a texel inside
// the frame's square; `w` is zeroed outside it.
fn imp_atlas(f: u32, uv: vec2<f32>) -> vec2<f32> {
  let g = imp_grid();
  // Frames across and down: the lower hemisphere's stand right of the
  // upper one's.
  let cells = vec2<f32>(f32(select(g, 2u * g, imp_below())), f32(g));
  let size = vec2<f32>(textureDimensions(base_color_tex, 0));
  let half = 0.5 * cells / size;
  let inside = clamp(vec2<f32>(uv.x * 0.5 + 0.5, 0.5 - uv.y * 0.5), half, vec2<f32>(1.0) - half);
  let k = f % (g * g);
  return (vec2<f32>(f32(k % g + (f / (g * g)) * g), f32(k / g)) + inside) / cells;
}

fn imp_inside(uv: vec2<f32>) -> f32 {
  return select(0.0, 1.0, all(abs(uv) <= vec2<f32>(1.0)));
}

// Where frame `f` sees the surface the eye's ray through the card's point
// meets, from its image `uv` of the card's point moved by the depth it
// holds (`par`: imp_frame_par of that ray; `pd`: imp_depths). The depth is
// read two levels coarser than the colour, smooth where it falls steeply
// at the silhouette; a point moved past the frame's square is uncovered.
fn imp_parallax(f: u32, uv: vec2<f32>, par: vec2<f32>, pd: f32) -> vec2<f32> {
  // Two fixed-point steps: the depth read where the last step landed (one
  // step read it at the card's own image, off by the depth's slope).
  var at = uv;
  for (var k = 0; k < 2; k++) {
    let depth = textureSampleLevel(normal_tex, material_sampler, imp_atlas(f, at), imp_level + 2.0).a * 2.0 - 1.0;
    at = uv + par * (depth - pd);
  }
  return at;
}

// The frames' images of the card's point, each moved by one parallax step.
struct ImpUvs {
  uv01: vec4<f32>,
  uv2: vec2<f32>,
};

fn imp_uvs(uv01: vec4<f32>, uv2: vec2<f32>, par01: vec4<f32>, par2: vec2<f32>, frames: vec3<u32>, pd: vec3<f32>) -> ImpUvs {
  var o: ImpUvs;
  o.uv01 = vec4<f32>(imp_parallax(frames.x, uv01.xy, par01.xy, pd.x), imp_parallax(frames.y, uv01.zw, par01.zw, pd.y));
  o.uv2 = imp_parallax(frames.z, uv2, par2, pd.z);
  return o;
}

// Whether a pixel of a card is dropped (scatter_dropped's threshold): it
// keeps its pixels whose noise is at least `lo` and below `hi`, so a copy
// crossing to its impostor draws the pixels its mesh drops and no others.
fn imp_dropped(pixel: vec2<f32>, lo: f32, hi: f32) -> bool {
  let n = scatter_noise(pixel);
  return n < lo || n >= hi;
}

// A copy at `at`: its card's (lo, hi) of imp_dropped, the share its mesh
// shows to all of it. The distance fade shrinks the card as it does the
// mesh (scatter_scale), never by dropping pixels. A model impostor (no
// impostor distance) draws whole.
fn imp_fade(at: vec3<f32>) -> vec2<f32> {
  if (scatter.impostor_grid.y <= 0.0) { return vec2<f32>(0.0, 1.0); }
  return vec2<f32>(scatter_near(at), 1.0);
}

// How the wind moves the card's vertex `card` of the copy `i` scaled by
// `size`: as scatter_sway moves its mesh's point at the same height in the
// model's space, so the card leans and sways with the mesh it stands for
// through the crossfade. A model impostor's the same, by og.Model.sway
// (scatter.wind.z; 0: still).
fn imp_sway(card: ImpCard, i: ScatterInstance, size: f32, previous: bool) -> vec3<f32> {
  return scatter_sway(card.local, i.m3, length(i.m1) * size, previous);
}

// The card's size over the distance fade (scatter_scale; a model impostor's 1).
fn imp_scale(at: vec3<f32>) -> f32 {
  if (scatter.impostor_grid.y <= 0.0) { return 1.0; }
  return scatter_scale(at);
}

// The three frames' coverage by weight (the blend's alpha), each also
// returned scaled into `wa`.
struct ImpCoverage {
  wa: vec3<f32>,
  alpha: f32,
  a0: vec2<f32>,
  a1: vec2<f32>,
  a2: vec2<f32>,
};

fn imp_coverage(uv01: vec4<f32>, uv2: vec2<f32>, frames: vec3<u32>, weights: vec3<f32>) -> ImpCoverage {
  var o: ImpCoverage;
  o.a0 = imp_atlas(frames.x, uv01.xy);
  o.a1 = imp_atlas(frames.y, uv01.zw);
  o.a2 = imp_atlas(frames.z, uv2);
  let c = vec3<f32>(
    textureSampleLevel(base_color_tex, material_sampler, o.a0, imp_level).a * imp_inside(uv01.xy),
    textureSampleLevel(base_color_tex, material_sampler, o.a1, imp_level).a * imp_inside(uv01.zw),
    textureSampleLevel(base_color_tex, material_sampler, o.a2, imp_level).a * imp_inside(uv2),
  );
  o.wa = weights * c;
  o.alpha = (o.wa.x + o.wa.y + o.wa.z) * material.base_color.a;
  return o;
}

// How far the surface stands from the card's point along the eye (radii),
// each frame's from its depth (imp_depths), weighted as their coverage is.
fn imp_blended_step(c: ImpCoverage, dd: ImpDepths) -> f32 {
  let total = max(c.wa.x + c.wa.y + c.wa.z, 1e-5);
  let d = vec3<f32>(
    textureSampleLevel(normal_tex, material_sampler, c.a0, imp_level).a,
    textureSampleLevel(normal_tex, material_sampler, c.a1, imp_level).a,
    textureSampleLevel(normal_tex, material_sampler, c.a2, imp_level).a,
  ) * 2.0 - 1.0;
  return dot((d - dd.pd) / dd.vd, c.wa) / total;
}

// The atlases' level of detail for this pixel (imp_set_level), shared by
// every frame's samples: taken from the card's own coordinates, not the
// parallax-moved ones, whose derivatives jump where the depth falls
// steeply at the silhouette and would pick a coarse level there (a saw-
// tooth rim, by 2 × 2 quads).
var<private> imp_level: f32;

// Sets imp_level from frame coordinates `uv` ([-1, 1]² a frame) before
// any sample; called first in a fragment stage, in uniform control flow.
fn imp_set_level(uv: vec2<f32>) {
  let texels = f32(textureDimensions(base_color_tex, 0).y) / (2.0 * f32(imp_grid()));
  let d = max(length(dpdxFine(uv)), length(dpdyFine(uv))) * texels;
  imp_level = max(log2(max(d, 1e-6)), 0.0);
}

// The cut: the material's alpha test, half coverage without one.
fn imp_cutoff() -> f32 {
  return select(0.5, material.emissive.w, material.emissive.w >= 0.0);
}
