//! Deterministic math on fixed inputs.
//!
//! The vectors are the correctly rounded f32 results, produced independently
//! of this crate (CPython's `math` in f64, then rounded once to f32). Every
//! host must reproduce them bit for bit; that is the cross-host contract.

use og_core::Rng;
use og_core::math::{
    Quat, Vec3, acos, atan, atan2, cos, exp, ln, pow, quat_from_axis_angle, quat_slerp, sin,
    sin_cos,
};

const SIN: &[(u32, u32)] = &[
    (0x00000000, 0x00000000), // sin(0.0)
    (0x0da24260, 0x0da24260), // sin(1.0000000031710769e-30)
    (0x3727c5ac, 0x3727c5ac), // sin(9.999999747378752e-06)
    (0x3f000000, 0x3ef57744), // sin(0.5)
    (0x3f800000, 0x3f576aa4), // sin(1.0)
    (0xbf800000, 0xbf576aa4), // sin(-1.0)
    (0x3f490fdb, 0x3f3504f3), // sin(0.7853981852531433)
    (0x3fc90fdb, 0x3f800000), // sin(1.5707963705062866)
    (0x40490fdb, 0xb3bbbd2e), // sin(3.1415927410125732)
    (0x40000000, 0x3f68c7b7), // sin(2.0)
    (0x40400000, 0x3e1081c3), // sin(3.0)
    (0xc0200000, 0xbf193578), // sin(-2.5)
    (0x41200000, 0xbf0b44f8), // sin(10.0)
    (0x42c80000, 0xbf01a12e), // sin(100.0)
    (0x447a0000, 0x3f53ae61), // sin(1000.0)
    (0x4640e6b6, 0xbf344b08), // sin(12345.677734375)
    (0x47c35000, 0x3d126d55), // sin(100000.0)
    (0x49742400, 0xbeb33259), // sin(1000000.0)
    (0x4affffff, 0x3f4fc8cd), // sin(8388607.5)
    (0x4b7fffff, 0xbf72bf60), // sin(16777215.0)
    (0x4b800000, 0xbf47917c), // sin(16777216.0)
    (0x4b189680, 0x3ed7520a), // sin(10000000.0)
    (0x4e26e49c, 0x3f5bcbf6), // sin(700000000.0)
    (0x501502f9, 0xbef99a64), // sin(10000000000.0)
    (0x60ad78ec, 0x3f281569), // sin(1.0000000200408773e+20)
    (0x7f61b1e6, 0x3f5ff9c4), // sin(3.0000000054977558e+38)
    (0xff61b1e6, 0xbf5ff9c4), // sin(-3.0000000054977558e+38)
];
const COS: &[(u32, u32)] = &[
    (0x00000000, 0x3f800000), // cos(0.0)
    (0x0da24260, 0x3f800000), // cos(1.0000000031710769e-30)
    (0x3727c5ac, 0x3f800000), // cos(9.999999747378752e-06)
    (0x3f000000, 0x3f60a940), // cos(0.5)
    (0x3f800000, 0x3f0a5140), // cos(1.0)
    (0xbf800000, 0x3f0a5140), // cos(-1.0)
    (0x3f490fdb, 0x3f3504f3), // cos(0.7853981852531433)
    (0x3fc90fdb, 0xb33bbd2e), // cos(1.5707963705062866)
    (0x40490fdb, 0xbf800000), // cos(3.1415927410125732)
    (0x40000000, 0xbed51133), // cos(2.0)
    (0x40400000, 0xbf7d7026), // cos(3.0)
    (0xc0200000, 0xbf4d17bf), // cos(-2.5)
    (0x41200000, 0xbf56cd64), // cos(10.0)
    (0x42c80000, 0x3f5cc0ee), // cos(100.0)
    (0x447a0000, 0x3f0ff813), // cos(1000.0)
    (0x4640e6b6, 0x3f35be20), // cos(12345.677734375)
    (0x47c35000, 0xbf7fd61c), // cos(100000.0)
    (0x49742400, 0x3f6fcefd), // cos(1000000.0)
    (0x4affffff, 0xbf1589c3), // cos(8388607.5)
    (0x4b7fffff, 0xbea29962), // cos(16777215.0)
    (0x4b800000, 0x3f2056b4), // cos(16777216.0)
    (0x4b189680, 0xbf6842df), // cos(10000000.0)
    (0x4e26e49c, 0x3f033ede), // cos(700000000.0)
    (0x501502f9, 0x3f5f84c5), // cos(10000000000.0)
    (0x60ad78ec, 0x3f411723), // cos(1.0000000200408773e+20)
    (0x7f61b1e6, 0xbef7f57c), // cos(3.0000000054977558e+38)
    (0xff61b1e6, 0xbef7f57c), // cos(-3.0000000054977558e+38)
];
const ATAN: &[(u32, u32)] = &[
    (0x00000000, 0x00000000), // atan(0.0)
    (0x1e3ce508, 0x1e3ce508), // atan(9.999999682655225e-21)
    (0x3dcccccd, 0x3dcc1f14), // atan(0.10000000149011612)
    (0x3e892a30, 0x3e86048d), // atan(0.2678999900817871)
    (0x3e99999a, 0x3e9539d4), // atan(0.30000001192092896)
    (0x3f000000, 0x3eed6338), // atan(0.5)
    (0x3f13cd36, 0x3f060a8f), // atan(0.5773500204086304)
    (0x3f800000, 0x3f490fdb), // atan(1.0)
    (0xbf800000, 0xbf490fdb), // atan(-1.0)
    (0x3fc00000, 0x3f7b985f), // atan(1.5)
    (0x40000000, 0x3f8db70d), // atan(2.0)
    (0x41200000, 0x3fbc4de9), // atan(10.0)
    (0xc2f6cccd, 0xbfc80651), // atan(-123.4000015258789)
    (0x501502f9, 0x3fc90fdb), // atan(10000000000.0)
    (0x7f61b1e6, 0x3fc90fdb), // atan(3.0000000054977558e+38)
];
const ATAN2: &[(u32, u32, u32)] = &[
    (0x3f800000, 0x3f800000, 0x3f490fdb), // atan2(1.0, 1.0)
    (0x3f800000, 0xbf800000, 0x4016cbe4), // atan2(1.0, -1.0)
    (0xbf800000, 0xbf800000, 0xc016cbe4), // atan2(-1.0, -1.0)
    (0xbf800000, 0x3f800000, 0xbf490fdb), // atan2(-1.0, 1.0)
    (0x3f000000, 0x40000000, 0x3e7adbb0), // atan2(0.5, 2.0)
    (0x40400000, 0xbe800000, 0x3fd3b43a), // atan2(3.0, -0.25)
    (0x0da24260, 0x3f800000, 0x0da24260), // atan2(1.0000000031710769e-30, 1.0)
    (0x7149f2ca, 0x0da24260, 0x3fc90fdb), // atan2(1.0000000150474662e+30, 1.0000000031710769e-30)
    (0xc0000000, 0x3a83126f, 0xbfc8ff78), // atan2(-2.0, 0.0010000000474974513)
    (0x40e00000, 0x40e000d2, 0x3f490f63), // atan2(7.0, 7.000100135803223)
    (0xb8d1b717, 0xc0a00000, 0xc0490f87), // atan2(-9.999999747378752e-05, -5.0)
    (0x42f6e979, 0xc4454000, 0x403f20d8), // atan2(123.45600128173828, -789.0)
];
const EXP: &[(u32, u32)] = &[
    (0x00000000, 0x3f800000), // exp(0.0)
    (0x322bcc77, 0x3f800000), // exp(9.99999993922529e-09)
    (0x3f000000, 0x3fd3094c), // exp(0.5)
    (0x3f800000, 0x402df854), // exp(1.0)
    (0xbf800000, 0x3ebc5ab2), // exp(-1.0)
    (0x40135d8d, 0x411ffffe), // exp(2.3025848865509033)
    (0x41200000, 0x46ac14ee), // exp(10.0)
    (0xc1200000, 0x383e6bce), // exp(-10.0)
    (0x42480000, 0x638c881f), // exp(50.0)
    (0x42b00000, 0x7ef882b7), // exp(88.0)
    (0x42b170a4, 0x7f7f4648), // exp(88.72000122070312)
    (0x42b20000, 0x7f800000), // exp(89.0)
    (0xc2ae0000, 0x00b33687), // exp(-87.0)
    (0xc2c80000, 0x0000001b), // exp(-100.0)
    (0xc2cfcccd, 0x00000001), // exp(-103.9000015258789)
    (0xc2dc0000, 0x00000000), // exp(-110.0)
];
const LN: &[(u32, u32)] = &[
    (0x3f800000, 0x00000000), // ln(1.0)
    (0x40000000, 0x3f317218), // ln(2.0)
    (0x3f000000, 0xbf317218), // ln(0.5)
    (0x41200000, 0x40135d8e), // ln(10.0)
    (0x0da24260, 0xc28a27b5), // ln(1.0000000031710769e-30)
    (0x7f61b1e6, 0x42b13196), // ln(3.0000000054977558e+38)
    (0x3f800001, 0x33ffffff), // ln(1.0000001192092896)
    (0x3f7fffff, 0xb3800000), // ln(0.9999999403953552)
    (0x00000001, 0xc2ce8ed0), // ln(1.401298464324817e-45)
    (0x40e00000, 0x3ff91395), // ln(7.0)
    (0x3fb504f3, 0x3eb17217), // ln(1.4142135381698608)
    (0x47f12065, 0x413b940e), // ln(123456.7890625)
];
const POW: &[(u32, u32, u32)] = &[
    (0x40000000, 0x41200000, 0x44800000), // pow(2.0, 10.0)
    (0x40000000, 0x3f000000, 0x3fb504f3), // pow(2.0, 0.5)
    (0x41200000, 0xc0400000, 0x3a83126f), // pow(10.0, -3.0)
    (0x3f000000, 0x40533333, 0x3dcfefc6), // pow(0.5, 3.299999952316284)
    (0x3f800347, 0x461c4000, 0x402dfd7e), // pow(1.000100016593933, 10000.0)
    (0xc0000000, 0x40400000, 0xc1000000), // pow(-2.0, 3.0)
    (0xc0000000, 0x40000000, 0x40800000), // pow(-2.0, 2.0)
    (0x40400000, 0x3eaaaaaa, 0x3fb89ba2), // pow(3.0, 0.3333333134651184)
    (0x501502f9, 0x40400000, 0x7149f2ca), // pow(10000000000.0, 3.0)
    (0x40e00000, 0xc2200000, 0x0750c653), // pow(7.0, -40.0)
    (0x3f666666, 0x42c90000, 0x37d360ea), // pow(0.8999999761581421, 100.5)
    (0x40000000, 0xc3150000, 0x00000001), // pow(2.0, -149.0)
    (0x3fc00000, 0xc0100000, 0x3ecd9ea2), // pow(1.5, -2.25)
    (0xbf000000, 0xc0400000, 0xc1000000), // pow(-0.5, -3.0)
];

