// Scattered copies over a terrain (scatter.rs): instances cached per chunk,
// each a world transform, shrunk and thinned out with the distance from the
// camera (scatter_scale; no dither), crossfaded to their impostor by the
// one dithered transition (common.wgsl dither_cut; scatter_hidden),
// swayed by the wind.
// Shared by the lit, prepass and shadow passes, so their coverage agrees.

struct ScatterUniform {
  camera: vec4<f32>,          // xyz: the camera, w: the distance copies reach (m)
  fade: vec4<f32>,            // x: where the fade starts (m), y: unused, zw: a crowd card's views and frames (crowd_card_common.wgsl)
  view_proj: mat4x4<f32>,     // unjittered (prepass velocity)
  prev_view_proj: mat4x4<f32>,
  wind: vec4<f32>,            // xy: og.Wind's horizontal direction (x, z), z: the scatter's sway (0: still), w: the trample's recovery (s)
  wind_shape: vec4<f32>,      // x: unused, y: the mesh's top above its origin (m), zw: the frame's origin's x and z (origin.rs)
  impostor: vec4<f32>,        // the impostor's sphere in the model's space (impostor.rs): xyz its centre, w its radius
  impostor_grid: vec4<f32>,   // x: frames per side of its atlas, y: the distance past which copies draw as impostors (0: never), z: 1 with alpha to coverage, w: 1 when its material's roughness map is the material atlas (impostor.wgsl imp_material)
  impostor_light: vec4<f32>,  // xyz: towards the sun (the shadow cascades' cards face it), w: an impostor's 1 when its atlases hold the lower hemisphere's frames (impostor.rs Impostor::below; a crowd card's elevation)
  impostor_trim: array<vec4<f32>, 2>, // the impostor card's outline (impostor.rs CardTrim): its strips' boundary half-widths, bottom to top ([0].xyzw, [1].x), and its bottom and top ([1].yz)
  trample: vec4<f32>,         // the trample map (trample.rs): min corner (x, z from the frame's origin), cell, cells a side (0: none)
  canopy: vec4<f32>,          // xyz: the mesh's canopy centre in its own space, w: 1 when it has foliage (meshlet.rs MeshFoliage)
};
@group(2) @binding(0) var<uniform> scatter: ScatterUniform;
// The frame's wind (wind.rs WindField) and the simulated field's lean
// every vegetation tier reads (wind_field.rs; wind_common.wgsl wind_lean).
@group(2) @binding(2) var<uniform> scatter_wind: WindField;
@group(2) @binding(3) var scatter_wind_tex: texture_2d<f32>;
// The trample map (trample.rs; wind_lean.wgsl vegetation_trample).
@group(2) @binding(4) var scatter_trample_tex: texture_2d<f32>;

struct ScatterVertex {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};

// A copy's transform: its columns' xyz (instance.wgsl).
struct ScatterInstance {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
  // Its look (pbr.wgsl InstanceFxIn): zero for a copy; a model impostor's
  // carries how much of it is hidden (instance.wgsl look_hidden).
  @location(8) look: vec4<f32>,
};

// How much of a copy standing at `at` shows: 1 inside the fade, 0 past the distance.
fn scatter_fade(at: vec3<f32>) -> f32 {
  let d = length(at.xz - scatter.camera.xz);
  if (d > scatter.camera.w) { return 0.0; }
  if (scatter.camera.w <= scatter.fade.x) { return 1.0; }
  return 1.0 - smoothstep(scatter.fade.x, scatter.camera.w, d);
}

// How much of a copy standing at `at` draws as its mesh: 1 nearer than
// the last tenth before the impostor distance, 0 past it (0 distance:
// always 1). The impostor draws the rest (impostor.wgsl).
fn scatter_near(at: vec3<f32>) -> f32 {
  let far = scatter.impostor_grid.y;
  if (far <= 0.0) { return 1.0; }
  return 1.0 - smoothstep(0.9 * far, far, length(at.xz - scatter.camera.xz));
}

// How much of a copy's mesh the dither cuts (common.wgsl dither_cut)
// as it crosses to its impostor: it fades out against the impostor by the
// share it shows (`near`, scatter_near), so each pixel is drawn once, by
// the mesh or by the card (impostor_common.wgsl imp_hidden).
fn scatter_hidden(near: f32) -> f32 {
  return near - 1.0;
}

// A clip position outside the view: the copy's triangles are culled.
const SCATTER_GONE = vec4<f32>(2.0, 2.0, 2.0, 1.0);

// A hash in [0, 1) of the copy standing at `base` (its place in the world,
// the same wherever the frame's origin is: scatter.rs places the copies in
// the world first).
fn scatter_copy_unit(base: vec3<f32>) -> f32 {
  let at = base.xz + scatter.wind_shape.zw;
  return hash_to_unit(hash_lowbias(bitcast<u32>(at.x) ^ hash_lowbias(bitcast<u32>(at.y))));
}

// How large the copy standing at `at` draws over the distance fade, 0..1:
// each copy shrinks to nothing over its own half of the fade, its place's
// hash choosing which, so the field thins out evenly while every copy
// shrinks smoothly (grass.wgsl's blades thin and shrink the same way). No
// pixel is dithered, so TAA sees no grain.
fn scatter_scale(at: vec3<f32>) -> f32 {
  let f = scatter_fade(at);
  if (f >= 1.0) { return 1.0; }
  return clamp(2.0 * f - scatter_copy_unit(at), 0.0, 1.0);
}

// `model` scaled by `s` about its origin (the copy's root).
fn scatter_shrunk(model: mat4x4<f32>, s: f32) -> mat4x4<f32> {
  return mat4x4<f32>(model[0] * s, model[1] * s, model[2] * s, model[3]);
}

// How the wind moves a vertex at `local` (mesh space) of the copy standing
// at `base` with uniform scale `scale`, now or as last drawn (`previous`):
// by the lean of the wind field at the copy's root (wind_common.wgsl
// wind_lean: the field every tier shares, or og.Wind's analytic lean
// without it) times the scatter's sway times its height times its height
// over the mesh's top, lowered to keep its distance from the root. Each
// copy reads the field a little upwind or downwind of its root (by its
// place's hash, up to a wavelength's sixth), so neighbours do not sway in
// step. Trampled copies (trample.rs) part away from what pressed them.
fn scatter_sway(local: vec3<f32>, base: vec3<f32>, scale: f32, previous: bool) -> vec3<f32> {
  let sway = scatter.wind.z;
  let top = scatter.wind_shape.y;
  if (sway <= 0.0 || top <= 0.0 || local.y <= 0.0) {
    return vec3<f32>(0.0);
  }
  let jitter = scatter_copy_unit(base) * WIND_WAVELENGTH / (2.0 * PI);
  let lean4 = wind_lean(scatter_wind_tex, scatter_wind, base.xz - scatter.wind.xy * jitter);
  // Trampled (G3): parted away from the pusher, as the grass is.
  let tall = top * scale;
  let tr = vegetation_trample(scatter_trample_tex, scatter.trample, base, tall, scatter.wind.w);
  let lean = select(lean4.xy, lean4.zw, previous) * sway * (1.0 - clamp(tr.z, 0.0, 1.0)) + tr.xy * 1.2;
  let h = local.y * scale;
  let d = lean * h * min(local.y / top, 1.0);
  let dl = min(length(d), h);
  let drop = h - sqrt(max(h * h - dl * dl, 0.0));
  return vec3<f32>(d.x, -drop, d.y);
}
