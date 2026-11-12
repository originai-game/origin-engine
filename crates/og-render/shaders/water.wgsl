// Planar water (the E5 round): a grid over the surface's local xz plane,
// displaced by an optional height field (og-physics' wave simulation),
// with animated normals from a sum of travelling waves, colour by depth
// (shallow to deep over depthFade, from the depth prepass), foam where the
// water meets geometry, refraction of a copy of the scene drawn so far,
// and reflection: screen-space (a march through the depth prepass), the
// sky's prefiltered cubemap where it misses, or the planar reflection
// pass's mirrored scene. Phase 2: the
// directional light's shadows on the surface, and the view from below
// (what is above inside Snell's window, the water's colour reflected
// outside it). Drawn after the opaque scene and the sky, before
// transparent meshes, writing depth. Prepended by frame.wgsl and
// lighting.wgsl.
//
// R5: a sea (Water::wind) is a clipmap about the camera (water.rs
// `clipmap`), out to a horizon ring, displaced over the absolute world's xz
// by every wave the tick floats bodies on (og_core::ocean, as Gerstner
// waves) and the FFT's cascades, the waves shorter than the shortest of them
// (ocean_fft), every one faded where the cells or the pixel cannot hold it;
// its normal and its facets' slope covariance from the cascades' slopes and
// their moments (LEAN). Shaded and foamed as web-ocean's sea (MIT,
// third_party/ocean/): anisotropic glitter, the grazing tilt of the visible
// facets, backlit crests' subsurface light, and foam that persists on the
// tick's time (ocean_fft/foam.rs). A flow map (Water::flow_map) turns the
// flow per point.

struct WaterUniform {
  model: mat4x4<f32>,
  size: vec4<f32>,        // xy: extent along local x and z, z: segments, w: 1 with a height field, 2 with its ripples
  shallow: vec4<f32>,     // rgb, w: depth fade (m)
  deep: vec4<f32>,        // rgb, w: opacity
  foam: vec4<f32>,        // rgb, w: foam width (m)
  waves: vec4<f32>,       // x: scale (waves per metre), y: speed, z: strength, w: refraction (px)
  look: vec4<f32>,        // x: reflectivity, y: 1 for screen-space reflections, z: height field size (m), w: resolution
  screen: vec4<f32>,      // w, h, 1/w, 1/h
  extra: vec4<f32>,       // x: shadows, y: 1 with the planar reflection, z: F0, w: reflection bias
  under: vec4<f32>,       // rgb: the underwater colour, w: three.js Water's glint (0: the half-vector highlight)
  normals: vec4<f32>,     // x: the normal map's repeats, yz: its flow (0: the lake's layers), w: 1 with a normal map
  ao: vec4<f32>,          // x: PostEffects.aoDirect, how much the screen AO darkens the reflection and the body (0: not at all), y: 1 with a flow map, z: 1 over a mesh
  layers: array<vec4<f32>, 2>,  // the lake's four layers' uv at the frame's origin, whole repeats dropped (xy, zw; origin.rs)
  ocean: vec4<f32>,       // x: 1 with a sea, y: unused, z: the Jacobian under which a crest foams as it folds, w: how fast it grows
  sea_wind: vec4<f32>,    // xy: the wind's direction in the frame's xz, z: its speed (m/s; 0 without a sea), w: foamStreaks
  clip: vec4<f32>,        // the sea's clipmap: x levels, y cells along a level's edge, z the horizon ring's radius, w 1 endless
  levels: array<vec4<f32>, 10>,  // per level: its centre (frame xz), its cell (m)
  inverse: mat4x4<f32>,   // the surface's frame from the world
  fft: vec4<f32>,         // x cascades, y unused, z the slope variance under the finest cascade, w Hs (m)
  cascades: array<vec4<f32>, 3>,  // per cascade: 1 / tile (1/m), its uv at the frame's origin, the tile (m)
  bands: vec4<f32>,       // each cascade's band's middle wavenumber (xyz: what the seabed's shoaling keeps of it)
  // The persistent foam's two levels (ocean_fft/foam.rs), in the fields' layers after the cascades: per level 1 / its
  // edge (1/m), its uv at the frame's origin, its edge (m).
  foam_levels: array<vec4<f32>, 2>,
  // The foam micro-structure's uv at the frame's origin (whole repeats dropped): its lace (xy), finer lace (zw); its
  // bubbles (xy), its streaks along and across the wind (zw).
  foam_uv: array<vec4<f32>, 2>,
  // Per wave of og_core::ocean: (k d.x, k d.z, amplitude, phase at the frame's origin), (lean d.x, lean d.z, k lean, 0)
  // (water.rs ocean_gpu).
  swell: array<vec4<f32>, 64>,
  // W1: the seabed (water.rs shore_gpu): [0] its sample (0, 0) in the frame's xz, its cell (m), 1 with one; [1] its
  // columns and rows, the shore trains' wavenumber and amplitude; [2] their angular frequency and phase now, the surf's
  // foam.
  shore: array<vec4<f32>, 3>,
  // W4: the sea's spray (water.rs spray_gpu): x how much (0: none), y its clock (the sea's seconds folded into
  // SPRAY_FOLD), z the droplets of each population drawn, w the water's own clock folded so (every surface).
  spray: vec4<f32>,
  // W2: the water's absorption and scattering per metre (rgb; w 1 for the physical body, water_optics).
  optics: array<vec4<f32>, 2>,
  // W6: x 1 when the near plane crosses the surface (water.rs waterline), y the underwater fog's density, z the
  // meniscus's width (px).
  waterline: vec4<f32>,
  // W3: the hulls' wakes (sea_wakes.rs): xy the grid's centre (frame xz), z one over its side (1/m), w 1 with one.
  wake: vec4<f32>,
};

@group(1) @binding(0) var<uniform> water: WaterUniform;

// Which kinds of surface a pipeline draws (pipeline_features.rs): a sea
// (og_core's waves, its foam and spray), a flowing surface (a current, a
// flow map or a river) and the ripples' height field. A kind's code sits
// inside `if (OG_<kind>)`, so a pipeline made without it has that code
// folded away before the backend inlines it.
override OG_SEA: bool = true;
override OG_FLOW: bool = true;
override OG_RIPPLES: bool = true;

fn is_sea() -> bool {
  return OG_SEA && water.ocean.x > 0.5;
}

fn flows() -> bool {
  return OG_FLOW && dot(water.normals.yz, water.normals.yz) >= 1e-12;
}

// The ripples' heights, read per pixel (WaterGpu.size.w 2).
fn rippled() -> bool {
  return OG_RIPPLES && water.size.w > 1.5;
}