const ACOS: &[(u32, u32)] = &[
    (0x3f800000, 0x00000000), // acos(1.0)
    (0xbf800000, 0x40490fdb), // acos(-1.0)
    (0x00000000, 0x3fc90fdb), // acos(0.0)
    (0x3f000000, 0x3f860a92), // acos(0.5)
    (0xbf000000, 0x40060a92), // acos(-0.5)
    (0x3f7ffffe, 0x3a000000), // acos(0.9999998807907104)
    (0x3f3504f3, 0x3f490fdb), // acos(0.7071067690849304)
    (0xbe800000, 0x3fe967ae), // acos(-0.25)
    (0x3dcccccd, 0x3fbc3d91), // acos(0.10000000149011612)
    (0x3f733333, 0x3ea29749), // acos(0.949999988079071)
    (0x33d6bf95, 0x3fc90fda), // acos(1.0000000116860974e-07)
];

fn f(bits: u32) -> f32 {
    f32::from_bits(bits)
}

fn check1(name: &str, table: &[(u32, u32)], fun: fn(f32) -> f32) {
    for &(x, want) in table {
        let got = fun(f(x)).to_bits();
        assert_eq!(
            got,
            want,
            "{name}({:e}) = {:e} ({got:#010x}), expected {:e} ({want:#010x})",
            f(x),
            f(got),
            f(want)
        );
    }
}

