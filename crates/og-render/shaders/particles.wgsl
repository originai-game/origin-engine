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
};

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
  if (emitter.render.x > 0.5 && speed > 1e-4) {
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
  o.uv = vec2<f32>(0.5);
  o.local = vec2<f32>(0.0);
  o.color = color_curve(emitter, t) * emitter.color * m.color;
  o.depth = dot(world - view.cam_pos.xyz, view.cam_forward.xyz);
  o.world = world;
  o.normal = normalize(nrm);
  return o;
}

@fragment
fn fs_mesh(in: ParticleOut) -> @location(0) vec4<f32> {
  var c = in.color;
  if (emitter.look.x > 0.5) {
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
  let tex = textureSample(sprite_tex, sprite_sampler, in.uv);
  var c = in.color;
  if (emitter.sheet2.y > 0.5) {
    c *= tex;
  } else {
    // No texture: a soft round dot.
    let d = clamp(1.0 - length(in.local) * 2.0, 0.0, 1.0);
    c.a *= d * d;
  }
  if (emitter.look.x > 0.5) {
    c = vec4<f32>(c.rgb * particle_light(in.clip.xy, in.world, ball_normal(in.facing)), c.a);
  }
  if (emitter.render.z > 0.0) {
    // Clamped: a pass without a depth of its own binds a 1x1 far plane.
    let at = min(vec2<i32>(in.clip.xy), vec2<i32>(textureDimensions(scene_depth)) - vec2<i32>(1));
    let scene = textureLoad(scene_depth, at, 0).r;
    c.a *= clamp((scene - in.depth) / emitter.render.z, 0.0, 1.0);
  }
  c.a *= fog_keep(in.world);
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return c;
}
