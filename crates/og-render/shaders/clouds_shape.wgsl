// The clouds' shape (clouds.rs): the noise volume and the density it
// gives a point of the layer. Before sky.wgsl in the sky's module and
// before cloud_shadow_bake.wgsl in the shadow bake's.

// The clouds' noise volume (clouds.rs): r Perlin-Worley, g and b Worley
// detail, a coverage; tiling, sampled with repeat.
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
  let c = frame.clouds;
  let hf = (h - c[0].z) / max(c[0].w, 1.0);
  if (hf <= 0.0 || hf >= 1.0) { return 0.0; }
  // (Less the offset: the wind carries the shapes along it.)
  return cloud_density_in(vec3<f32>(p.x + c[10].x, h, p.z + c[10].y) - c[1].xyz, hf, cheap);
}

// The density at `at`, a point of the noise in metres (the world's x and
// z and the height, less the wind's offset), `hf` of the way up the layer
// (0 to 1, exclusive): what the sky marches and what the clouds' shadow
// is baked from (cloud_shadow_bake.wgsl), so a shadow lies under its cloud.
fn cloud_density_in(at: vec3<f32>, hf: f32, cheap: bool) -> f32 {
  let c = frame.clouds;
  let uvw = at / c[1].w;
  // The lobes vary in size: the shape is read at two scales, blended by a
  // slow noise across the sky (small popcorn here, broad towers there).
  let small = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy, 0.0).r;
  let broad = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.5 + vec3<f32>(0.31, 0.17, 0.53), 0.0).r;
  let cover_px = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.25, 0.0);
  let cover = cover_px.a;
  let shape = mix(small, broad, smoothstep(0.3, 0.7, textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.25 + vec3<f32>(0.6, 0.0, 0.2), 0.0).a));
  // The coverage varies across the sky most when it is broken, and not
  // at all when clear or overcast (an overcast stays a ceiling).
  // (The power: the cells at different heights overlap, so a column
  // meets cloud more often than one level does; with it the sky over a
  // viewer is about `coverage` covered, 0.3 a fair-weather sky.)
  let c0 = c[0].x;
  let local = clamp(pow(c0, 1.25) + (cover - 0.5) * 0.5 * min(1.0, 4.0 * c0 * (1.0 - c0)), 0.0, 1.0);
  // The shape channel is uniform: a threshold of 1 - local leaves a
  // fraction `local` in cloud near the base. Higher up the threshold
  // rises, so only the strongest cells reach the top (domes); the base is
  // cut flat at the condensation level, the cloud densest just above it.
  let th = mix(1.0 - local, 1.0, smoothstep(0.3, 1.0, hf) * 0.85);
  var base = cloud_remap(shape, th, min(th + 0.35, 1.0)) * smoothstep(0.0, 0.025, hf) * (1.0 + 0.5 * (1.0 - smoothstep(0.02, 0.2, hf)));
  if (!cheap && c[2].w > 0.0) {
    // The fine noise eats the tops into cauliflower and leaves the flat
    // bases whole.
    let fine = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 5.0, 0.0);
    let erode = mix(fine.g, fine.b, hf) * c[2].w * (0.25 + 1.1 * hf);
    base = cloud_remap(base, erode * 0.6, 1.0);
  }
  return base * c[0].y;
}
