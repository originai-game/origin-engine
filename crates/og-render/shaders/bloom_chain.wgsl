// The bloom chain's two filters, shared by the frame's bloom (post.wgsl)
// and a view's glow (views_glow.wgsl), so the two chains spread light
// alike. Each reads its module's `source` through `linear_sampler`, a tap
// at a time through its module's `bloom_tap`.

// The 13-tap downsample (Jimenez, "Next generation post processing in Call of Duty").
fn downsample13(uv: vec2<f32>) -> vec3<f32> {
  let t = 1.0 / vec2<f32>(textureDimensions(source));
  let s = bloom_tap(uv);
  let a = bloom_tap(uv + t * vec2<f32>(-2.0, -2.0));
  let b = bloom_tap(uv + t * vec2<f32>(0.0, -2.0));
  let c = bloom_tap(uv + t * vec2<f32>(2.0, -2.0));
  let d = bloom_tap(uv + t * vec2<f32>(-2.0, 0.0));
  let f = bloom_tap(uv + t * vec2<f32>(2.0, 0.0));
  let g = bloom_tap(uv + t * vec2<f32>(-2.0, 2.0));
  let h = bloom_tap(uv + t * vec2<f32>(0.0, 2.0));
  let i = bloom_tap(uv + t * vec2<f32>(2.0, 2.0));
  let j = bloom_tap(uv + t * vec2<f32>(-1.0, -1.0));
  let k = bloom_tap(uv + t * vec2<f32>(1.0, -1.0));
  let l = bloom_tap(uv + t * vec2<f32>(-1.0, 1.0));
  let m = bloom_tap(uv + t * vec2<f32>(1.0, 1.0));
  return s * 0.125 + (a + c + g + i) * 0.03125 + (b + d + f + h) * 0.0625 + (j + k + l + m) * 0.125;
}

// A 3x3 tent over the smaller level, `radius` texels wide, which the
// upsample adds onto the larger one by blending.
fn bloom_tent(uv: vec2<f32>, radius: f32) -> vec3<f32> {
  let t = radius / vec2<f32>(textureDimensions(source));
  var c = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb * 4.0;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, 0.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, 0.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(0.0, -1.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(0.0, 1.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, -1.0), 0.0).rgb;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, -1.0), 0.0).rgb;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, 1.0), 0.0).rgb;
  c += textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, 1.0), 0.0).rgb;
  return c / 16.0;
}
