// 2D fluid as metaballs (the E5 round): each particle is drawn through the
// sprite quad as a soft kernel added into channels of a field target (its
// fluid's, or by its mix its group's), then each surface is composited in
// the sprite order by a threshold over the sum of its channels: an
// antialiased edge, a body that deepens where the field is dense and takes
// its fluids' colours by their share there, a bright rim inside the edge.
// Prepended by frame.wgsl.

struct BlobIn {
  @location(0) position_radius: vec4<f32>,  // world xyz, radius
  @location(1) channel: vec4<f32>,          // which field channel it adds to
};

struct BlobOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) corner: vec2<f32>,
  @location(1) channel: vec4<f32>,
};

@vertex
fn vs_field(@builtin(vertex_index) vi: u32, b: BlobIn) -> BlobOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
  let c = corners[vi];
  var o: BlobOut;
  let p = vec3<f32>(b.position_radius.xy + c * b.position_radius.w, b.position_radius.z);
  o.clip = frame.view_proj * vec4<f32>(p, 1.0);
  o.corner = c;
  o.channel = b.channel;
  return o;
}

@fragment
fn fs_field(in: BlobOut) -> @location(0) vec4<f32> {
  let d2 = dot(in.corner, in.corner);
  if (d2 >= 1.0) { discard; }
  let k = 1.0 - d2;
  return in.channel * (k * k);
}

struct Fluid {
  colors: array<vec4<f32>, 4>,  // per channel, the linear rgba of its fluid's body
  rim: vec4<f32>,               // rgb: rim colour, w: rim width (field units)
  channel: vec4<f32>,           // the surface's channels
  params: vec4<f32>,            // x: threshold, y: 1/w, z: 1/h, w: 1 for one fluid (colors[0])
};

@group(1) @binding(0) var field: texture_2d<f32>;
@group(1) @binding(1) var<uniform> fluid: Fluid;

struct FullOut {
  @builtin(position) clip: vec4<f32>,
};

@vertex
fn vs_composite(@builtin(vertex_index) vi: u32) -> FullOut {
  let uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
  var o: FullOut;
  o.clip = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
  return o;
}

@fragment
fn fs_composite(in: FullOut) -> @location(0) vec4<f32> {
  let raw = textureLoad(field, vec2<i32>(in.clip.xy), 0) * fluid.channel;
  let f = raw.x + raw.y + raw.z + raw.w;
  let th = fluid.params.x;
  let aa = max(fwidth(f), 1e-4);
  let alpha = smoothstep(th - aa, th + aa, f);
  if (alpha <= 0.0) { discard; }
  let depth = clamp((f - th) / max(th * 3.0, 1e-4), 0.0, 1.0);
  // One fluid: its colour; a group: its fluids' colours by their share.
  var body = fluid.colors[0];
  if (fluid.params.w < 0.5) {
    body = (fluid.colors[0] * raw.x + fluid.colors[1] * raw.y + fluid.colors[2] * raw.z
      + fluid.colors[3] * raw.w) / max(f, 1e-6);
  }
  var c = body.rgb * (1.15 - 0.4 * depth);
  let rim = 1.0 - smoothstep(th, th + fluid.rim.w, f);
  c = mix(c, fluid.rim.rgb, rim * 0.85);
  return vec4<f32>(c, alpha * body.a);
}
