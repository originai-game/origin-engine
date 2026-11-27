// Particle draw: one instanced quad per slot of a system's ring (dead slots
// collapse), camera-facing or stretched along the velocity, sized and
// coloured by the emitter's curves, animated through a sprite sheet, and
// softened against the depth prepass. The instance data is the simulation's
// storage buffer or the CPU path's upload. Prepended by frame.wgsl, the
// light tables, lighting.wgsl and area.wgsl (renderer::fluid_shared_source:
// lit particles), noise.wgsl and particle_common.wgsl.
//
// `vs_mesh`/`fs_mesh` draw a mesh per particle instead of a quad (the mesh's
// vertices beside the particle as instance data); `fs_distort` draws a
// billboard as the opaque scene behind it, shifted (heat haze), from the
// main pass's scene copy.
//
// An emitter with an effect material (effect.rs; group 3, effect.wgsl and
// the hook contract prepended) shades through `effect_shade` instead of
// its fixed look; its driven values are evaluated per vertex. A mesh
// particle whose emitter.look.z is 2 is an effect mesh (effect.rs
// EffectMesh): placed whole by the entry's transform, its life the
// particle's.

@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(2) @binding(0) var<uniform> emitter: EmitterDraw;
@group(2) @binding(1) var<uniform> view: ParticleView;
@group(2) @binding(2) var scene_depth: texture_2d<f32>;
// The opaque scene copied before the transparent draws (white where the pass
// has none: its distortion systems are not drawn there).
@group(2) @binding(3) var scene_color: texture_2d<f32>;

struct ParticleIn {
  @location(0) pos_age: vec4<f32>,
  @location(1) vel_life: vec4<f32>,
  @location(2) misc: vec4<f32>,
};

struct ParticleOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) local: vec2<f32>,
  @location(2) color: vec4<f32>,
  @location(3) depth: f32,
  @location(4) world: vec3<f32>,
  @location(5) normal: vec3<f32>,
  // Where the fragment lies on a billboard as drawn (turned by the
  // particle's rotation, along the camera's right and up, a half-size
  // from the centre at 0.5): its lit normal (`ball_normal`). `local` is
  // the corner before the rotation, which the texture and the round dot use.
  @location(6) facing: vec2<f32>,
  // oit.wgsl `oit_key`: the particle and which of its triangles.
  @location(7) @interpolate(flat, either) oit_key: vec2<u32>,
  // The effect material's driven values (effect.wgsl EffectDriven) and
  // the surface's own 0 to 1 coordinates.
  @location(8) @interpolate(flat, either) fx_a: vec4<f32>,
  @location(9) @interpolate(flat, either) fx_b: vec4<f32>,
  @location(10) @interpolate(flat, either) fx_c1: vec4<f32>,
  @location(11) @interpolate(flat, either) fx_c2: vec4<f32>,
  @location(12) shape_uv: vec2<f32>,
  // A blended sheet's next frame (xy) and how far into it (z; 0 without).
  @location(13) uv_next: vec3<f32>,
};

// Corner `c` (-0.5 to 0.5) of frame `f` on a sheet of `columns` × `rows`.
fn sheet_uv(f: f32, c: vec2<f32>, columns: f32, rows: f32) -> vec2<f32> {
  let col = f % columns;
  let row = floor(f / columns);
  return vec2<f32>((col + c.x + 0.5) / columns, (row + 0.5 - c.y) / rows);
}

// The particle's texture where the fragment is: a blended sheet's frame
// cross-faded into the next (sampled with the first's gradients, the same
// on every frame of a sheet).
fn particle_tex(in: ParticleOut) -> vec4<f32> {
  let a = textureSample(sprite_tex, sprite_sampler, in.uv);
  if (emitter.sheet2.z < 0.5) { return a; }
  let b = textureSampleGrad(sprite_tex, sprite_sampler, in.uv_next.xy, dpdx(in.uv), dpdy(in.uv));
  return mix(a, b, in.uv_next.z);
}

