use super::*;

#[test]
fn arena_budget_has_exact_pixel_boundaries() {
    // 3x2 pixels, one count word and one seven-word node each:
    // 16 + 6 * 32. One byte less cannot hold a node for every pixel.
    assert_eq!(ArenaSize::new(3, 2, 4, 207), None);
    let one = ArenaSize::new(3, 2, 4, 208).unwrap();
    assert_eq!(one.nodes_per_pixel, 1);
    assert_eq!(one.bytes, 208);
    assert_eq!(one.header(), [3, 2, 1, 0]);
    // Two nodes need 16 + 6 * 60 = 376.
    assert_eq!(ArenaSize::new(3, 2, 4, 375).unwrap().nodes_per_pixel, 1);
    assert_eq!(ArenaSize::new(3, 2, 4, 376).unwrap().nodes_per_pixel, 2);
    assert_eq!(ArenaSize::new(3, 2, 1, 400).unwrap().nodes_per_pixel, 1);
    // The clear covers the overflow count and the six counts only.
    assert_eq!(one.cleared(), (12, 4 + 6 * 4));
    assert_eq!(ArenaSize::new(0, 2, 4, 1 << 20), None);
}

#[test]
fn the_device_budget_resolves_the_requested_layers() {
    // WebGPU's baseline storage binding (128 MiB).
    let limit = 128 * 1024 * 1024;
    assert_eq!(
        ArenaSize::new(1280, 720, 0, limit).unwrap().nodes_per_pixel,
        4
    );
    assert_eq!(
        ArenaSize::new(1920, 1080, 0, limit)
            .unwrap()
            .nodes_per_pixel,
        2
    );
    assert_eq!(ArenaSize::new(3840, 2160, 0, limit), None);
    let high = ArenaSize::new(3840, 2160, 4, 1024 * 1024 * 1024).unwrap();
    assert_eq!(high.nodes_per_pixel, 4);
    assert!(high.bytes <= 1024 * 1024 * 1024);
    assert_eq!(
        ArenaSize::new(1, 1, 100, limit).unwrap().nodes_per_pixel,
        MAX_TRANSPARENCY_LAYERS
    );
    // Word indices stay within u32 whatever the device allows.
    let huge = ArenaSize::new(16384, 16384, 32, u64::MAX).unwrap();
    assert!(huge.bytes <= u64::from(u32::MAX) * 4);
}

/// IEEE half precision, round to nearest even (what an RGBA16F blend
/// stores).
fn half(x: f32) -> f32 {
    let b = x.to_bits();
    let sign = (b >> 16) & 0x8000;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let mant = b & 0x7f_ffff;
    // Drops `shift` bits of `m`, rounding to nearest even.
    let round = |m: u32, shift: u32| {
        let kept = m >> shift;
        let rest = m & ((1 << shift) - 1);
        let halfway = 1 << (shift - 1);
        kept + u32::from(rest > halfway || (rest == halfway && kept & 1 == 1))
    };
    let h = if exp >= 31 {
        sign | 0x7c00
    } else if exp <= 0 {
        if exp < -10 {
            sign
        } else {
            sign | round(mant | 0x80_0000, (14 - exp) as u32)
        }
    } else {
        // A carry out of the mantissa is the next exponent's, as it
        // should be (and 0x7c00 is infinity).
        sign | round(((exp as u32) << 23) | mant, 13)
    };
    let (s, e, m) = (h >> 15, (h >> 10) & 31, h & 1023);
    let magnitude = match e {
        0 => m as f32 * 2f32.powi(-24),
        31 => f32::INFINITY,
        e => (1.0 + m as f32 / 1024.0) * 2f32.powi(e as i32 - 15),
    };
    if s == 1 { -magnitude } else { magnitude }
}

/// `oit.wgsl` `oit_weight`, on the CPU.
fn weight(alpha: f32, depth: f32) -> f32 {
    let a = (alpha * 10.0).min(1.0) + 0.01;
    let z = 1.0 - depth.clamp(0.0, 1.0) * 0.9;
    (a * a * a * 1e8 * z * z * z).clamp(1e-2, 3e3) / 3e3
}

/// One red channel through the weighted targets in `order`: the
/// colour sum, the weight sum and the revealage, each blend rounded
/// to half precision as RGBA16F and R16F store it.
fn accumulate(order: &[(f32, f32, f32)]) -> (f32, f32, f32) {
    let (mut color, mut weights, mut reveal) = (0.0f32, 0.0f32, 1.0f32);
    for &(red, alpha, depth) in order {
        let w = weight(alpha, depth);
        color = half(color + red * alpha * w);
        weights = half(weights + alpha * w);
        reveal = half(reveal * (1.0 - alpha));
    }
    (color, weights, reveal)
}

fn srgb_code(linear: f32) -> u8 {
    let x = linear.clamp(0.0, 1.0);
    let s = if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// The handoff's counterexample (OIT closing report): four fragments
/// (linear red, alpha, window depth) whose weighted result is not
/// the same in both orders, red 218 against 219 after the sRGB8
/// encode. The weighted fallback is held to the bound its RGBA16F
/// rounding allows ([`weighted_order_codes`]), at most one code a
/// channel, never to bit equality.
#[test]
// The report's values, as it gives them.
#[allow(clippy::excessive_precision)]
fn weighted_results_of_two_orders_stay_within_the_half_float_bound() {
    let forward = [
        (0.926_398_86, 0.691_721_2, 0.901_566_6),
        (0.289_832_96, 0.384_999_8, 0.392_899_4),
        (0.998_792_5, 0.580_259, 0.360_709_3),
        (0.428_052_75, 0.297_639_72, 0.048_268_097),
    ];
    let reverse: Vec<_> = forward.iter().rev().copied().collect();
    let f = accumulate(&forward);
    let r = accumulate(&reverse);
    // The sums the report gives for each order.
    assert_eq!(f, (1.458_984_4, 1.955_078_1, 0.055_908_203));
    assert_eq!(r, (1.459_960_9, 1.954_101_6, 0.055_877_686));
    let resolve = |(c, w, rev): (f32, f32, f32)| c / w * (1.0 - rev);
    let (a, b) = (resolve(f), resolve(r));
    assert_eq!((srgb_code(a), srgb_code(b)), (218, 219));
    let mean = f.0 / f.1;
    assert!((a - b).abs() <= weighted_order_bound(4, mean, f.2));
    let codes = weighted_order_codes(4, mean, f.2, a);
    assert_eq!(codes, 1, "the tolerance for four fragments here");
    assert!(u32::from(srgb_code(b).abs_diff(srgb_code(a))) <= codes);
}

#[test]
fn half_rounding_matches_ieee() {
    assert_eq!(half(1.0), 1.0);
    assert_eq!(half(65504.0), 65504.0);
    assert!(half(70000.0).is_infinite());
    // 1 + 2^-11 is a tie between 1 and 1 + 2^-10: to the even, 1.
    assert_eq!(half(1.0 + 2f32.powi(-11)), 1.0);
    assert_eq!(half(1.0 + 3.0 * 2f32.powi(-11)), 1.0 + 2.0 * 2f32.powi(-10));
    assert_eq!(half(2f32.powi(-24)), 2f32.powi(-24));
}
