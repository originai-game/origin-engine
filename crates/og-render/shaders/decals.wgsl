// Shared projected decals. The receiving pass declares fx.decal_info,
// decal_atlas, fx_sampler, decal_list and decal_cells at its own group.
struct DecalGpu {
  inv: mat4x4<f32>,
  axis: vec4<f32>,
  uv_rect: vec4<f32>,
  color: vec4<f32>,
  emissive: vec4<f32>,
  normal_rect: vec4<f32>,
  normal: vec4<f32>,
};

struct DecalResult {
  albedo: vec3<f32>,
  roughness: f32,
  emissive: vec3<f32>,
  normal: vec4<f32>,
};

fn decal_stored(c: vec3<f32>) -> vec3<f32> {
  return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

fn frame_decals(frag: vec2<f32>, p: vec3<f32>) -> vec2<u32> {
  if (fx.decal_info.x < 0.0 || frame.cluster.x <= 0.0) {
    return vec2<u32>(NO_CLUSTER, u32(abs(fx.decal_info.x)));
  }
  let word = decal_cells[cluster_at(frag, p)];
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
      var m = decal_sample(d.normal_rect, t, gx, gy).xyz;
      if (fx.decal_info.y > 1.5) { m = decal_stored(m); }
      let ts = vec3<f32>((m.xy * 2.0 - 1.0) * d.normal.x, max(m.z * 2.0 - 1.0, 1e-3));
      let ux = vec3<f32>(d.inv[0].x, d.inv[1].x, d.inv[2].x);
      var tu = ux - n * dot(n, ux);
      if (dot(tu, tu) > 1e-12) {
        tu = normalize(tu);
        let tv = cross(n, tu);
        let bent = normalize(tu * ts.x + tv * ts.y + n * ts.z);
        let w = clamp(fade, 0.0, 1.0);
        r.normal = vec4<f32>(normalize(mix(r.normal.xyz, bent, w)), max(r.normal.w, w));
      }
    }
  }
  return r;
}
