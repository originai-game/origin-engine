//! The platform's rng vectors (produced by the Web scaffold's `rng.ts` under
//! bun, inherited by the Godot module's `rng.rs`): a seed names the same
//! sequence on every substrate.

use og_core::Rng;
use og_core::reflect::{from_bytes, to_bytes};

#[test]
fn matches_the_web_substrate_sequence() {
    let cases: [(u32, u32, [f64; 4], u32); 4] = [
        (
            0,
            1,
            [
                0.479614513926208,
                0.48086497001349926,
                0.8585285847075284,
                0.9917182873468846,
            ],
            2027808485,
        ),
        (
            1,
            2261973619,
            [
                0.9481164838653058,
                0.5605214422103018,
                0.1859289831481874,
                0.14164146734401584,
            ],
            4289782103,
        ),
        (
            42,
            671623878,
            [
                0.09396462352015078,
                0.5757808552589267,
                0.19534794357605278,
                0.7601067263167351,
            ],
            2699432362,
        ),
        (
            4294967295,
            2578835075,
            [
                0.3764090109616518,
                0.41705114021897316,
                0.07903929799795151,
                0.706570005742833,
            ],
            311676263,
        ),
    ];
    for (seed, state0, draws, after) in cases {
        let mut rng = Rng::new(seed);
        assert_eq!(rng.save(), state0, "seed {seed}");
        for expected in draws {
            assert_eq!(rng.next_f64(), expected, "seed {seed}");
        }
        assert_eq!(rng.save(), after, "seed {seed}");
    }
}

#[test]
fn forks_derive_the_web_substrate_seed() {
    let mut enemies = Rng::new(42).fork("enemies");
    assert_eq!(enemies.seed(), 3383749726);
    assert_eq!(enemies.next_f64(), 0.12230518204160035);
    assert_eq!(enemies.next_f64(), 0.9143574410118163);
    let mut weather = Rng::new(7).fork("天气");
    assert_eq!(weather.seed(), 3296486234);
    assert_eq!(weather.next_f64(), 0.2081162198446691);
}

#[test]
fn integer_and_range_follow_the_same_draws() {
    let mut rng = Rng::new(42);
    assert_eq!(
        [rng.integer(1, 6), rng.integer(1, 6), rng.integer(1, 6)],
        [1, 4, 2]
    );
    assert_eq!(rng.range(-2.0, 3.0), 1.8005336315836757);
}

#[test]
fn save_restore_and_reflected_bytes_resume_the_sequence() {
    let mut rng = Rng::new(9);
    rng.next_f64();
    let saved = rng.save();
    let bytes = to_bytes(&rng).unwrap();
    let a = rng.next_f64();
    rng.restore(saved);
    assert_eq!(rng.next_f64(), a);
    let mut copy: Rng = from_bytes(&bytes).unwrap();
    assert_eq!(copy.next_f64(), a);
}
