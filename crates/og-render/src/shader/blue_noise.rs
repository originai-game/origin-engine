//! The blue-noise threshold tile `DitherPattern::BlueNoise` reads
//! (stylise.wgsl `dither_at`): 64 × 64 thresholds, each of the 256 levels
//! on 16 texels, laid out by void and cluster (Ulichney 1993) so every
//! level's texels are spread as evenly as the tile allows and the tile
//! repeats without a seam. Made once by [`tests::void_and_cluster`] (a
//! deterministic routine: integer energies, a fixed seed, ties to the
//! lowest index) and compiled in as its 4096 bytes, uploaded as one
//! `R8Unorm` texture with the post passes; the test checks the committed
//! bytes are the routine's (`OG_BLESS_BLUE_NOISE=1` writes them again).

/// The tile's side in texels.
pub(crate) const BLUE_NOISE_SIZE: u32 = 64;

/// The thresholds, row-major, one byte a texel (`R8Unorm`).
pub(crate) static BLUE_NOISE: &[u8; (BLUE_NOISE_SIZE * BLUE_NOISE_SIZE) as usize] =
    include_bytes!("../../data/blue-noise-64.r8");
