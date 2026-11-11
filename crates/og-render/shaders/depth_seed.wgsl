// The main pass's depth seeded from the prepass's (depth_seed.rs): one
// triangle over the target that writes each pixel's prepass depth, moved
// back by SLACK, and leaves the colour as it is (the pipeline's blend).
@group(0) @binding(0) var pre_depth: texture_2d<f32>;

// How far behind the prepass's depth the seed sits: 2^-18, 64 steps of the
// depth buffer beyond twice the near plane. A lit vertex stage is compiled
// apart from its prepass one and may land a rounding away from it; under
// this the surface still passes the main pass's Less.
const SLACK: f32 = 0.000003814697265625;

@vertex
fn vs_seed(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  return vec4<f32>(x, y, 1.0, 1.0);
}

struct Seed {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_seed(@builtin(position) at: vec4<f32>) -> Seed {
  var o: Seed;
  o.color = vec4<f32>(0.0);
  // The far plane where the prepass drew nothing: the pass's own clear.
  o.depth = min(textureLoad(pre_depth, vec2<i32>(at.xy), 0).r + SLACK, 1.0);
  return o;
}
