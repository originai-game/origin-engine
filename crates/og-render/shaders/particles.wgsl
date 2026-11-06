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
@group(2) @binding(0) var<uniform> emitter: Emitter;
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
  @location(7) @interpolate(flat) oit_key: vec2<u32>,
  // The effect material's driven values (effect.wgsl EffectDriven) and
  // the surface's own 0 to 1 coordinates.
  @location(8) @interpolate(flat) fx_a: vec4<f32>,
  @location(9) @interpolate(flat) fx_b: vec4<f32>,
  @location(10) @interpolate(flat) fx_c1: vec4<f32>,
  @location(11) @interpolate(flat) fx_c2: vec4<f32>,
  @location(12) shape_uv: vec2<f32>,
};

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
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5),
    vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, 0.5), vec2<f32>(-0.5, 0.5));
  let c = corners[vi];
  o.oit_key = oit_key4(p.pos_age, vec3<f32>(f32(vi / 3u), p.misc.x, 0.0));
  let age = p.pos_age.w;
  let t = clamp(age / max(p.vel_life.w, 1e-6), 0.0, 1.0);
  let size = p.misc.x * size_curve(emitter, t);
  var center = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  if (emitter.ranges2.z > 0.5) {
    center = (emitter.transform * vec4<f32>(center, 1.0)).xyz;
    vel = (emitter.transform * vec4<f32>(vel, 0.0)).xyz;
  }
  let right = view.cam_right.xyz;
  let up = view.cam_up.xyz;
  var offset: vec3<f32>;
  let v2 = vec2<f32>(dot(vel, right), dot(vel, up));
  let speed = length(v2);
  if (emitter.render.x > 0.5 && speed > 1e-4) {
    let axis = v2 / speed;
    let len = size + speed * emitter.render.y;
    let along = right * axis.x + up * axis.y;
    let across = right * -axis.y + up * axis.x;
    offset = along * (c.y * len) + across * (c.x * size);
  } else {
    let rot = p.misc.y + p.misc.z * age;
    let cs = cos(rot);
    let sn = sin(rot);
    offset = (right * (c.x * cs - c.y * sn) + up * (c.x * sn + c.y * cs)) * size;
  }
  let world = center + offset;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.facing = vec2<f32>(dot(offset, right), dot(offset, up)) / max(size, 1e-6);
  // The sprite sheet frame.
  let columns = max(emitter.sheet.x, 1.0);
  let rows = max(emitter.sheet.y, 1.0);
  let frames = max(emitter.sheet.z, 1.0);
  let mode = u32(emitter.sheet.w);
  var f = 0.0;
  if (mode == 1u) { f = floor(t * emitter.sheet2.x * frames) % frames; }
  if (mode == 2u) { f = floor(age * emitter.sheet2.x) % frames; }
  if (mode == 3u) { f = min(floor(p.misc.w * frames), frames - 1.0); }
  let col = f % columns;
  let row = floor(f / columns);
  o.uv = vec2<f32>((col + c.x + 0.5) / columns, (row + 0.5 - c.y) / rows);
  o.local = c;
  o.shape_uv = vec2<f32>(c.x + 0.5, 0.5 - c.y);
  particle_effect(&o, t, p.misc.w);
  o.color = color_curve(emitter, t) * emitter.color;
  o.depth = dot(world - view.cam_pos.xyz, view.cam_forward.xyz);
  o.world = world;
  o.normal = -view.cam_forward.xyz;
  return o;
}

// The light a particle at `p` with normal `n` receives (emitter.look.x):
// the sun with its cascades, the image-based light, the cluster's local
// lights, on a rough white surface.
fn particle_light(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
  var s: Surface;
  s.albedo = vec3<f32>(1.0);
  s.metallic = 0.0;
  s.roughness = 1.0;
  s.f0 = vec3<f32>(0.04);
  s.coat = 0.0;
  s.coat_roughness = 1.0;
  s.coat_normal = n;
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow(p, n, n_dot_l); }
  var c = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  c += ambient_light_level(n, p) * frame.params.x;
  c += surface_local_lights(frag, s, p, n, v);
  return c;
}

