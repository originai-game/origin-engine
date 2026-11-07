// The standard lit fragment shader, after pbr.wgsl. The physical
// pipelines use physical.wgsl's instead.

// False in the lean pipelines (pbr_lean.wgsl) whose fragments take no
// screen derivative: a plain material has no alpha cut-off and no
// dissolve, so it never discards, and a shader without a discard keeps
// the early depth test (Apple's hidden surface removal among them): the
// same pixels, each shaded once. Where derivatives are taken (flat
// shading, the frame's decals) the discard stays even though it is never
// taken: on Metal a shader with a discard computes screen derivatives
// differently from one without (flat-shaded normals moved by up to
// 122/255, decal texels by 5/255 in og-gfx runs), and the lean pixels
// must be the full path's.
override DISCARD: bool = true;

// The visibility buffer's resolve (meshlet.wgsl) shades pixels of one
// full-screen pass whose 2x2 neighbours are other triangles, other
// batches' or no one's, so screen derivatives there mean nothing (a
// texture would sample its coarsest mip). With ANALYTIC the material's
// texture samples and the position's derivatives take the gradients the
// rasteriser computed for the pixel (`analytic_*`, set by the resolve).
override ANALYTIC: bool = false;
// d(uv)/dx and d(uv)/dy, d(world)/dx and d(world)/dy, and d(local)/dx
// and d(local)/dy (the material style's surface, set when it is on).
var<private> analytic_uv: vec4<f32>;
var<private> analytic_dx: vec3<f32>;
var<private> analytic_dy: vec3<f32>;
var<private> analytic_local_dx: vec3<f32>;
var<private> analytic_local_dy: vec3<f32>;

// The gradient along `d` (uv per pixel) of slot `slot`'s uv.
fn analytic_grad(slot: u32, uv: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
  return material_uv(slot, uv + d) - material_uv(slot, uv);
}

// A material map at slot `slot`'s uv (the analytic gradients with ANALYTIC).
fn material_sample(t: texture_2d<f32>, slot: u32, uv: vec2<f32>) -> vec4<f32> {
  let at = material_uv(slot, uv);
  if (ANALYTIC) {
    return textureSampleGrad(t, material_sampler, at, analytic_grad(slot, uv, analytic_uv.xy), analytic_grad(slot, uv, analytic_uv.zw));
  }
  return textureSample(t, material_sampler, at);
}

// How high a point lies in the material's relief for the weather (0 a
// hollow, 1 a crest): from its AO map, whose occluded texels are its
// crevices; 0.5 (none known) without one.
fn weather_cavity(uv: vec2<f32>) -> f32 {
  if (material.flags.y < 0.0) { return 0.5; }
  return 0.2 + 0.6 * material_sample(ao_tex, UV_AO, uv).r;
}

// A toon metal's reflection as bands: the environment along the reflected
// ray at the roughness's level, its brightness k against the flat colour
// (toon ambient's: the reflection straight up at the roughest level),
// stepped by the material's ramp at 0.45 k²: the sky (k about 1 and over)
// in the top band, the horizon in the middle one and the ground it
// reflects (k under about 0.4) in the darkest, scaled so the top band is
// 1. A cel-shaded metal is shaded as a matte surface whose colour is f0
// times those steps (toon_metal_lit), lit by the same ramp and flat
// ambient: only the reflection's brightness is stepped, so gold keeps its
// hue and saturation in every band, and the ground's dark band sits under
// the sky's (a metal matcap's grey bands times the base colour; Guilty
// Gear Xrd's and Genshin's stepped metals). Lit as a surface in f0 alone,
// chrome drew as a white ball and gold as cream. 1 where the environment
// is black: nothing to reflect.
fn toon_metal_bands(s: Surface, n: vec3<f32>, v: vec3<f32>, p: vec3<f32>) -> f32 {
  let w = vec3<f32>(0.2126, 0.7152, 0.0722);
  let flat_env = dot(reflected_light(vec3<f32>(0.0, 1.0, 0.0), p, frame.params.y), w);
  if (flat_env < 1e-3) { return 1.0; }
  let k = dot(reflected_light(reflect(-v, n), p, s.roughness * frame.params.y), w) / flat_env;
  return toon_ramp(0.45 * k * k) / max(toon_ramp(1.0), 1e-3);
}

