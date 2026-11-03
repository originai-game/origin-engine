// Grass blades (grass.rs): what every grass draw (lit, prepass, shadow)
// needs to turn a packed blade and a vertex index into a point of the
// blade, and its colour. Group 1 of every grass pipeline. Needs
// grass_common.wgsl before it.
//
// A blade is a strip of rows from its root to its tip, `rows` per level
// of detail (grass.rs LOD_ROWS): row r is vertices 2r and 2r + 1, the tip
// vertex 2 * rows. Its draw for level l passes base vertex
// l * GRASS_LOD_STRIDE, so the level is known from the vertex index alone
// (no first instance, an optional feature on the web).
//
// Ported from bevy_grass v0.20.3 (src/shaders/blade.wesl, look.wesl;
// third_party/bevy-grass). MIT License, Copyright (c) 2026 Gediminas
// Mikelenas: the full notice is in grass_common.wgsl and
// third_party/bevy-grass/LICENSE.

// grass.rs DrawGpu.
struct GrassUniform {
  view_proj: mat4x4<f32>,       // unjittered (the prepass's motion)
  prev_view_proj: mat4x4<f32>,
  eye: vec4<f32>,       // xyz: the camera (frame coordinates), w: 1 orthographic
  forward: vec4<f32>,   // xyz: its forward, w: metres a pixel covers per unit of clip w
  origin: vec4<f32>,    // xyz: where blade positions are stored from, w: their extent either side (m)
  limits: vec4<f32>,    // x: the height range roots are stored over (m either side), y: the narrowest a blade draws (px), z: how far an edge-on blade turns to the eye, w: the root's occlusion
  sun: vec4<f32>,       // xyz: towards the sun, w: 1 with one
  shape: vec4<f32>,     // x: the tallest blade (m), y: its root width (m), z: curve (droop), w: flutter
  base_color: vec4<f32>,  // rgb: the roots' colour (linear), w: translucency
  tip_color: vec4<f32>,   // rgb: the tips', w: sheen
  dry_color: vec4<f32>,   // rgb: dry grass, w: roughness
  variation: vec4<f32>,   // x: colour variation, y: dryness, z: patch size (m), w: the field's distance (m)
  rows: vec4<u32>,      // rows of each level of detail
  wind: WindField,
};

@group(1) @binding(0) var<uniform> grass: GrassUniform;
// [0]: xyz each level's first blade; then the blades (grass_generate.wgsl).
@group(1) @binding(1) var<storage, read> grass_blades: array<vec4<u32>>;

struct GrassRef {
  blade: GrassBlade,
  lod: u32,
  local: u32,   // the vertex within the level's strip
};

fn grass_ref(vertex_index: u32, instance_index: u32) -> GrassRef {
  var r: GrassRef;
  r.lod = min(vertex_index / GRASS_LOD_STRIDE, 2u);
  r.local = vertex_index % GRASS_LOD_STRIDE;
  let first = grass_blades[0];
  r.blade = grass_unpack(grass_blades[first[r.lod] + instance_index], grass.origin, grass.limits.x);
  return r;
}

// Whom a blade is built for.
struct GrassEye {
  position: vec3<f32>,
  // Orthographic: the direction it looks from (a shadow pass's sun).
  toward: vec3<f32>,
  orthographic: bool,
  view_proj: mat4x4<f32>,
  // Metres a pixel covers per unit of clip w (0: no narrowest width).
  pixel: f32,
  // Build the blade as it stood last frame (motion vectors).
  previous: bool,
};

struct GrassVertex {
  world: vec3<f32>,
  normal: vec3<f32>,
  t: f32,   // 0 at the root, 1 at the tip
};

fn grass_random(b: GrassBlade, salt: u32) -> f32 {
  let bits = u32(round(b.seed * 255.0)) | (u32(b.facing * 256.0) << 8u) | (u32(round(b.clump * 255.0)) << 16u);
  return wind_unit(wind_hash(bits ^ salt));
}

