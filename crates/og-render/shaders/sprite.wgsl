// Sprites and MSDF text: instanced quads in the world's xy plane, alpha
// blended (or added) into the HDR target after the 3D scene. One instance
// per sprite or glyph; the quad's corners come from the vertex index (no
// vertex buffer); in the xy plane or, a billboard, facing the camera
// (stand). Per instance: a hit flash, 2D lighting from the light map
// (lit layers), a soft round dot for untextured particles, and a glow
// strength (`fs_glow` adds glowing sprites into the glow buffer).
// A drop shadow fades where its ground already lies in the sun's cascaded
// shadow. Prepended by frame.wgsl and lighting.wgsl.

struct SpriteParams {
  // x: MSDF distance range in atlas pixels (0: a plain sprite), yz: atlas size in pixels.
  msdf: vec4<f32>,
};
@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var<uniform> sprite_params: SpriteParams;

struct SpriteFxFrame {
  screen: vec4<f32>,  // w, h, 1/w, 1/h
};
@group(2) @binding(0) var light_map: texture_2d<f32>;
@group(2) @binding(1) var light_sampler: sampler;
@group(2) @binding(2) var<uniform> sprite_fx: SpriteFxFrame;

struct SpriteIn {
  @location(0) position_rotation: vec4<f32>,  // xyz: anchor position, w: rotation (radians)
  @location(1) size_anchor: vec4<f32>,        // xy: size, zw: anchor (fraction from bottom-left)
  @location(2) uv_rect: vec4<f32>,            // u0, v0, u1, v1 (v down)
  @location(3) color: vec4<f32>,
  @location(4) flash: vec4<f32>,              // rgb: flash colour, a: amount
  @location(5) fx: vec3<f32>,                 // x: glow, y: lit, z: soft dot; a drop shadow's yz: its ground's rise per metre along x and z
  @location(6) ground: vec4<f32>,             // a drop shadow: x 1, yz: x and z moved per metre of height, w: the ground's height under the anchor; x 2: a bent shadow's cell (bent_corner)
  @location(8) facing: u32,                   // how the quad stands (stand)
  @location(9) outline: vec4<f32>,            // a pixel outline: xy one logical pixel in uv, z its width in logical pixels (0 none)
  @location(10) outline_color: vec4<f32>,
};

struct SpriteOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) flash: vec4<f32>,
  @location(3) fx: vec3<f32>,
  @location(4) corner: vec2<f32>,
  @location(5) world: vec3<f32>,
  @location(6) shadow: f32,                   // 1 for a drop shadow
  @location(7) @interpolate(flat) region: vec4<f32>,         // the uv region: its smallest u and v (xy), its largest (zw)
  @location(8) @interpolate(flat) outline: vec4<f32>,
  @location(9) @interpolate(flat) outline_color: vec4<f32>,
};

// The instance's uv region as min (xy) and max (zw): a flipped region's
// corners in order.
fn region_of(r: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(min(r.xy, r.zw), max(r.xy, r.zw));
}

// Where the quad's point `q` (its offset from the anchor `a` in the quad's
// own x and y, rotated) stands in the world by the instance's facing
// (sprites.rs face_axes and sun_card_right): 0 the xy plane facing +z, 1
// the view plane of the camera this frame group is drawn for, 2 upright
// turned about y towards it (its right laid level), 3 upright facing the
// sun (a faced sprite's drop shadow, the same from every camera).
fn stand(a: vec3<f32>, q: vec2<f32>, facing: u32) -> vec3<f32> {
  if (facing == 0u) { return vec3<f32>(a.xy + q, a.z); }
  var right = frame.view_right.xyz;
  var up = frame.view_up.xyz;
  if (facing >= 2u) {
    var h = vec3<f32>(right.x, 0.0, right.z);
    if (facing == 3u) { h = vec3<f32>(frame.light_dir.z, 0.0, -frame.light_dir.x); }
    right = select(vec3<f32>(1.0, 0.0, 0.0), normalize(h), dot(h, h) > 1e-8);
    up = vec3<f32>(0.0, 1.0, 0.0);
  }
  return a + right * q.x + up * q.y;
}

// A drop shadow's corner: the standing quad's corner carried along the sun
// down to the level ground at g.w (sprites.rs shadow_ground); others
// unchanged.
fn ground_shadow(p: vec3<f32>, g: vec4<f32>) -> vec3<f32> {
  if (g.x < 0.5) { return p; }
  let h = p.y - g.w;
  return vec3<f32>(p.x + g.y * h, g.w, p.z + g.z * h);
}

