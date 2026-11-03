// Forward+ PBR (metallic-roughness): the directional light with 3 cascaded
// shadow maps, the point and spot lights of the fragment's cluster (the
// first two shadowed; clusters.rs), split-sum IBL. Linear HDR output; the
// post passes tonemap. Prepended by frame.wgsl, the light tables
// (clusters.rs), noise.wgsl, material.wgsl, the hooks and lighting.wgsl (the
// BRDF with the clear coat, the shadow lookups, the water's caustics);
// skin_*.wgsl supplies `joint_matrix` for the skinned entry points.
//
// Effects (each behind a branch, so a material or frame without it shades
// exactly as before): projected decals before lighting, screen-space
// ambient occlusion on the image-based light, toon bands, rim light, the
// dissolve (discard plus a glowing edge) and the hit flash (the material's,
// and the instance's own with its tint: `InstanceFxIn`). `vs_prepass` /
// `fs_prepass` write the depth-normal prepass: world normal and
// screen-space velocity, view depth, and the material's outline.

// `FxUniform` in fx.rs: per-frame effect inputs of the lit passes.
struct DecalGpu {
  inv: mat4x4<f32>,       // world -> decal box (-0.5..0.5)
  axis: vec4<f32>,        // xyz: the box's +z in world (surfaces facing it are covered), w: cos of the cut-off angle
  uv_rect: vec4<f32>,     // u0, v0, u1, v1 in the atlas
  color: vec4<f32>,       // linear rgba
  emissive: vec4<f32>,    // rgb: emitted light, w: roughness (<0 keep)
  normal_rect: vec4<f32>, // u0, v0, u1, v1 of its normal map in the atlas
  normal: vec4<f32>,      // x: the normal map's scale (0: none)
};
struct Fx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,      // w, h, 1/w, 1/h
  ao: vec4<f32>,          // x: 1 when the AO texture is valid, y: intensity
  decal_info: vec4<f32>,  // x: decals (negative: no decal clusters, walk them all), y: atlas (1 linear, 2 sRGB), z: cos fade width, w: the previous joints' section (0: none; vs_prepass_skinned)
  planar: vec4<f32>,      // the planar reflection's plane (n, n·at); n 0: none, then w -1 while drawing it
};
@group(2) @binding(0) var<uniform> fx: Fx;
@group(2) @binding(1) var ssao_tex: texture_2d<f32>;
@group(2) @binding(2) var decal_atlas: texture_2d<f32>;
@group(2) @binding(3) var fx_sampler: sampler;
// The decal records and their clusters (fx.rs, clusters.rs `DecalBins`):
// one word per cluster (first reference, 24 bits; count, 8 bits), then the
// references, one record index a word.
@group(2) @binding(5) var<storage, read> decal_list: array<DecalGpu>;
@group(2) @binding(6) var<storage, read> decal_cells: array<u32>;
// planar_reflection (the planar reflection for material shaders): planar.wgsl,
// or its stub where the lit pipelines have no texture slot left (lib.rs).

struct VertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  // xyz along +u, w the bitangent's sign (0: none).
  @location(15) tangent: vec4<f32>,
};

// The instance's transform: its columns' xyz (instance.wgsl).
struct InstanceIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
};

// What an instance carries beside its transform (instances.rs
// `GpuTransform`, lib.rs `INSTANCE_LAYOUT` location 8): x an array-map
// instance's layer plus one (pbr_array.wgsl), negated less one when the
// instance receives no shadows (instances.rs `layer_bits`: `look_layer`,
// `look_receives`), then its own look (instances.rs `fx_bits`): y the hit
// flash, z the flash's colour and w the tint's distance from white, each
// colour 8:8:8 in an exact integer. All zero: no look, receiving shadows.
struct InstanceFxIn {
  @location(8) look: vec4<f32>,
};

// A skinned instance: its transform and where its joints start in the palette.
struct SkinnedInstanceIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
  @location(8) palette: vec4<f32>,  // x: first joint, y: joints, z: first morph header, w: morph targets
  @location(11) morph0: vec4<f32>,  // x: first weight slot + 1, y: first deformed slot + 1 (0: none; skin.rs, the palette's layout)
  @location(12) morph1: vec4<f32>,  // a baked clip's previous frame (skin.rs)
  @location(13) look: vec4<f32>,    // as InstanceFxIn's (x: no layer, 0 or -1)
};