// A particle's effect inputs: its driven values at life `t`, from its
// random value (misc.w) as bits.
fn particle_effect(o: ptr<function, ParticleOut>, t: f32, random: f32) {
  if (!effect_on()) { return; }
  let d = effect_driven(t, u32(random * 16777216.0) ^ (emitter.info.y * 2246822519u));
  (*o).fx_a = d.a;
  (*o).fx_b = d.b;
  (*o).fx_c1 = d.c1;
  (*o).fx_c2 = d.c2;
}

fn particle_driven(in: ParticleOut) -> EffectDriven {
  var d: EffectDriven;
  d.a = in.fx_a;
  d.b = in.fx_b;
  d.c1 = in.fx_c1;
  d.c2 = in.fx_c2;
  return d;
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, p: ParticleIn) -> ParticleOut {
  var o: ParticleOut;
  if (p.pos_age.w >= p.vel_life.w) {
    o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    return o;
  }
  let q = quad_corner(vi);
  let c = vec2<f32>(
    mix(emitter.crop.x, emitter.crop.z, q.x) - 0.5,
    0.5 - mix(emitter.crop.w, emitter.crop.y, q.y),
  );
  o.oit_key = oit_key4(p.pos_age, vec3<f32>(f32(vi / 3u), p.misc.x, 0.0));
  let age = p.pos_age.w;
  let t = clamp(age / max(p.vel_life.w, 1e-6), 0.0, 1.0);
  let size = particle_size(p.misc.x, t);
  var center = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  if (emitter.sheet2.w > 0.5) {
    center = (emitter.transform * vec4<f32>(center, 1.0)).xyz;
    vel = (emitter.transform * vec4<f32>(vel, 0.0)).xyz;
  }
  var right = view.cam_right.xyz;
  var up = view.cam_up.xyz;
  var offset: vec3<f32>;
  let v2 = vec2<f32>(dot(vel, right), dot(vel, up));
  let speed = length(v2);
  let align = particle_align();
  // What of a pixel's width a streak covers: 1 for one a pixel wide or
  // more (below).
  var coverage = 1.0;
  if (align == 1u && speed > 1e-4) {
    let axis = v2 / speed;
    let len = size.y + speed * emitter.render.y;
    let along = right * axis.x + up * axis.y;
    let across = right * -axis.y + up * axis.x;
    // A streak narrower than a pixel (a raindrop, a far spark) is drawn a
    // pixel wide at the share of it it covers (Persson's phone-wire
    // anti-aliasing, 2011): thinner than that it would fall between the
    // pixel centres and flicker in and out along its length.
    let at = view.view_proj * vec4<f32>(center, 1.0);
    let per_unit = abs((view.view_proj * vec4<f32>(up, 0.0)).y) * view.screen.y;
    let pixel = 2.0 * max(at.w, 1e-4) / max(per_unit, 1e-6);
    let width = max(size.x, pixel);
    coverage = size.x / width;
    offset = along * (c.y * len) + across * (c.x * width);
  } else {
    if (align >= 2u) {
      let b = particle_basis(align, center);
      right = b[0];
      up = b[1];
    }
    let rot = particle_rotation(p.misc, age, p.vel_life.w, t);
    let cs = cos(rot);
    let sn = sin(rot);
    // The height as a share of the width: exactly 1 for a square one,
    // whose corner is then the same arithmetic as before heights.
    let cy = c.y * (size.y / max(size.x, 1e-6));
    offset = (right * (c.x * cs - cy * sn) + up * (c.x * sn + cy * cs)) * size.x;
  }
  let world = center + offset;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.facing = vec2<f32>(dot(offset, right), dot(offset, up)) / max(max(size.x, size.y), 1e-6);
  // The sprite sheet frame.
  let columns = max(emitter.sheet.x, 1.0);
  let rows = max(emitter.sheet.y, 1.0);
  let frames = max(emitter.sheet.z, 1.0);
  let mode = u32(emitter.sheet.w);
  // Where it is in the frames played so far (a blended sheet fades by
  // its fraction into the next; one playing once holds its last frame).
  var played = 0.0;
  if (mode == 1u) { played = t * emitter.sheet2.x * frames; }
  if (mode == 2u) { played = age * emitter.sheet2.x; }
  var f = floor(played) % frames;
  if (mode == 3u) { f = min(floor(p.misc.w * frames), frames - 1.0); }
  o.uv = sheet_uv(f, c, columns, rows);
  var fade = 0.0;
  if (emitter.sheet2.z > 0.5 && (mode == 1u || mode == 2u)) {
    fade = fract(played);
    if (mode == 1u && played >= ceil(emitter.sheet2.x) * frames - 1.0) { fade = 0.0; }
  }
  o.uv_next = vec3<f32>(sheet_uv((f + 1.0) % frames, c, columns, rows), fade);
  // A streak drawn wider than it is keeps its own width's place across
  // it (its round fall-off across a pixel-wide quad left it dashed), and
  // along it: a drop crossing the exposure at its speed is as bright all
  // along its streak, its ends rounded over its own width, which is under
  // a pixel (the dot's fall-off along it drew a short dash).
  o.local = c * coverage;
  o.shape_uv = vec2<f32>(c.x + 0.5, 0.5 - c.y);
  particle_effect(&o, t, p.misc.w);
  o.color = color_curve(t, p.misc.w) * emitter.color;
  o.color.a *= coverage;
  o.depth = dot(world - view.cam_pos.xyz, view.cam_forward.xyz);
  o.world = world;
  o.normal = -view.cam_forward.xyz;
  return o;
}

