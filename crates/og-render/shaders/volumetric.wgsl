// Volumetric fog (volumetric.rs), after frame.wgsl, the light tables,
// lighting.wgsl and area.wgsl. The medium: `density` per metre plus height
// fog (`height density` below `height`, thinning above it by `falloff`),
// scattering `colour × scattering` of the light it is lit by with a
// Henyey-Greenstein phase of `anisotropy`.
//
// The froxel grid: `inject` lights every froxel (160 × 90 × 64 cells over the
// view, slices quadratically deeper to `distance`) by the sun through its
// cascades and the cluster's point, spot and rect lights through their
// shadows and cookies; `integrate` sums them front to back into the
// in-scattered light and the transmittance at each slice's far edge;
// `fs_froxel` reads that at each pixel's depth. The raymarch: `fs_raymarch`
// marches each pixel's view ray in steps, lit by the sun and the first
// RAYMARCH_LIGHTS lights of each step's cluster. Both
// output (in-scattered light, transmittance), blended over the lit image
// as colour × transmittance + light.
//
// Under water (`VOL_WATER`, volumetric.rs `Medium::Water`) the medium is the water the eye is in and the pass draws
// the sun's own light in it (`water_in`): the light shafts. The frame's fog has already drawn the water's even light
// and dimmed what is seen through it, so the transmittance it writes is 1.

// True in the water's pipelines: each medium compiles its own, so the fog's carries none of the water's code.
override VOL_WATER: bool = false;

struct Vol {
  scatter: vec4<f32>,  // rgb: colour × scattering, w: anisotropy
  medium: vec4<f32>,   // x: density, y: height, z: height density, w: falloff
  extent: vec4<f32>,   // x: distance, y: raymarch steps, zw: the target's size in pixels
  grid: vec4<f32>,     // xyz: froxels across, down and deep, w: the frame's noise seed
};

@group(1) @binding(0) var<uniform> vol: Vol;

fn vol_density(p: vec3<f32>) -> f32 {
  // The water's: its fog's, below its surface.
  if (VOL_WATER) { return select(0.0, max(frame.fog_distance.x, 1e-5), p.y < frame.fog_height.x); }
  return vol.medium.x + vol.medium.z * exp(-vol.medium.w * max(p.y - vol.medium.y, 0.0));
}

fn hg_phase(cos_theta: f32, g: f32) -> f32 {
  let g2 = g * g;
  return (1.0 - g2) / (4.0 * PI * pow(max(1.0 + g2 - 2.0 * g * cos_theta, 1e-4), 1.5));
}

// The view depth of a slice edge at fraction f of the grid's depth.
fn slice_depth(f: f32) -> f32 {
  return vol.extent.x * f * f;
}

// The world direction of the view ray through uv (0..1, y down).
fn view_ray(uv: vec2<f32>) -> vec3<f32> {
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a = frame.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b = frame.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
  return normalize(b.xyz / b.w - a.xyz / a.w);
}

// The sun's light scattered towards the eye at p (dir: the view ray).
fn sun_in(p: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let l = normalize(frame.light_dir.xyz);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow_lookup(p, l, 1.0, false); }
  return frame.light_color.rgb * shadow * hg_phase(dot(dir, l), vol.scatter.w);
}

