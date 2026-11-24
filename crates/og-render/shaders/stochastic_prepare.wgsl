// Empirical per-channel transforms for Heitz/Neyret (HPG 2018), using
// Burley's truncated-Gaussian formulation (JCGT 2019). Original implementation.
// Each texel contributes once, integer counts; no readback or generated image.
struct Histogram { counts: array<atomic<u32>, 384>, };
struct Lookup { entries: array<vec4<u32>, 128>, };
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> histogram: Histogram;
@group(0) @binding(2) var<storage, read_write> lookup: Lookup;

@compute @workgroup_size(8, 8)
fn count(@builtin(global_invocation_id) id: vec3<u32>) {
  let size = textureDimensions(source);
  if (any(id.xy >= size)) { return; }
  let value = clamp(textureLoad(source, vec2<i32>(id.xy), 0).rgb, vec3<f32>(0.0), vec3<f32>(1.0));
  let bin = vec3<u32>(round(value * 127.0));
  atomicAdd(&histogram.counts[bin.r], 1u);
  atomicAdd(&histogram.counts[128u + bin.g], 1u);
  atomicAdd(&histogram.counts[256u + bin.b], 1u);
}

// Winitzki's matched erf/inverse-erf approximation (a=0.147); Gaussian
// coordinate 0..1 represents -3..3 standard deviations. Avoid infinite tails.
fn gaussian(p: f32) -> f32 {
  let x = 2.0 * clamp(p, 0.00135, 0.99865) - 1.0;
  let l = log(1.0 - x * x);
  let a = 2.0 / (3.141592653589793 * 0.147) + l * 0.5;
  return 0.5 + sign(x) * sqrt(max(sqrt(a * a - l / 0.147) - a, 0.0)) * 1.4142135623730951 / 6.0;
}

fn probability(g: f32) -> f32 {
  let x = (g - 0.5) * 6.0 / 1.4142135623730951;
  return 0.5 + 0.5 * sign(x) * sqrt(1.0 - exp(-x * x * (4.0 / 3.141592653589793 + 0.147 * x * x) / (1.0 + 0.147 * x * x)));
}

@compute @workgroup_size(128)
fn transform(@builtin(local_invocation_index) bin: u32) {
  let size = textureDimensions(source);
  let total = f32(size.x) * f32(size.y);
  let quantile = probability(f32(bin) / 127.0) * total;
  var entry = vec4<u32>(0u);
  for (var channel = 0u; channel < 3u; channel++) {
    var before = 0.0;
    var inverse = 0.0;
    var found = false;
    var cumulative = 0.0;
    for (var k = 0u; k < 128u; k++) {
      let count = f32(atomicLoad(&histogram.counts[channel * 128u + k]));
      if (k < bin) { before += count; }
      cumulative += count;
      if (!found && cumulative >= quantile && count > 0.0) {
        inverse = f32(k) / 127.0;
        found = true;
      }
    }
    let here = f32(atomicLoad(&histogram.counts[channel * 128u + bin]));
    entry[channel] = pack2x16unorm(vec2<f32>(gaussian((before + here * 0.5) / total), inverse));
  }
  lookup.entries[bin] = entry;
}