// The light a sea's crests carry through them (rgb) and its strength (w): web-ocean's `sssCol`.
const SEA_SSS: vec4<f32> = vec4<f32>(0.035, 0.12, 0.085, 2.6);
// The sun's disc as a variance of slopes (its 0.27° radius, halved at reflection, squared): the glitter's least
// spread, so a calm sea mirrors a disc, not a point.
const SUN_DISC: f32 = 5.5e-6;
// A height field (layer 0, the height in r), or a sea's FFT fields: per cascade c, layer 3c (x displacement,
// height, z displacement, d Dx/dz), 3c + 1 (the height's slopes along x and z, d Dx/dx, d Dz/dz), 3c + 2 (the slopes'
// squares and product), mipped; after the cascades the persistent foam's two levels (r its cover, g how fresh it is)
// and its micro-structure (r lace, g finer lace, b soft density, a streaks).
@group(1) @binding(1) var heights: texture_2d_array<f32>;
@group(1) @binding(2) var scene_color: texture_2d<f32>;
@group(1) @binding(3) var scene_depth: texture_2d<f32>;
@group(1) @binding(4) var water_sampler: sampler;
// The planar reflection pass's frame (the mirrored camera's view, flipped
// left to right), when this surface is the one drawn with it.
@group(1) @binding(5) var reflection_tex: texture_2d<f32>;
// The ripples' tangent-space normal map (Water::normal_map).
@group(1) @binding(6) var normal_tex: texture_2d<f32>;
@group(1) @binding(7) var ripple_sampler: sampler;
// The frame's screen-space AO (white without one).
@group(1) @binding(8) var ao_tex: texture_2d<f32>;
// The flow map (Water::flow_map): rg the flow's direction * 0.5 + 0.5.
@group(1) @binding(9) var flow_tex: texture_2d<f32>;
// The seabed (Water::shore, og_core::shore): per sample its depth under the rest level, the signed distance to the
// waterline and that distance's gradient (xz).
@group(1) @binding(10) var shore_tex: texture_2d<f32>;
// W3: the hulls' wakes over a sea (sea_wakes.wgsl): (height, its slopes along x and z, foam), a camera-centred square.
@group(1) @binding(11) var wake_tex: texture_2d<f32>;

struct WaterOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) local: vec2<f32>,
  @location(3) slope: f32,
  // Where the normal map is read: the mesh's uv, or the grid's metres.
  @location(4) uv: vec2<f32>,
  // The sea's rest point (frame xz) the displaced vertex came from.
  @location(5) rest: vec2<f32>,
};

// W2: the water's optics along a view ray from the surface into it (`v` towards the eye, `l` the sun's direction above
// the surface), `thickness` its view depth to what lies below (0 to the far plane: the open sea), `forward` the view's.
struct Optics {
  // What reaches the eye of the light from below (rgb), and how opaque the body is (its mean, for the extras).
  through: vec3<f32>,
  opacity: f32,
  // The sun's light scattered to the eye per unit of its irradiance and phase, and the sky's per unit of its ambient
  // level (as an albedo's: its light from every side, scattered evenly, its path down about the depth).
  scatter: vec3<f32>,
  scatter_sky: vec3<f32>,
};