fn check2(name: &str, table: &[(u32, u32, u32)], fun: fn(f32, f32) -> f32) {
    for &(a, b, want) in table {
        let got = fun(f(a), f(b)).to_bits();
        assert_eq!(
            got,
            want,
            "{name}({:e}, {:e}) = {:e} ({got:#010x}), expected {:e} ({want:#010x})",
            f(a),
            f(b),
            f(got),
            f(want)
        );
    }
}

#[test]
fn sin_and_cos_match_the_vectors() {
    check1("sin", SIN, sin);
    check1("cos", COS, cos);
}

#[test]
fn atan_and_atan2_match_the_vectors() {
    check1("atan", ATAN, atan);
    check2("atan2", ATAN2, atan2);
    check1("acos", ACOS, acos);
    assert!(acos(1.5).is_nan() && acos(f32::NAN).is_nan());
}

#[test]
fn exp_ln_pow_match_the_vectors() {
    check1("exp", EXP, exp);
    check1("ln", LN, ln);
    check2("pow", POW, pow);
}

fn bits(x: f32) -> u32 {
    x.to_bits()
}

#[test]
fn special_values_follow_c99() {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
    let (inf, nan) = (f32::INFINITY, f32::NAN);
    assert_eq!(bits(sin(-0.0)), bits(-0.0));
    assert_eq!(bits(cos(-0.0)), bits(1.0));
    assert!(sin(inf).is_nan() && cos(-inf).is_nan() && sin(nan).is_nan());
    assert_eq!(sin_cos(0.25), (sin(0.25), cos(0.25)));

    assert_eq!(bits(atan(-0.0)), bits(-0.0));
    assert_eq!(atan(inf), FRAC_PI_2);
    assert_eq!(atan(-inf), -FRAC_PI_2);
    assert_eq!(bits(atan2(0.0, 1.0)), bits(0.0));
    assert_eq!(bits(atan2(-0.0, 1.0)), bits(-0.0));
    assert_eq!(atan2(0.0, -1.0), PI);
    assert_eq!(atan2(-0.0, -1.0), -PI);
    assert_eq!(atan2(0.0, -0.0), PI);
    assert_eq!(bits(atan2(0.0, 0.0)), bits(0.0));
    assert_eq!(atan2(2.0, 0.0), FRAC_PI_2);
    assert_eq!(atan2(-2.0, -0.0), -FRAC_PI_2);
    assert_eq!(atan2(inf, inf), FRAC_PI_4);
    assert_eq!(atan2(-inf, -inf), -3.0 * FRAC_PI_4);
    assert_eq!(atan2(1.0, -inf), PI);
    assert_eq!(bits(atan2(-1.0, inf)), bits(-0.0));
    assert_eq!(atan2(inf, 5.0), FRAC_PI_2);
    assert!(atan2(nan, 1.0).is_nan() && atan2(1.0, nan).is_nan());

    assert_eq!(exp(-inf), 0.0);
    assert_eq!(exp(inf), inf);
    assert!(exp(nan).is_nan());
    assert_eq!(ln(0.0), -inf);
    assert_eq!(ln(inf), inf);
    assert!(ln(-1.0).is_nan() && ln(nan).is_nan());

    assert_eq!(pow(nan, 0.0), 1.0);
    assert_eq!(pow(1.0, nan), 1.0);
    assert!(pow(nan, 2.0).is_nan() && pow(2.0, nan).is_nan());
    assert!(pow(-8.0, 1.0 / 3.0).is_nan());
    assert_eq!(pow(-1.0, inf), 1.0);
    assert_eq!(pow(0.5, inf), 0.0);
    assert_eq!(pow(0.5, -inf), inf);
    assert_eq!(pow(2.0, inf), inf);
    assert_eq!(pow(2.0, -inf), 0.0);
    assert_eq!(pow(0.0, -1.0), inf);
    assert_eq!(pow(-0.0, -3.0), -inf);
    assert_eq!(pow(-0.0, -2.0), inf);
    assert_eq!(bits(pow(-0.0, 3.0)), bits(-0.0));
    assert_eq!(bits(pow(-0.0, 2.0)), bits(0.0));
    assert_eq!(pow(-inf, 3.0), -inf);
    assert_eq!(bits(pow(-inf, -3.0)), bits(-0.0));
    assert_eq!(pow(inf, -1.0), 0.0);
    assert_eq!(pow(-2.0, 3.0), -8.0);
    assert_eq!(pow(-2.0, 1e9), inf);
}

