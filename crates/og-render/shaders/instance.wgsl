// An instance's transform as the instance buffers hold it (instances.rs
// `GpuTransform`): the x, y and z of its four columns (then, in 16 more
// bytes, an array-map instance's layer and the instance's look, pbr.wgsl
// `InstanceFxIn`); their w is (0, 0, 0, 1), an instance's transform being
// affine.
fn instance_matrix(c0: vec3<f32>, c1: vec3<f32>, c2: vec3<f32>, c3: vec3<f32>) -> mat4x4<f32> {
  return mat4x4<f32>(vec4<f32>(c0, 0.0), vec4<f32>(c1, 0.0), vec4<f32>(c2, 0.0), vec4<f32>(c3, 1.0));
}

// How much of an instance fading in is still hidden, in 255 steps (0:
// whole), and in bit 8 whether it fades out against another fading in, from
// its look's flash word: 512 a step above the outline group (instances.rs
// `fx_bits`, environment.rs `InstanceFx::hidden_bits`).
fn look_hidden_steps(look: vec4<f32>) -> u32 {
  return u32(max(floor(look.y / 512.0), 0.0)) & 511u;
}

// The share of an instance hidden, 0 to 1 (0: whole), from its steps;
// negative for one fading out against another (common.wgsl
// `dither_cut`).
fn hidden_share(steps: u32) -> f32 {
  let share = f32(steps & 255u) / 255.0;
  return select(share, -share, (steps & 256u) != 0u);
}

fn look_hidden(look: vec4<f32>) -> f32 {
  return hidden_share(look_hidden_steps(look));
}

// A normal's world direction: through the inverse transpose of the model's
// upper 3x3 (its cofactor matrix, flipped for a mirroring transform), so a
// non-uniform scale bends normals the right way. Every vertex stage takes
// its normal from here (the lit, shadow, visibility-buffer and custom
// ones), so they agree to the bit.
fn world_normal(model: mat4x4<f32>, n: vec3<f32>) -> vec3<f32> {
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  let mirrored = dot(a, cross(b, c)) < 0.0;
  return normalize(cofactor * n) * select(1.0, -1.0, mirrored);
}

// `m * p` summed in column order, each product fused: the arithmetic the
// shader compiler gives `m * p` when p's w is a value it cannot know. A
// point an instance matrix moves has the w 1 the compiler can see, and
// then it reorders the plain product's sum (the w term first), which moves
// the point by an ulp and flips edge pixels; the projections of such
// points (into the view, the shadow maps, last frame's view) go through
// here so they stay the full 4x4's, bit for bit.
fn mul_point(m: mat4x4<f32>, p: vec4<f32>) -> vec4<f32> {
  return fma(m[3], vec4<f32>(p.w), fma(m[2], vec4<f32>(p.z), fma(m[1], vec4<f32>(p.y), m[0] * p.x)));
}
