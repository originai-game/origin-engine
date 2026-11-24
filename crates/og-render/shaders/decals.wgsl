// Shared projected decals. The receiving pass declares fx.decal_info,
// decal_atlas, fx_sampler and decal_data at its own group.
struct DecalGpu {
  inv: mat4x4<f32>,
  axis: vec4<f32>,
  uv_rect: vec4<f32>,
  color: vec4<f32>,
  emissive: vec4<f32>,
  normal_rect: vec4<f32>,
  normal: vec4<f32>,
};

const DECAL_VEC4S: u32 = 10u;

// Word `i` of the decals' clusters and references.
fn decal_cell(i: u32) -> u32 {
  let w = i + 4u;
  return decal_data[w >> 2u][w & 3u];
}

fn decal_at(i: u32) -> DecalGpu {
  let b = decal_data[0].x + i * DECAL_VEC4S;
  let f = array<vec4<f32>, 10>(
    bitcast<vec4<f32>>(decal_data[b]), bitcast<vec4<f32>>(decal_data[b + 1u]),
    bitcast<vec4<f32>>(decal_data[b + 2u]), bitcast<vec4<f32>>(decal_data[b + 3u]),
    bitcast<vec4<f32>>(decal_data[b + 4u]), bitcast<vec4<f32>>(decal_data[b + 5u]),
    bitcast<vec4<f32>>(decal_data[b + 6u]), bitcast<vec4<f32>>(decal_data[b + 7u]),
    bitcast<vec4<f32>>(decal_data[b + 8u]), bitcast<vec4<f32>>(decal_data[b + 9u]),
  );
  return DecalGpu(mat4x4<f32>(f[0], f[1], f[2], f[3]), f[4], f[5], f[6], f[7], f[8], f[9]);
}

struct DecalResult {
  albedo: vec3<f32>,
  roughness: f32,
  emissive: vec3<f32>,
  normal: vec4<f32>,
  coverage: f32,
};

fn frame_decals(frag: vec2<f32>, p: vec3<f32>) -> vec2<u32> {
  if (fx.decal_info.x < 0.0 || frame.cluster.x <= 0.0) {
    return vec2<u32>(NO_CLUSTER, u32(abs(fx.decal_info.x)));
  }
  let word = decal_cell(cluster_at(frag, p));
  return vec2<u32>(word & 0xffffffu, word >> 24u);
}

// Cap the mip footprint inside the atlas region, as on mesh receivers.
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

// Whether a pipeline has the decals' code (pipeline_features.rs).
override OG_DECALS: bool = true;

// A lit surface under the frame's decals: its colour and gloss, its normal
// bent by their normal maps, and what they emit.
struct Decaled {
  s: Surface,
  n: vec3<f32>,
  emissive: vec3<f32>,
};

// The decals over the surface `s` at `p` (pixel `frag`; `dpx` and `dpy`
// the point's change across the pixel, for the atlas' mip): `facing` is
// the direction a decal must project against to land (the geometric
// normal; the ground's for what stands on it), `n` the shading normal
// their normal maps bend. A metal's f0 follows its colour.
fn surface_decals(s_in: Surface, n: vec3<f32>, facing: vec3<f32>, frag: vec2<f32>, p: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> Decaled {
  var o = Decaled(s_in, n, vec3<f32>(0.0));
  if (abs(fx.decal_info.x) < 0.5) { return o; }
  let d = apply_decals(frag, p, facing, s_in.albedo, s_in.roughness, dpx, dpy, false);
  o.s.albedo = d.albedo;
  o.s.roughness = clamp(d.roughness, 0.04, 1.0);
  o.s.f0 = s_in.f0 + s_in.metallic * (d.albedo - s_in.albedo);
  o.emissive = d.emissive;
  o.n = decal_shading_normal(n, d.normal, d.coverage);
  return o;
}

// Normal composition is affine until the final normalization. A wet
// layer covers earlier bends and receiver detail; mix layers keep the
// original additive bend, including its exact zero-coverage arithmetic.
fn decal_shading_normal(n: vec3<f32>, bend: vec4<f32>, coverage: f32) -> vec3<f32> {
  if (coverage > 0.0) { return safe_normalize(n * (1.0 - coverage) + bend.xyz, n); }
  if (bend.w > 0.0) { return safe_normalize(n + bend.xyz, n); }
  return n;
}

// The prepass must see a wet decal's mirror too. Ordinary mix decals
// keep their existing prepass behavior so existing scenes do not move.
// xyz: shading normal, w: roughness.
fn wet_decal_gloss(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, facing: vec3<f32>, roughness: f32) -> vec4<f32> {
  let d = apply_decals(frag, p, facing, vec3<f32>(1.0), roughness, dpdxFine(p), dpdyFine(p), true);
  return vec4<f32>(decal_shading_normal(n, d.normal, d.coverage), d.roughness);
}

fn apply_decals(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, albedo: vec3<f32>, roughness: f32, dpx: vec3<f32>, dpy: vec3<f32>, wet_only: bool) -> DecalResult {
  var r: DecalResult;
  r.albedo = albedo;
  r.roughness = roughness;
  r.emissive = vec3<f32>(0.0);
  // xyz: the decals' normal maps' bend of `n`, summed; w: their reach.
  r.normal = vec4<f32>(0.0);
  r.coverage = 0.0;
  if (!OG_DECALS || abs(fx.decal_info.x) < 0.5) { return r; }
  let range = frame_decals(frag, p);
  let cells = CLUSTER_X * CLUSTER_Y * CLUSTER_Z;
  for (var k = 0u; k < range.y; k++) {
    var i = k;
    if (range.x != NO_CLUSTER) { i = decal_cell(cells + range.x + k); }
    let d = decal_at(i);
    let wet = d.normal.y > 0.5;
    if (wet_only && !wet) { continue; }
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
    if (wet) {
      r.albedo *= mix(vec3<f32>(1.0), c.rgb, a);
    } else {
      r.albedo = mix(r.albedo, c.rgb, a);
    }
    if (d.emissive.w >= 0.0) { r.roughness = mix(r.roughness, d.emissive.w, a); }
    r.emissive += d.emissive.rgb * texel.rgb * a;
    var target_normal = n;
    if (d.normal.x != 0.0 && fx.decal_info.y > 0.5) {
      var m = decal_sample(d.normal_rect, t, gx, gy).xyz;
      if (fx.decal_info.y > 1.5) { m = srgb_encode(m); }
      let ts = vec3<f32>((m.xy * 2.0 - 1.0) * d.normal.x, max(m.z * 2.0 - 1.0, 1e-3));
      let ux = vec3<f32>(d.inv[0].x, d.inv[1].x, d.inv[2].x);
      var tu = ux - n * dot(n, ux);
      if (dot(tu, tu) > 1e-12) {
        tu = normalize(tu);
        let tv = cross(n, tu);
        let bent = normalize(tu * ts.x + tv * ts.y + n * ts.z);
        target_normal = bent;
        // Weight by decal opacity * angle fade so transparent margins and
        // animated fade-outs leave the underlying surface normal untouched.
        if (!wet) { r.normal = vec4<f32>(r.normal.xyz + (bent - n) * a, max(r.normal.w, a)); }
      }
    }
    if (wet) {
      r.normal = vec4<f32>(mix(r.normal.xyz, target_normal, a), max(r.normal.w, a));
      r.coverage = mix(r.coverage, 1.0, a);
    }
  }
  return r;
}
