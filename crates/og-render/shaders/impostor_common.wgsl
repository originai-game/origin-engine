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
// crossfade. IMP_ROWS: og-render-data's CARD_ROWS (`march_wgsl`).

// The card's corner `vertex` (0..6 × IMP_ROWS, two triangles a strip) of
// the copy `model` facing `toward` (a point, or a direction when `is_dir`).
// The strips span the cook's outline of what the frames draw (impostor.rs
// CardTrim), not the sphere's square: the rest only shaded to discard,
// in every pass. Each strip boundary's width is shared by the strips
// either side, so the outline has no cracks.
fn imp_card(model: mat4x4<f32>, vertex: u32, toward: vec3<f32>, is_dir: bool) -> ImpCard {
  let corner = quad_corner(vertex) * 2.0 - 1.0;
  let strip = min(vertex / 6u, IMP_ROWS - 1u);
  let row = strip + select(0u, 1u, corner.y > 0.0);
  let trim = scatter.impostor_trim;
  let width = select(trim[0][min(row, 3u)], trim[1].x, row == IMP_ROWS);
  let q = vec2<f32>(corner.x * width, mix(trim[1].y, trim[1].z, f32(row) / f32(IMP_ROWS)));
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

// Frame `f`'s depth towards its viewer (radii) at its point `at`, read at
// `level`, and whether a surface covers that point.
struct ImpDepthAt {
  depth: f32,
  covered: bool,
};

fn imp_depth_at(f: u32, at: vec2<f32>, level: f32) -> ImpDepthAt {
  let a = imp_atlas(f, at);
  var o: ImpDepthAt;
  o.depth = textureSampleLevel(normal_tex, material_sampler, a, level).a * 2.0 - 1.0;
  o.covered = textureSampleLevel(base_color_tex, material_sampler, a, level).a * imp_inside(at) > 0.5;
  return o;
}

// The march's rules (IMP_FIXED_STEPS, IMP_COARSE, IMP_LANDED, IMP_STEP,
// IMP_STEPS, IMP_BISECT, IMP_NEAR, IMP_AGREE, IMP_MARCH_LEVEL, IMP_THICK)
// and IMP_ROWS are og-render-data's (impostor.rs `march`, written before
// this module by `march_wgsl`), which the cook bounds a card's outline by.

// Where frame `f` sees the surface the eye's ray through the card's point
// meets, from its image `uv` of the card's point (`par`: imp_frame_par of
// that ray; `pd`: imp_depths). The ray's point at frame depth z is
// uv + par (z - pd); the surface is the first z, from the eye's side, whose
// frame depth is at least z on a covered texel. A point moved past the
// frame's square, or a ray that meets nothing, is uncovered. Returns the
// point in the frame's square and z there.
//
// IMP_FIXED_STEPS fixed-point steps on the depth IMP_COARSE levels coarser find it where the
// depth is smooth (most of a card, and every card seen from above). Seen
// from below, a thin trunk stands before the crown: the fixed point jumps
// between the two sides of that step in depth, each frame lands on a
// different one, and the blend drew a blotched trunk with loose pieces of
// crown beside it. With `march` (the eye below the card's horizon), a
// landing the fine depth disagrees with, or an uncovered one beside the
// model, marches the ray instead. Only cards read at the atlas's finest
// level march: a coarser card's trunk is a few pixels wide, and marching
// a screen of them cost the main pass as much again (Metal, 300 cards).
fn imp_parallax(f: u32, uv: vec2<f32>, par: vec2<f32>, pd: f32, march: bool) -> vec3<f32> {
  var at = uv;
  var z = 0.0;
  for (var k = 0u; k < IMP_FIXED_STEPS; k++) {
    z = imp_depth_at(f, at, imp_level + IMP_COARSE).depth;
    at = uv + par * (z - pd);
  }
  if (!march || imp_level >= IMP_MARCH_LEVEL) {
    return vec3<f32>(at, z);
  }
  // The fine depth where the fixed point landed agrees with it.
  let landed = imp_depth_at(f, at, imp_level);
  if (landed.covered && abs(landed.depth - z) < IMP_LANDED * exp2(imp_level)) {
    return vec3<f32>(at, z);
  }
  if (!landed.covered && textureSampleLevel(base_color_tex, material_sampler, imp_atlas(f, at), imp_level + IMP_NEAR).a <= 0.0) {
    return vec3<f32>(at, z);
  }
  // The ray at frame depth 0, and the part of it inside the model's
  // sphere (every surface the frame holds is): |o + par z|² + z² ≤ 1.
  let o = uv - par * pd;
  let qa = dot(par, par) + 1.0;
  let qb = dot(o, par);
  let qc = qb * qb - qa * (dot(o, o) - 1.0);
  if (qc <= 0.0) { return vec3<f32>(4.0, 4.0, 0.0); }
  let z_in = (sqrt(qc) - qb) / qa;
  let z_out = (-sqrt(qc) - qb) / qa;
  // A step per IMP_STEP texels the ray crosses at the level read.
  let texels = f32(textureDimensions(base_color_tex, 0).y) / (2.0 * f32(imp_grid())) * exp2(-imp_level);
  let n = clamp(ceil(length(par) * (z_in - z_out) * texels / IMP_STEP), 2.0, IMP_STEPS);
  let dz = (z_in - z_out) / n;
  var front = z_in;
  for (var k = 0.0; k <= n; k += 1.0) {
    let zk = z_in - dz * k;
    // Depth alone first: coverage is read only where the surface stands
    // before the ray. (The sphere lies inside the frame's square.)
    let a = imp_atlas(f, o + par * zk);
    var depth = textureSampleLevel(normal_tex, material_sampler, a, imp_level).a * 2.0 - 1.0;
    if (depth < zk || textureSampleLevel(base_color_tex, material_sampler, a, imp_level).a <= 0.5) {
      front = zk;
      continue;
    }
    // The surface lies between the last step before it and this one:
    // bisect to it, so the point found is the surface's to a fraction of
    // a step, close enough for the other frames to confirm (imp_seen).
    var hit = zk;
    if (k > 0.0) {
      for (var b = 0u; b < IMP_BISECT; b++) {
        let zm = 0.5 * (front + hit);
        let s = imp_depth_at(f, o + par * zm, imp_level);
        if (s.covered && s.depth >= zm) {
          hit = zm;
          depth = s.depth;
        } else {
          front = zm;
        }
      }
    }
    // The frame holds one layer: a ray meeting it well behind its front
    // surface passed behind that surface, where the frame cannot see what
    // the ray meets, so it shows none (a crown's edge seen past the trunk).
    if (depth - hit > IMP_THICK) { break; }
    return vec3<f32>(o + par * hit, hit);
  }
  return vec3<f32>(4.0, 4.0, 0.0);
}

// Frame `f`'s image of the point the eye's ray meets `step` radii from
// the card's point (imp_uvs), where the frame's own depth there agrees
// that it sees that point. A frame that sees another surface there (one
// standing before the point from its side), or a ray that met nothing,
// draws none: falling back to the frame's own fixed point drew loose
// pieces of crown beside the trunk.
fn imp_seen(f: u32, uv: vec2<f32>, par: vec2<f32>, pd: f32, vd: f32, step: f32, found: bool) -> vec2<f32> {
  let at = uv + par * (step * vd);
  if (found && all(abs(at) <= vec2<f32>(1.0))) {
    let s = imp_depth_at(f, at, imp_level);
    if (s.covered && abs(s.depth - (pd + step * vd)) < IMP_AGREE) { return at; }
  }
  return vec2<f32>(4.0);
}

// The frames' images of the card's point, each moved by one parallax step.
struct ImpUvs {
  uv01: vec4<f32>,
  uv2: vec2<f32>,
  // The frames' weights in the blend: the card's, or seen from below
  // those of the frames that see the point the march found, rescaled to
  // the whole (a frame that sees another surface there has no say).
  weights: vec3<f32>,
};

fn imp_uvs(uv01: vec4<f32>, uv2: vec2<f32>, par01: vec4<f32>, par2: vec2<f32>, frames: vec3<u32>, dd: ImpDepths, weights: vec3<f32>, march: bool) -> ImpUvs {
  var o: ImpUvs;
  if (!march || imp_level >= IMP_MARCH_LEVEL) {
    o.uv01 = vec4<f32>(imp_parallax(frames.x, uv01.xy, par01.xy, dd.pd.x, false).xy, imp_parallax(frames.y, uv01.zw, par01.zw, dd.pd.y, false).xy);
    o.uv2 = imp_parallax(frames.z, uv2, par2, dd.pd.z, false).xy;
    o.weights = weights;
    return o;
  }
  // Seen from below, the frame weighted most finds where the eye's ray
  // meets the model, and every frame shows that one point: its image moved
  // along the ray by the same step. Each frame finding its own (the fixed
  // point on one, the march on another) put the three on points apart in
  // depth by as much as the trunk's width: up close from 60 degrees below
  // a band of crown drew across the trunk, and the blended surface point
  // jumped from pixel to pixel. It also marched up to three rays a pixel,
  // which made a screen of near cards seen from below cost the main pass
  // a third more than from above (lavapipe; +69% on the L4).
  let k1 = weights.y > weights.x && weights.y >= weights.z;
  let k2 = weights.z > weights.x && weights.z > weights.y;
  let uv = select(select(uv01.xy, uv01.zw, k1), uv2, k2);
  let par = select(select(par01.xy, par01.zw, k1), par2, k2);
  let f = select(select(frames.x, frames.y, k1), frames.z, k2);
  let pd = select(select(dd.pd.x, dd.pd.y, k1), dd.pd.z, k2);
  let vd = select(select(dd.vd.x, dd.vd.y, k1), dd.vd.z, k2);
  let hit = imp_parallax(f, uv, par, pd, true);
  // The step along the eye (radii), as imp_blended_step has it.
  let step = (hit.z - pd) / vd;
  let found = all(abs(hit.xy) <= vec2<f32>(1.0));
  o.uv01 = vec4<f32>(imp_seen(frames.x, uv01.xy, par01.xy, dd.pd.x, dd.vd.x, step, found), imp_seen(frames.y, uv01.zw, par01.zw, dd.pd.y, dd.vd.y, step, found));
  o.uv2 = imp_seen(frames.z, uv2, par2, dd.pd.z, dd.vd.z, step, found);
  // The frame that found the point shows it as found.
  if (k2) {
    o.uv2 = hit.xy;
  } else if (k1) {
    o.uv01 = vec4<f32>(o.uv01.xy, hit.xy);
  } else {
    o.uv01 = vec4<f32>(hit.xy, o.uv01.zw);
  }
  let sees = vec3<f32>(
    imp_inside(o.uv01.xy),
    imp_inside(o.uv01.zw),
    imp_inside(o.uv2),
  );
  let w = weights * max(sees, vec3<f32>(f32(!k1 && !k2), f32(k1), f32(k2)));
  o.weights = w / max(w.x + w.y + w.z, 1e-5);
  return o;
}

// How much of the card of copy `i` the dither cuts (instance.wgsl
// dither_cut): a scattered copy's card draws the pixels its mesh drops
// (scatter_hidden) and no others; a model impostor (no impostor distance)
// its look's share (look_hidden: its model's level cross-fading to or
// from it, a reveal). The distance fade shrinks the card as it does the
// mesh (scatter_scale), never by dropping pixels.
fn imp_hidden(i: ScatterInstance) -> f32 {
  if (scatter.impostor_grid.y <= 0.0) { return look_hidden(i.look); }
  return scatter_near(i.m3);
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