// A smooth lit mesh particle (emitter.look.w, its roughness, below 1):
// its body `c` lit as a matte one and blended at its alpha, with what its
// surface reflects (the sun's GGX highlight, the local lights', the sky's
// at Schlick's Fresnel) added over it at full strength, the alpha raised to
// carry it: glass shards that stay clear and glint where they face the
// light, as a transparent material keeps its specular.
fn particle_gloss(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, c: vec4<f32>, rough: f32) -> vec4<f32> {
  var s: Surface;
  s.albedo = vec3<f32>(0.0);
  s.metallic = 0.0;
  s.roughness = clamp(rough, 0.02, 1.0);
  s.f0 = vec3<f32>(0.04);
  s.coat = 0.0;
  s.coat_roughness = 1.0;
  s.coat_normal = n;
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow(p, n, n_dot_l); }
  var spec = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  spec += surface_local_lights(frag, s, p, n, v);
  let fres = f_schlick_roughness(max(dot(n, v), 0.0), s.f0, s.roughness);
  spec += fres * reflected_light(reflect(-v, n), p, s.roughness * frame.params.y) * frame.params.x;
  let body = c.rgb * particle_light(frag, p, n) * c.a;
  let a = clamp(c.a + max(spec.r, max(spec.g, spec.b)), 0.0, 1.0);
  if (a <= 0.0) { return vec4<f32>(0.0); }
  return vec4<f32>((body + spec) / a, a);
}

// A billboard's normal at `facing` (ParticleOut's, the rotation applied:
// a puff's lit side faces the light whatever its spin): a ball facing the
// camera, so a lit puff shades away from the light. The ball spans the middle of the quad (its rim at
// 0.6 of the half-size), where a soft texture is still dense: a ball as
// wide as the quad tilts only at the edges, which a puff's alpha has
// already faded, and both sides then read alike.
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
  let size = p.misc.x * size_curve(emitter, t);
  var center = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  if (emitter.ranges2.z > 0.5) {
    center = (emitter.transform * vec4<f32>(center, 1.0)).xyz;
    vel = (emitter.transform * vec4<f32>(vel, 0.0)).xyz;
  }
  var pos = m.position * size;
  var nrm = m.normal;
  let speed = length(vel);
  if (emitter.look.z > 1.5) {
    // An effect mesh: the entry's transform places it whole.
    center = (emitter.transform * vec4<f32>(m.position, 1.0)).xyz;
    pos = vec3<f32>(0.0);
    nrm = (emitter.transform * vec4<f32>(m.normal, 0.0)).xyz;
  } else if (emitter.render.x > 0.5 && speed > 1e-4) {
    // +y along the velocity, stretched by the speed.
    let y = vel / speed;
    var x = cross(y, vec3<f32>(0.0, 0.0, 1.0));
    if (dot(x, x) < 1e-6) { x = cross(y, vec3<f32>(1.0, 0.0, 0.0)); }
    x = normalize(x);
    let z = cross(x, y);
    let stretch = 1.0 + speed * emitter.render.y / max(size, 1e-4);
    pos = x * pos.x + y * (pos.y * stretch) + z * pos.z;
    nrm = x * nrm.x + y * (nrm.y / stretch) + z * nrm.z;
  } else {
    // Tumbling about an axis of its own (its random value).
    let a = p.misc.w * 43.758;
    let axis = normalize(vec3<f32>(sin(a), cos(a * 1.7), sin(a * 2.3 + 1.0)) + vec3<f32>(1e-3, 0.0, 0.0));
    let rot = p.misc.y + p.misc.z * age;
    pos = rotate(pos, axis, rot);
    nrm = rotate(nrm, axis, rot);
  }
  let world = center + pos;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = m.uv;
  o.local = vec2<f32>(0.0);
  o.shape_uv = m.uv;
  particle_effect(&o, t, p.misc.w);
  o.color = color_curve(emitter, t) * emitter.color * m.color;
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
  f.textured = emitter.sheet2.y;
  f.world = in.world;
  f.normal = normal;
  f.driven = particle_driven(in);
  return effect_shade(f, sprite_tex);
}

