// A custom post pass (post_custom.rs): the fullscreen triangle, what the
// pass reads, then its parameters and the game's `post` function.
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  o.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
  return o;
}

struct PostUniform {
  params: array<vec4<f32>, 8>,
  screen: vec4<f32>,  // xy: size in pixels, z: seconds
};
@group(0) @binding(0) var previous_tex: texture_2d<f32>;
@group(0) @binding(1) var scene_tex: texture_2d<f32>;
@group(0) @binding(2) var depth_tex: texture_2d<f32>;
@group(0) @binding(3) var gbuffer_tex: texture_2d<f32>;
@group(0) @binding(4) var post_sampler: sampler;
@group(0) @binding(5) var<uniform> u: PostUniform;

struct PostInput {
  uv: vec2<f32>,     // 0..1 from the top left
  pixel: vec2<f32>,  // the pixel's centre
  size: vec2<f32>,   // the target's size in pixels
  time: f32,         // the scene clock in seconds
};

// The pass before this one (the lit image for the first), linear HDR.
fn previous(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(previous_tex, post_sampler, uv, 0.0);
}

// The lit image before any custom pass, linear HDR.
fn scene_color(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(scene_tex, post_sampler, uv, 0.0);
}

fn texel_of(uv: vec2<f32>) -> vec2<i32> {
  let size = vec2<i32>(textureDimensions(depth_tex));
  return clamp(vec2<i32>(uv * vec2<f32>(size)), vec2<i32>(0), size - 1);
}

// The view depth along the camera's axis in world units (the sky: 1e9 or more).
fn depth(uv: vec2<f32>) -> f32 {
  return textureLoad(depth_tex, texel_of(uv), 0).r;
}

// The world normal (zero: the sky).
fn normal(uv: vec2<f32>) -> vec3<f32> {
  let e = textureLoad(gbuffer_tex, texel_of(uv), 0).xy;
  var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    let s = select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0));
    n = vec3<f32>((1.0 - abs(n.yx)) * s, n.z);
  }
  return normalize(n);
}

@fragment
fn fs_custom_post(in: VsOut) -> @location(0) vec4<f32> {
  var f: PostInput;
  f.uv = in.uv;
  f.pixel = in.pos.xy;
  f.size = u.screen.xy;
  f.time = u.screen.z;
  return post(f, post_params());
}