fn ordered(x: f32) -> i64 {
    let b = x.to_bits();
    if b & 0x8000_0000 != 0 {
        -i64::from(b & 0x7fff_ffff)
    } else {
        i64::from(b)
    }
}

fn ulps(a: f32, b: f32) -> i64 {
    (ordered(a) - ordered(b)).abs()
}

/// Accuracy across ranges, against the host's f64 library rounded to f32:
/// never more than 1 ulp, and correctly rounded in all but at most 1 in
/// 10,000 results per function (the implementation measured 140,000 of
/// 140,000).
#[test]
fn results_are_correctly_rounded_across_ranges() {
    let mut rng = Rng::new(2026);
    // Per function: (results, results not correctly rounded).
    let mut stats = std::collections::BTreeMap::<&str, (usize, usize)>::new();
    let mut tally = |d: i64, what: &'static str, x: f32, y: f32| {
        assert!(d <= 1, "{what}({x:e}, {y:e}) is {d} ulps off");
        let s = stats.entry(what).or_default();
        s.0 += 1;
        s.1 += usize::from(d != 0);
    };
    for &(lo, hi) in &[
        (-4.0, 4.0),
        (-100.0, 100.0),
        (-1e5, 1e5),
        (-3e7, 3e7),
        (-1e30, 1e30),
    ] {
        for _ in 0..4000 {
            let x = rng.range(lo, hi) as f32;
            let xd = f64::from(x);
            tally(ulps(sin(x), xd.sin() as f32), "sin", x, 0.0);
            tally(ulps(cos(x), xd.cos() as f32), "cos", x, 0.0);
            tally(ulps(atan(x), xd.atan() as f32), "atan", x, 0.0);
            let y = rng.range(lo, hi) as f32;
            tally(
                ulps(atan2(y, x), f64::from(y).atan2(xd) as f32),
                "atan2",
                y,
                x,
            );
        }
    }
    for _ in 0..20000 {
        let x = rng.range(-104.0, 89.0) as f32;
        tally(ulps(exp(x), f64::from(x).exp() as f32), "exp", x, 0.0);
        let p = f32::from_bits(rng.integer(1, 0x7f7f_ffff) as u32);
        tally(ulps(ln(p), f64::from(p).ln() as f32), "ln", p, 0.0);
        let base = rng.range(0.0, 50.0) as f32;
        let e = rng.range(-20.0, 20.0) as f32;
        tally(
            ulps(pow(base, e), f64::from(base).powf(f64::from(e)) as f32),
            "pow",
            base,
            e,
        );
    }
    for (what, (total, off)) in stats {
        assert!(
            off * 10_000 <= total,
            "{what}: {off} of {total} not correctly rounded"
        );
    }
}