@fragment
fn fs_mesh(in: ParticleOut) -> @location(0) vec4<f32> {
  if (effect_on()) {
    var c = particle_effect_color(in, 1.0, in.normal);
    c.a *= particle_soft(in);
    c = vec4<f32>(apply_fog(c.rgb, in.world), c.a);
    if (c.a <= 0.0) { discard; }
    if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
    return c;
  }
  var c = in.color;
  if (emitter.look.x > 0.5 && emitter.look.w < 1.0) {
    c = particle_gloss(in.clip.xy, in.world, normalize(in.normal), c, emitter.look.w);
  } else if (emitter.look.x > 0.5) {
    c = vec4<f32>(c.rgb * particle_light(in.clip.xy, in.world, normalize(in.normal)), c.a);
  }
  c = vec4<f32>(apply_fog(c.rgb, in.world), c.a);
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return c;
}

// Distortion: the scene copy behind the particle, shifted along the
// texture's red and green (a normal map) or, without a texture, by a
// smooth wavering field in screen space that rises over time (heat haze).
// The field is the screen's, not the particle's, so overlapping particles
// at a pixel bend it the same way and blend softly instead of pasting
// differently shifted copies over each other (shards).
@fragment
fn fs_distort(in: ParticleOut) -> @location(0) vec4<f32> {
  let tex = textureSample(sprite_tex, sprite_sampler, in.uv);
  var bend: vec2<f32>;
  var mask: f32;
  if (emitter.sheet2.y > 0.5) {
    bend = tex.rg * 2.0 - 1.0;
    mask = tex.a;
  } else {
    let r = length(in.local) * 2.0;
    mask = smoothstep(0.0, 0.6, clamp(1.0 - r, 0.0, 1.0));
    let cells = in.clip.xy / f32(textureDimensions(scene_color).y) * 18.0;
    let q = vec3<f32>(cells.x, cells.y + frame.time.x * 1.2, frame.time.x * 0.6);
    bend = vec2<f32>(value_noise3(q, 11u), value_noise3(q, 23u)) * 2.0 - 1.0;
  }
  var w = in.color.a * mask;
  if (emitter.render.z > 0.0) {
    let at = min(vec2<i32>(in.clip.xy), vec2<i32>(textureDimensions(scene_depth)) - vec2<i32>(1));
    let scene = textureLoad(scene_depth, at, 0).r;
    w *= clamp((scene - in.depth) / emitter.render.z, 0.0, 1.0);
  }
  if (w <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  let size = vec2<i32>(textureDimensions(scene_color));
  // Screen rows run down: the bend's +y is up.
  let shift = vec2<f32>(bend.x, -bend.y) * emitter.look.y * f32(size.y) * w;
  let at = clamp(vec2<i32>(in.clip.xy + shift), vec2<i32>(0), size - vec2<i32>(1));
  return vec4<f32>(textureLoad(scene_color, at, 0).rgb, clamp(w, 0.0, 1.0));
}

@fragment
fn fs(in: ParticleOut) -> @location(0) vec4<f32> {
  return particle_fragment(in);
}

// An alpha system in the main camera's order-independent passes (oit.rs).
@fragment
fn fs_oit(in: ParticleOut) -> OitOutput {
  return oit_weighted(particle_fragment(in), in.clip.z);
}

@fragment
fn fs_oit_list(in: ParticleOut, @builtin(sample_mask) mask: u32) -> OitOutput {
  let c = particle_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: ParticleOut) -> OitOutput {
  let c = particle_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: ParticleOut) {
  let c = particle_fragment(in);
  oit_front_mark(in.clip, c.a);
}

fn particle_fragment(in: ParticleOut) -> vec4<f32> {
  let tex = textureSample(sprite_tex, sprite_sampler, in.uv);
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
  if (emitter.look.x > 0.5) {
    c = vec4<f32>(c.rgb * particle_light(in.clip.xy, in.world, ball_normal(in.facing)), c.a);
  }
  c.a *= particle_soft(in);
  c.a *= fog_keep(in.world);
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return c;
}