fn water_optics(v: vec3<f32>, l: vec3<f32>, thickness: f32, forward: vec3<f32>) -> Optics {
  let a = water.optics[0].rgb;
  let b = water.optics[1].rgb;
  let ext = max(a + b, vec3<f32>(1e-5));
  // Along the view ray, refracted (Snell, 1.33) under the surface; the sun's path slants by its own refracted angle.
  let down = refract(-v, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let path = min(thickness / max(dot(-v, forward), 0.05), 1e4);
  let vy = max(-down.y, 0.05);
  let l_in = refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let ly = max(-l_in.y, 0.05);
  // The depth the ray reaches, and the light's path down to it.
  let depth = path * vy;
  var o: Optics;
  o.through = exp(-ext * (path + depth / ly));
  o.opacity = 1.0 - dot(exp(-ext * path), vec3<f32>(0.3333));
  // ∫ b e^{-ext s} e^{-ext s vy / ly} ds over the path: the light at each point came down a slant of s vy / ly.
  let k_sun = 1.0 + vy / ly;
  o.scatter = b * (1.0 - exp(-ext * path * k_sun)) / (ext * k_sun);
  let k_sky = 1.0 + vy;
  o.scatter_sky = b * (1.0 - exp(-ext * path * k_sky)) / (ext * k_sky);
  return o;
}

// Sea water's phase function for the sun's light: a Henyey–Greenstein lobe of g 0.9 (particles scatter forward;
// Mobley's Light and Water, ch. 3), mixed with a fifth of an even share for the water's own molecular scattering.
fn hg_water(cos_t: f32) -> f32 {
  let g = 0.9;
  let hg = (1.0 - g * g) / (12.5663706 * pow(1.0 + g * g - 2.0 * g * cos_t, 1.5));
  return mix(hg, 0.0795775, 0.2);
}

// The height field's texels at local point (x, z) of the surface, bilinear: r the height; with ripples
// (water_ripples.wgsl) g the ripples alone and b their foam.
fn field_at(local: vec2<f32>) -> vec4<f32> {
  if (water.size.w < 0.5) { return vec4<f32>(0.0); }
  let n = i32(water.look.w);
  let cell = water.look.z / f32(n - 1);
  let g = (local + vec2<f32>(water.look.z * 0.5)) / cell;
  let i = clamp(vec2<i32>(floor(g)), vec2<i32>(0), vec2<i32>(n - 2));
  let f = clamp(g - vec2<f32>(i), vec2<f32>(0.0), vec2<f32>(1.0));
  let a = textureLoad(heights, i, 0, 0);
  let b = textureLoad(heights, i + vec2<i32>(1, 0), 0, 0);
  let c = textureLoad(heights, i + vec2<i32>(0, 1), 0, 0);
  let d = textureLoad(heights, i + vec2<i32>(1, 1), 0, 0);
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// The height field at local point (x, z) of the surface, bilinear.
fn height_at(local: vec2<f32>) -> f32 {
  return field_at(local).r;
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> WaterOut {
  let segs = u32(water.size.z);
  let quad = vi / 6u;
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0));
  if (OG_SEA) {
    if (water.fft.x > 0.5) { return vs_sea(vi); }
  }
  let c = corners[vi % 6u];
  let cell = vec2<f32>(f32(quad % segs), f32(quad / segs)) + c;
  let local = (cell / f32(segs) - 0.5) * water.size.xy;
  let h = height_at(local);
  // The height field's slope gives the geometric normal.
  let e = water.look.z / max(water.look.w - 1.0, 1.0);
  let hx = height_at(local + vec2<f32>(e, 0.0)) - height_at(local - vec2<f32>(e, 0.0));
  let hz = height_at(local + vec2<f32>(0.0, e)) - height_at(local - vec2<f32>(0.0, e));
  let n_local = normalize(vec3<f32>(-hx / (2.0 * e), 1.0, -hz / (2.0 * e)));
  let world = water.model * vec4<f32>(local.x, h, local.y, 1.0);
  let rest = world.xz;
  var o: WaterOut;
  o.clip = frame.view_proj * world;
  o.world = world.xyz;
  o.rest = rest;
  o.normal = normalize((water.model * vec4<f32>(n_local, 0.0)).xyz);
  o.local = local;
  o.slope = length(vec2<f32>(hx, hz)) / (2.0 * e);
  o.uv = local;
  return o;
}

// A sea's vertex (sea_vertex): its rest point on the surface's level, displaced; the geometric normal is the level's.
fn vs_sea(vi: u32) -> WaterOut {
  let v = sea_vertex(vi);
  var o: WaterOut;
  o.normal = normalize((water.model * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);
  o.slope = 0.0;
  if (v.skip) {
    // A cell under the level inside: all six corners at one point outside the view, drawing nothing.
    o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    o.world = vec3<f32>(0.0);
    o.rest = vec2<f32>(0.0);
    o.local = vec2<f32>(0.0);
    o.uv = vec2<f32>(0.0);
    return o;
  }
  let level = water.model[3].y;
  let world = vec3<f32>(v.rest.x, level, v.rest.y) + v.d;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.world = world;
  o.rest = v.rest;
  o.local = (water.inverse * vec4<f32>(v.rest.x, level, v.rest.y, 1.0)).xz;
  o.uv = o.local;
  return o;
}

// A surface drawn over a mesh (Water::mesh): its vertices and normals in the
// surface's frame; the waves run over its local xz.
@vertex
fn vs_mesh(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>) -> WaterOut {
  let world = water.model * vec4<f32>(position, 1.0);
  var o: WaterOut;
  o.clip = frame.view_proj * world;
  o.world = world.xyz;
  o.normal = normalize((water.model * vec4<f32>(normal, 0.0)).xyz);
  o.local = position.xz;
  o.slope = 0.0;
  o.uv = uv;
  o.rest = world.xz;
  return o;
}

fn view_depth(p: vec3<f32>) -> f32 {
  return dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
}

fn screen_uv(p: vec3<f32>) -> vec3<f32> {
  let c = frame.view_proj * vec4<f32>(p, 1.0);
  let ndc = c.xyz / max(c.w, 1e-6);
  return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, c.w);
}

// A screen-space reflection: march the reflected ray through the depth
// prepass; the colour where it passes behind the scene (w: 1), else w 0.
fn ssr(origin: vec3<f32>, r: vec3<f32>) -> vec4<f32> {
  let steps = 28;
  var t = 0.2;
  var last = 0.0;
  for (var i = 0; i < steps; i++) {
    let p = origin + r * t;
    let s = screen_uv(p);
    if (s.z <= 0.0 || any(s.xy < vec2<f32>(0.0)) || any(s.xy > vec2<f32>(1.0))) { break; }
    let px = vec2<i32>(s.xy * water.screen.xy);
    let scene = textureLoad(scene_depth, px, 0).r;
    let d = view_depth(p);
    if (d > scene && d - scene < 0.6 + t * 0.15) {
      // Refine between the last two samples.
      var lo = last;
      var hi = t;
      for (var j = 0; j < 4; j++) {
        let mid = 0.5 * (lo + hi);
        let q = origin + r * mid;
        let sq = screen_uv(q);
        let sd = textureLoad(scene_depth, vec2<i32>(sq.xy * water.screen.xy), 0).r;
        if (view_depth(q) > sd) { hi = mid; } else { lo = mid; }
      }
      let hit = screen_uv(origin + r * hi);
      // On a sea, a hit is where the refined ray meets what the screen holds: a ray that only passed behind a
      // floater's silhouette (the march's thickness is generous) mirrored its dark side into the near water.
      let held = textureLoad(scene_depth, vec2<i32>(hit.xy * water.screen.xy), 0).r;
      if (is_sea() && view_depth(origin + r * hi) - held > 0.25 + 0.02 * hi) {
        last = t;
        t = t * 1.25 + 0.15;
        continue;
      }
      let edge = min(min(hit.x, 1.0 - hit.x), min(hit.y, 1.0 - hit.y));
      let fade = clamp(edge * 8.0, 0.0, 1.0) * (1.0 - f32(i) / f32(steps));
      // The scene colour there was already darkened by its own screen AO (the whole colour × mix(0.35, 1, ao) by
      // aoDirect, pbr_fs.wgsl): taken back out, so that `fs` darkens the reflection once, by the AO at the water (as
      // a mirror drawn without AO and darkened as a whole by three.js-era post AO). aoDirect 0 or no AO: no change.
      let k_hit = mix(1.0, mix(0.35, 1.0, textureSampleLevel(ao_tex, water_sampler, hit.xy, 0.0).r), water.ao.x);
      return vec4<f32>(textureSampleLevel(scene_color, water_sampler, hit.xy, 0.0).rgb / k_hit, fade);
    }
    last = t;
    t = t * 1.25 + 0.15;
  }
  return vec4<f32>(0.0);
}

// A texel of the depth below, clamped to it (a target smaller than the
// screen the uniform names reads its edge).
fn depth_texel(p: vec2<f32>) -> vec2<i32> {
  return min(vec2<i32>(p), vec2<i32>(textureDimensions(scene_depth)) - vec2<i32>(1));
}

// `r` moved out of the sun's disc in the environment to its edge (the sky just beside the sun, continuous with the sky
// outside the disc): the sun at the light's direction, its disc the frame's sky look's (its outer edge) or the
// built-in sky's (ibl.wgsl fs_sky: 1.8°, its soft edge out to 2.16°), widened by the blur of the mip read: a texel of
// the 128-texel faces (0.7°) and the GGX lobe that mip was filtered with (its roughness the mip over the last, some
// two alphas wide).
fn off_sun(r: vec3<f32>, mip: f32) -> vec3<f32> {
  let l = normalize(frame.light_dir.xyz);
  let outer = select(0.99929, frame.sky_sun.x, frame.sky_zenith.w > 0.5 && frame.sky_sun.z > 0.0);
  let rough = clamp(mip / max(frame.params.y, 1.0), 0.0, 1.0);
  let edge = min(acos(clamp(outer, -1.0, 1.0)) + radians(0.7) + 2.0 * rough * rough, 1.0);
  let c = dot(r, l);
  if (c <= cos(edge)) { return r; }
  let side = r - l * c;
  let len = length(side);
  // (Straight at the sun: any side.)
  let t = select(normalize(cross(l, select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(l.y) > 0.9))), side / len, len > 1e-5);
  return l * cos(edge) + t * sin(edge);
}

