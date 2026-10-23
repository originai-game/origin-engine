// Volumetric fog (volumetric.rs), after frame.wgsl, the light tables,
// lighting.wgsl and area.wgsl. The medium: `density` per metre plus height
// fog (`height density` below `height`, thinning above it by `falloff`),
// scattering `colour × scattering` of the light it is lit by with a
// Henyey-Greenstein phase of `anisotropy`.
//
// Full tier: `inject` lights every froxel (160 × 90 × 64 cells over the
// view, slices quadratically deeper to `distance`) by the sun through its
// cascades and the cluster's point, spot and rect lights through their
// shadows and cookies; `integrate` sums them front to back into the
// in-scattered light and the transmittance at each slice's far edge;
// `fs_froxel` reads that at each pixel's depth. WebGL2: `fs_raymarch`
// marches each pixel's view ray in steps, lit by the sun and the first
// RAYMARCH_LIGHTS lights of each step's cluster. Both
// output (in-scattered light, transmittance), blended over the lit image
// as colour × transmittance + light.

struct Vol {
  scatter: vec4<f32>,  // rgb: colour × scattering, w: anisotropy
  medium: vec4<f32>,   // x: density, y: height, z: height density, w: falloff
  extent: vec4<f32>,   // x: distance, y: raymarch steps, zw: the target's size in pixels
  grid: vec4<f32>,     // xyz: froxels across, down and deep, w: the frame's noise seed
};

@group(1) @binding(0) var<uniform> vol: Vol;

fn vol_density(p: vec3<f32>) -> f32 {
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
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow(p, l, 1.0); }
  return frame.light_color.rgb * shadow * hg_phase(dot(dir, l), vol.scatter.w);
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
    let light = light_list[cluster_ref(first + k)];
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
        atten *= local_shadow(light, p, l);
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
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  return o;
}

@group(1) @binding(4) var scene_depth: texture_2d<f32>;
fn pixel_depth(frag: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(scene_depth));
  let at = clamp(vec2<i32>(frag * vec2<f32>(size) / vol.extent.zw), vec2<i32>(0), size - 1);
  return textureLoad(scene_depth, at, 0).r;
}

