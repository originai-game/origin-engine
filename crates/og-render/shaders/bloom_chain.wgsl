// The bloom chain's two filters, shared by the frame's bloom (post.wgsl)
// and a view's glow (views_glow.wgsl), so the two chains spread light
// alike. Each reads its module's `source` through `linear_sampler`, a tap
// at a time through its module's `bloom_tap`, and takes the source's size
// in pixels from its module's `bloom_source_size`: a glow level is sampled
// through its view's base mip level, whose textureDimensions is not the
// level's size.

// The 13-tap downsample (Jimenez, "Next generation post processing in Call of Duty").
fn downsample13(uv: vec2<f32>) -> vec3<f32> {
  let t = 1.0 / bloom_source_size();
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

// The first downsample, from the lit image (the frame's prefilter): the
// same 13 taps as five overlapping boxes of four (the middle one weighted
// 0.5, the four corner ones 0.125, which is downsample13's weights), each
// box's mean weighted by 1 / (1 + its luma) and the sum normalised: the
// partial Karis average of Jimenez's talk (slide 139; Karis, "Real Shading
// in Unreal Engine 4", 2013), so one pixel far brighter than its
// neighbours counts as about one box's worth of light instead of spreading
// its whole energy into every mip as a block (fireflies). `scale` brings
// a tap to the exposed units the luma is judged in; the result is not
// scaled.
fn downsample13_karis(uv: vec2<f32>, scale: f32) -> vec3<f32> {
  let t = 1.0 / bloom_source_size();
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
  var sum = vec3<f32>(0.0);
  var norm = 0.0;
  let boxes = array<vec3<f32>, 5>(
    (j + k + l + m) * 0.25,
    (a + b + d + s) * 0.25,
    (b + c + s + f) * 0.25,
    (d + s + g + h) * 0.25,
    (s + f + h + i) * 0.25,
  );
  for (var n = 0; n < 5; n++) {
    let box = boxes[n];
    let w = select(0.125, 0.5, n == 0) / (1.0 + dot(box * scale, vec3<f32>(0.2126, 0.7152, 0.0722)));
    sum += box * w;
    norm += w;
  }
  return sum / norm;
}

// A 3x3 tent over the smaller level, `radius` texels wide, which the
// upsample adds onto the larger one by blending.
fn bloom_tent(uv: vec2<f32>, radius: f32) -> vec3<f32> {
  let t = radius / bloom_source_size();
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
