// Particle draw: one instanced quad per slot of a system's ring (dead slots
// collapse), camera-facing or stretched along the velocity, sized and
// coloured by the emitter's curves, animated through a sprite sheet, and
// softened against the depth prepass. The instance data is the simulation's
// storage buffer (full tier) or the CPU path's upload. Prepended by
// frame.wgsl, noise.wgsl and particle_common.wgsl.

@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(2) @binding(0) var<uniform> emitter: Emitter;
@group(2) @binding(1) var<uniform> view: ParticleView;
@group(2) @binding(2) var scene_depth: texture_2d<f32>;

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
  return o;
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
  if (emitter.render.z > 0.0) {
    let scene = textureLoad(scene_depth, vec2<i32>(in.clip.xy), 0).r;
    c.a *= clamp((scene - in.depth) / emitter.render.z, 0.0, 1.0);
  }
  c.a *= fog_keep(in.world);
  if (c.a <= 0.0) { discard; }
  return c;
}
