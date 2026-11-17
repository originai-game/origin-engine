// The clouds' shape (clouds.rs): the noise volume and the density it
// gives a point of the layer. Before sky.wgsl in the sky's module and
// before cloud_shadow_bake.wgsl in the shadow bake's.

// The clouds' noise volume (clouds.rs, clouds_noise.wgsl): r the base
// shape, g and b Worley detail, a the weather map; tiling, sampled with
// repeat.
@group(1) @binding(1) var cloud_noise: texture_3d<f32>;
@group(1) @binding(2) var cloud_sampler: sampler;

fn cloud_remap(v: f32, lo: f32, hi: f32) -> f32 {
  return clamp((v - lo) / max(hi - lo, 1e-4), 0.0, 1.0);
}

// The cloud density at `p` (metres, planet-centred, the viewer over the
// origin's xz) of height `h` above the ground. The clouds stand over the
// world: the viewer's place in it ([10].xy) and the wind's offset ([1])
// place the point in the noise, each wrapped at four tiles, which every
// scale below repeats in (a scale four tiles are no period of changed
// every cloud's shape at a wrap).
fn cloud_density(p: vec3<f32>, h: f32, cheap: bool) -> f32 {
  return cloud_density_at(p, h, select(1u, 0u, cheap));
}

// Never less than cloud_density with its detail at `p`: what the march
// strides through clear air on (sky.wgsl clouds_march), so a stride
// steps over no billow the detail pushes out.
fn cloud_density_bound(p: vec3<f32>, h: f32) -> f32 {
  return cloud_density_at(p, h, 2u);
}

fn cloud_density_at(p: vec3<f32>, h: f32, detail: u32) -> f32 {
  let c = frame.clouds;
  let hf = (h - c[0].z) / max(c[0].w, 1.0);
  if (hf <= 0.0 || hf >= 1.0) { return 0.0; }
  // (Less the offset: the wind carries the shapes along it.)
  return cloud_density_of(vec3<f32>(p.x + c[10].x, h, p.z + c[10].y) - c[1].xyz, hf, detail);
}

// How much of the weather map stands in cloud for a cover of `c0`: the
// map is uniform, a column stands in cloud where it passes 1 - this, and
// with it the sky over a viewer is about `c0` covered (measured over the
// volume: 0.3 a fair-weather sky; an overcast passes everywhere).
fn cloud_stand(c0: f32) -> f32 {
  return mix(0.19 * min(c0 * 10.0, 1.0) + 1.09 * c0, 1.4, smoothstep(0.9, 1.0, c0));
}

// The density at `at`, a point of the noise in metres (the world's x and
// z and the height, less the wind's offset), `hf` of the way up the layer
// (0 to 1, exclusive): what the sky marches and what the clouds' shadow
// is baked from (cloud_shadow_bake.wgsl), so a shadow lies under its cloud.
// The cloudscape of Schneider 2015 and 2017 (clouds_noise.wgsl):
//   - the weather map says where clouds stand and how much (`cover`): a
//     column outside it holds none, so no puff floats loose of the layer;
//   - the height gradient stands every cloud on the layer's base, densest
//     just above it and thinning to its top, which rises with the cover
//     (low humilis at a field's edge, towers in its middle, the whole
//     layer under an overcast);
//   - the base shape, thinned by the gradient, passes the cover's
//     threshold: only its strongest cells reach the top (domes) and the
//     base is cut flat at the condensation level;
//   - the Worley detail moves the surface in and out about a mean
//     erosion: wisps at the base, billows (the cells' round middles pushed
//     out) above it.
// `cheap`: the detail at its mean, for the light's and the shadow's
// samples, which are wider than it (its most, which the strides read,
// cast every shadow wider than its cloud).
fn cloud_density_in(at: vec3<f32>, hf: f32, cheap: bool) -> f32 {
  return cloud_density_of(at, hf, select(1u, 0u, cheap));
}

// `detail`: 0 at its mean, 1 the detail read, 2 at its most.
fn cloud_density_of(at: vec3<f32>, hf: f32, detail: u32) -> f32 {
  let c = frame.clouds;
  let uvw = at / c[1].w;
  let c0 = c[0].x;
  let map = textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(uvw.x * 0.25, uvw.z * 0.25, 0.5), 0.0).a;
  // A broken sky's clouds stop short of a solid mass, whose top would be
  // the layer's own plane; an overcast is one.
  let cover = clamp((map - 1.0 + cloud_stand(c0)) * 2.5, 0.0, 0.8 + 0.2 * smoothstep(0.7, 1.0, c0));
  if (cover <= 0.0) { return 0.0; }
  let top = 0.3 + 0.7 * smoothstep(0.1, 0.8, cover);
  let gradient = clamp(hf / 0.07, 0.0, 1.0) * clamp((top - hf) / (top * 0.75), 0.0, 1.0);
  if (gradient <= 0.0) { return 0.0; }
  var shape = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy, 0.0).r * gradient;
  // The detail moves the cloud's surface in and out about its mean (half
  // the cover's share), before the threshold: eroding the density after it
  // only thinned a rim a few metres wide and left the body a smooth lump.
  // Its spread is two and a half times the Worley fBms' own (about 0.17
  // either side of 0.48) from a fifth of the way up, and at least that of a
  // cover of 0.6: at the fBms' own, or scaled down with a thin cover, a
  // near cloud's billows were too shallow to see and its edge a smooth
  // lump. At the base, and as the cover closes into a deck (a stratiform
  // overcast has no billows), it falls back to the fBms' own: an overcast
  // seen from under it drew as a mottle of dark rings at the wider one.
  let k = c[2].w * 0.8;
  let edge = max(cover, 0.6);
  let wide = mix(2.5, 1.0, smoothstep(0.7, 0.95, c0));
  let spread = 0.17 * mix(1.0, wide, clamp(hf / 0.2, 0.0, 1.0)) * edge;
  // How far past the shape alone the detail can push the surface out (the
  // bound's, so the strides and the gate below miss no billow).
  let out = max(spread - 0.5 * cover, 0.0) * k;
  if (detail == 2u) {
    shape += out;
  } else if (detail == 0u) {
    shape -= 0.5 * cover * k;
  } else if (shape + out > 1.0 - cover && k > 0.0) {
    // (Not worth the detail's read where the shape cannot pass the
    // threshold with it.)
    let fine = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 4.0, 0.0);
    let e = clamp((0.65 * fine.g + 0.35 * fine.b - 0.48) / 0.17, -1.0, 1.0);
    shape -= (0.5 * cover + mix(e, -e, clamp(hf * 6.0, 0.0, 1.0)) * spread) * k;
  }
  // (Full 0.15 past the threshold, of a cover of at least 0.6: a cumulus
  // has an edge some tens of metres deep, not a haze that thickens to its
  // middle, and a thin cover's small clouds no sharper one.)
  let base = cloud_remap(shape, 1.0 - cover, 1.0 - cover + 0.15 * edge) * cover;
  return base * c[0].y;
}
