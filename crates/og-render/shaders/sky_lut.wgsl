// Hillaire (EGSR 2020 §5.3) Sky-View LUT compute shader (`media/sky_lut.rs`):
// precomputes the 32-step spherical physical atmosphere scattering (sun + moon)
// into a 192x108 rgba16float lookup texture with non-linear horizon-focused
// elevation mapping and an explicit horizon seam split so sky and ground never
// bleed across the horizon during hardware bilinear filtering.
//
// Prepended by `common.wgsl` and `sky_scatter.wgsl` (with `SCATTER` rewritten
// to `lut.scatter`).

struct SkyLutParams {
  scatter: array<vec4<f32>, 38>,
  // xyz: unit sun direction, w: viewer altitude h0 (km)
  sun_dir: vec4<f32>,
  // rgb: sun light above air (at illuminance 1)
  sun_light: vec4<f32>,
  // xyz: unit moon direction, w: moon enabled (> 0.5)
  moon_dir: vec4<f32>,
  // rgb: moon light above air
  moon_light: vec4<f32>,
};

@group(0) @binding(0) var<uniform> lut: SkyLutParams;
@group(0) @binding(1) var sky_lut_out: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_sky_lut(@builtin(global_invocation_id) id: vec3<u32>) {
  let dims = textureDimensions(sky_lut_out);
  if (id.x >= dims.x || id.y >= dims.y) { return; }

  let bottom = lut.scatter[3].x;
  let h0 = max(lut.sun_dir.w, 0.001);
  let horizon = -acos(clamp(bottom / (bottom + h0), 0.0, 1.0));

  // Periodic azimuth with duplicated endpoints (id.x = 0 and dims.x - 1 both
  // map to +-PI) so ClampToEdge filtering never introduces an azimuth seam.
  let phi = (f32(id.x) / f32(max(dims.x - 1u, 1u))) * (2.0 * PI) - PI;

  // Non-linear latitude mapping concentrated around the horizon, split cleanly
  // between upper rows (sky, >= dims.y / 2) and lower rows (ground, < dims.y / 2).
  let half_h = dims.y / 2u;
  let span = max(f32(half_h) - 1.0, 1.0);
  var beta: f32;
  if (id.y >= half_h) {
    let adj = f32(id.y - half_h) / span;
    beta = min(horizon + 1e-4 + (0.5 * PI - horizon - 1e-4) * adj * adj, 0.5 * PI);
  } else {
    let adj = f32(half_h - 1u - id.y) / span;
    beta = max(horizon - 1e-4 - (0.5 * PI + horizon - 1e-4) * adj * adj, -0.5 * PI);
  }

  let cb = cos(beta);
  let d = normalize(vec3<f32>(cb * cos(phi), sin(beta), cb * sin(phi)));
  let sun = normalize(lut.sun_dir.xyz);
  let moon = select(sun, normalize(lut.moon_dir.xyz), lut.moon_dir.w > 0.5);
  let moon_light = select(vec3<f32>(0.0), lut.moon_light.rgb, lut.moon_dir.w > 0.5);

  let rad = physical_sky_lit(d, sun, lut.sun_light.rgb, moon, moon_light, h0, 32u);
  textureStore(
    sky_lut_out,
    vec2<i32>(id.xy),
    vec4<f32>(clamp(rad, vec3<f32>(0.0), vec3<f32>(65504.0)), 1.0),
  );
}