// toon_aa for a fragment whose N·L towards the sun is `n_dot_l`: called in
// uniform control flow (screen derivatives); with ANALYTIC (the visibility
// buffer's resolve, whose neighbours are other triangles) a fixed hundredth
// per pixel.
fn toon_edges(n_dot_l: f32) {
  let px = frame.toon.x;
  if (ANALYTIC) {
    toon_aa = 0.01 * px;
  } else {
    toon_aa = clamp(fwidthFine(n_dot_l), 1e-4, 0.25) * px;
  }
}

fn world_dx(p: vec3<f32>) -> vec3<f32> {
  if (ANALYTIC) { return analytic_dx; }
  return dpdxFine(p);
}

fn world_dy(p: vec3<f32>) -> vec3<f32> {
  if (ANALYTIC) { return analytic_dy; }
  return dpdyFine(p);
}

@fragment
fn fs(in: VertexOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
  hook_front = select(-1.0, 1.0, front_facing);
  return lit_fragment(in);
}

// A blended material in the main camera's order-independent passes
// (oit.rs): the weighted targets, and with the linked lists its pixel's
// nodes too. The screen-space AO and contact shadows are the opaque
// surface's behind it, so they are left out (OIT_PASS).
@fragment
fn fs_oit(in: VertexOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  return oit_weighted(lit_fragment(in), in.clip.z);
}

@fragment
fn fs_oit_list(in: VertexOut, @builtin(front_facing) front_facing: bool, @builtin(sample_mask) mask: u32) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lit_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: VertexOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lit_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: VertexOut, @builtin(front_facing) front_facing: bool) {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lit_fragment(in);
  oit_front_mark(in.clip, c.a);
}

