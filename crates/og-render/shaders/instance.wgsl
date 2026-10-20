// An instance's transform as the instance buffers hold it (instances.rs
// `GpuTransform`): the x, y and z of its four columns (then, in 4 more
// bytes, an array-map instance's layer); their w is (0, 0, 0, 1), an
// instance's transform being affine.
fn instance_matrix(c0: vec3<f32>, c1: vec3<f32>, c2: vec3<f32>, c3: vec3<f32>) -> mat4x4<f32> {
  return mat4x4<f32>(vec4<f32>(c0, 0.0), vec4<f32>(c1, 0.0), vec4<f32>(c2, 0.0), vec4<f32>(c3, 1.0));
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
