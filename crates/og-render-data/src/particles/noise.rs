use super::*;

// ---------------------------------------------------------------------------
// Hashing and noise (mirrored in shaders/hash.wgsl and noise.wgsl).
// ---------------------------------------------------------------------------

pub(crate) use og_core::hash::pcg;

#[inline]
pub fn hash_unit(v: u32) -> f32 {
    og_core::hash::unit24(pcg(v))
}

/// A 32-bit seed from a 64-bit one.
pub fn hash_seed(seed: u64) -> u32 {
    pcg(seed as u32 ^ pcg((seed >> 32) as u32))
}

fn lattice3(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let h = (x as u32).wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ (z as u32).wrapping_mul(83_492_791)
        ^ seed;
    hash_unit(h)
}

/// Trilinear value noise in [0, 1) with a smoothstep fade.
pub fn value_noise3(p: Vec3, seed: u32) -> f32 {
    value_noise3_at(p, [0; 3], seed)
}

/// [`value_noise3`] at `p` plus the whole lattice cells `whole`: the noise
/// of a point far from the world's zero, taken from its place relative to
/// the render origin and the origin's whole cells (`origin.rs`).
pub fn value_noise3_at(p: Vec3, whole: [i32; 3], seed: u32) -> f32 {
    let fl = p.floor();
    let (ix, iy, iz) = (
        fl.x as i32 + whole[0],
        fl.y as i32 + whole[1],
        fl.z as i32 + whole[2],
    );
    let f = p - fl;
    let u = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let c = |dx, dy, dz| lattice3(ix + dx, iy + dy, iz + dz, seed);
    let x00 = c(0, 0, 0) + (c(1, 0, 0) - c(0, 0, 0)) * u.x;
    let x10 = c(0, 1, 0) + (c(1, 1, 0) - c(0, 1, 0)) * u.x;
    let x01 = c(0, 0, 1) + (c(1, 0, 1) - c(0, 0, 1)) * u.x;
    let x11 = c(0, 1, 1) + (c(1, 1, 1) - c(0, 1, 1)) * u.x;
    let y0 = x00 + (x10 - x00) * u.y;
    let y1 = x01 + (x11 - x01) * u.y;
    y0 + (y1 - y0) * u.z
}

/// Smooth 1D noise in -1..1 (camera shake).
pub fn noise1(t: f32, seed: u32) -> f32 {
    value_noise3(Vec3::new(t, 0.37, 0.71), seed) * 2.0 - 1.0
}

/// The random value `stream` of particle `n`.
#[inline]
pub(super) fn rand(seed: u32, n: u32, stream: u32) -> f32 {
    hash_unit(seed ^ pcg(n.wrapping_add(stream.wrapping_mul(0x9e37_79b9))))
}