// The same onto a slope: the plane through g.w under the anchor `a`
// rising by `rise` per metre along x and z (the terrain under a sprite).
fn sloped_shadow(p: vec3<f32>, g: vec4<f32>, a: vec3<f32>, rise: vec2<f32>) -> vec3<f32> {
  // Down the sun by t metres of height: p + (g.y, -1, g.z) t on the plane.
  let above = p.y - g.w - rise.x * (p.x - a.x) - rise.y * (p.z - a.z);
  let t = above / max(1.0 + rise.x * g.y + rise.y * g.z, 0.2);
  return vec3<f32>(p.x + g.y * t, p.y - t, p.z + g.z * t);
}

// A cell of a bent drop shadow (sprites.rs bent_shadow, x 2): the cell's
// corner `k` as a corner of the whole quad (the cell's column and row in
// the flash's first two bytes, 2 x 4 cells) and the ground's height under
// it (the cell's corners: w bottom left, fx.x bottom right, fx.y top
// right, fx.z top left). Neighbouring cells compute a shared corner alike,
// so the shadow has no seams.
struct BentCorner {
  c: vec2<f32>,
  height: f32,
};

fn bent_corner(k: vec2<f32>, flash: vec4<f32>, g: vec4<f32>, fx: vec3<f32>) -> BentCorner {
  var b: BentCorner;
  let cell = round(flash.xy * 255.0);
  b.c = (cell + k) * vec2<f32>(0.5, 0.25);
  let bottom = select(g.w, fx.x, k.x > 0.5);
  let top = select(fx.z, fx.y, k.x > 0.5);
  b.height = select(bottom, top, k.y > 0.5);
  return b;
}

// How much of a drop shadow shows at p: none where the sun's cascades
// already shadow the ground (a shadow inside a shadow is no darker).
fn drop_shadow_visible(p: vec3<f32>) -> f32 {
  let l = normalize(frame.light_dir.xyz);
  return cascade_shadow(p, vec3<f32>(0.0, 1.0, 0.0), max(l.y, 0.05));
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, s: SpriteIn) -> SpriteOut {
  // Two triangles: (0,0) (1,0) (1,1) / (0,0) (1,1) (0,1), counter-clockwise.
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  if (s.ground.x > 1.5) {
    return bent_vertex(corners[vi], s);
  }
  let c = corners[vi];
  // A pixel outline grows the quad by its width on every side (in uv, then
  // as a fraction of the quad), so the line fits outside the art; without
  // one the corner is c exactly.
  let span = max(abs(s.uv_rect.zw - s.uv_rect.xy), vec2<f32>(1e-6));
  let grow = select(vec2<f32>(0.0), s.outline.z * s.outline.xy / span, s.outline.z > 0.0);
  let q = c * (1.0 + 2.0 * grow) - grow;
  let local = (q - s.size_anchor.zw) * s.size_anchor.xy;
  let cs = cos(s.position_rotation.w);
  let sn = sin(s.position_rotation.w);
  let rotated = vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
  let standing = stand(s.position_rotation.xyz, rotated, s.facing);
  var world = ground_shadow(standing, s.ground);
  if (s.ground.x > 0.5 && (s.fx.y != 0.0 || s.fx.z != 0.0)) {
    world = sloped_shadow(standing, s.ground, s.position_rotation.xyz, s.fx.yz);
  }
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  // Bottom of the quad samples the bottom of the region (v grows downwards).
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, q.x), mix(s.uv_rect.w, s.uv_rect.y, q.y));
  o.color = s.color;
  o.flash = s.flash;
  o.fx = select(s.fx, vec3<f32>(0.0), s.ground.x > 0.5);
  o.corner = c;
  o.world = world;
  o.shadow = select(0.0, 1.0, s.ground.x > 0.5);
  o.region = region_of(s.uv_rect);
  o.outline = s.outline;
  o.outline_color = s.outline_color;
  return o;
}

