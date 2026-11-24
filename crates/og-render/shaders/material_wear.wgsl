// Authored aging, independent of OG_WEATHER and the current WeatherState.
// After lighting.wgsl and weather_rain.wgsl (the shared, frozen rain runs).
override OG_MATERIAL_WEAR: bool = true;

fn material_wear(s_in: Surface, uv: vec2<f32>, geo: vec3<f32>, p_frame: vec3<f32>, footprint: f32) -> Surface {
  var s = s_in;
  if (!OG_MATERIAL_WEAR) { return s; }
  let dirt = material.grime.w;
  let height = material.variation.x;
  let stains = material.variation.y;
  if (max(dirt, stains) <= 0.0) { return s; }
  let p = p_frame + frame.origin.xyz;
  let wall = 1.0 - smoothstep(0.35, 0.75, abs(geo.y));
  if (dirt > 0.0 && height > 0.0) {
    // Metric input, never the map's scaled, rotated or translated uv.
    // Only raggedness multiplies the falloff: no dirt above the authored height.
    let falloff = 1.0 - smoothstep(0.0, height, max(-uv.y, 0.0));
    let cover = dirt * falloff * wall * (0.65 + 0.35 * weather_noise(p.xz * 2.3));
    s.albedo = mix(s.albedo, material.grime.rgb, cover);
    s.metallic *= 1.0 - cover;
    s.roughness = mix(s.roughness, max(s.roughness, 0.9), cover);
    s.f0 = mix(s.f0, vec3<f32>(0.04), cover);
  }
  if (stains > 0.0 && wall > 0.0) {
    let stain = weather_streaks(p, geo, 0.0, footprint) * stains * wall;
    s.albedo *= 1.0 - stain;
  }
  return s;
}