// The lit shading of a fragment (`fs`, and `fs_lean`).
fn lit_fragment(in: VertexOut) -> vec4<f32> {
  let uv = stylised_uv(in.uv, in.affine);
  let base_uv = material_uv(UV_MAP, uv);
  var base_sample: vec4<f32>;
  if (ANALYTIC) {
    let gx = analytic_grad(UV_MAP, uv, analytic_uv.xy);
    let gy = analytic_grad(UV_MAP, uv, analytic_uv.zw);
    base_sample = textureSampleGrad(base_color_tex, material_sampler, base_uv, gx, gy);
  } else {
    base_sample = textureSample(base_color_tex, material_sampler, base_uv);
  }
  // A material shader's hooks (identities with the built-in hooks).
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.origin = frame.origin.xyz;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  let params = shader_params();
  let base = surface_color(hook, params, look_tint(material.base_color * base_sample * in.color, in.look));
  var dissolve = vec4<f32>(0.0);
  if (EFFECTS) { dissolve = dissolve_at(uv, in.local_pos); }
  // One discard for the alpha cut-off and the dissolve (see dissolve_at);
  // none in most lean pipelines (DISCARD).
  let mode = u32(material.flags.w);
  // A closed, single-sided surface (mode 32): its shadows compare past its depth (lighting.wgsl shadow_closed).
  shadow_closed = (mode & 32u) != 0u;
  // Alpha to coverage (mode 16, an alpha-tested material): alpha past the cut-off over one pixel's change of it is the
  // share of samples covered (three.js's ALPHA_TO_COVERAGE alpha test); it is cut only where nothing is covered.
  let a2c = (mode & 16u) != 0u && material.emissive.w >= 0.0;
  let coverage = clamp((base.a - material.emissive.w) / max(fwidthFine(base.a), 1e-4) + 0.5, 0.0, 1.0);
  if (DISCARD && ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || dissolve.w > 0.5)) {
    discard;
  }
  let dissolve_edge = dissolve.rgb;
  // Blended materials keep their alpha; opaque ones write 1 (alpha-to-coverage ones their coverage).
  let alpha = select(select(1.0, coverage, a2c), base.a, (mode & 4u) != 0u);
  if (frame.light_color.w > 6.5) { return debug_unsupported(in.clip.xy); }
  if ((mode & 2u) != 0u) {
    // Unlit (MeshBasicMaterial): the colour plus the emissive term.
    let e = surface_emissive(hook, params, material.emissive.rgb * material_sample(emissive_tex, UV_EMISSIVE, uv).rgb);
    var unlit = base.rgb + e + dissolve_edge;
    if (EFFECTS && material.toon.w > 0.0) { unlit = mix(unlit, material.flash.rgb, material.toon.w); }
    unlit = look_flash(unlit, in.look);
    if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) {
      // Debug (DebugView::AmbientOcclusion): the ambient occlusion here, as on lit surfaces.
      let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
      return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
    }
    if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(base.rgb, alpha); }
    if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
    return vec4<f32>(apply_fog(unlit, in.world_pos), alpha);
  }
  let metalness_sample = material_sample(metalness_tex, UV_METALNESS, uv);
  let roughness_sample = material_sample(roughness_tex, UV_METALNESS, uv);
  let dpx = world_dx(in.world_pos);
  let dpy = world_dy(in.world_pos);
  var s: Surface;
  s.metallic = clamp(material.mr.x * metalness_sample.b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * roughness_sample.g, 0.04, 1.0);
  s.albedo = base.rgb;

  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    // Flat shading: the face normal from the position derivatives.
    n = normalize(cross(dpx, dpy));
  }
  // Perspective: towards the eye; orthographic: against the view direction.
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let n_vertex = n;
  let face = select(1.0, -1.0, dot(n, v) < 0.0);
  n = n * face;
  var decal_emissive = vec3<f32>(0.0);
  var decal_normal = vec4<f32>(0.0);
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) {
    let d = apply_decals(in.clip.xy, in.world_pos, n, s.albedo, s.roughness, dpx, dpy);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    decal_emissive = d.emissive;
    decal_normal = d.normal;
  }
  // three.js's geometryRoughness with RenderSettings.specular three (lighting.wgsl three_roughness).
  let turn = normal_turn(n_vertex);
  s.roughness = three_roughness(s.roughness, turn);
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  let normal_uv = material_uv(UV_NORMAL, uv);
  var normal_sample: vec4<f32>;
  if (ANALYTIC) {
    let gx = analytic_grad(UV_NORMAL, uv, analytic_uv.xy);
    let gy = analytic_grad(UV_NORMAL, uv, analytic_uv.zw);
    normal_sample = textureSampleGrad(normal_tex, material_sampler, normal_uv, gx, gy);
  } else {
    normal_sample = textureSample(normal_tex, material_sampler, normal_uv);
  }
  if (material.flags.x > 0.5) {
    var map = normal_sample.xyz * 2.0 - 1.0;
    if (material.flags.x > 1.5) {
      // Two-channel (BC5, EAC RG11): z from x and y.
      map.z = sqrt(max(1.0 - dot(map.xy, map.xy), 0.0));
    }
    map = vec3<f32>(map.xy * material.mr.zw, map.z);
    // The vertex tangent frame (glTF's: the file's tangents, the cook's
    // MikkTSpace ones, or generated at upload; scene.rs Vertex::tangent).
    var t = in.tangent.xyz - n_vertex * dot(n_vertex, in.tangent.xyz);
    if (dot(t, t) > 1e-12) {
      t = normalize(t);
      let b = cross(n_vertex, t) * select(1.0, -1.0, in.tangent.w < 0.0);
      // A back face flips the whole frame with its normal.
      n = normalize(mat3x3<f32>(t, b, n_vertex) * map) * face;
    }
  }
  n = surface_normal(hook, params, n);
  // The decals' normal maps over the surface's own (apply_decals).
  if (decal_normal.w > 0.0) { n = normalize(mix(n, decal_normal.xyz, decal_normal.w)); }
  // The material style's made surface (MaterialStyle.surface, frame.wgsl
  // material_surface), in the mesh's own space; a material with styled
  // false (mode 8) keeps its own. The branch is on uniforms only, so the
  // screen derivatives below are taken in uniform control flow.
  let ms = frame.material_surface;
  var wrap = 0.0;
  var wrap_tint = vec3<f32>(1.0);
  if (ms.x > 0.5 && (mode & 8u) == 0u) {
    let kind = u32(ms.x + 0.5);
    if (kind != 2u) {
      // Clay and toys are painted or moulded, not photographed: the base
      // texture's detail gives way to its local average (a coarse mip).
      // (A bias of 6 is the gradients 2^6 times longer.)
      var coarse_sample: vec4<f32>;
      if (ANALYTIC) {
        let gx = analytic_grad(UV_MAP, uv, analytic_uv.xy) * 64.0;
        let gy = analytic_grad(UV_MAP, uv, analytic_uv.zw) * 64.0;
        coarse_sample = textureSampleGrad(base_color_tex, material_sampler, base_uv, gx, gy);
      } else {
        coarse_sample = textureSampleBias(base_color_tex, material_sampler, base_uv, 6.0);
      }
      let coarse = look_tint(material.base_color * coarse_sample * in.color, in.look).rgb;
      s.albedo = mix(s.albedo, coarse, select(0.5, 0.7, kind == 3u) * ms.y);
    }
    // The pattern fades where it spans fewer than about four pixels, so a
    // far surface keeps its form instead of shimmering grain (the pixel's
    // footprint in the mesh's own space, where the pattern lies).
    var footprint: f32;
    if (ANALYTIC) {
      footprint = max(length(analytic_local_dx), length(analytic_local_dy));
    } else {
      footprint = max(length(dpdxFine(in.local_pos)), length(dpdyFine(in.local_pos)));
    }
    let fade = clamp((ms.z / max(footprint, 1e-6) - 3.0) / 6.0, 0.0, 1.0);
    let made = material_surface(kind, ms.y, in.local_pos / ms.z, s, fade);
    s = made.s;
    wrap = made.wrap;
    wrap_tint = made.tint;
    // The height (metres) bends the normal by its screen-space surface
    // gradient (Mikkelsen 2010): no tangents needed, so it holds on every
    // generated, decimated or bevelled mesh.
    let hm = made.height * ms.z;
    let r1 = cross(dpy, n);
    let r2 = cross(n, dpx);
    let det = dot(dpx, r1);
    // The height's screen derivatives: with ANALYTIC the pattern again
    // at the next pixel right and down on the triangle's plane, as the
    // hardware's quad takes them.
    var dh: vec2<f32>;
    if (ANALYTIC) {
      let hx = material_surface(kind, ms.y, (in.local_pos + analytic_local_dx) / ms.z, s, fade).height * ms.z;
      let hy = material_surface(kind, ms.y, (in.local_pos + analytic_local_dy) / ms.z, s, fade).height * ms.z;
      dh = vec2<f32>(hx - hm, hy - hm);
    } else {
      dh = vec2<f32>(dpdxFine(hm), dpdyFine(hm));
    }
    let grad = sign(det) * (dh.x * r1 + dh.y * r2);
    // det is n · (dpx × dpy): where the shading normal lies in the face (a
    // smooth mesh's silhouette, a rim turning away) the frame is degenerate,
    // abs(det) n - grad tends to zero and its normalize to NaN, which the
    // frame's blurs spread into a black disc. The bump fades out as the
    // normal turns into the face, and where nothing is left the normal stays.
    let facing = abs(det) / max(length(cross(dpx, dpy)), 1e-20);
    let bumped = abs(det) * n - grad * smoothstep(0.02, 0.1, facing);
    n = select(n, normalize(bumped), dot(bumped, bumped) > 1e-30);
    s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  }
  // Rain, streaks, puddles, snow and dust (weather_surface.wgsl): the
  // frame's uniforms, the material's weathering (uv[3].w) and its relief
  // from the AO map, whose crevices hold water and dust first.
  let weathered = weather_surface_at(s, n, normalize(n_vertex * face), in.world_pos, weather_cavity(uv), material.uv[3].w, WEATHER_STANDS);
  s = weathered.s;
  n = weathered.n;
  s.coat = material.coat.x;
  s.coat_roughness = three_roughness(clamp(material.coat.y, 0.04, 1.0), turn);
  s.coat_normal = n_vertex * face;
  s.translucency = material.coat.z;

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  let receives = look_receives(in.look);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && (n_dot_l > 0.0 || s.translucency > 0.0) && receives) {
    shadow = cascade_shadow(in.world_pos, leaf_shadow_normal(s, normalize(in.normal), l), abs(dot(n, l)));
  }
  if (EFFECTS && material.toon.x > 0.5) {
    toon_edges(dot(n, l));
    if (s.metallic > 0.0) {
      toon_metal_lit = toon_metal_bands(s, n, v, in.world_pos);
    }
  }
  var color: vec3<f32>;
  if (EFFECTS && material.toon.x > 0.5) {
    color = toon_light(s, n, v, l, shadow) * frame.light_color.rgb;
  } else {
    color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  }
  if (wrap > 0.0) {
    // Light wrapped past the terminator, as through a soft body or a
    // sheet (the surface's): energy-conserving wrap, (n.l + w) / (1 + w)²,
    // so the lit side gives up what the terminator gains; what is gained
    // is tinted as it passes. Shadowed where the face is lit.
    let raw = dot(n, l);
    let wrapped = max(raw + wrap, 0.0) / ((1.0 + wrap) * (1.0 + wrap));
    let extra = wrapped - max(raw, 0.0);
    let gain = max(extra, 0.0) * wrap_tint + vec3<f32>(min(extra, 0.0));
    color += s.albedo / PI * (1.0 - s.metallic) * gain * frame.light_color.rgb * select(1.0, shadow, raw > 0.0);
  }
  // Caustics where the surface lies under the frame's water (0 without).
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world_pos);
  // The material style's shadow darkness: where the sun does not reach, the
  // surface loses that share of the sun's ramp and of the ambient light.
  let keep = shadow_keep(n_dot_l * shadow, mode);
  color *= keep;
  // The material style's shadow tint: shadowed sunlight moves towards a
  // colour instead of black (the cool or warm shadow of illustration). A
  // toon material's tint takes what its ramp lost to the shadow, once, from
  // the final shadow factor (as the terrain's, terrain_lit.wgsl): inside the
  // dark band a deeper shadow loses nothing more and gains no more tint, so
  // overlapping shadows never come out lighter.
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0 && (mode & 8u) == 0u) {
    var tinted = (1.0 - shadow) * n_dot_l;
    var tint = frame.shadow_tint.rgb * frame.shadow_tint.w;
    if (EFFECTS && material.toon.x > 0.5) {
      tinted = toon_ramp(n_dot_l) - toon_ramp(n_dot_l * shadow);
      // Never more than the diffuse light the ramp took (1 / π of the albedo): a white tint at 1 fills the shadow, never outshines the lit side.
      tint = min(tint, vec3<f32>(1.0 / PI));
    }
    color += tinted * tint * s.albedo * frame.light_color.rgb;
  }

  // Point and spot lights: those of this fragment's cluster, in scene
  // order, or every light in a reflection probe's capture (probes.rs),
  // which has no clusters. One loop, so local_light is inlined once.
  let lights = frame_lights(in.clip.xy, in.world_pos);
  for (var k = 0u; k < lights.y; k++) {
    color += local_light(frame_light(lights, k), s, in.world_pos, n, v, receives);
  }

  // Image-based light (split sum).
  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  // The baked lightmap or light volumes where there are (lightmap.rs), at
  // the environment's intensity; the hemisphere light adds at its own.
  // The probe GI's one gather here (gi_sample.wgsl), for the diffuse
  // light along the ambient normal and the reflections' occlusion.
  let ambient_n = select(n, ambient_normal(in.clip.xy, n), EFFECTS);
  let gi = gi_sample_at(in.world_pos, ambient_n);
  let irradiance = ambient_irradiance(ambient_light_from(ambient_n, in.world_pos, gi), in.world_pos, n);
  var diffuse_ibl = (irradiance * frame.params.x + hemi_irradiance(n)) * s.albedo;
  // three.js's getIBLRadiance (r170): the reflection bent towards the normal by roughness squared before the prefiltered map is
  // read (rough surfaces reflect around their normal, not the mirror direction). A back-facing leaf card's normal turned
  // away from the eye (three's back-face rule) otherwise mirrors the bright sky behind it: grey-blue crowns.
  let r = normalize(mix(reflect(-v, n), n, s.roughness * s.roughness));
  let mip = s.roughness * frame.params.y;
  // Relit by the realtime light volumes where there are (lightmap.rs).
  var prefiltered = gi_reflected_from(volume_reflected(reflected_light(r, in.world_pos, mip), in.world_pos, r, s.roughness), r, in.world_pos, gi) * frame.params.x;
  // Screen-space reflections over it (ssr.rs), in the opaque main pass.
  // (Not a blended surface's: it lies over the one they were traced for.)
  if (!oit_pass && (u32(material.flags.w) & 4u) == 0u) { prefiltered = ssr_reflected(prefiltered, in.world_pos, in.clip.xy); }
  let lut = ibl_dfg(textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg, n_dot_v, s.roughness);
  // Split sum as Karis (and three.js): F0 scales the LUT, whose bias term already holds the Fresnel rise at grazing
  // angles; f_ibl * lut.x counted it twice (rough leaves seen edge-on reflected 3-4x the sky).
  var specular_ibl = prefiltered * (s.f0 * lut.x + lut.y);
  if (EFFECTS && material.toon.x > 0.5 && frame.toon.y < 1.0) {
    // Toon ambient (frame.toon.y 0): one flat colour, the light from above
    // and below averaged (MToon's equalised ambient), and the environment's
    // reflection as its roughest, flat colour at f0, so the ramp's bands sit
    // on flat colour instead of the PBR gradient that washed them out.
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let flat_irradiance = 0.5 * (
      ambient_irradiance(ambient_light_level(up, in.world_pos), in.world_pos, up) + ambient_irradiance(ambient_light_level(-up, in.world_pos), in.world_pos, -up));
    let flat_hemi = 0.5 * (frame.hemi_sky.rgb + frame.hemi_ground.rgb) * frame.hemi_sky.w;
    let flat_diffuse = (flat_irradiance * frame.params.x + flat_hemi) * s.albedo;
    // A metal takes the flat ambient as a matte surface in f0 times its
    // reflection's bands (toon_metal_bands), not the reflection's colour:
    // the ambient's colour and brightness, the metal's hue.
    let flat_specular = mix(
      reflected_light(up, in.world_pos, frame.params.y) * s.f0 * frame.params.x,
      (flat_irradiance * frame.params.x + flat_hemi) * s.f0 * toon_metal_lit,
      s.metallic);
    diffuse_ibl = mix(flat_diffuse, diffuse_ibl, frame.toon.y);
    specular_ibl = mix(flat_specular, specular_ibl, frame.toon.y);
  }
  let ao_sample = material_sample(ao_tex, UV_AO, uv).r;
  // What the material's AO and envMapIntensity keep of the image-based light (the environment's intensity is in
  // diffuse_ibl and specular_ibl already).
  var ibl_scale = material.flags.z * keep;
  if (material.flags.y >= 0.0) {
    // Ambient occlusion darkens only the image-based light.
    ibl_scale *= mix(1.0, ao_sample, material.flags.y);
  }
  // What `ao.z` keeps of the whole colour (three.js-era post AO multiplies everything, the emitted light too).
  var ao_all = 1.0;
  // Screen-space GI's bounce (ssgi.rs: the AO texture's gba with `ao.w`), the light a diffuse surface receives
  // from what the screen shows around it; the material's AO and the shadow darkness keep their share of it.
  var bounce = vec3<f32>(0.0);
  if (EFFECTS && fx.ao.x > 0.5 && !oit_pass) {
    // Screen-space ambient occlusion, likewise; `ao.z` lets it reach the direct and local light too, and moves the
    // image-based light to the same floor (0.35, as three.js-era post AO: the whole colour × mix(0.35, 1, ao)).
    let ao_texel = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0);
    let ssao_sample = ao_texel.r;
    if (fx.ao.w > 0.5 && fx.ao.w < 1.5) { bounce = ao_texel.gba * (ibl_scale / max(material.flags.z, 1e-4)); }
    ibl_scale *= mix(mix(1.0, ssao_sample, fx.ao.y), mix(0.35, 1.0, ssao_sample), fx.ao.z);
    ao_all = mix(1.0, mix(0.35, 1.0, ssao_sample), fx.ao.z);
    color *= ao_all;
  }
  var ibl = kd_ibl * diffuse_ibl + specular_ibl;
  // Branch on the material itself, not on `s`: Tint's uniformity analysis
  // takes all of `s` as varying once a member comes from a texture, and the
  // LUT is read without derivatives (it has one level).
  if (material.coat.x > 0.0) {
    // The coat's reflection of the environment, and what it takes from the base.
    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let fc = f_schlick(c_dot_v, vec3<f32>(0.04)).x * s.coat;
    let rc = reflect(-v, s.coat_normal);
    let coat_env = reflected_light(rc, in.world_pos, s.coat_roughness * frame.params.y);
    let coat_lut = ibl_dfg(textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness), 0.0).rg, c_dot_v, s.coat_roughness);
    ibl = ibl * (1.0 - fc) + coat_env * (0.04 * coat_lut.x + coat_lut.y) * s.coat * frame.params.x;
  }
  let dbg_direct = color;
  color += ibl * ibl_scale + kd_ibl * s.albedo * bounce;

  color += surface_emissive(hook, params, material.emissive.rgb * material_sample(emissive_tex, UV_EMISSIVE, uv).rgb) * ao_all;
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve_edge; }
  if (EFFECTS && material.toon.y > 0.0) {
    // Rim (fresnel) light at grazing angles.
    color += material.rim.rgb * material.toon.y * pow(1.0 - n_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }
  color = look_flash(color, in.look);

  if (frame.light_color.w > 2.5) {
    // Debug (DebugView::lit_code): 3 albedo, 4 diffuse image-based light, 5 specular image-based light, 6 direct light.
    let m = u32(frame.light_color.w + 0.5);
    if (m == 3u) { return vec4<f32>(s.albedo, 1.0); }
    if (m == 4u) { return vec4<f32>(kd_ibl * (diffuse_ibl * ibl_scale + s.albedo * bounce), 1.0); }
    if (m == 5u) { return vec4<f32>(specular_ibl * ibl_scale, 1.0); }
    return vec4<f32>(dbg_direct, 1.0);
  }
  if (frame.light_color.w > 1.5) {
    // Debug: the ambient occlusion term.
    let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
  }
  if (frame.light_color.w > 0.5) {
    // Debug: the cascade covering this pixel.
    return vec4<f32>(cascade_tint(cascade_index(in.world_pos)) * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, in.world_pos), alpha);
}