// The sun's light in the water at p scattered towards the eye (dir: the view ray), as a sample `width` metres
// across holds it, per unit of the water's extinction: the sun's share of the water's own light there
// (frame.wgsl `underwater_sun` of `underwater_body`'s source, the fog colour falling off with depth), times what
// the waves make of the sun at p (caustics.wgsl `water_focus`, the light field that draws the caustics on the bed),
// the sun's shadow there, and a Henyey-Greenstein phase about the sun's refracted direction over an even one (water
// scatters forwards: brighter looking up the light). Summed along a view ray that is the light shafts: parallel
// beams along the refracted sun, each under the patch of surface that focuses it, cut where something shadows
// them, converging on the sun's place in Snell's window only as perspective makes parallel lines converge.
// (Light scattered once in a medium lit through a refracting surface: Nishita and Nakamae, "Method of displaying
// optical effects within water using accumulation buffer", SIGGRAPH 1994; Iwasaki, Dobashi and Nishita, "An
// efficient method for rendering underwater optical effects using graphics hardware", CGF 2002.)
fn water_in(p: vec3<f32>, dir: vec3<f32>, width: f32) -> vec3<f32> {
  let l = normalize(frame.light_dir.xyz);
  let depth = frame.fog_height.x - p.y;
  if (depth <= 0.0 || l.y <= 0.0) { return vec3<f32>(0.0); }
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow_lookup(p, l, 1.0, false); }
  let up_light = -refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let phase = 4.0 * PI * hg_phase(dot(dir, up_light), vol.scatter.w);
  let light = underwater_sun() * exp(-frame.fog_height.y * depth) * phase * shadow * max(1.0 + water_focus(p, width), 0.0);
  return frame.fog_color.rgb * light;
}

// The light scattered towards the eye at p (frag: its pixel; `most`: as `locals_in`), a sample `width` metres
// across: the medium's.
fn medium_in(p: vec3<f32>, dir: vec3<f32>, frag: vec2<f32>, most: u32, width: f32) -> vec3<f32> {
  if (VOL_WATER) { return water_in(p, dir, width); }
  return sun_in(p, dir) + locals_in(p, dir, frag, most);
}

// The cluster's lights scattered towards the eye at p (frag: its pixel),
// the first `most` of them (the cluster keeps the brightest at the camera
// first): every one in the froxel grid, RAYMARCH_LIGHTS in the raymarch.
fn locals_in(p: vec3<f32>, dir: vec3<f32>, frag: vec2<f32>, most: u32) -> vec3<f32> {
  var sum = vec3<f32>(0.0);
  if (frame.params.z < 0.5) { return sum; }
  let word = cluster_word(cluster_at(frag, p));
  let first = word & 0xffffffu;
  let count = min(word >> 24u, most);
  for (var k = 0u; k < count; k++) {
    let light = light_at(cluster_ref(first + k));
    let to_light = light.position_range.xyz - p;
    let d2 = max(dot(to_light, to_light), 1e-4);
    let range = light.position_range.w;
    let ratio = d2 / (range * range);
    let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
    var atten = window * window / d2;
    let l = to_light * inverseSqrt(d2);
    let kind = light.color_kind.w;
    if (kind < -0.5) {
      // A rect light as a small panel: its area, facing p.
      let side = cross(light.direction_slot.xyz, light.cone.xyz);
      let area = 4.0 * length(side);
      var facing = dot(l, normalize(side));
      if (kind < -1.5) { facing = abs(facing); }
      atten *= area * max(facing, 0.0);
      if (atten > 0.0) {
        atten *= rect_shadow(light, p, l);
      }
    } else {
      if (kind > 0.5) {
        let t = clamp((dot(-l, light.direction_slot.xyz) - light.cone.x) * light.cone.y, 0.0, 1.0);
        atten *= t * t;
      }
      if (atten > 0.0) {
        atten *= local_shadow_lookup(light, p, l, false);
      }
    }
    if (atten <= 0.0) { continue; }
    sum += light.color_kind.rgb * atten * light_cookie(light, -l) * hg_phase(dot(dir, l), vol.scatter.w);
  }
  return sum;
}

struct VolOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_volumetric(@builtin(vertex_index) i: u32) -> VolOut {
  var o: VolOut;
  o.pos = fullscreen_clip(i);
  return o;
}

@group(1) @binding(4) var scene_depth: texture_2d<f32>;
fn pixel_depth(frag: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(scene_depth));
  let at = clamp(vec2<i32>(frag * vec2<f32>(size) / vol.extent.zw), vec2<i32>(0), size - 1);
  return textureLoad(scene_depth, at, 0).r;
}

