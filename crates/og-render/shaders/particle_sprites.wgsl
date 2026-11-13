// Sprite-mode particles simulated in compute, as the 2D batch draws them:
// one instance a slot of the system's ring (`SpriteInstance` in sprites.rs,
// written word by word: 31 words), sized, coloured, turned and framed as
// the CPU path's `appearance` does; a dead slot an empty quad. Prepended by
// particle_common.wgsl.

struct SpriteParams {
  first: u32,      // the range's first instance
  max: u32,        // the ring's slots
  frames: u32,     // the sheet's frames (an array emitter's: the array's)
  columns: u32,
  rows: u32,
  array: u32,      // 1: an array emitter (the frame is the layer)
  soft: u32,       // 1: the soft dot's round edge
  _pad0: u32,
  z: f32,          // the system's z (the sprite order's)
  glow: f32,
  lit: f32,        // the light map its instances take
  _pad1: f32,
};

@group(0) @binding(0) var<uniform> emitter: Emitter;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(2) var<uniform> params: SpriteParams;
@group(0) @binding(3) var<storage, read_write> instances: array<u32>;

const WORDS: u32 = 31u;
// The flash bytes of an instance without one (rgb 255, a 0: NO_FLASH).
const NO_FLASH: u32 = 0x00ffffffu;

fn put(at: u32, v: vec4<f32>) {
  instances[at] = bitcast<u32>(v.x);
  instances[at + 1u] = bitcast<u32>(v.y);
  instances[at + 2u] = bitcast<u32>(v.z);
  instances[at + 3u] = bitcast<u32>(v.w);
}

@compute @workgroup_size(64)
fn expand(@builtin(global_invocation_id) id: vec3<u32>) {
  let i = id.x;
  if (i >= params.max) { return; }
  let p = particles[i];
  let at = (params.first + i) * WORDS;
  let age = p.pos_age.w;
  let alive = age < p.vel_life.w;
  let t = clamp(age / max(p.vel_life.w, 1e-6), 0.0, 1.0);
  var size = p.misc.x * size_curve(t);
  var color = color_curve(t) * emitter.color;
  if (!alive) {
    size = 0.0;
    color = vec4<f32>(0.0);
  }
  let rotation = particle_rotation(p.misc, age, p.vel_life.w, t);
  let frames = max(params.frames, 1u);
  let mode = u32(emitter.sheet.w);
  var frame = 0u;
  if (mode == 1u) { frame = u32(t * emitter.sheet2.x * f32(frames)) % frames; }
  if (mode == 2u) { frame = u32(age * emitter.sheet2.x) % frames; }
  if (mode == 3u) { frame = min(u32(p.misc.w * f32(frames)), frames - 1u); }
  var pos = p.pos_age.xyz;
  if (emitter.ranges2.z > 0.5) {
    pos = (emitter.transform * vec4<f32>(pos, 1.0)).xyz;
  }
  var uv = vec4<f32>(0.0, 0.0, 1.0, 1.0);
  var layer = 0u;
  if (params.array == 1u) {
    layer = frame;
  } else {
    let du = 1.0 / f32(params.columns);
    let dv = 1.0 / f32(params.rows);
    let col = f32(frame % params.columns);
    let row = f32(frame / params.columns);
    uv = vec4<f32>(col * du, row * dv, (col + 1.0) * du, (row + 1.0) * dv);
  }
  put(at, vec4<f32>(pos.x, pos.y, params.z, rotation));
  put(at + 4u, vec4<f32>(size, size, 0.5, 0.5));
  put(at + 8u, uv);
  put(at + 12u, color);
  instances[at + 16u] = NO_FLASH;
  instances[at + 17u] = bitcast<u32>(params.glow);
  instances[at + 18u] = bitcast<u32>(params.lit);
  instances[at + 19u] = bitcast<u32>(f32(params.soft));
  put(at + 20u, vec4<f32>(0.0));
  instances[at + 24u] = layer;
  instances[at + 25u] = 0u;
  put(at + 26u, vec4<f32>(0.0));
  instances[at + 30u] = 0u;
}
