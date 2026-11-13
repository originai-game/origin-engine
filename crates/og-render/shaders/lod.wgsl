// The level of detail rule (lod.rs), written once for every cull that
// picks a level on the GPU.

// lod.rs LOD_PIXEL_ERROR (a test holds the two equal).
const LOD_PIXEL_ERROR: f32 = 1.0;

// lod.rs `select` without hysteresis, a level at a time from the second:
// whether a level `error` from the full mesh still draws, seen at
// `per_unit` pixels per unit of length, at the instance's `scale` and
// `bias`. `worst` carries the largest error so far (0 before the first
// call): a level's error counts as at least the one before it. The caller
// stops at the first level that does not hold.
fn lod_holds(worst: ptr<function, f32>, error: f32, per_unit: f32, scale: f32, bias: f32) -> bool {
  *worst = max(*worst, error);
  return *worst * per_unit * scale * bias < LOD_PIXEL_ERROR;
}
