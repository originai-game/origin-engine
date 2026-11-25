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
fn material_sample(t: texture_2d<f32>, slot: u32, uv: vec2<f32>, histogram: u32) -> vec4<f32> {
  let at = material_uv(slot, uv);
  if (ANALYTIC) {
    return stochastic_texel(t, at, analytic_grad(slot, uv, analytic_uv.xy), analytic_grad(slot, uv, analytic_uv.zw), histogram);
  }
  return stochastic_texel(t, at, dpdxFine(at), dpdyFine(at), histogram);
}

// How high a point lies in the material's relief for the weather (0 a
// hollow, 1 a crest): from its AO map, whose occluded texels are its
// crevices; 0.5 (none known) without one.
fn weather_cavity(uv: vec2<f32>) -> f32 {
  if (material.flags.y < 0.0) { return 0.5; }
  return 0.2 + 0.6 * material_sample(ao_tex, UV_AO, uv, 5u).r;
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
  let flat_env = luma(reflected_light(vec3<f32>(0.0, 1.0, 0.0), p, frame.params.y));
  if (flat_env < 1e-3) { return 1.0; }
  let k = luma(reflected_light(reflect(-v, n), p, s.roughness * frame.params.y)) / flat_env;
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
  let mode = u32(material.flags.w);
  let uv = stylised_uv(in.uv, in.affine);
  let base_uv = material_uv(UV_MAP, uv);
  var base_sample = vec4<f32>(1.0);
  if ((mode & 64u) != 0u) {
    if (ANALYTIC) {
      let gx = analytic_grad(UV_MAP, uv, analytic_uv.xy);
      let gy = analytic_grad(UV_MAP, uv, analytic_uv.zw);
      base_sample = base_texel(base_uv, gx, gy);
    } else {
      base_sample = base_texel(base_uv, dpdxFine(base_uv), dpdyFine(base_uv));
    }
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
  hook.wind = frame.foliage_wind[0][0];
  let params = shader_params();
  let base = surface_color(hook, params, look_tint(season_base(base_sample, in.color, in.season), in.look));
  var dissolve = vec4<f32>(0.0);
  if (DISCARD && EFFECTS) { dissolve = dissolve_at(uv, in.local_pos); }
  // A closed, single-sided surface (mode 32): its shadows compare past its depth (lighting.wgsl shadow_closed).
  shadow_closed = (mode & 32u) != 0u;
  // Alpha to coverage (mode 16, an alpha-tested material): alpha past the cut-off over one pixel's change of it is the
  // share of samples covered (three.js's ALPHA_TO_COVERAGE alpha test); it is cut only where nothing is covered.
  let a2c = DISCARD && (mode & 16u) != 0u && material.emissive.w >= 0.0;
  var coverage = 1.0;
  if (DISCARD) {
    if (!ANALYTIC) {
      coverage = clamp((base.a - material.emissive.w) / max(fwidthFine(base.a), 1e-4) + 0.5, 0.0, 1.0);
    } else if (material.emissive.w >= 0.0) {
      coverage = select(0.0, 1.0, base.a >= material.emissive.w);
    }
    // An instance fading in is cut by the dither (common.wgsl `dither_cut`).
    let hidden = dither_cut(look_hidden(in.look), in.clip.xy, frame.drawn.w);
    if ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || dissolve.w > 0.5 || hidden) {
      discard;
    }
  }
  let dissolve_edge = dissolve.rgb;
  // Blended materials keep their alpha; opaque ones write 1 (alpha-to-coverage ones their coverage).
  let alpha = select(select(1.0, coverage, a2c), base.a, (mode & 4u) != 0u);
  if (frame.light_color.w > 6.5) { return debug_unsupported(in.clip.xy); }
  if ((mode & 2u) != 0u) {
    // Unlit (MeshBasicMaterial): the colour plus the emissive term.
    var emissive_in = material.emissive.rgb;
    if ((mode & 512u) != 0u && any(emissive_in != vec3<f32>(0.0))) {
      emissive_in *= material_sample(emissive_tex, UV_EMISSIVE, uv, 4u).rgb;
    }
    let e = surface_emissive(hook, params, emissive_in);
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
  var metalness_factor = material.mr.x;
  if ((mode & 128u) != 0u && metalness_factor > 0.0) {
    metalness_factor *= material_sample(metalness_tex, UV_METALNESS, uv, 1u).b;
  }
  var roughness_factor = material.mr.y;
  if ((mode & 256u) != 0u) {
    roughness_factor *= material_sample(roughness_tex, UV_METALNESS, uv, 2u).g;
  }
  let dpx = world_dx(in.world_pos);
  let dpy = world_dy(in.world_pos);
  var s: Surface;
  s.metallic = clamp(surface_metalness(hook, params, metalness_factor), 0.0, 1.0);
  s.roughness = clamp(surface_roughness(hook, params, roughness_factor), 0.04, 1.0);
  s.albedo = base.rgb;

  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    // Framebuffer y runs down: undo its winding, then recover the mesh's
    // oriented normal before applying the material's side below.
    n = -normalize(cross(dpx, dpy)) * hook_front;
  }
  // Perspective: towards the eye; orthographic: against the view direction.
  let v = toward_eye(in.world_pos);
  let n_vertex = n;
  let face = material_face(hook_front);
  n = n * face;
  var decal_emissive = vec3<f32>(0.0);
  var decal_normal = vec4<f32>(0.0);
  var decal_coverage = 0.0;
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) {
    let d = apply_decals(in.clip.xy, in.world_pos, n, s.albedo, s.roughness, s.metallic, dpx, dpy, false);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    s.metallic = clamp(d.metallic, 0.0, 1.0);
    decal_emissive = d.emissive;
    decal_normal = d.normal;
    decal_coverage = d.coverage;
  }
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  if (material.flags.x > 0.5) {
    let normal_uv = material_uv(UV_NORMAL, uv);
    var normal_sample: vec4<f32>;
    if (ANALYTIC) {
      let gx = analytic_grad(UV_NORMAL, uv, analytic_uv.xy);
      let gy = analytic_grad(UV_NORMAL, uv, analytic_uv.zw);
      normal_sample = normal_texel(normal_uv, gx, gy);
    } else {
      normal_sample = normal_texel(normal_uv, dpdxFine(normal_uv), dpdyFine(normal_uv));
    }
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
  n = decal_shading_normal(n, decal_normal, decal_coverage);
  // The material style's made surface (MaterialStyle.surface, frame.wgsl
  // material_surface), in the mesh's own space; a material with styled
  // false (mode 8) keeps its own. The branch is on uniforms only, so the
  // screen derivatives below are taken in uniform control flow.
  let ms = frame.material_surface;
  var wrap = 0.0;
  var wrap_tint = vec3<f32>(1.0);
  if (OG_SURFACE) {
    if (ms.x > 0.5 && (mode & 8u) == 0u) {
      let kind = u32(ms.x + 0.5);
      if (kind != 2u && (mode & 64u) != 0u) {
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
        let coarse = look_tint(season_base(coarse_sample, in.color, in.season), in.look).rgb;
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
  }
  let geo_n = normalize(n_vertex * face);
  s = material_wear(s, in.uv, geo_n, in.world_pos, length(dpx) + length(dpy));
  // Rain, streaks, puddles, snow and dust (weather_surface.wgsl): the
  // frame's uniforms, the material's weathering (uv[3].w) and its relief
  // from the AO map, whose crevices hold water and dust first.
  var weathered: Weathered;
  if (ANALYTIC) {
    weathered = weather_surface_grad(s, n, geo_n, in.world_pos, weather_cavity(uv), material.uv[3].w, select(WEATHER_STANDS, WEATHER_MOVES, in.moves != 0u), length(dpx) + length(dpy), 0.0);
  } else {
    weathered = weather_surface_at(s, n, geo_n, in.world_pos, weather_cavity(uv), material.uv[3].w, select(WEATHER_STANDS, WEATHER_MOVES, in.moves != 0u));
  }
  s = weathered.s;
  n = weathered.n;
  if (!ANALYTIC) {
    s.roughness = specular_aa(n, s.roughness, dpdxFine(n), dpdyFine(n));
  }
  s.coat = material.coat.x;
  s.coat_roughness = clamp(material.coat.y, 0.04, 1.0);
  s.coat_normal = geo_n;
  s.translucency = material.coat.z;

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  let receives = look_receives(in.look);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && (n_dot_l > 0.0 || s.translucency > 0.0) && receives) {
    shadow = cascade_shadow(in.world_pos, leaf_shadow_normal(s, geo_n, l), abs(dot(n, l)));
  }
  // Inside a tree's crown, what its leaves let through (crown_sun).
  let map_shadow = shadow;
  shadow = crown_sun(shadow, in.crown.y);
  if (EFFECTS && OG_TOON && material.toon.x > 0.5) {
    toon_edges(dot(n, l));
    if (s.metallic > 0.0) {
      toon_metal_lit = toon_metal_bands(s, n, v, in.world_pos);
    }
  }
  var color: vec3<f32>;
  if (EFFECTS && OG_TOON && material.toon.x > 0.5) {
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
  // What the crown's leaves pass on of the sun they took (crown_passed_sun).
  color += crown_passed_sun(s, map_shadow, in.crown.y);
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
    if (EFFECTS && OG_TOON && material.toon.x > 0.5) {
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
    color += local_light(frame_light(lights, k), s, in.world_pos, n, v, in.clip.xy, receives);
  }

  // The light around it (surface_shade.wgsl): every lit surface's one
  // gather. Screen-space reflections only in the opaque main pass (a
  // blended surface lies over the one they were traced for).
  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);
  let at = ShadeAt(in.world_pos, n, v, in.clip.xy, EFFECTS, !oit_pass && (mode & 4u) == 0u, false);
  // A leaf takes the light around it along its canopy's normal (its
  // material's foliage, coat.w), but its own face mirrors the sky: a
  // crown of leaves, each at its own angle, has no grazing rim, and an
  // inner leaf whose canopy normal lies across the view would otherwise
  // take the rim's Fresnel sheen of the sky.
  var facet = n;
  if (material.coat.w > 0.0) {
    let face = normalize(cross(dpx, dpy));
    facet = normalize(mix(n, face * select(-1.0, 1.0, dot(face, v) >= 0.0), clamp(material.coat.w, 0.0, 1.0)));
  }
  var around = surface_ambient(s, at, facet);
  let r_base = normalize(mix(reflect(-v, facet), facet, s.roughness * s.roughness));
  around.specular *= specular_horizon_occlusion(r_base, geo_n);
  // A leaf lets the light around it through from behind, as it does the
  // sun's (lighting.wgsl leaf_transmission): the sky and ground behind it,
  // tinted by its colour.
  if (material.coat.z > 0.0) {
    let back = -n;
    let behind = ambient_irradiance(ambient_light_level(back, in.world_pos) * frame.params.x, in.world_pos, back) + hemi_irradiance(back);
    around.diffuse += behind * s.albedo * s.translucency;
  }
  around = crown_ambient(around, s, clamp(dot(facet, v), 1e-4, 1.0), in.crown.x);
  if (EFFECTS && OG_TOON && material.toon.x > 0.5 && frame.toon.y < 1.0) {
    // Toon ambient (frame.toon.y 0): one flat colour, the light from above
    // and below averaged (MToon's equalised ambient), and the environment's
    // reflection as its roughest, flat colour at f0, so the ramp's bands sit
    // on flat colour instead of the PBR gradient that washed them out.
    let up = vec3<f32>(0.0, 1.0, 0.0);
    // From above, then below, in one loop (one copy of the light
    // volumes' code: DXC inlines every call).
    var flat_sum = vec3<f32>(0.0);
    for (var j = 0u; j < 2u; j++) {
      let d = select(-up, up, j == 0u);
      flat_sum += ambient_irradiance(ambient_light_level(d, in.world_pos) * frame.params.x, in.world_pos, d);
    }
    let flat_irradiance = 0.5 * flat_sum;
    let flat_hemi = 0.5 * (frame.hemi_sky.rgb + frame.hemi_ground.rgb) * frame.hemi_sky.w;
    let flat_diffuse = (flat_irradiance + flat_hemi) * s.albedo;
    // A metal takes the flat ambient as a matte surface in f0 times its
    // reflection's bands (toon_metal_bands), not the reflection's colour:
    // the ambient's colour and brightness, the metal's hue.
    let flat_specular = mix(
      reflected_light(up, in.world_pos, frame.params.y) * s.f0 * frame.params.x,
      (flat_irradiance + flat_hemi) * s.f0 * toon_metal_lit,
      s.metallic);
    around.diffuse = mix(flat_diffuse, around.diffuse, frame.toon.y);
    around.specular = mix(flat_specular, around.specular, frame.toon.y);
  }
  // Branch on the material itself, not on `s`: Tint's uniformity analysis
  // takes all of `s` as varying once a member comes from a texture, and the
  // LUT is read without derivatives (it has one level).
  if (material.coat.x > 0.0) {
    // The coat's reflection of the environment, occluded by probe GI and
    // combined with screen-space reflections like the base layer.
    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let rc = normalize(mix(reflect(-v, s.coat_normal), s.coat_normal, s.coat_roughness * s.coat_roughness));
    let coat_gi = gi_sample_at(in.world_pos, s.coat_normal);
    var coat_env = surface_reflected(reflected_light(rc, in.world_pos, s.coat_roughness * frame.params.y) * frame.params.x, at, rc, s.coat_roughness, coat_gi);
    if (at.opaque) { coat_env = ssr_reflected(coat_env, in.world_pos, in.clip.xy); }
    let coat_lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness), 0.0).rg;
    let coat_f = (0.04 * coat_lut.x + coat_lut.y) * s.coat;
    let coat_spec_ao = specular_occlusion(c_dot_v, around.occlusion.ambient, s.coat_roughness);
    let coat_horizon = specular_horizon_occlusion(rc, geo_n);
    around.diffuse *= 1.0 - coat_f;
    around.specular = around.specular * (1.0 - coat_f)
      + coat_env * coat_f * coat_horizon * (coat_spec_ao / max(around.occlusion.ambient, 1e-4));
  }
  // What the material's AO map keeps of the light around it (it darkens
  // only that), with the style's shadow darkness.
  var ao = 1.0;
  if (material.flags.y >= 0.0) {
    ao = mix(1.0, material_sample(ao_tex, UV_AO, uv, 5u).r, material.flags.y);
  }
  let kept = keep * clamp(surface_ao(hook, params, ao), 0.0, 1.0);
  var emissive_in = material.emissive.rgb;
  if ((mode & 512u) != 0u && any(emissive_in != vec3<f32>(0.0))) {
    emissive_in *= material_sample(emissive_tex, UV_EMISSIVE, uv, 4u).rgb;
  }
  let emitted = surface_emissive(hook, params, emissive_in);
  let shaded = surface_sum(s, color, around, emitted, material.flags.z, kept);
  color = shaded_color(shaded);

  if (EFFECTS && abs(fx.decal_info.x) > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve_edge; }
  if (EFFECTS && material.toon.y > 0.0) {
    // Silhouette light follows the underlying surface, not the virtual
    // creases of a normal map, procedural bump, decal or weather layer.
    let rim_dot_v = clamp(dot(n_vertex * face, v), 0.0, 1.0);
    color += material.rim.rgb * material.toon.y * pow(1.0 - rim_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }
  color = look_flash(color, in.look);

  if (frame.light_color.w > 2.5) {
    return vec4<f32>(shaded_debug(shaded, s.albedo, u32(frame.light_color.w + 0.5)), 1.0);
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