#[test]
fn deterministic_rotation_matches_its_definition() {
    let q = quat_from_axis_angle(Vec3::Y, 1.0);
    let (s, c) = sin_cos(0.5);
    assert_eq!((q.x, q.y, q.z, q.w), (0.0, s, 0.0, c));
}

/// `quat_slerp` turns along the arc by the share asked: from a turn of 0.3
/// about +Y to one of 2.1, a quarter of the way is a turn of 0.75. Its bits
/// are pinned, so a host whose results differ fails here (glam's `slerp`
/// uses the host's `sinf` and did, by an ulp, on Windows).
#[test]
fn quat_slerp_turns_by_the_share_and_holds_its_bits() {
    let a = quat_from_axis_angle(Vec3::Y, 0.3);
    let b = quat_from_axis_angle(Vec3::Y, 2.1);
    let q = quat_slerp(a, b, 0.25);
    let want = quat_from_axis_angle(Vec3::Y, 0.75);
    assert!(same(q, want), "{q:?} vs {want:?}");
    let got = [q.x, q.y, q.z, q.w].map(f32::to_bits);
    assert_eq!(got, PINNED_SLERP, "{got:#010x?}");
    // The shorter arc: -b is the same rotation as b.
    assert!(same(quat_slerp(a, -b, 0.25), want));
    // The ends, and the lerp of nearly equal rotations.
    assert!(same(quat_slerp(a, b, 0.0), a));
    assert!(same(quat_slerp(a, b, 1.0), b));
    assert_eq!(
        quat_slerp(Quat::IDENTITY, Quat::IDENTITY, 0.5),
        Quat::IDENTITY
    );
}