// A cell of a bent drop shadow's corner `k`: the whole quad's corner it is
// (bent_corner) carried down the sun to the ground's height under it.
fn bent_vertex(k: vec2<f32>, s: SpriteIn) -> SpriteOut {
  let b = bent_corner(k, s.flash, s.ground, s.fx);
  let c = b.c;
  let local = (c - s.size_anchor.zw) * s.size_anchor.xy;
  let cs = cos(s.position_rotation.w);
  let sn = sin(s.position_rotation.w);
  let rotated = vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
  let standing = stand(s.position_rotation.xyz, rotated, s.facing);
  let world = ground_shadow(standing, vec4<f32>(1.0, s.ground.yz, b.height));
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, c.x), mix(s.uv_rect.w, s.uv_rect.y, c.y));
  o.color = s.color;
  o.flash = vec4<f32>(0.0);
  o.fx = vec3<f32>(0.0);
  o.corner = c;
  o.world = world;
  o.shadow = 1.0;
  o.region = region_of(s.uv_rect);
  o.outline = vec4<f32>(0.0);
  o.outline_color = vec4<f32>(0.0);
  return o;
}

// Whether uv lies in the sprite's region (a sheet's frame).
fn in_region(uv: vec2<f32>, region: vec4<f32>) -> bool {
  return all(uv >= region.xy) && all(uv <= region.zw);
}

// The texel's alpha at uv, clear outside the region.
fn region_alpha(uv: vec2<f32>, region: vec4<f32>) -> f32 {
  if (!in_region(uv, region)) { return 0.0; }
  return textureSampleLevel(sprite_tex, sprite_sampler, uv, 0.0).a;
}

// A pixel outline (scene.rs SpriteOutline) at a clear point: whether an
// opaque logical pixel lies within the width (a diamond, |dx| + |dy| <=
// width) of the logical pixel the point is in, tested at the centres of
// the logical pixels counted from the region's corner.
fn on_outline(uv: vec2<f32>, region: vec4<f32>, outline: vec4<f32>) -> bool {
  let px = outline.xy;
  let centre = region.xy + (floor((uv - region.xy) / px) + 0.5) * px;
  let w = min(outline.z, 4.0);
  let n = i32(ceil(w));
  for (var j = -n; j <= n; j++) {
    for (var i = -n; i <= n; i++) {
      let d = f32(abs(i) + abs(j));
      if (d == 0.0 || d > w + 1e-3) { continue; }
      if (region_alpha(centre + vec2<f32>(f32(i), f32(j)) * px, region) >= 0.5) { return true; }
    }
  }
  return false;
}

// The sprite's colour before lighting: texture (through cleanEdge when its
// filter is, sprite_params.msdf.w), the pixel outline, tint, soft dot,
// flash. Without an outline or cleanEdge, the texture sampled as ever.
fn sprite_color(in: SpriteOut) -> vec4<f32> {
  var t = textureSample(sprite_tex, sprite_sampler, in.uv);
  if (sprite_params.msdf.w > 0.5) {
    t = clean_edge(in.uv, in.region);
  }
  var c = t * in.color;
  if (in.outline.z > 0.0) {
    // The grown quad reaches past the region: clear there.
    if (!in_region(in.uv, in.region)) { c = vec4<f32>(0.0); }
    // The line lies under the art: a clear texel takes it, a soft edge
    // (a linear texture's filtered alpha) shows it through.
    if (c.a < 1.0 && on_outline(in.uv, in.region, in.outline)) {
      let la = in.outline_color.a * in.color.a;
      let a = c.a + la * (1.0 - c.a);
      c = vec4<f32>(mix(in.outline_color.rgb, c.rgb, c.a / max(a, 1e-4)), a);
    }
  }
  if (in.fx.z > 0.5) {
    let d = clamp(1.0 - length(in.corner * 2.0 - 1.0), 0.0, 1.0);
    c.a *= d * d;
  }
  if (in.flash.a > 0.0) {
    c = vec4<f32>(mix(c.rgb, in.flash.rgb, in.flash.a), c.a);
  }
  return c;
}

@fragment
fn fs(in: SpriteOut) -> @location(0) vec4<f32> {
  var c = sprite_color(in);
  if (in.shadow > 0.5) {
    c.a *= drop_shadow_visible(in.world);
  }
  if (c.a <= 0.0) { discard; }
  if (in.fx.y > 0.5) {
    let light = textureSampleLevel(light_map, light_sampler, in.clip.xy * sprite_fx.screen.zw, 0.0).rgb;
    c = vec4<f32>(c.rgb * light, c.a);
  }
  return vec4<f32>(apply_fog(c.rgb, in.world), c.a);
}