// The light a particle at `p` with normal `n` receives (emitter.look.x),
// sent out along `v`: the sun with its cascades, the image-based light,
// the cluster's local lights, on a rough white surface, plus optional
// Henyey-Greenstein backlit sun transmittance for lit smoke billboards.
fn particle_light(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, scatter: f32) -> vec3<f32> {
  var s: Surface;
  s.albedo = vec3<f32>(1.0);
  s.metallic = 0.0;
  s.roughness = 1.0;
  s.f0 = vec3<f32>(0.04);
  s.coat = 0.0;
  s.coat_roughness = 1.0;
  s.coat_normal = n;
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow(p, n, n_dot_l); }
  // The caustics, the local lights and the diffuse light around it, as
  // every lit surface's (surface_shade.wgsl).
  let at = ShadeAt(p, n, v, frag, false, false, false);
  var lit = surface_matte(s, at, brdf(s, n, v, l) * frame.light_color.rgb * shadow, n_dot_l * shadow);
  if (scatter > 0.0) {
    let cos_t = clamp(dot(-v, l), -1.0, 1.0);
    lit += frame.light_color.rgb * shadow * (1.5 * hg_phase(cos_t, 0.55) * scatter);
  }
  return lit;
}

// A smooth lit mesh particle (emitter.look.w, its roughness, below 1):
// evaluates diffuse (premultiplied by c.a) and dielectric specular (at full
// strength) in a single `surface_shade` pass, raising alpha to carry bright
// glints on clear shards without duplicating shadow or cluster light loops.
fn particle_gloss(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, c: vec4<f32>, rough: f32) -> vec4<f32> {
  var s: Surface;
  s.albedo = c.rgb * c.a;
  s.metallic = 0.0;
  s.roughness = clamp(rough, 0.02, 1.0);
  s.f0 = vec3<f32>(0.04);
  s.coat = 0.0;
  s.coat_roughness = 1.0;
  s.coat_normal = n;
  let v = toward_eye(p);
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow(p, n, n_dot_l); }
  let at = ShadeAt(p, n, v, frag, false, false, false);
  let sun_total = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  let lit = surface_shade(s, at, StyleSun(sun_total, 1.0), n_dot_l * shadow, vec3<f32>(0.0), 1.0, 1.0);
  var spec_s = s;
  spec_s.albedo = vec3<f32>(0.0);
  let spec = lit.specular + brdf(spec_s, n, v, l) * frame.light_color.rgb * shadow;
  let a = clamp(c.a + max(spec.r, max(spec.g, spec.b)), 0.0, 1.0);
  if (a <= 0.0) { return vec4<f32>(0.0); }
  return vec4<f32>((lit.direct + lit.diffuse + lit.specular) / a, a);
}