const PINNED_SLERP: [u32; 4] = [0x00000000, 0x3ebb8813, 0x00000000, 0x3f6e35bf];

/// The same rotation within 1e-6 per component (q and -q are one rotation).
fn same(a: Quat, b: Quat) -> bool {
    (a - b).length().min((a + b).length()) < 1e-6
}

/// `world_pose` down a chain of three: a grandparent moved and doubled, a
/// parent turned a quarter about +Y, a child offset along +X. A missing
/// transform and a cycle end the chain.
#[test]
fn world_pose_composes_down_the_parent_chain() {
    use og_core::math::{Pose, Quat, world_pose};
    let quarter = quat_from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
    // 0: the child, 1: its parent, 2: the grandparent, 3: no transform.
    let parents = [Some(1usize), Some(2), None, None];
    let poses: [Option<Pose>; 4] = [
        None,
        Some((Vec3::new(0.0, 1.0, 0.0), quarter, Vec3::ONE)),
        Some((Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY, Vec3::splat(2.0))),
        None,
    ];
    let local = (Vec3::X, Quat::IDENTITY, Vec3::ONE);
    let (p, q, s) = world_pose(0, local, |c| parents[c], |c| poses[c]);
    // The parent turns +X to -Z and lifts it 1; the grandparent doubles
    // that and moves it 10 along +X.
    assert!((p - Vec3::new(10.0, 2.0, -2.0)).length() < 1e-5, "{p}");
    assert!(q.dot(quarter).abs() > 1.0 - 1e-6, "{q}");
    assert_eq!(s, Vec3::splat(2.0));
    // A root is its own pose, bit for bit.
    let root = world_pose(2, local, |_| None::<usize>, |c| poses[c]);
    assert_eq!(root, local);
    // A parent without a transform ends the chain.
    let (p, _, _) = world_pose(0, local, |c| (c == 0).then_some(3), |c| poses[c]);
    assert_eq!(p, Vec3::X);
    // A cycle ends after 256 steps instead of looping.
    let unit = (Vec3::new(0.0, 0.0, 1.0), Quat::IDENTITY, Vec3::ONE);
    let (p, _, _) = world_pose(0usize, local, |c| Some(1 - c), |_| Some(unit));
    assert_eq!(p, Vec3::new(1.0, 0.0, 256.0));
}