// An untonemapped sprite (`SpriteFx::tone_mapped` false): its colour as
// given, unfogged. Its pipeline blends the HDR target's alpha towards 0
// under it (the target holds 1 there, `fs_cover`), and the tonemap shows
// such pixels unexposed and uncurved (post.wgsl `tonemapped`).
@fragment
fn fs_untonemapped(in: SpriteOut) -> @location(0) vec4<f32> {
  var c = sprite_color(in);
  if (c.a <= 0.0) { discard; }
  if (in.fx.y > 0.5) {
    let light = textureSampleLevel(light_map, light_sampler, in.clip.xy * sprite_fx.screen.zw, 0.0).rgb;
    c = vec4<f32>(c.rgb * light, c.a);
  }
  return c;
}

// Before a frame's first untonemapped sprite: the HDR target's alpha set
// to 1 everywhere (its colour kept by the blend), so the alpha under the
// sprites is the fraction of each pixel the tonemap curves.
@vertex
fn vs_cover(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
  let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
  return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_cover() -> @location(0) vec4<f32> {
  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// Into the glow buffer: the sprite's light, whatever covers it.
@fragment
fn fs_glow(in: SpriteOut) -> @location(0) vec4<f32> {
  let c = sprite_color(in);
  return vec4<f32>(c.rgb * c.a * in.fx.x, 0.0);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
  return max(min(r, g), min(max(r, g), b));
}

@fragment
fn fs_msdf(in: SpriteOut) -> @location(0) vec4<f32> {
  let m = textureSample(sprite_tex, sprite_sampler, in.uv).rgb;
  // The distance range in screen pixels: atlas range scaled by how many
  // screen pixels one atlas pixel covers here.
  let unit_range = vec2<f32>(sprite_params.msdf.x) / sprite_params.msdf.yz;
  let screen_tex_size = vec2<f32>(1.0) / fwidth(in.uv);
  let screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);
  let sd = median(m.r, m.g, m.b) - 0.5;
  let fill = clamp(screen_px_range * sd + 0.5, 0.0, 1.0);
  // An outline (fx.x: its width as a fraction of half the distance range,
  // flash.rgb its colour) grows the shape outwards and shows behind the fill.
  let w = clamp(in.fx.x, 0.0, 1.0) * 0.5;
  let outer = clamp(screen_px_range * (sd + w) + 0.5, 0.0, 1.0);
  let opacity = select(fill, outer, w > 0.0) * in.color.a;
  if (opacity <= 0.0) { discard; }
  let rgb = select(in.color.rgb, mix(in.flash.rgb, in.color.rgb, fill / max(outer, 1e-4)), w > 0.0);
  return vec4<f32>(apply_fog(rgb, in.world), opacity);
}

// ── cleanEdge ─────────────────────────────────────────────────────────────
// TextureFilter::CleanEdge: torcado's cleanEdge, "a pixel art upscaling
// algorithm for clean rotations" (the shadertoy shader of
// gist.github.com/torcado194/e2794f5a4b22049ac0a41f972d14c329, with SLOPE
// and CLEANUP on, highestColor white, similarThreshold 0 and lineWidth 1),
// as og-cook's port (conform/texture2d/cleanedge.rs) samples it for the
// prerotated sprites. MIT, Copyright (c) 2022 torcado: the notice is
// third_party/cleanedge/LICENSE. Every sample is one of the region's
// texels, never a blend; texels outside the region read as clear.

struct CeN {
  ub: vec4<f32>, u: vec4<f32>, uf: vec4<f32>, uff: vec4<f32>,
  b: vec4<f32>, c: vec4<f32>, f: vec4<f32>, ff: vec4<f32>,
  db: vec4<f32>, d: vec4<f32>, df: vec4<f32>, dff: vec4<f32>,
  ddb: vec4<f32>, dd: vec4<f32>, ddf: vec4<f32>,
};

struct CeHit {
  hit: bool,
  col: vec4<f32>,
};

// `similar` at similarThreshold 0: equal, or both clear.
fn ce_similar(a: vec4<f32>, b: vec4<f32>) -> bool {
  return (a.a == 0.0 && b.a == 0.0) || all(a == b);
}

fn ce_similar3(a: vec4<f32>, b: vec4<f32>, c: vec4<f32>) -> bool {
  return ce_similar(a, b) && ce_similar(b, c);
}

fn ce_similar4(a: vec4<f32>, b: vec4<f32>, c: vec4<f32>, d: vec4<f32>) -> bool {
  return ce_similar(a, b) && ce_similar(b, c) && ce_similar(c, d);
}

fn ce_higher(t: vec4<f32>, o: vec4<f32>) -> bool {
  if (ce_similar(t, o)) { return false; }
  if (t.a == o.a) {
    return distance(t.rgb, vec3<f32>(1.0)) < distance(o.rgb, vec3<f32>(1.0));
  }
  return t.a > o.a;
}

fn ce_cd(a: vec4<f32>, b: vec4<f32>) -> f32 {
  return distance(a, b);
}

fn ce_dist_to_line(test: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, dir: vec2<f32>) -> f32 {
  let line = p2 - p1;
  let perp = vec2<f32>(line.y, -line.x);
  let n = normalize(perp);
  let s = select(-1.0, 1.0, dot(perp, dir) > 0.0);
  return s * dot(n, p1 - test);
}

// The colour (one of three) nearer `a`, where the slice's distance is <= 0.
fn ce_pick(dist: f32, a: vec4<f32>, x: vec4<f32>, y: vec4<f32>) -> CeHit {
  return CeHit(dist <= 0.0, select(y, x, ce_cd(a, x) <= ce_cd(a, y)));
}

// `sliceDist`: the colour a slice from a neighbour lays over the point.
fn ce_slice(point_in: vec2<f32>, main_dir: vec2<f32>, pd: vec2<f32>, n: CeN) -> CeHit {
  let none = CeHit(false, vec4<f32>(0.0));
  let lw = clamp(1.0, 0.45, 1.142);
  let point = main_dir * (point_in - 0.5) + 0.5;
  let against = 4.0 * ce_cd(n.f, n.d) + ce_cd(n.uf, n.c) + ce_cd(n.c, n.db) + ce_cd(n.ff, n.df) + ce_cd(n.df, n.dd);
  let towards = 4.0 * ce_cd(n.c, n.df) + ce_cd(n.u, n.f) + ce_cd(n.f, n.dff) + ce_cd(n.b, n.d) + ce_cd(n.d, n.ddf);
  var should = against < towards || (against < towards + 0.001 && !ce_higher(n.c, n.f));
  if (ce_similar4(n.f, n.d, n.b, n.u) && ce_similar4(n.uf, n.df, n.db, n.ub) && !ce_similar(n.c, n.f)) {
    should = false;
  }
  if (!should) { return none; }
  var flip = false;
  let center = vec2<f32>(0.5);
  if (ce_similar3(n.f, n.d, n.db) && !ce_similar3(n.f, n.d, n.b) && !ce_similar(n.uf, n.db)) {
    // Lower shallow 2:1 slant.
    if (!(ce_similar(n.c, n.df) && ce_higher(n.c, n.f))) {
      if (ce_higher(n.c, n.f)) { flip = true; }
      if (ce_similar(n.u, n.f) && !ce_similar(n.c, n.df) && !ce_higher(n.c, n.u)) { flip = true; }
    }
    var dist: f32;
    if (flip) {
      dist = lw - ce_dist_to_line(point, center + vec2<f32>(1.5, -1.0) * pd, center + vec2<f32>(-0.5, 0.0) * pd, -pd);
    } else {
      dist = ce_dist_to_line(point, center + vec2<f32>(1.5, 0.0) * pd, center + vec2<f32>(-0.5, 1.0) * pd, pd);
    }
    if (!flip && ce_similar(n.c, n.uf) && !(ce_similar3(n.c, n.uf, n.uff) && !ce_similar3(n.c, n.uf, n.ff) && !ce_similar(n.d, n.uff))) {
      dist = min(dist, ce_dist_to_line(point, center + vec2<f32>(2.0, -1.0) * pd, center + vec2<f32>(0.0, 1.0) * pd, pd));
    }
    return ce_pick(dist - lw / 2.0, n.c, n.f, n.d);
  } else if (ce_similar3(n.uf, n.f, n.d) && !ce_similar3(n.u, n.f, n.d) && !ce_similar(n.uf, n.db)) {
    // Forward steep 2:1 slant.
    if (!(ce_similar(n.c, n.df) && ce_higher(n.c, n.d))) {
      if (ce_higher(n.c, n.d)) { flip = true; }
      if (ce_similar(n.b, n.d) && !ce_similar(n.c, n.df) && !ce_higher(n.c, n.d)) { flip = true; }
    }
    var dist: f32;
    if (flip) {
      dist = lw - ce_dist_to_line(point, center + vec2<f32>(0.0, -0.5) * pd, center + vec2<f32>(-1.0, 1.5) * pd, -pd);
    } else {
      dist = ce_dist_to_line(point, center + vec2<f32>(1.0, -0.5) * pd, center + vec2<f32>(0.0, 1.5) * pd, pd);
    }
    if (!flip && ce_similar(n.c, n.db) && !(ce_similar3(n.c, n.db, n.ddb) && !ce_similar3(n.c, n.db, n.dd) && !ce_similar(n.f, n.ddb))) {
      dist = min(dist, ce_dist_to_line(point, center + vec2<f32>(1.0, 0.0) * pd, center + vec2<f32>(-1.0, 2.0) * pd, pd));
    }
    return ce_pick(dist - lw / 2.0, n.c, n.f, n.d);
  } else if (ce_similar(n.f, n.d)) {
    // 45-degree diagonal.
    if (ce_similar(n.c, n.df) && ce_higher(n.c, n.f)) {
      if (!ce_similar(n.c, n.dd) && !ce_similar(n.c, n.ff)) { flip = true; }
    } else {
      if (ce_higher(n.c, n.f)) { flip = true; }
      if (!ce_similar(n.c, n.b) && ce_similar4(n.b, n.f, n.d, n.u)) { flip = true; }
    }
    if (((ce_similar(n.f, n.db) && ce_similar3(n.u, n.f, n.df)) || (ce_similar(n.uf, n.d) && ce_similar3(n.b, n.d, n.df))) && !ce_similar(n.c, n.df)) {
      flip = true;
    }
    var dist: f32;
    if (flip) {
      dist = lw - ce_dist_to_line(point, center + vec2<f32>(1.0, -1.0) * pd, center + vec2<f32>(-1.0, 1.0) * pd, -pd);
    } else {
      dist = ce_dist_to_line(point, center + vec2<f32>(1.0, 0.0) * pd, center + vec2<f32>(0.0, 1.0) * pd, pd);
    }
    if (!flip && ce_similar3(n.c, n.uf, n.uff) && !ce_similar3(n.c, n.uf, n.ff) && !ce_similar(n.d, n.uff)) {
      dist = max(dist, ce_dist_to_line(point, center + vec2<f32>(1.5, 0.0) * pd, center + vec2<f32>(-0.5, 1.0) * pd, pd));
    }
    if (!flip && ce_similar3(n.ddb, n.db, n.c) && !ce_similar3(n.dd, n.db, n.c) && !ce_similar(n.ddb, n.f)) {
      dist = max(dist, ce_dist_to_line(point, center + vec2<f32>(1.0, -0.5) * pd, center + vec2<f32>(0.0, 1.5) * pd, pd));
    }
    return ce_pick(dist - lw / 2.0, n.c, n.f, n.d);
  } else if (ce_similar3(n.ff, n.df, n.d) && !ce_similar3(n.ff, n.df, n.c) && !ce_similar(n.uff, n.d)) {
    // Far corner of a shallow slant.
    if (!(ce_similar(n.f, n.dff) && ce_higher(n.f, n.ff))) {
      if (ce_higher(n.f, n.ff)) { flip = true; }
      if (ce_similar(n.uf, n.ff) && !ce_similar(n.f, n.dff) && !ce_higher(n.f, n.uf)) { flip = true; }
    }
    var dist: f32;
    if (flip) {
      dist = lw - ce_dist_to_line(point, center + vec2<f32>(2.5, -1.0) * pd, center + vec2<f32>(0.5, 0.0) * pd, -pd);
    } else {
      dist = ce_dist_to_line(point, center + vec2<f32>(2.5, 0.0) * pd, center + vec2<f32>(0.5, 1.0) * pd, pd);
    }
    return ce_pick(dist - lw / 2.0, n.f, n.ff, n.df);
  } else if (ce_similar3(n.f, n.df, n.dd) && !ce_similar3(n.c, n.df, n.dd) && !ce_similar(n.f, n.ddb)) {
    // Far corner of a steep slant.
    if (!(ce_similar(n.d, n.ddf) && ce_higher(n.d, n.dd))) {
      if (ce_higher(n.d, n.dd)) { flip = true; }
      if (ce_similar(n.db, n.dd) && !ce_similar(n.d, n.ddf) && !ce_higher(n.d, n.dd)) { flip = true; }
    }
    var dist: f32;
    if (flip) {
      dist = lw - ce_dist_to_line(point, center + vec2<f32>(0.0, 0.5) * pd, center + vec2<f32>(-1.0, 2.5) * pd, -pd);
    } else {
      dist = ce_dist_to_line(point, center + vec2<f32>(1.0, 0.5) * pd, center + vec2<f32>(0.0, 2.5) * pd, pd);
    }
    return ce_pick(dist - lw / 2.0, n.d, n.df, n.dd);
  }
  return none;
}

// One texel of the region (`lo` its first texel, `size` its texels), read
// from the texel the point's quadrant faces (`px` + d * pd); clear outside.
fn ce_read(lo: vec2<i32>, size: vec2<i32>, px: vec2<f32>, pd: vec2<f32>, d: vec2<f32>) -> vec4<f32> {
  let t = vec2<i32>(floor(px + d * pd));
  if (any(t < vec2<i32>(0)) || any(t >= size)) { return vec4<f32>(0.0); }
  return textureLoad(sprite_tex, lo + t, 0);
}

// The colour cleanEdge shows at uv within the region.
fn clean_edge(uv: vec2<f32>, region: vec4<f32>) -> vec4<f32> {
  let tex = vec2<f32>(textureDimensions(sprite_tex));
  let lo = floor(region.xy * tex + 0.5);
  let size = vec2<i32>(floor(region.zw * tex + 0.5) - lo);
  let p = uv * tex - lo;
  let local = p - floor(p);
  let px = ceil(p) - 0.5;
  let pd = floor(local + 0.5) * 2.0 - 1.0;
  let l = vec2<i32>(lo);
  let uub = ce_read(l, size, px, pd, vec2<f32>(-1.0, -2.0));
  let uu = ce_read(l, size, px, pd, vec2<f32>(0.0, -2.0));
  let uuf = ce_read(l, size, px, pd, vec2<f32>(1.0, -2.0));
  let ubb = ce_read(l, size, px, pd, vec2<f32>(-2.0, -2.0));
  let ub = ce_read(l, size, px, pd, vec2<f32>(-1.0, -1.0));
  let u = ce_read(l, size, px, pd, vec2<f32>(0.0, -1.0));
  let uf = ce_read(l, size, px, pd, vec2<f32>(1.0, -1.0));
  let uff = ce_read(l, size, px, pd, vec2<f32>(2.0, -1.0));
  let bb = ce_read(l, size, px, pd, vec2<f32>(-2.0, 0.0));
  let b = ce_read(l, size, px, pd, vec2<f32>(-1.0, 0.0));
  let c = ce_read(l, size, px, pd, vec2<f32>(0.0, 0.0));
  let f = ce_read(l, size, px, pd, vec2<f32>(1.0, 0.0));
  let ff = ce_read(l, size, px, pd, vec2<f32>(2.0, 0.0));
  let dbb = ce_read(l, size, px, pd, vec2<f32>(-2.0, 1.0));
  let db = ce_read(l, size, px, pd, vec2<f32>(-1.0, 1.0));
  let d = ce_read(l, size, px, pd, vec2<f32>(0.0, 1.0));
  let df = ce_read(l, size, px, pd, vec2<f32>(1.0, 1.0));
  let dff = ce_read(l, size, px, pd, vec2<f32>(2.0, 1.0));
  let ddb = ce_read(l, size, px, pd, vec2<f32>(-1.0, 2.0));
  let dd = ce_read(l, size, px, pd, vec2<f32>(0.0, 2.0));
  let ddf = ce_read(l, size, px, pd, vec2<f32>(1.0, 2.0));
  var col = c;
  // As the shader: the back slice's `uff` is `ubb`, two up and two back.
  let c_col = ce_slice(local, vec2<f32>(1.0, 1.0), pd, CeN(ub, u, uf, uff, b, c, f, ff, db, d, df, dff, ddb, dd, ddf));
  let b_col = ce_slice(local, vec2<f32>(-1.0, 1.0), pd, CeN(uf, u, ub, ubb, f, c, b, bb, df, d, db, dbb, ddf, dd, ddb));
  let u_col = ce_slice(local, vec2<f32>(1.0, -1.0), pd, CeN(db, d, df, dff, b, c, f, ff, ub, u, uf, uff, uub, uu, uuf));
  if (c_col.hit) { col = c_col.col; }
  if (b_col.hit) { col = b_col.col; }
  if (u_col.hit) { col = u_col.col; }
  return col;
}