fn grass_vertex(r: GrassRef, eye: GrassEye) -> GrassVertex {
  let b = r.blade;
  let height = b.height * grass.shape.x;
  let root_width = b.width * GRASS_WIDTH_RANGE * grass.shape.y;
  let rows = grass.rows[r.lod];
  let is_tip = r.local >= 2u * rows;
  let row = select(r.local / 2u, rows, is_tip);
  let t = f32(row) / f32(rows);
  let side = select(f32(r.local % 2u) - 0.5, 0.0, is_tip);

  // The blade's flat side faces its facing; turned edge-on to the eye it
  // turns part of the way towards it, so a field seen side-on does not
  // thin out to lines.
  var to_eye = select(eye.position.xz - b.root.xz, eye.toward.xz, eye.orthographic);
  to_eye = select(vec2<f32>(0.0, 1.0), normalize(to_eye), dot(to_eye, to_eye) > 1e-8);
  let yaw = b.facing * GRASS_TAU;
  var face = vec2<f32>(sin(yaw), cos(yaw));
  let facing_eye = dot(face, to_eye);
  let edge_on = 1.0 - abs(facing_eye);
  face = normalize(mix(face, to_eye * select(-1.0, 1.0, facing_eye >= 0.0), edge_on * grass.limits.z));
  let across = vec3<f32>(face.y, 0.0, -face.x);

  // Tall blades droop the most, each its own amount.
  let droop = grass.shape.z * grass_random(b, 0x7c2u) * (0.5 + 0.5 * b.height);
  let bend = select(b.bend, b.previous_bend, eye.previous);
  let curve = grass_curve(height, bend, droop);
  let u = 1.0 - t;
  var point = curve[0] * (u * u) + curve[1] * (2.0 * u * t) + curve[2] * (t * t);
  let tangent = normalize((curve[1] - curve[0]) * (2.0 * u) + (curve[2] - curve[1]) * (2.0 * t) + vec3<f32>(0.0, 1e-5, 0.0));
  if (r.lod == 0u) {
    point += across * grass_flutter(grass.wind, grass.shape.w, b.seed, t, height, eye.previous);
  }

  // A blade narrower on screen than the narrowest width widens, keeping
  // its taper: thinner ones shimmer.
  let per_pixel = (eye.view_proj * vec4<f32>(b.root, 1.0)).w * eye.pixel;
  let widen = max(1.0, grass.limits.y * per_pixel / max(root_width, 1e-6));
  let width = root_width * widen * (1.0 - pow(t, 1.4));
  // Rounded normals: each edge's normal tilts out by about 30 degrees, so
  // the flat strip shades as a curved leaf.
  let normal = normalize(cross(across, tangent));
  var out: GrassVertex;
  out.t = t;
  out.normal = normalize(normal + across * side * 2.0 * 0.577);
  out.world = b.root + point + across * side * width;
  return out;
}

// The colour of a blade at `t`: dark roots to bright tips, brighter or
// darker per clump and a little per blade, in patches several metres
// across, a share of the clumps dry and the tips drying.
fn grass_color(b: GrassBlade, t: f32) -> vec3<f32> {
  let v = grass.variation.x;
  let shade = mix(1.0 - 0.35 * v, 1.0 + 0.25 * v, b.clump) * (1.0 + v * (0.2 * b.seed - 0.1));
  var c = mix(grass.base_color.rgb, grass.tip_color.rgb, pow(t, 0.7)) * shade;
  // Patches: the noise, turned so its lattice does not run along the world's axes.
  let at = (b.root.xz + grass.wind.time.zw) / max(grass.variation.z, 0.1);
  let patch_uv = vec2<f32>(at.x * 0.83 - at.y * 0.56, at.x * 0.56 + at.y * 0.83);
  let patch_noise = wind_noise(patch_uv + vec2<f32>(91.0, 47.0));
  c *= mix(1.0, mix(0.85, 1.12, patch_noise), v);
  // Dry clumps (a share `dryness` of them) and dry tips.
  let dryness = grass.variation.y;
  let dry_clump = grass_dry(b.clump, b.root.xz + grass.wind.time.zw, dryness);
  let dry_tip = dryness * smoothstep(0.45, 1.0, t) * (0.4 + 1.2 * grass_random(b, 0x7d1u));
  let dry = grass.dry_color.rgb * shade * mix(0.6, 1.0, pow(t, 0.7));
  c = mix(c, dry, clamp(max(dry_clump * 0.75, dry_tip), 0.0, 1.0));
  return c;
}

// The neighbours' shade on a blade at `t`: darkest at the root.
fn grass_occlusion(t: f32) -> f32 {
  return mix(1.0 - grass.limits.w, 1.0, smoothstep(0.0, 0.6, t));
}