// The sky mirrored along `r` (`c`, from the reflection maps, which hold no clouds) under the frame's cloud cover:
// greyed as the clouds' own ambient is (sky.wgsl clouds_over), and under an overcast the deck itself, brighter
// overhead than at the horizon (a clear horizon mirrored at the sea's grazing rim drew a white line under a storm).
fn overcast(c: vec3<f32>, r: vec3<f32>) -> vec3<f32> {
  if (frame.clouds[3].w < 0.5) { return c; }
  let grey = mix(c, vec3<f32>(dot(c, vec3<f32>(0.2126, 0.7152, 0.0722))), frame.clouds[0].x * 0.7);
  let dk = cloud_deck(r);
  return mix(grey, dk.rgb * (1.0 + 0.6 * max(r.y, 0.0)), dk.a);
}

@fragment
fn fs(in: WaterOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5 && (frame.light_color.w < 1.5 || frame.light_color.w > 2.5)) {
    return debug_unsupported(in.clip.xy);
  }
  let t = frame.time.x;
  // (The map's gradients are taken in uniform control flow: the branch is on a uniform.)
  // The pixel's footprint on the surface (m) the detail waves are filtered over; a sea fades its own below.
  let dlx = dpdxFine(in.local);
  let dly = dpdyFine(in.local);
  let reach = select(max(length(dlx), length(dly)), 0.0, is_sea());
  // Its area (m², the glints' cells: lake_glints).
  let foot_area = abs(dlx.x * dly.y - dlx.y * dly.x);
  var detail = wave_slope(in.local, t, reach);
  if (water.normals.w > 0.5) {
    detail = map_slope(in.world.xz, in.uv, t);
  } else if (OG_FLOW) {
    if (flows()) { detail = flowed_slope(in.local, in.uv, t, reach); }
  }
  var slope = detail.slope;
  let tangent_x = normalize((water.model * vec4<f32>(1.0, 0.0, 0.0, 0.0)).xyz);
  let tangent_z = normalize((water.model * vec4<f32>(0.0, 0.0, 1.0, 0.0)).xyz);
  // The sea's own normal under the detail (a level surface: its waves are the world's), and its foam.
  var base = in.normal;
  var sea_cov = vec3<f32>(0.0);
  var sea_f = SeaFoam(0.0, 0.0, 0.0, 0.0, 0.0);
  if (OG_SEA) {
if (is_sea()) {
    var footprint = sea_footprint(in.world);
    // The pixel's footprint on the level surface, from where its view ray meets the level: smooth across the screen,
    // where the rest point's derivatives jump at every edge of the displaced triangles (and their mips drew the grid).
    // (A ray that never meets the level, a crest above the eye, takes the point itself.)
    let to = in.world - frame.view_pos.xyz;
    let along = (water.model[3].y - frame.view_pos.y) / min(to.y, -1e-6);
    let flat_hit = frame.view_pos.xz + to.xz * select(1.0, along, to.y < -0.01 * length(to) && along > 0.0);
    var gx = dpdxFine(flat_hit);
    var gy = dpdyFine(flat_hit);
    if (frame.view_pos.w > 0.5) {
      gx = dpdxFine(in.rest);
      gy = dpdyFine(in.rest);
      footprint = max(length(gx), length(gy));
    }
    let sea = ocean_normal(in.rest, in.world.xz, gx, gy);
    base = sea.n;
    // web-ocean: the slopes' change across the pixel widens the lobe (specular anti-aliasing).
    let dsx = dpdxFine(sea.slope);
    let dsy = dpdyFine(sea.slope);
    let aa = vec3<f32>(dsx.x * dsx.x + dsy.x * dsy.x, dsx.y * dsx.y + dsy.y * dsy.y, dsx.x * dsx.y + dsy.x * dsy.y) * 0.5;
    sea_cov = sea.cov + aa * min(1.0, 0.03 / max(max(aa.x, aa.y), 1e-8));
    sea_f = sea_foam(in.rest, sea.jac);
    // The detail ripples are the sea's own shortest waves: kept only where a pixel holds them (else they alias into
    // stripes towards the horizon).
    slope *= 1.0 - smoothstep(0.785, 1.571, water.waves.x * 6.2831853 * 3.9 * footprint);
  }
  }
  // With ripples (water_ripples.wgsl): their own slope per pixel bends the grid's normal (finer than its vertices;
  // zero on open water, so its glare stays the grid's), and their foam.
  var ripple_foam = 0.0;
  if (OG_RIPPLES) {
if (rippled() && !is_sea()) {
    let e = water.look.z / max(water.look.w - 1.0, 1.0);
    let rx = field_at(in.local + vec2<f32>(e, 0.0)).g - field_at(in.local - vec2<f32>(e, 0.0)).g;
    let rz = field_at(in.local + vec2<f32>(0.0, e)).g - field_at(in.local - vec2<f32>(0.0, e)).g;
    base = normalize(base - tangent_x * (rx / (2.0 * e)) - tangent_z * (rz / (2.0 * e)));
    ripple_foam = field_at(in.local).b;
  }
  }
  var n = normalize(base - tangent_x * slope.x - tangent_z * slope.y);
  let v = select(normalize(frame.view_pos.xyz - in.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  // Seen from below: the camera is under the surface.
  let below_view = dot(in.normal, v) < 0.0;
  // three.js Water / Water2 (below) never turn a ripple's normal to the eye: a wavelet facing away at a grazing view
  // reflects fully (theta = max(dot(V, N), 0), Fresnel 1), its body scatters nothing, the mirror bends by its own N.xz.
  let three_water = water.under.w > 0.0 || (water.normals.w > 0.5 && flows());
  if (is_sea()) {
    // A sea's swell turns its back slopes from the eye past a crest: a flipped normal mirrored the bright sky there
    // at Fresnel 1, a bright line along every crest's silhouette. Bend it to just face the eye instead (its
    // reflection then points down, into the sea: see below).
    n = normalize(n + v * max(0.05 - dot(n, v), 0.0));
  } else if (dot(n, v) < 0.0 && !three_water) { n = -n; }
  // The lake glitter's mean slope and its change across the pixel (below), taken here: every derivative comes before
  // the first return on a per-pixel value (`below_view`), as WGSL's uniformity rules require (Tint refuses the whole
  // module otherwise, and the frame drew black in the browser).
  let ms = n.xz / max(n.y, 1e-3);
  let dsx = dpdxFine(ms);
  let dsy = dpdyFine(ms);
  let l = normalize(frame.light_dir.xyz);
  var sun = 1.0;
  if (water.extra.x > 0.0 && frame.light_dir.w > 0.5) {
    let up = select(in.normal, -in.normal, dot(in.normal, l) < 0.0);
    sun = mix(1.0, cascade_shadow(in.world, up, max(dot(up, l), 0.0)), water.extra.x);
  }

  let px = in.clip.xy;
  let uv = px * water.screen.zw;
  let here = view_depth(in.world);
  let scene_here = textureLoad(scene_depth, depth_texel(px), 0).r;
  // Refraction: the scene behind, shifted by the detail normal, unless the
  // shifted sample is in front of the water.
  // three.js Water2 (a normal map with a flow): its look, term for term (see below). three.js Water (glint > 0): its body.
  let water2 = water.normals.w > 0.5 && flows();
  let water_js = water.under.w > 0.0;
  var shift = (n - in.normal).xz * water.waves.w * water.screen.zw * 40.0 / max(here * 0.2, 1.0);
  if (water2) {
    // Water2: uv + coord.z · N.xz · 0.05 (coord.z, the depth in [0, 1], is about 1), with screen v down here.
    let d = (n - in.normal).xz;
    shift = vec2<f32>(d.x, -d.y) * 0.05 * water.waves.w;
  }
  var ruv = clamp(uv + shift, vec2<f32>(0.0), vec2<f32>(1.0));
  var scene_at = textureLoad(scene_depth, depth_texel(ruv * water.screen.xy), 0).r;
  if (scene_at < here) {
    ruv = uv;
    scene_at = scene_here;
  }
  let below = textureSampleLevel(scene_color, water_sampler, ruv, 0.0).rgb;
  if (below_view) {
    // From below: what is above shows inside Snell's window (the view
    // steeper than the critical angle, about 49 degrees from the normal),
    // outside it the surface reflects the water's own colour.
    // The share that comes through is Fresnel's at the angle the ray leaves into the air (Snell, 1.33), none past the
    // critical angle: there the surface mirrors the water below it, the body's own light along the mirrored ray (dark
    // looking down, so the window stands bright in a dark surface the waves break up).
    let ci = abs(dot(n, v));
    let st2 = 1.7689 * (1.0 - ci * ci);
    let ct = sqrt(max(1.0 - st2, 0.0));
    let window = (0.98 - 0.98 * pow(1.0 - ct, 5.0)) * step(st2, 1.0);
    var mirror = water.under.rgb;
    if (frame.fog_color.w > 1.5) { mirror = underwater_body(in.world, reflect(-v, n), 1e4, 1.0, 0.0); }
    let colour = mix(mirror, below * water.shallow.rgb, window);
    return vec4<f32>(apply_fog(colour, in.world), 1.0);
  }
  let thickness = max(scene_at - here, 0.0);
  let absorb = 1.0 - exp(-thickness / max(water.shallow.w, 1e-3));
  // The body scatters the light that reaches it: its deep colour is authored for daylight and darkens under less, at dusk, at night or under a storm's deck
  // (else the sea glowed a daylight teal under a dark sky). Shadowed water scatters less light still.
  let light_in = frame.light_color.rgb * max(l.y, 0.0) + ambient_light_level(vec3<f32>(0.0, 1.0, 0.0), in.world) * frame.params.x;
  // Full from 1.5 (the sun some 25° up on a clear day), then falling with the square of the light: a moonlit body
  // (the moon's light is a fair share of the sun's in the day-night look, so land reads at night) sits under the night
  // sky it lies beneath, as a real sea's does (its water-leaving light is a few percent of the sky's).
  let daylight_k = min(dot(light_in, vec3<f32>(0.2126, 0.7152, 0.0722)) / 1.5, 1.0);
  let daylight = daylight_k * daylight_k;
  var deep = water.deep.rgb * mix(0.4, 1.0, sun) * daylight;
  let sea = is_sea();
  // The sky's light on the sea from above and along its normal (web-ocean's skyUp and skyN).
  let amb_up = ambient_light_level(in.normal, in.world) * frame.params.x;
  let amb_n = ambient_light_level(n, in.world) * frame.params.x;
  // Under a closed cloud deck the light reaches the sea as the deck's own radiance, from the whole sky above (its a:
  // how much of the sky it closes): the foam takes it (below). The sun itself is hidden by the deck
  // for every surface at its source (Clouds::light). The foam's light under a live sky is the captured deck that
  // amb_up and amb_n already carry (the analytical deck over it lit the foam twice).
  let deck = cloud_deck(vec3<f32>(0.0, 1.0, 0.0));
  let deck_up = select(deck, vec4<f32>(0.0), frame.clouds[4].w > 0.5);
  let sun_e = frame.light_color.rgb * sun;
  if (OG_SEA) {
if (sea) {
    // web-ocean's body (`ocean.js` fs; the Sea of Thieves talk's subsurface term): light carried through the thin
    // upper part of a backlit crest, strongest looking towards the sun; and under foam the bubbles brighten and tint
    // the water beneath it.
    // How high up its wave the point lies: over the sea's own height (a 10 m gale's crest is not lit through from 1 m
    // up, which lit the whole upper half of every wave alike), a breeze's 0.9 m as before.
    let crest = clamp((in.world.y - water.model[3].y - 0.05) / max(0.9, 0.35 * water.fft.w), 0.0, 1.0);
    let to_sun = max(dot(l, -v), 0.0);
    let thin = pow(clamp(0.5 - 0.5 * dot(l, n), 0.0, 1.0), 3.0);
    let dist = length(frame.view_pos.xyz - in.world);
    let k1 = SEA_SSS.w * crest * crest * pow(to_sun, 4.0) * thin * exp(-dist / 600.0);
    deep += SEA_SSS.rgb * sun_e * k1;
    let ambient = amb_up * 1.3 + amb_n * 0.5;
    deep += SEA_SSS.rgb * (sun_e * 0.05 + ambient * 0.35) * clamp(sea_f.cover, 0.0, 1.0);
    deep += vec3<f32>(0.10, 0.32, 0.30) * amb_up * sea_f.young * 0.35;
  }
  }
  if (water_js) {
    // three.js Water's body, (sunColor · diffuse · 0.3 + scatter) · shadow: diffuse = max(L·N, 0) · sunColor · 0.5,
    // scatter = max(N·V, 0) · waterColor (the deep colour), sunColor = the light × glint / 2 (glint = spec 2 × sunColor
    // over the light). The ripples shade it, as they do the original's.
    let sc = frame.light_color.rgb * water.under.w * 0.5;
    deep = (sc * sc * 0.15 * max(dot(l, n), 0.0) + water.deep.rgb * max(dot(n, v), 0.0)) * sun;
  }
  var color = mix(below * water.shallow.rgb, deep, clamp(absorb * water.deep.w, 0.0, 1.0));
  if (water.optics[0].w > 0.5) {
    // W2: the physical body. The light seen through the water travels the view ray's path in it, from the surface to
    // what lies below (the depth prepass's point, `thickness` along the view's forward), and had come down to that
    // point from the surface along the refracted sun: Beer–Lambert over both. The water scatters the light it lets in
    // back along the view ray (single scattering, the light attenuated down to each point and back along the ray),
    // the sun through a forward lobe and the sky evenly (UE's single-layer water, Crest's volume shading).
    let o = water_optics(v, l, thickness, frame.view_forward.xyz);
    let body = o.scatter * sun_e * hg_water(dot(l, -v)) + o.scatter_sky * amb_up;
    color = below * o.through + body;
    // A sea's crests and foam scatter their light on top, as `deep` carries for the colour model.
    if (OG_SEA) {
if (sea) { color += (deep - water.deep.rgb * mix(0.4, 1.0, sun) * daylight) * o.opacity; }
  }
  }

  // The sun's glitter on a sea, kept for the wet lace that lies over it (below).
  var glitter = vec3<f32>(0.0);
  // Reflection with Schlick's Fresnel.
  var r = reflect(-v, n);
  let f0 = water.extra.z;
  var fresnel = (f0 + (1.0 - f0) * pow(1.0 - saturate(dot(n, v)), 5.0)) * water.look.x;
  // The detail's slopes a pixel leaves out (LEAN, wave_slope) blur the mirrored sky as a sea's facets blur it.
  var mip = max(sqrt(sqrt(detail.cov.x + detail.cov.y)) * frame.params.y, 0.5);
  var sig_v = 0.0;
  if (OG_SEA) {
if (sea) {
    // web-ocean's tilt of the visible facets (`ocean.js` fs): at a grazing view the facets the eye sees lean towards
    // it (projected area and masking), so the reflector tilts up and mirrors higher sky. The Fresnel is Bruneton,
    // Neyret and Holzschuch's mean over a rough sea's facets (2010), which dims the grazing mirror as the slopes along
    // the view spread: web-ocean's partly smoothed normal kept it near 1 at the horizon, and with downward rays folded
    // up into the sky the far sea outshone the sky it mirrors (0.47 against 0.27 under the storm's deck). The sky is
    // blurred by the slopes' spread.
    let vh = normalize(vec3<f32>(v.x, 0.0, v.z) + vec3<f32>(1e-5, 0.0, 0.0));
    sig_v = sqrt(max(vh.x * vh.x * sea_cov.x + vh.z * vh.z * sea_cov.y + 2.0 * vh.x * vh.z * sea_cov.z, 1e-6));
    let a_v = max(v.y, 1e-3) / max(sqrt(1.0 - v.y * v.y), 1e-3) / (1.4142 * sig_v);
    let tilt = 0.32 * sig_v * (1.0 - smoothstep(0.0, 2.5, a_v));
    let nv = normalize(n + vh * tilt);
    r = reflect(-v, nv);
    fresnel = (f0 + (1.0 - f0) * sea_fresnel(saturate(dot(n, v)), sig_v)) * water.look.x;
    mip = max(sqrt(sqrt(sea_cov.x + sea_cov.y)) * frame.params.y, 0.5);
  }
  }
  // The environment, or the reflection probe holding the surface (probes.rs), as bright as the sky is drawn: without the
  // twilight's adaptation (water.ao.w), which lifts the light on surfaces but not the sky they mirror. Under a cloud
  // deck the sky mirrored is the clouds' grey, as they light it (sky.wgsl).
  let sky_mirror = frame.params.x * water.ao.w * 1.8;
  // A lake mirrors the sun by its glitter alone (below), its facets counted: the environment's sun disc, mirrored by
  // every ripple face turned to it, drew a soft disc twice the sun's width under each glint, and bloomed.
  let lake = !sea && !water2 && !water_js;
  let r_env = select(r, off_sun(r, mip), lake);
  var reflected = overcast(reflected_light(r_env, in.world, mip) * sky_mirror, r_env);
  if (water.extra.y > 0.5) {
    // The mirrored scene, flipped left to right, shifted by the waves.
    var shift_r = (n - in.normal).xz * water.screen.zw * 60.0;
    if (water.normals.w > 0.5 && !flows()) {
      // three.js Water (the lake's normal map): the mirror bends by N.xz (0.001 + 1 / distance) x distortionScale 1.2, in uv.
      shift_r = (n - in.normal).xz * (0.001 + 1.0 / max(length(frame.view_pos.xyz - in.world), 0.1)) * 1.2;
    } else if (water2) {
      // Water2's own reflector, read at the refraction's uv (uv + coord.z · N.xz · 0.05, mirrored left to right).
      shift_r = shift;
    }
    let puv = clamp(vec2<f32>(1.0 - uv.x, uv.y) + vec2<f32>(-shift_r.x, shift_r.y), vec2<f32>(0.0), vec2<f32>(1.0));
    reflected = textureSampleLevel(reflection_tex, water_sampler, puv, 0.0).rgb;
  } else if (water.look.y > 0.5) {
    let hit = ssr(in.world, r);
    reflected = mix(reflected, hit.rgb, hit.a);
  }
  if (!three_water) {
    // A ray reflected downwards meets the next wave, which shows its body and, dimly, the sky in turn (it never
    // reaches the sky, nor the ground under the horizon the blurred environment holds there). The facets spread the
    // reflection over about two of their slopes' deviations, so it turns down gradually. On a lake too: a ripple's
    // face tilted from the eye (the rim of a floating body's funnel) reflected down into the bed, the bank and the
    // body itself, through the screen's trace and the environment's ground, as a ring of brown grit.
    let up_r = vec3<f32>(r.x, abs(r.y), r.z);
    let up = overcast(reflected_light(up_r, in.world, mip) * sky_mirror, up_r);
    let spread = select(sqrt(max(detail.cov.x + detail.cov.y, 0.0)), sig_v, sea);
    let w = 2.0 * spread + 0.05;
    reflected = mix(deep + up * 0.25, reflected, smoothstep(-w, w, r.y));
  }
  let mirrored = reflected;
  reflected = water.extra.w + (1.0 - water.extra.w) * reflected;
  if (water2) {
    // Water2: color × mix(refraction, reflection, F): the tint is on the reflection too.
    reflected *= water.shallow.rgb;
  }
  // Screen-space AO: the scene behind (`below`) was darkened by it as it was lit; the reflection and the body are the
  // water's own, darkened here the same way (the whole colour × mix(0.35, 1, ao), as pbr_fs with aoDirect 1 and as
  // three.js-era post AO, which darkened the water's pixel whole), by PostEffects.aoDirect (water.ao.x): at the
  // default 0 the water, like every opaque surface's direct light, is not darkened by the screen AO.
  let ao_raw = textureSampleLevel(ao_tex, water_sampler, uv, 0.0).r;
  // Debug (DebugView::AmbientOcclusion): the ambient occlusion here, as pbr_fs.wgsl and scatter.wgsl.
  if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) { return vec4<f32>(vec3<f32>(ao_raw), 1.0); }
  let ao_k = mix(1.0, mix(0.35, 1.0, ao_raw), water.ao.x);
  reflected *= ao_k;
  if (water_js) { color *= ao_k; }
  color = mix(color, reflected, clamp(fresnel, 0.0, 1.0));

  if (water_js) {
    // three.js Water's glint: pow(eye . reflect(-L, N), 100) x the sun (glint = spec 2 x its sunColor over the light's
    // intensity) on the mirrored image, through the Fresnel mix: every wavelet facing the sun flashes with the sky.
    // Unshadowed, as there (its shadow mask is on the body alone).
    let g = pow(max(dot(v, reflect(-l, n)), 0.0), lean_power(100.0, detail.cov)) * lean_energy(100.0, detail.cov)
      * water.under.w;
    color += mirrored * frame.light_color.rgb * g * clamp(fresnel, 0.0, 1.0);
  } else if (sea) {
    // The sun's glitter, web-ocean's (`ocean.js` fs): the facets' slopes an anisotropic Gaussian about the pixel's
    // mean slope with the unresolved covariance (and the sun's disc), Smith's masking of the view and the light,
    // Schlick's Fresnel at the half vector (none in shadow).
    let h = normalize(l + v);
    let hs = -h.xz / max(h.y, 1e-3);
    let dsl = hs + n.xz / max(n.y, 1e-3);
    let cov = sea_cov + vec3<f32>(SUN_DISC, SUN_DISC, 0.0);
    let det = max(cov.x * cov.y - cov.z * cov.z, 1e-10);
    let e = (dsl.x * dsl.x * cov.y + dsl.y * dsl.y * cov.x - 2.0 * dsl.x * dsl.y * cov.z) / det;
    let pdf = exp(-0.5 * e) / (6.2831853 * sqrt(det));
    let hy2 = h.y * h.y;
    let dm = pdf / max(hy2 * hy2, 1e-6);
    let g = 1.0 / (1.0 + smith_lambda(v, cov) + smith_lambda(l, cov));
    let fh = f0 + (1.0 - f0) * pow(1.0 - clamp(dot(v, h), 0.0, 1.0), 5.0);
    let spec = fh * dm * g / (4.0 * max(v.y, 0.02)) * step(0.0, l.y);
    glitter = sun_e * spec * water.look.x;
    color += glitter;
  } else if (!water2) {
    // (Water2 has no highlight of its own: the sun shows only in what it reflects.)
    // The sun's glitter, the sea's (web-ocean's Gaussian of the facets' slopes) about the lake's own ripples: the
    // slopes a pixel leaves out (LEAN), the slopes' change across it and the capillary ripples no wave draws widen
    // it, and the sun's disc bounds how narrow it gets. That lobe is what a pixel reflects on average; the glints are
    // the few facets in it that face the sun (`lake_glints`). (A Blinn-Phong lobe of power 220 is some 5° wide,
    // twenty suns, so every ripple face near the sun's mirror angle lit whole and clipped into a flat white field.)
    let h = normalize(l + v);
    let hs = -h.xz / max(h.y, 1e-3);
    let aa = vec3<f32>(dsx.x * dsx.x + dsy.x * dsy.x, dsx.y * dsx.y + dsy.y * dsy.y, dsx.x * dsx.y + dsy.x * dsy.y) * 0.5;
    let capillary = CAPILLARY * clamp(water.waves.z / 0.35, 0.0, 1.0);
    let micro = detail.cov + aa * min(1.0, 0.03 / max(max(aa.x, aa.y), 1e-8)) + vec3<f32>(capillary, capillary, 0.0);
    let cov = micro + vec3<f32>(SUN_DISC, SUN_DISC, 0.0);
    let dsl = hs + ms;
    let det = max(cov.x * cov.y - cov.z * cov.z, 1e-10);
    let e = (dsl.x * dsl.x * cov.y + dsl.y * dsl.y * cov.x - 2.0 * dsl.x * dsl.y * cov.z) / det;
    let pdf = exp(-0.5 * e) / (6.2831853 * sqrt(det));
    let hy2 = h.y * h.y;
    let dm = pdf / max(hy2 * hy2, 1e-6);
    let g = 1.0 / (1.0 + smith_lambda(v, cov) + smith_lambda(l, cov));
    let fh = f0 + (1.0 - f0) * pow(1.0 - clamp(dot(v, h), 0.0, 1.0), 5.0);
    let spec = fh * dm * g / (4.0 * max(v.y, 0.02)) * step(0.0, l.y);
    // The share of the facets whose slope lies on the sun's disc (its area in slope space, 2π its variance).
    let share = clamp(pdf * 6.2831853 * SUN_DISC, 0.0, 1.0);
    let glint = sun_e * spec * water.look.x * lake_glints(in.local, foot_area, share, t);
    // Each glint soft-capped at some eight times the sunlit white: a facet mirrors the sun's own radiance, which no
    // display holds, and uncapped each pinpoint bloomed into a halo many pixels wide.
    let gl = dot(glint, vec3<f32>(0.2126, 0.7152, 0.0722));
    let cap = 8.0 * max(dot(sun_e, vec3<f32>(0.2126, 0.7152, 0.0722)), 1e-4);
    color += glint / (1.0 + gl / cap);
  }

  // Foam: a band where the water meets geometry, broken up by noise, and on steep crests.
  let edge = 1.0 - clamp(thickness / max(water.foam.w, 1e-3), 0.0, 1.0);
  let noise = foam_noise(in.local * 6.0 + vec2<f32>(t * 0.3, -t * 0.2));
  let crest = clamp((in.slope - 0.35) * 3.0, 0.0, 1.0);
  var band = edge * edge * (0.55 + 0.6 * noise);
  let flowing = flows() && water.normals.w < 0.5 && !is_sea();
  if (OG_FLOW) {
if (flowing) {
    // A stream's foam comes and goes along its banks: patches carried down by the flow, more of it where the water
    // runs fast and shallow, none where it is still (a uniform band read as a seam).
    var f = water.normals.yz;
    if (water.ao.y > 0.5) { f = flow_at(in.uv) * length(water.normals.yz); }
    let speed = length(f);
    // Carried by the flow in two phases (Vlachos, "Water Flow in Portal 2", SIGGRAPH 2010): each copy of the patches
    // drifts for a period and restarts half a period after the other, cross-faded so neither restart shows, its own
    // offset to tell the copies apart. A drift of f·t grew without bound and lost its precision in a long session
    // (at 10⁶ s and 1 m/s the noise's lattice was read in steps of 1/16 m).
    let period = 8.0;
    let ph0 = fract(water.spray.w / period);
    let ph1 = fract(water.spray.w / period + 0.5);
    let pa = bank_patches(in.local - f * (ph0 * period) + vec2<f32>(0.0, 0.0));
    let pb = bank_patches(in.local - f * (ph1 * period) + vec2<f32>(17.3, 9.1));
    let patches = mix(pa, pb, abs(ph0 - 0.5) * 2.0);
    // Faded by depth: none at the bank's very contact (a band peaked there read as one bright line along the whole
    // bank), most a third of the band out where the water piles against it.
    let d = 1.0 - edge;
    band = smoothstep(0.02, 0.3, d) * (1.0 - smoothstep(0.3, 1.0, d)) * (0.3 + 0.8 * noise) * patches * smoothstep(0.05, 0.6, speed);
  }
  }
  // The ripples' foam: lumps with gaps between them, its cover growing with how much of it there is (a soft disc of
  // it read as a glow), drifting slowly.
  let lumps = foam_noise(in.local * 2.3 + vec2<f32>(t * 0.15, t * 0.1)) * 0.6 + foam_noise(in.local * 7.1 - vec2<f32>(t * 0.2, 0.0)) * 0.4;
  let amount = clamp(ripple_foam, 0.0, 1.0);
  let lumpy = smoothstep(1.0 - amount, 1.12 - amount, lumps) * (0.5 + 0.5 * amount);
  let foam = clamp(band + crest * noise + lumpy, 0.0, 1.0);
  // Foam is a cloud of bubbles, lit through its depth from any side as the river's white water (wrapped): under
  // Lambert's cosine the flecks on a funnel's or a crest's face turned from the sun went ambient grey over the water and
  // read as grit.
  let lit = clamp((dot(in.normal, l) + 0.6) / 1.6, 0.0, 1.0) * frame.light_color.rgb * sun + vec3<f32>(frame.params.x * 0.6);
  color = mix(color, water.foam.rgb * lit, foam * step(0.001, water.foam.w));
  if (water.waterline.w > 0.5) {
    // W5: a river. Below a fall its pool churns white (the flow map's b: 1 at the fall's foot, fading over some one and
    // a half widths), in patches carried down by the current in two phases as the bank foam; and where the ribbon hangs (its
    // normal more than some 40 degrees off up: the course falls there) the water is a sheet falling some 7 m/s,
    // streaked along its fall.
    let ft = flow_texel(in.uv);
    let current = (ft.rg * 2.0 - 1.0) * length(water.normals.yz);
    let ph0 = fract(water.spray.w / 8.0);
    let ph1 = fract(water.spray.w / 8.0 + 0.5);
    let pa = bank_patches(in.local * 1.7 - current * (ph0 * 8.0));
    let pb = bank_patches(in.local * 1.7 - current * (ph1 * 8.0) + vec2<f32>(5.3, 11.7));
    let churned = ft.b * (0.3 + 0.7 * mix(pa, pb, abs(ph0 - 0.5) * 2.0));
    let hang = 1.0 - smoothstep(0.62, 0.8, in.normal.y);
    let fall = water.spray.w * 7.0;
    let streak = foam_noise(vec2<f32>(in.uv.x * 40.0, in.world.y * 0.9 + fall * 0.9)) * 0.65
      + foam_noise(vec2<f32>(in.uv.x * 113.0, in.world.y * 2.7 + fall * 2.7)) * 0.35;
    // The streaks thin where the ribbon only starts to hang (the lip, still glassy) and close up where it falls.
    let white_water = smoothstep(0.3, 0.7, streak) * mix(0.55, 1.0, hang);
    let fall_white = hang * white_water;
    let river_white = clamp(max(churned, fall_white), 0.0, 1.0);
    // White water is a cloud of bubbles and drops: lit through its depth by the sun from any side (wrapped, not Lambert's
    // cosine, which left a sheet facing away from the sun grey) and by the whole sky.
    let wrap = (dot(in.normal, l) + 0.6) / 1.6;
    // (Lambert's 1/pi on the sun's irradiance, as the sea's foam: the river's had none and glowed over the ground.)
    let white_lit = clamp(wrap, 0.0, 1.0) * frame.light_color.rgb * sun * 0.31831 + vec3<f32>(frame.params.x * 0.6);
    color = mix(color, water.foam.rgb * white_lit * mix(0.85, 1.0, white_water), river_white);
  }
  if (OG_SEA) {
if (sea) {
    // A sea's foam, whatever the shore foam's width, web-ocean's (`ocean.js` fs): a matte, diffuse layer over the
    // water, its albedo the foam colour varied by the bubbles inside it, lit by the sun (half of it: the lumpy foam
    // shadows itself at a grazing sun) and by the whole sky above, nearly grey (foam scatters every part of it).
    // The sun's share is Lambert's, its irradiance over pi as web-ocean's (0.318) and pbr_fs's: the port had dropped
    // the 1/pi, lighting sunlit foam three times too bright.
    // W4: the thin lace an old whitecap leaves is wet, darker than the dry, fresh foam of a crest breaking now.
    let wet = sea_f.lace / max(sea_f.mask, 1e-3);
    // (The lace's bubbles are too fine to see apart: their mean, so a thin thread is not drawn as a chain of beads.)
    let alb = water.foam.rgb * (0.78 + 0.3 * mix(sea_f.bubbles, 0.5, wet)) * mix(1.0, 0.65, wet);
    let amb = amb_up + amb_n * 0.3;
    let amb_grey = mix(amb, vec3<f32>(dot(amb, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.85);
    // The deck's irradiance over its radiance: its cosine-weighted mean, brighter overhead (overcast()'s 1 + 0.6 r.y)
    // as 1 + 0.6 * 2/3.
    let sky_light = mix(amb_grey * 1.55, deck_up.rgb * 1.4, deck_up.a);
    let foam_lit = alb * (sun_e * max(dot(n, l), 0.0) * 0.5 * 0.31831 + sky_light);
    color = mix(color, foam_lit, sea_f.mask * 0.95);
    // The lace is a film between bubbles, not a matte sheet: the water's glitter shows through some of it (more drew
    // each thread as a string of glints).
    color += glitter * sea_f.lace * 0.95 * 0.3;
  }
  }
  return vec4<f32>(apply_fog(color, in.world), 1.0);
}

// The surface's view depth for the screen AO (lib.rs, after the prepass): three.js' Water and Water2 meshes wrote depth,
// so the original's post AO saw the water's surface, not the bed under it. Drawn into a copy of the prepass depth,
// tested against the prepass, so the AO alone sees the water (the water itself still reads the bed's depth).
@fragment
fn fs_ao_depth(in: WaterOut) -> @location(0) vec4<f32> {
  return vec4<f32>(view_depth(in.world), 0.0, 0.0, 1.0);
}