struct SkinIn {
  @location(9) joints: vec4<u32>,
  @location(10) weights: vec4<f32>,
};

// The previous frame's transform of the instance (motion vectors), as
// InstanceIn's.
struct PrevIn {
  @location(11) p0: vec3<f32>,
  @location(12) p1: vec3<f32>,
  @location(13) p2: vec3<f32>,
  @location(14) p3: vec3<f32>,
};

struct VertexOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  // The world tangent and the bitangent's sign (w 0: none).
  @location(5) tangent: vec4<f32>,
  // Affine texturing (frame.stylise.z): uv times the clip w, and that w,
  // so the fragment can undo the perspective-correct interpolation.
  @location(6) affine: vec3<f32>,
  // The instance's look (InstanceFxIn; zero: none).
  @location(8) @interpolate(flat) look: vec4<f32>,
};

// A colour packed 8:8:8 in an exact integer (instances.rs `fx_bits`; flat,
// so it arrives exact: no rounding, which at 2^24 - 1 would carry).
fn look_rgb(packed: f32) -> vec3<f32> {
  let u = u32(packed);
  return vec3<f32>(f32((u >> 16u) & 255u), f32((u >> 8u) & 255u), f32(u & 255u)) / 255.0;
}

// The base colour times the instance's tint (look.w: its distance from
// white; 0 leaves the colour untouched).
fn look_tint(c: vec4<f32>, look: vec4<f32>) -> vec4<f32> {
  if (look.w <= 0.0) { return c; }
  return vec4<f32>(c.rgb * (vec3<f32>(1.0) - look_rgb(look.w)), c.a);
}

// The lit colour mixed towards the instance's flash colour by its flash.
fn look_flash(c: vec3<f32>, look: vec4<f32>) -> vec3<f32> {
  if (look.y <= 0.0) { return c; }
  return mix(c, look_rgb(look.z), look.y);
}

// Whether the instance is darkened by the shadow maps (look.x >= 0;
// instances.rs `layer_bits`): one that is not is lit as if nothing stood
// between it and any light.
fn look_receives(look: vec4<f32>) -> bool {
  return look.x >= 0.0;
}

// An array-map instance's layer plus one, 0 for the material's own
// (look.x without the receive flag).
fn look_layer(look: vec4<f32>) -> f32 {
  return select(-look.x - 1.0, look.x, look_receives(look));
}

// The uv a fragment samples with: the interpolated one, or (retro 3D) the
// affine one, interpolated without perspective as early hardware did.
fn stylised_uv(uv: vec2<f32>, affine: vec3<f32>) -> vec2<f32> {
  if (frame.stylise.z <= 0.0 || affine.z <= 0.0) { return uv; }
  return mix(uv, affine.xy / affine.z, frame.stylise.z);
}

// Retro 3D (frame.stylise): the clip position snapped to a screen grid of
// stylise.xy cells and its depth quantised to stylise.w bits, as fixed-point
// hardware did; the identity when both are 0.
fn retro_clip(clip: vec4<f32>) -> vec4<f32> {
  var c = clip;
  if (c.w <= 0.0) { return c; }
  if (frame.stylise.x > 0.0) {
    let half = frame.stylise.xy * 0.5;
    let ndc = c.xy / c.w;
    c = vec4<f32>(round(ndc * half) / half * c.w, c.zw);
  }
  if (frame.stylise.w > 0.0) {
    let steps = exp2(frame.stylise.w);
    c.z = floor(c.z / c.w * steps) / steps * c.w;
  }
  return c;
}

struct PrepassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) cur_clip: vec4<f32>,
  @location(6) prev_clip: vec4<f32>,
  // Which object this is (`object_id`), for the outline pass.
  @location(8) @interpolate(flat) object: u32,
};