// A billboard's normal at `facing` (ParticleOut's, the rotation applied:
// a puff's lit side faces the light whatever its spin): a ball facing the
// camera, so a lit puff shades away from the light. The ball spans the middle of the quad (its rim at
// 0.6 of the half-size), where a soft texture is still dense: a ball as
// wide as the quad tilts only at the edges, which a puff's alpha has
// already faded, and both sides then read alike.
// A particle's trail (CpuParticles::trail): its samples, one instance per
// particle beside it.
struct TrailIn {
  @location(3) s0: vec4<f32>,
  @location(4) s1: vec4<f32>,
  @location(5) s2: vec4<f32>,
  @location(6) s3: vec4<f32>,
  @location(7) s4: vec4<f32>,
  @location(8) s5: vec4<f32>,
  @location(9) s6: vec4<f32>,
  @location(10) s7: vec4<f32>,
};

const TRAIL_POINTS: u32 = 8u;

// Point `k` of a particle's trail: 0 where it is (`head`), then its
// samples newest first (emitter.trail.w the newest's ring index); xyz its
// place in the simulation space, w its age in seconds.
fn trail_point(ring: ptr<function, array<vec4<f32>, 8>>, head: vec3<f32>, k: u32) -> vec4<f32> {
  if (k == 0u) { return vec4<f32>(head, 0.0); }
  let newest = u32(emitter.trail.w + 0.5);
  let i = (newest + TRAIL_POINTS - (k - 1u) % TRAIL_POINTS) % TRAIL_POINTS;
  return vec4<f32>((*ring)[i].xyz, emitter.counts2.z + f32(k - 1u) * emitter.trail.z);
}

// A particle's trail: a camera-facing strip through where it is and its
// samples (TRAIL_POINTS segments of a quad each), cut where it is
// emitter.trail.x seconds old, its width (a share emitter.trail.y of the
// particle's) and alpha falling to nothing there. u runs along it from
// the particle, v across; `local` across, for the untextured round fall
// off.
@vertex
fn vs_trail(@builtin(vertex_index) vi: u32, p: ParticleIn, h: TrailIn) -> ParticleOut {
  var o: ParticleOut;
  o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
  if (p.pos_age.w >= p.vel_life.w) { return o; }
  var ring = array<vec4<f32>, 8>(h.s0, h.s1, h.s2, h.s3, h.s4, h.s5, h.s6, h.s7);
  let life = emitter.trail.x;
  let c = quad_corner(vi);
  let seg = vi / 6u;
  let head = p.pos_age.xyz;
  let a = trail_point(&ring, head, seg);
  var b = trail_point(&ring, head, seg + 1u);
  if (a.w >= life) { return o; }
  if (b.w > life) {
    b = vec4<f32>(mix(a.xyz, b.xyz, (life - a.w) / max(b.w - a.w, 1e-6)), life);
  }
  // c.y: the segment's end nearer the particle (0) or its far one.
  let k = seg + u32(c.y > 0.5);
  let at = select(a, b, c.y > 0.5);
  // Its direction there: between the points either side.
  var tangent = trail_point(&ring, head, max(k, 1u) - 1u).xyz
    - trail_point(&ring, head, min(k + 1u, TRAIL_POINTS)).xyz;
  // Where a sample lies on the particle (taken this tick), the way it
  // moves; else the camera's right below.
  if (dot(tangent, tangent) < 1e-12) { tangent = p.vel_life.xyz; }
  var pos = at.xyz;
  if (emitter.sheet2.w > 0.5) {
    pos = (emitter.transform * vec4<f32>(pos, 1.0)).xyz;
    tangent = (emitter.transform * vec4<f32>(tangent, 0.0)).xyz;
  }
  let t = clamp(p.pos_age.w / max(p.vel_life.w, 1e-6), 0.0, 1.0);
  let fade = 1.0 - clamp(at.w / life, 0.0, 1.0);
  let width = particle_size(p.misc.x, t).x * emitter.trail.y * fade;
  let side = cross(tangent, toward_eye(pos));
  let across = select(view.cam_right.xyz, normalize(side), dot(side, side) > 1e-12);
  let world = pos + across * ((c.x - 0.5) * width);
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = vec2<f32>(at.w / life, c.x);
  o.local = vec2<f32>(c.x - 0.5, 0.0);
  o.facing = o.local;
  o.shape_uv = o.uv;
  o.oit_key = oit_key4(p.pos_age, vec3<f32>(f32(vi / 3u), p.misc.x, 1.0));
  particle_effect(&o, t, p.misc.w);
  o.color = color_curve(t, p.misc.w) * emitter.color;
  o.color.a *= fade;
  o.depth = dot(world - view.cam_pos.xyz, view.cam_forward.xyz);
  o.world = world;
  o.normal = toward_eye(world);
  return o;
}

