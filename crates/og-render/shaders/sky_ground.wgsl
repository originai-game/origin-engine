// The grounded ground as a shadow catcher (ground_heights.rs): the sky
// pass's variant for a grounded environment whose ground takes the shadows
// of what stands on it. After sky.wgsl and lighting.wgsl, in their own
// module (sky.rs `sky_ground_source`), so a sky without it keeps its own.
//
// Where the view ray meets the dome's flat ground, the ground the map shows
// is drawn times the light that point gets with the scene's props over the
// light it would get without them (Debevec, "Rendering Synthetic Objects
// into Real Scenes", 1998: differential rendering; Blender's shadow
// catcher): the map's light from above, times how much of it the props
// leave open (ground_open.wgsl, baked from the ground's height map), and
// the sun's, times its shadow (the cascades).

// The ground's openness (ground_open.wgsl), read with the IBL sampler.
@group(2) @binding(0) var ground_open_map: texture_2d<f32>;

// The share of the sky's light the props leave open at the ground point
// `p` (frame coordinates): the baked openness over the 4x4 window its
// sixteen phases fill, faded to all open over the map's outer eighth so a
// prop at its edge leaves no step.
fn ground_open(p: vec3<f32>) -> f32 {
  let half = frame.env_dome[1].w;
  let d = (p.xz - (frame.env_dome[0].xyz).xz) / half;
  let edge = max(abs(d.x), abs(d.y));
  if (edge >= 1.0) { return 1.0; }
  let uv = vec2<f32>(0.5 + 0.5 * d.x, 0.5 - 0.5 * d.y);
  let texel = 1.0 / vec2<f32>(textureDimensions(ground_open_map));
  var open = 0.0;
  for (var k = 0; k < 4; k++) {
    let o = vec2<f32>(f32(k & 1) * 2.0 - 1.0, f32(k >> 1u) * 2.0 - 1.0);
    open += textureSampleLevel(ground_open_map, ibl_sampler, uv + o * texel, 0.0).r;
  }
  return mix(open * 0.25, 1.0, smoothstep(0.85, 0.97, edge));
}

// What the ground keeps of its light at `p`, per channel: (E_map V + E_sun S)
// / (E_map + E_sun), moved towards 1 as the ground takes less shadow.
fn ground_kept(p: vec3<f32>) -> vec3<f32> {
  let up = vec3<f32>(0.0, 1.0, 0.0);
  let map = ambient_light_level(up, p) * frame.params.x + hemi_irradiance(up);
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(l.y, 0.0);
  let sun = frame.light_color.rgb * n_dot_l / PI;
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) {
    shadow = cascade_shadow(p, up, n_dot_l);
  }
  let open = ground_open(p);
  let kept = (map * open + sun * shadow) / max(map + sun, vec3<f32>(1e-6));
  return mix(vec3<f32>(1.0), kept, frame.env_dome[1].y);
}

@fragment
fn fs_sky_ground(in: SkyOut) -> @location(0) vec4<f32> {
  let dir = pixel_ray(in.ndc);
  let spread = max(length(dpdx(dir)), length(dpdy(dir)));
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  // The view ray as the sky's (sky.wgsl fs_sky): from the pixel's near point along pixel_ray.
  let near = frame.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
  let eye = near.xyz / near.w;
  var color = sky_background(eye, dir) * (1.0 - frame.view_up.w);
  // Where the ray meets the dome's ground before it leaves the dome (as
  // frame.wgsl dome_lookup finds it).
  let h = frame.env_dome[0].w;
  let o = eye - frame.env_dome[0].xyz;
  let r = frame.env_dome[1].x;
  let b = dot(o, dir);
  let disc = b * b - (dot(o, o) - r * r);
  if (h > 0.0 && disc >= 0.0 && dir.y < 0.0) {
    let tg = (-h - o.y) / dir.y;
    if (tg > 0.0 && tg < -b + sqrt(disc)) {
      color *= ground_kept(eye + dir * tg);
    }
  }
  let background = sky_fog(color, dir);
  if (frame.clouds[3].w > 0.5) {
    return sky_clouds_stored(clouds_over(dir, in.pos.xy, in.grey, spread), background, dir);
  }
  return vec4<f32>(background, sky_tone_fraction());
}