// An object's id in the prepass's outline target, 1 to 255, from its
// instance's transform (0 is terrain and scattered copies): the outline
// pass lines a crease only between two objects, never between the faces of
// one. Neighbouring objects share one in 255.
fn object_id(m0: vec3<f32>, m1: vec3<f32>, m2: vec3<f32>, m3: vec3<f32>) -> u32 {
  var h = 2166136261u;
  for (var c = 0u; c < 3u; c++) {
    h = (h ^ bitcast<u32>(m3[c])) * 16777619u;
    h = (h ^ bitcast<u32>(m0[c] + m1[c] * 3.0 + m2[c] * 7.0)) * 16777619u;
  }
  h = h ^ (h >> 16u);
  return 1u + h % 255u;
}

// A normal's world direction: through the inverse transpose of the model's
// upper 3x3 (its cofactor matrix, flipped for a mirroring transform), so a
// non-uniform scale bends normals the right way.
fn world_normal(model: mat4x4<f32>, n: vec3<f32>) -> vec3<f32> {
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  let mirrored = dot(a, cross(b, c)) < 0.0;
  return normalize(cofactor * n) * select(1.0, -1.0, mirrored);
}

// Where a lit vertex lands: its clip position, world position (moved by
// a material shader's vertex offset) and world normal. Every lit vertex
// stage, the lean ones (pbr_lean.wgsl) included, computes them here, so
// they agree to the bit.
struct LitPlace {
  clip: vec4<f32>,
  world: vec3<f32>,
  normal: vec3<f32>,
};

fn lit_place(model: mat4x4<f32>, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>, color: vec4<f32>) -> LitPlace {
  var world = model * vec4<f32>(position, 1.0);
  let n = world_normal(model, normal);
  // A material shader's vertex offset (none with the built-in hooks).
  var h: Hook;
  h.world = world.xyz;
  h.local = position;
  h.normal = n;
  h.uv = uv;
  h.color = color;
  h.time = frame.time.x;
  h.origin = frame.origin.xyz;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  world = vec4<f32>(world.xyz + vertex_offset(h, shader_params()), world.w);
  var p: LitPlace;
  p.clip = retro_clip(moved_clip(frame.view_proj, world));
  p.world = world.xyz;
  p.normal = n;
  return p;
}

fn lit_vertex(model: mat4x4<f32>, v: VertexIn) -> VertexOut {
  let p = lit_place(model, v.position, v.normal, v.uv, v.color);
  var o: VertexOut;
  o.clip = p.clip;
  o.affine = vec3<f32>(v.uv * o.clip.w, o.clip.w);
  o.world_pos = p.world;
  o.normal = p.normal;
  o.uv = v.uv;
  o.color = v.color;
  o.local_pos = v.position;
  o.look = vec4<f32>(0.0);
  // A tangent follows the surface: through the model itself.
  let t = (model * vec4<f32>(v.tangent.xyz, 0.0)).xyz;
  o.tangent = vec4<f32>(select(vec3<f32>(0.0), normalize(t), dot(t, t) > 1e-20), v.tangent.w);
  return o;
}

// The weighted joint matrices of one vertex (identity without weights).
fn skin_matrix(s: SkinIn, palette: vec4<f32>) -> mat4x4<f32> {
  let total = s.weights.x + s.weights.y + s.weights.z + s.weights.w;
  if (total < 1e-6) {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
  let w = s.weights / total;
  return palette_joint(palette, s.joints.x) * w.x
    + palette_joint(palette, s.joints.y) * w.y
    + palette_joint(palette, s.joints.z) * w.z
    + palette_joint(palette, s.joints.w) * w.w;
}

@vertex
fn vs(v: VertexIn, i: InstanceIn, k: InstanceFxIn) -> VertexOut {
  var o = lit_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  o.look = k.look;
  return o;
}




fn morph_vertex(v: VertexIn, vi: u32, i: SkinnedInstanceIn) -> VertexIn {
  let m = morphed(v.position, v.normal, vi, i.palette, i.morph0.x, i.morph0.y);
  var o = v;
  o.position = m.position;
  o.normal = m.normal;
  return o;
}

@vertex
fn vs_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> VertexOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);
  var o = lit_vertex(model, morph_vertex(v, vi, i));
  o.look = i.look;
  return o;
}

fn prepass_vertex(model: mat4x4<f32>, prev: mat4x4<f32>, v: VertexIn) -> PrepassOut {
  return prepass_vertex_from(model, prev, v, v.position);
}