// How its quads face: 0 the camera, 1 along the velocity, 2 upright on
// the emitter's axis, 3 across it.
fn particle_align() -> u32 {
  return u32(emitter.render.x + 0.5);
}

// An upright or facing particle's right, up and normal (towards the eye
// for an upright one) at `center`, in the world.
fn particle_basis(align: u32, center: vec3<f32>) -> mat3x3<f32> {
  let axis = normalize((emitter.transform * vec4<f32>(emitter.axis.xyz, 0.0)).xyz);
  if (align == 2u) {
    let side = cross(axis, toward_eye(center));
    // Seen along its axis it has no side to turn to: the camera's right.
    let right = select(view.cam_right.xyz, normalize(side), dot(side, side) > 1e-8);
    return mat3x3<f32>(right, axis, cross(right, axis));
  }
  // Up: the emitter's +y laid into the plane, else its +x.
  let ey = (emitter.transform * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz;
  let ex = (emitter.transform * vec4<f32>(1.0, 0.0, 0.0, 0.0)).xyz;
  var up = ey - axis * dot(ey, axis);
  if (dot(up, up) < 1e-6 * dot(ey, ey)) { up = ex - axis * dot(ex, axis); }
  up = normalize(up);
  return mat3x3<f32>(cross(up, axis), up, axis);
}

fn ball_normal(local: vec2<f32>) -> vec3<f32> {
  var q = local * (2.0 / 0.6);
  let r = length(q);
  if (r > 1.0) { q = q / r; }
  let z = sqrt(max(1.0 - dot(q, q), 0.0));
  return normalize(view.cam_right.xyz * q.x + view.cam_up.xyz * q.y - view.cam_forward.xyz * (z + 0.1));
}

struct MeshIn {
  @location(5) position: vec3<f32>,
  @location(6) normal: vec3<f32>,
  @location(7) color: vec4<f32>,
  @location(8) uv: vec2<f32>,
};

// Rotates `v` by `angle` about the unit `axis` (Rodrigues).
fn rotate(v: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
  let c = cos(angle);
  let s = sin(angle);
  return v * c + cross(axis, v) * s + axis * dot(axis, v) * (1.0 - c);
}

@vertex
fn vs_mesh(p: ParticleIn, m: MeshIn) -> ParticleOut {
  var o: ParticleOut;
  if (p.pos_age.w >= p.vel_life.w) {
    o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    return o;
  }
  let age = p.pos_age.w;
  let t = clamp(age / max(p.vel_life.w, 1e-6), 0.0, 1.0);
  let size = particle_size(p.misc.x, t);
  var center = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  if (emitter.sheet2.w > 0.5) {
    center = (emitter.transform * vec4<f32>(center, 1.0)).xyz;
    vel = (emitter.transform * vec4<f32>(vel, 0.0)).xyz;
  }
  var pos = m.position * vec3<f32>(size.x, size.y, size.x);
  var nrm = m.normal;
  let speed = length(vel);
  let align = particle_align();
  if (align >= 2u) {
    // Its own +z across the quad's plane, spun about it by its rotation.
    let b = particle_basis(align, center);
    let rot = particle_rotation(p.misc, age, p.vel_life.w, t);
    let z = vec3<f32>(0.0, 0.0, 1.0);
    pos = rotate(pos, z, rot);
    nrm = rotate(nrm, z, rot);
    pos = b[0] * pos.x + b[1] * pos.y + b[2] * pos.z;
    nrm = b[0] * nrm.x + b[1] * nrm.y + b[2] * nrm.z;
  } else if (align == 1u && speed > 1e-4) {
    // +y along the velocity, stretched by the speed.
    let y = vel / speed;
    var x = cross(y, vec3<f32>(0.0, 0.0, 1.0));
    if (dot(x, x) < 1e-6) { x = cross(y, vec3<f32>(1.0, 0.0, 0.0)); }
    x = normalize(x);
    let z = cross(x, y);
    let stretch = 1.0 + speed * emitter.render.y / max(size.y, 1e-4);
    pos = x * pos.x + y * (pos.y * stretch) + z * pos.z;
    nrm = x * nrm.x + y * (nrm.y / stretch) + z * nrm.z;
  } else {
    // Tumbling about an axis of its own (its random value).
    let a = p.misc.w * 43.758;
    let axis = normalize(vec3<f32>(sin(a), cos(a * 1.7), sin(a * 2.3 + 1.0)) + vec3<f32>(1e-3, 0.0, 0.0));
    let rot = particle_rotation(p.misc, age, p.vel_life.w, t);
    pos = rotate(pos, axis, rot);
    nrm = rotate(nrm, axis, rot);
  }
  let world = center + pos;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = m.uv;
  o.local = vec2<f32>(0.0);
  o.shape_uv = m.uv;
  o.oit_key = oit_key4(p.pos_age, vec3<f32>(p.misc.x, p.misc.w, 2.0));
  particle_effect(&o, t, p.misc.w);
  o.color = color_curve(t, p.misc.w) * emitter.color * m.color;
  o.depth = dot(world - view.cam_pos.xyz, view.cam_forward.xyz);
  o.world = world;
  o.normal = normalize(nrm);
  return o;
}

// The scene's view depth behind a fragment minus its own, through the
// soft distance (1 where the emitter has none).
fn particle_soft(in: ParticleOut) -> f32 {
  if (emitter.render.z <= 0.0) { return 1.0; }
  // Clamped: a pass without a depth of its own binds a 1x1 far plane.
  let at = min(vec2<i32>(in.clip.xy), vec2<i32>(textureDimensions(scene_depth)) - vec2<i32>(1));
  let scene = textureLoad(scene_depth, at, 0).r;
  return clamp((scene - in.depth) / emitter.render.z, 0.0, 1.0);
}

// The effect material's colour for a particle fragment (`shape` its
// untextured falloff).
fn particle_effect_color(in: ParticleOut, shape: f32, normal: vec3<f32>) -> vec4<f32> {
  var f: EffectFragment;
  f.uv = in.uv;
  f.shape_uv = in.shape_uv;
  f.color = in.color;
  f.shape = shape;
  f.profile = 1.0;
  f.textured = emitter.sheet2.y;
  f.world = in.world;
  f.normal = normal;
  f.driven = particle_driven(in);
  return effect_shade(f, sprite_tex);
}

// The system's hue turn (og.Vfx.hue) on the colour it draws, before light
// and fog; none at 0, so an unturned system draws exactly as it did.
fn particle_hue(c: vec4<f32>) -> vec4<f32> {
  if (emitter.counts2.y == 0.0) { return c; }
  return vec4<f32>(hue_rotate(c.rgb, emitter.counts2.y), c.a);
}

fn is_alpha_blend() -> bool {
  return abs(emitter.render.w - 1.0) < 0.5;
}

fn mesh_fragment(in: ParticleOut) -> vec4<f32> {
  let n = normalize(in.normal);
  var c: vec4<f32>;
  if (effect_on()) {
    c = particle_hue(particle_effect_color(in, 1.0, n));
    c.a *= particle_soft(in);
  } else {
    c = particle_hue(in.color);
  }
  let albedo = c.rgb;
  if (emitter.look.x > 0.5 && emitter.look.w < 1.0) {
    c = particle_gloss(in.clip.xy, in.world, n, c, emitter.look.w);
  } else if (emitter.look.x > 0.5) {
    c = vec4<f32>(c.rgb * particle_light(in.clip.xy, in.world, n, toward_eye(in.world), 0.0), c.a);
  }
  // Translucency: the share of the light on the far side (the sun with its
  // shadow, the sky, local lights) that passes through a thin piece.
  let through = emitter.counts2.w;
  if (emitter.look.x > 0.5 && through > 0.0) {
    c = vec4<f32>(c.rgb + albedo * through * particle_light(in.clip.xy, in.world, -n, toward_eye(in.world), 0.0), c.a);
  }
  c = fx_fog(c, in.world, in.clip.xy, is_alpha_blend());
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return blend_out(c);
}

@fragment
fn fs_mesh(in: ParticleOut) -> @location(0) vec4<f32> {
  return mesh_fragment(in);
}

@fragment
fn fs_mesh_oit(in: ParticleOut) -> OitOutput {
  return oit_weighted(mesh_fragment(in), in.clip.z);
}

@fragment
fn fs_mesh_oit_list(in: ParticleOut, @builtin(sample_mask) mask: u32) -> OitOutput {
  let c = mesh_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_mesh_oit_front(in: ParticleOut) -> OitOutput {
  let c = mesh_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_mesh_oit_mark(in: ParticleOut) {
  var a: f32;
  if (effect_on()) {
    a = particle_effect_color(in, 1.0, normalize(in.normal)).a * particle_soft(in);
  } else {
    a = in.color.a;
  }
  if (a <= 0.0) { discard; }
  oit_front_mark(in.clip, a);
}

// Distortion: the scene copy behind the particle, shifted along the
// texture's red and green (a normal map), an EffectMaterial, or a smooth
// wavering field in screen space that rises over time (heat haze).
// Samples the scene copy bilinearly and rejects shifted taps that lie on a
// foreground surface in front of the distortion particle.
@fragment
fn fs_distort(in: ParticleOut) -> @location(0) vec4<f32> {
  let tex = particle_tex(in);
  let r = length(in.local) * 2.0;
  let soft_dot = smoothstep(0.0, 0.6, clamp(1.0 - r, 0.0, 1.0));
  let size_f = max(vec2<f32>(textureDimensions(scene_color)), vec2<f32>(1.0));
  var bend: vec2<f32>;
  var w: f32;
  if (effect_on()) {
    let fx = particle_effect_color(in, soft_dot, ball_normal(in.facing));
    w = fx.a;
    if ((effect.flags.x & FX_MAIN) != 0u || emitter.sheet2.y > 0.5) {
      bend = fx.rg * 2.0 - 1.0;
    } else {
      let cells = in.clip.xy / size_f.y * 18.0;
      let q = vec3<f32>(cells.x, cells.y + frame.time.x * 1.2, frame.time.x * 0.6);
      bend = vec2<f32>(value_noise3(q, 11u), value_noise3(q, 23u)) * 2.0 - 1.0;
    }
  } else if (emitter.sheet2.y > 0.5) {
    bend = tex.rg * 2.0 - 1.0;
    w = in.color.a * tex.a;
  } else {
    w = in.color.a * soft_dot;
    let cells = in.clip.xy / size_f.y * 18.0;
    let q = vec3<f32>(cells.x, cells.y + frame.time.x * 1.2, frame.time.x * 0.6);
    bend = vec2<f32>(value_noise3(q, 11u), value_noise3(q, 23u)) * 2.0 - 1.0;
  }
  w *= particle_soft(in);
  if (w <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  // Screen rows run down: the bend's +y is up.
  let shift = vec2<f32>(bend.x, -bend.y) * emitter.look.y * size_f.y * w;
  let target_px = clamp(in.clip.xy + shift, vec2<f32>(0.5), size_f - vec2<f32>(0.5));
  let d_size = vec2<i32>(textureDimensions(scene_depth));
  let d_at = clamp(
    vec2<i32>(target_px * (vec2<f32>(d_size) / size_f)),
    vec2<i32>(0),
    max(d_size - vec2<i32>(1), vec2<i32>(0)),
  );
  let refracted_depth = textureLoad(scene_depth, d_at, 0).r;
  let safe_px = select(in.clip.xy, target_px, refracted_depth + 0.05 >= in.depth);
  let half_tex = vec2<f32>(0.5) / size_f;
  let uv = clamp(safe_px / size_f, half_tex, vec2<f32>(1.0) - half_tex);
  return vec4<f32>(textureSampleLevel(scene_color, sprite_sampler, uv, 0.0).rgb, clamp(w, 0.0, 1.0));
}

@fragment
fn fs(in: ParticleOut) -> @location(0) vec4<f32> {
  return particle_fragment(in, in.clip.xy);
}

// Half-resolution soft particle pass: scene_depth is bound at half resolution
// (matching in.clip.xy), while clustered lighting and froxel UVs expect
// full-resolution pixel coordinates.
@fragment
fn fs_half(in: ParticleOut) -> @location(0) vec4<f32> {
  return particle_fragment(in, in.clip.xy * 2.0);
}

// An alpha system in the main camera's order-independent passes (oit.rs).
@fragment
fn fs_oit(in: ParticleOut) -> OitOutput {
  return oit_weighted(particle_fragment(in, in.clip.xy), in.clip.z);
}

@fragment
fn fs_oit_list(in: ParticleOut, @builtin(sample_mask) mask: u32) -> OitOutput {
  let c = particle_fragment(in, in.clip.xy);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: ParticleOut) -> OitOutput {
  let c = particle_fragment(in, in.clip.xy);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: ParticleOut) {
  let tex = particle_tex(in);
  let dot_d = clamp(1.0 - length(in.local) * 2.0, 0.0, 1.0);
  var a = in.color.a;
  if (effect_on()) {
    a = particle_effect_color(in, dot_d * dot_d, ball_normal(in.facing)).a;
  } else if (emitter.sheet2.y > 0.5) {
    a *= tex.a;
  } else {
    a *= dot_d * dot_d;
  }
  a *= particle_soft(in);
  if (a <= 0.0) { discard; }
  oit_front_mark(in.clip, a);
}

fn particle_fragment(in: ParticleOut, light_frag: vec2<f32>) -> vec4<f32> {
  let tex = particle_tex(in);
  // No texture: a soft round dot.
  let dot_d = clamp(1.0 - length(in.local) * 2.0, 0.0, 1.0);
  var c = in.color;
  if (effect_on()) {
    // A billboard faces the eye: its normal turns out towards its rim,
    // so a fresnel fade reads it as a soft ball.
    c = particle_effect_color(in, dot_d * dot_d, ball_normal(in.facing));
  } else if (emitter.sheet2.y > 0.5) {
    c *= tex;
  } else {
    c.a *= dot_d * dot_d;
  }
  c = particle_hue(c);
  if (emitter.look.x > 0.5) {
    // A streak along its velocity is a drop drawn through the exposure: a
    // lens that shows the eye what lies beyond it, the light of the wide
    // field behind it (Garg and Nayar, 2006), so it takes the light of the
    // hemisphere facing away from the eye as that light leaves it, along
    // its normal. (Graded towards the eye it would be a surface seen from
    // behind, and Fresnel's grazing term would turn away the very light
    // behind it: the sun and the lamps it passes.)
    var n = ball_normal(in.facing);
    var v = toward_eye(in.world);
    var scatter = 1.0 - 0.65 * clamp(c.a, 0.0, 1.0);
    if (particle_align() == 1u) {
      n = -v;
      v = n;
      scatter = 0.0;
    }
    c = vec4<f32>(c.rgb * particle_light(light_frag, in.world, n, v, scatter), c.a);
  }
  c.a *= particle_soft(in);
  c = fx_fog(c, in.world, light_frag, is_alpha_blend());
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return blend_out(c);
}

// The colour a multiply system's blend (Dst times it) needs: its own,
// mixed towards white as its alpha falls, so a faint one changes little.
fn blend_out(c: vec4<f32>) -> vec4<f32> {
  if (emitter.render.w < 2.5) { return c; }
  return vec4<f32>(mix(vec3<f32>(1.0), c.rgb, clamp(c.a, 0.0, 1.0)), c.a);
}