// A made surface: what it does to the shading surface, the height that
// bends its normal (in units of the pattern's scale) and its wrapped light.
struct MadeSurface {
  s: Surface,
  height: f32,
  wrap: f32,
  tint: vec3<f32>,
};

// MaterialStyle.surface at strength `k` on the point `p` (the mesh's own
// position over the pattern's scale): 1 clay, 2 paper, 3 plastic. `detail`
// (0 to 1) is how much of the pattern this pixel resolves: the height and
// the colour grain go with it, the light response stays.
fn material_surface(kind: u32, k: f32, p: vec3<f32>, surface: Surface, detail: f32) -> MadeSurface {
  let kd = k * detail;
  var o: MadeSurface;
  o.s = surface;
  o.tint = vec3<f32>(1.0);
  if (kind == 1u) {
    // Clay: soft lumps over two octaves, thumb-pressed dents, matte, the
    // light wrapping warm past the terminator.
    let lump = value_noise3(p, 11u) * 0.65 + value_noise3(p * 2.3 + 7.1, 12u) * 0.35;
    let dent = smoothstep(0.6, 0.85, value_noise3(p * 1.6 + 3.7, 13u));
    o.height = (lump * 0.45 - dent * 0.3) * kd;
    o.s.roughness = max(o.s.roughness, 0.75);
    o.s.metallic = 0.0;
    o.s.albedo = o.s.albedo * (1.0 + (lump - 0.5) * 0.25 * kd - dent * 0.12 * kd);
    o.wrap = 0.5 * k;
    o.tint = vec3<f32>(1.0, 0.72, 0.55);
  } else if (kind == 2u) {
    // Paper: a mottled fibre grain with streaks along one direction, in
    // the colour and the normal, fully rough, a little light through.
    let mottle = value_noise3(p, 21u);
    let fibre = value_noise3(p * vec3<f32>(3.0, 0.5, 3.0) + 1.3, 22u);
    let g = mottle * 0.55 + fibre * 0.45;
    o.height = g * 0.2 * kd;
    o.s.roughness = 1.0;
    o.s.metallic = 0.0;
    o.s.albedo = o.s.albedo * (1.0 + (g - 0.5) * 0.6 * kd);
    o.wrap = 0.25 * k;
    o.tint = vec3<f32>(1.0, 0.96, 0.88);
  } else {
    // Toy plastic or painted wood: a satin sheen, a little more colour,
    // a faint moulding ripple, a soft wrap.
    o.height = value_noise3(p, 31u) * 0.04 * kd;
    o.s.roughness = mix(o.s.roughness, clamp(o.s.roughness, 0.3, 0.45), k);
    o.s.metallic = o.s.metallic * (1.0 - k);
    let luma = dot(o.s.albedo, vec3<f32>(0.2126, 0.7152, 0.0722));
    o.s.albedo = max(mix(vec3<f32>(luma), o.s.albedo, 1.0 + 0.35 * k), vec3<f32>(0.0));
    o.wrap = 0.3 * k;
    o.tint = vec3<f32>(1.0, 0.9, 0.82);
  }
  return o;
}