// `before`: where the vertex was in its own space the frame before (a
// morphing vertex's previous weights).
fn prepass_vertex_from(model: mat4x4<f32>, prev: mat4x4<f32>, v: VertexIn, before: vec3<f32>) -> PrepassOut {
  let o = lit_vertex(model, v);
  var p: PrepassOut;
  p.clip = o.clip;
  p.world_pos = o.world_pos;
  p.normal = o.normal;
  p.uv = o.uv;
  p.color = o.color;
  p.local_pos = o.local_pos;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(o.world_pos, 1.0);
  var previous = prev * vec4<f32>(before, 1.0);
  var h: Hook;
  h.world = previous.xyz;
  h.local = before;
  h.normal = world_normal(prev, v.normal);
  h.uv = v.uv;
  h.color = v.color;
  h.time = frame.time.w;
  h.origin = frame.origin.xyz;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  previous = vec4<f32>(previous.xyz + vertex_offset(h, previous_shader_params()), previous.w);
  p.prev_clip = mul_point(fx.prev_view_proj, previous);
  p.object = object_id(model[0].xyz, model[1].xyz, model[2].xyz, model[3].xyz);
  return p;
}

@vertex
fn vs_prepass(v: VertexIn, i: InstanceIn, prev: PrevIn) -> PrepassOut {
  return prepass_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), instance_matrix(prev.p0, prev.p1, prev.p2, prev.p3), v);
}

@vertex
fn vs_prepass_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> PrepassOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);
  // The previous frame's pose (skin.rs `append_previous`): the same
  // joints in the palette's second section, already in the world. A
  // vertex without weights (morph targets alone) moves with the camera.
  // Its blendshapes move it too: the previous weights sit at the same
  // offset in that section, so a morphing face has motion vectors; and a
  // garment's own positions (morph0.y), the previous frame's there too.
  var prev = model;
  let section = fx.decal_info.w;
  let now = morph_vertex(v, vi, i);
  var before = now.position;
  if (i.palette.w < -0.5) {
    // A baked clip: its previous transform and frame in the current
    // section (morph1), so it needs no previous palette.
    if (i.morph1.x > 0.5) {
      let then = vec4<f32>(i.morph1.y, i.palette.y, 0.0, -1.0 - i.morph1.z);
      prev = joint_palette[u32(i.morph1.x) - 1u] * skin_matrix(s, then);
    }
  } else if (section > 0.5) {
    if (dot(s.weights, vec4<f32>(1.0)) >= 1e-6) {
      prev = skin_matrix(s, vec4<f32>(i.palette.x + section, i.palette.yzw));
    }
    if (i.morph0.x > 0.5 || i.morph0.y > 0.5) {
      let weights = select(0.0, i.morph0.x + section, i.morph0.x > 0.5);
      let deformed = select(0.0, i.morph0.y + section, i.morph0.y > 0.5);
      before = morphed(v.position, v.normal, vi, i.palette, weights, deformed).position;
    }
  }
  var p = prepass_vertex_from(model, prev, now, before);
  // The instance's, not the skinned vertex's (its joints move it).
  p.object = object_id(i.m0, i.m1, i.m2, i.m3);
  return p;
}

// False for materials and frames without effects: their code folds away.
override EFFECTS: bool = true;

// The toon ramp: N·L (times shadow) to the light received, from the
// gradient map's first row without filtering, or three built-in bands.
fn toon_ramp(x: f32) -> f32 {
  let t = clamp(x, 0.0, 1.0);
  if (material.toon.x < 1.5) {
    let w = i32(textureDimensions(gradient_tex).x);
    let i = clamp(i32(t * f32(w)), 0, w - 1);
    return textureLoad(gradient_tex, vec2<i32>(i, 0), 0).r;
  }
  return select(select(0.22, 0.6, t > 0.08), 1.0, t > 0.45);
}

