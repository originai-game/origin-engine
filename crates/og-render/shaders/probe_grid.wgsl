// Realtime light volumes (probe_grid.rs): one probe's capture, six faces
// drawn by the reflection probes' face cameras (probes.rs), projected onto
// first-order spherical harmonics and written over the probe in the light
// volumes' buffer (frame binding 10, lightmap.rs) as the irradiance over π
// of red, green and blue, `[c1.x, c1.y, c1.z, c0]` (`c0 + c1 · n`).

struct GridParams {
  // The probe's first vec4 in the buffer.
  at: u32,
  // Scales what it captured.
  intensity: f32,
  pad0: u32,
  pad1: u32,
};

@group(0) @binding(0) var capture: texture_2d_array<f32>;
@group(0) @binding(1) var<storage, read_write> probes: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> grid: GridParams;

const THREADS: u32 = 64u;
const INV_4PI: f32 = 0.0795774715;
const INV_2PI: f32 = 0.1591549431;

var<workgroup> partial: array<array<vec4<f32>, 3>, 64>;

// The cube direction of a face texel (ibl.wgsl's `face_dir`: u right, v
// up, texture row 0 at v = +1).
fn grid_face_dir(face: u32, u: f32, v: f32) -> vec3<f32> {
  switch face {
    case 0u: { return normalize(vec3<f32>(1.0, v, -u)); }
    case 1u: { return normalize(vec3<f32>(-1.0, v, u)); }
    case 2u: { return normalize(vec3<f32>(u, 1.0, -v)); }
    case 3u: { return normalize(vec3<f32>(u, -1.0, v)); }
    case 4u: { return normalize(vec3<f32>(u, v, 1.0)); }
    default: { return normalize(vec3<f32>(-u, v, -1.0)); }
  }
}

// The world direction a capture texel shows: the face cameras are
// right-handed, so each face holds its view mirrored (ibl.wgsl's
// `fs_probe_copy` reads it the same way).
fn grid_world_dir(m: vec3<f32>) -> vec3<f32> {
  let x_major = abs(m.x) >= max(abs(m.y), abs(m.z));
  return select(vec3<f32>(-m.x, m.y, m.z), vec3<f32>(m.x, m.y, -m.z), x_major);
}

@compute @workgroup_size(64)
fn cs_project(@builtin(local_invocation_index) t: u32) {
  let n = textureDimensions(capture).x;
  let per_face = n * n;
  let texels = per_face * 6u;
  let inv = 1.0 / f32(n);
  var acc = array<vec4<f32>, 3>(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
  for (var i = t; i < texels; i += THREADS) {
    let face = i / per_face;
    let r = i % per_face;
    let px = r % n;
    let py = r / n;
    let u = (f32(px) + 0.5) * inv * 2.0 - 1.0;
    let v = 1.0 - (f32(py) + 0.5) * inv * 2.0;
    let d = grid_world_dir(grid_face_dir(face, u, v));
    // The texel's solid angle.
    let w = 4.0 * inv * inv / pow(1.0 + u * u + v * v, 1.5);
    var l = textureLoad(capture, vec2<u32>(px, py), face, 0).rgb;
    // Nothing negative, infinite or NaN from a face reaches the probe.
    l = select(vec3<f32>(0.0), min(l, vec3<f32>(6.0e4)), l > vec3<f32>(0.0));
    let basis = vec4<f32>(d * INV_2PI, INV_4PI) * w;
    acc[0] += basis * l.r;
    acc[1] += basis * l.g;
    acc[2] += basis * l.b;
  }
  partial[t] = acc;
  workgroupBarrier();
  for (var s = THREADS / 2u; s > 0u; s = s >> 1u) {
    if (t < s) {
      for (var c = 0u; c < 3u; c++) {
        partial[t][c] += partial[t + s][c];
      }
    }
    workgroupBarrier();
  }
  if (t == 0u) {
    for (var c = 0u; c < 3u; c++) {
      probes[grid.at + c] = partial[0][c] * max(grid.intensity, 0.0);
    }
  }
}