// Cel-shaded light: the ramp on the diffuse, a hard specular highlight.
fn toon_light(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, shadow: f32) -> vec3<f32> {
  let n_dot_l = max(dot(n, l), 0.0);
  let band = toon_ramp(n_dot_l * shadow);
  let h = normalize(v + l);
  let shine = exp2(10.0 * (1.0 - s.roughness) + 1.0);
  let spec = smoothstep(0.48, 0.52, pow(max(dot(n, h), 0.0), shine) * shadow);
  let kd = (1.0 - s.metallic) * s.albedo / PI;
  return kd * band + s.f0 * spec * (1.0 - s.roughness);
}

// The material style's shadow darkness (frame.outline_style.y): what a
// surface keeps of the sun's ramp and of the ambient light, from the sun it
// receives (N·L times the shadow). Toon: 1 in the ramp's lit band, 1 - d
// in its darkest, between in proportion to the ramp; otherwise the side
// turned away and the cast shadow, over a short terminator. 1 when off or
// for an unstyled material (mode 8).
fn shadow_keep(sun: f32, mode: u32) -> f32 {
  let d = frame.outline_style.y;
  if (d <= 0.0 || (mode & 8u) != 0u) { return 1.0; }
  var lit = smoothstep(0.0, 0.25, sun);
  if (EFFECTS && material.toon.x > 0.5) {
    let dark = toon_ramp(0.0);
    lit = clamp((toon_ramp(sun) - dark) / max(1.0 - dark, 1e-3), 0.0, 1.0);
  }
  return 1.0 - d * (1.0 - lit);
}

// `receives`: the instance is darkened by the light's shadow map
// (look_receives).
fn local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, receives: bool) -> vec3<f32> {
  // A rect light (area.wgsl): linearly transformed cosines, no shadow map.
  if (light.color_kind.w < -0.5) {
    let a = area_shade(light, s, p, n, v);
    return (a.diffuse + a.specular) * select(1.0, rect_shadow(light, p, n), receives);
  }
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  // Inverse square, windowed to zero at the range (UE4's falloff).
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0) { return vec3<f32>(0.0); }
  if (dot(n, l) <= 0.0) { return vec3<f32>(0.0); }
  var shadow = 1.0;
  if (receives) { shadow = local_shadow(light, p, n); }
  // Its cookie (area.wgsl; white without).
  let radiance = light.color_kind.rgb * light_cookie(light, -l);
  if (EFFECTS && material.toon.x > 0.5) {
    return toon_light(s, n, v, l, shadow) * radiance * atten;
  }
  return brdf(s, n, v, l) * radiance * atten * shadow;
}

struct DecalResult {
  albedo: vec3<f32>,
  roughness: f32,
  emissive: vec3<f32>,
  // xyz: the decals' normal in the world, w: how much of it covers (0: none).
  normal: vec4<f32>,
};

// An sRGB atlas's texel back to the values stored (a normal map in it).
fn decal_stored(c: vec3<f32>) -> vec3<f32> {
  return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

// The decals that can cover the fragment at pixel `frag`, world `p`: x the
// first reference and y their count, or x NO_CLUSTER and y every record in
// a view without decal clusters (a reflection, a render-to-texture view, a
// frame with no light clusters).
fn frame_decals(frag: vec2<f32>, p: vec3<f32>) -> vec2<u32> {
  if (fx.decal_info.x < 0.0 || frame.cluster.x <= 0.0) {
    return vec2<u32>(NO_CLUSTER, u32(abs(fx.decal_info.x)));
  }
  let word = decal_cells[cluster_at(frag, p)];
  return vec2<u32>(word & 0xffffffu, word >> 24u);
}

// Atlas region `rect` (u0, v0, u1, v1) at `t` (0..1 across the box), with
// the box's screen gradients `gx`, `gy`. The level is picked here, not by
// the sampler, and capped where the region's smaller side still spans
// eight texels: past it a level's texels average the region with its
// neighbours (at a grazing angle a small decal reached the atlas's 1x1
// level, half the normal map's opaque blue). The sample stays inside the
// region by its bilinear reach at the upper level of the trilinear pair
// (1.5 texels: the taps' texels, whose own footprints may straddle a
// boundary not aligned to the level), so the image is never pinned or
// smeared by a large footprint.
fn decal_sample(rect: vec4<f32>, t: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(decal_atlas));
  let lo = min(rect.xy, rect.zw);
  let hi = max(rect.xy, rect.zw);
  let span = rect.zw - rect.xy;
  let dx = gx * span * size;
  let dy = gy * span * size;
  let footprint = max(dot(dx, dx), dot(dy, dy));
  let region = max(min(hi.x - lo.x, hi.y - lo.y) * min(size.x, size.y), 1.0);
  let cap = max(log2(region / 8.0), 0.0);
  let lod = clamp(0.5 * log2(max(footprint, 1e-8)), 0.0, cap);
  let reach = 1.5 * exp2(ceil(lod)) / size;
  let mid = (lo + hi) * 0.5;
  let uv = clamp(mix(rect.xy, rect.zw, t), min(lo + reach, mid), max(hi - reach, mid));
  return textureSampleLevel(decal_atlas, fx_sampler, uv, lod);
}

// Projected decals: the boxes covering the point blend their image over
// the surface's albedo (and roughness) before it is lit, in record order;
// those with a normal map bend the normal (DecalResult.normal, blended over
// the surface's own after its normal map).
fn apply_decals(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, albedo: vec3<f32>, roughness: f32, dpx: vec3<f32>, dpy: vec3<f32>) -> DecalResult {
  var r: DecalResult;
  r.albedo = albedo;
  r.roughness = roughness;
  r.emissive = vec3<f32>(0.0);
  r.normal = vec4<f32>(n, 0.0);
  let range = frame_decals(frag, p);
  let cells = CLUSTER_X * CLUSTER_Y * CLUSTER_Z;
  for (var k = 0u; k < range.y; k++) {
    var i = k;
    if (range.x != NO_CLUSTER) { i = decal_cells[cells + range.x + k]; }
    let d = decal_list[i];
    let local = (d.inv * vec4<f32>(p, 1.0)).xyz;
    if (any(abs(local) > vec3<f32>(0.5))) { continue; }
    let facing = dot(n, d.axis.xyz);
    let fade = smoothstep(d.axis.w, d.axis.w + fx.decal_info.z, facing) * (1.0 - smoothstep(0.35, 0.5, abs(local.z)));
    if (fade <= 0.0) { continue; }
    let t = vec2<f32>(local.x + 0.5, 0.5 - local.y);
    var texel = vec4<f32>(1.0);
    let gx = (d.inv * vec4<f32>(dpx, 0.0)).xy * vec2<f32>(1.0, -1.0);
    let gy = (d.inv * vec4<f32>(dpy, 0.0)).xy * vec2<f32>(1.0, -1.0);
    if (fx.decal_info.y > 0.5) {
      texel = decal_sample(d.uv_rect, t, gx, gy);
    }
    let c = d.color * texel;
    let a = clamp(c.a * fade, 0.0, 1.0);
    r.albedo = mix(r.albedo, c.rgb, a);
    if (d.emissive.w >= 0.0) { r.roughness = mix(r.roughness, d.emissive.w, a); }
    r.emissive += d.emissive.rgb * texel.rgb * a;
    if (d.normal.x != 0.0 && fx.decal_info.y > 0.5) {
      // The box's +x (u) and +y (up the image) in the world, on the surface.
      var m = decal_sample(d.normal_rect, t, gx, gy).xyz;
      if (fx.decal_info.y > 1.5) { m = decal_stored(m); }
      let ts = vec3<f32>((m.xy * 2.0 - 1.0) * d.normal.x, max(m.z * 2.0 - 1.0, 1e-3));
      let ux = vec3<f32>(d.inv[0].x, d.inv[1].x, d.inv[2].x);
      var tu = ux - n * dot(n, ux);
      if (dot(tu, tu) > 1e-12) {
        tu = normalize(tu);
        let tv = cross(n, tu);
        let bent = normalize(tu * ts.x + tv * ts.y + n * ts.z);
        // By the box's fade, not the image's alpha: the normal map's flat
        // texels leave the surface as it is, and its relief often lies where
        // the image is clear (a crater's slopes between its pit and rim).
        let w = clamp(fade, 0.0, 1.0);
        r.normal = vec4<f32>(normalize(mix(r.normal.xyz, bent, w)), max(r.normal.w, w));
      }
    }
  }
  return r;
}
