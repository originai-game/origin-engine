//! Tweens through a world: what the og.tween system writes tick by tick.
//!
//! Ways this could fail: a field other than the target moves (a wrong
//! offset); the curve is applied to the wrong span (from read at the wrong
//! time, relative ignored); delay, loops or yoyo end at the wrong value or
//! tick; a sequence restarts its children or drops the leftover time; a
//! parallel group ends with its first child; time scale and hit stop are
//! ignored (or an unscaled tween honours them); a completion is sent for a
//! cancelled tween, twice, or never; a dead target keeps the tween alive;
//! a snapshot mid-tween restores a world that animates differently.

use og_core::Reflect;
use og_core::glam::Vec3;
use og_ecs::{Entity, World, WorldDesc};
use og_tween::*;

#[test]
fn gsap_names_and_changed_curves_match_asymmetric_mathematical_values() {
    // Independent double-precision evaluations of the documented GSAP 3.15
    // formula, not values produced by Ease::apply. The old Penner curves
    // differ substantially at 1/6 and 17/60, despite agreeing at endpoints.
    let points = [
        0.0,
        1.0 / 60.0,
        1.0 / 6.0,
        17.0 / 60.0,
        43.0 / 60.0,
        59.0 / 60.0,
        1.0,
    ];
    for (name, expected) in [
        (
            "expo.in",
            [
                0.0,
                0.0000182692601438,
                0.000534593338694,
                0.0023428102759,
                0.138942370563,
                0.891118326568,
                1.0,
            ],
        ),
        (
            "expo.out",
            [
                0.0,
                0.108881673432,
                0.681700118492,
                0.861057629437,
                0.997657189724,
                0.99998173074,
                1.0,
            ],
        ),
        (
            "expo.inOut",
            [
                0.0,
                0.0000205071905135,
                0.00209776957121,
                0.0212290594572,
                0.978770940543,
                0.999979492809,
                1.0,
            ],
        ),
        (
            "back.inOut",
            [
                0.0,
                -0.000895292962963,
                -0.044502962963,
                -0.0274043714815,
                1.02740437148,
                1.00089529296,
                1.0,
            ],
        ),
    ] {
        let ease = Ease::from_name(name).unwrap();
        for (t, expected) in points.into_iter().zip(expected) {
            assert!(
                (f64::from(ease.apply(t)) - expected).abs() < 0.000001,
                "{name} at {t}"
            );
        }
    }
    let near_zero = 1.0 / 1_048_576.0;
    assert!(Ease::ExpoIn.apply(near_zero) < 0.000000002);
    assert!(Ease::ExpoOut.apply(1.0 - near_zero) > 0.999999);
    for (name, value) in [
        ("none", 0.25),
        ("power1.out", 0.4375),
        ("power2.in", 0.015625),
        ("power3.inOut", 0.03125),
    ] {
        assert_eq!(Ease::from_name(name).unwrap().apply(0.25), value);
    }
    for old in ["linear", "quadOut", "expoIn", "backInOut"] {
        assert!(Ease::from_name(old).is_none(), "no alias for {old}");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "Body")]
struct Body {
    position: Vec3,
    opacity: f32,
    count: u32,
    on: bool,
}

impl Default for Body {
    fn default() -> Self {
        Body {
            position: Vec3::new(1.0, 2.0, 3.0),
            opacity: 1.0,
            count: 0,
            on: false,
        }
    }
}

fn setup() -> (World, TweenSystem, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    w.register::<Body>().unwrap();
    let t = install(&mut w, &["Body".to_string()]).unwrap();
    let e = w.spawn((Body::default(),)).unwrap();
    (w, t, e)
}

fn target(w: &World, e: Entity, path: &str) -> Target {
    let id = w.id::<Body>().unwrap();
    Target::resolve(w.schema(id).unwrap(), id.index(), e.to_bits(), path).unwrap()
}

fn body(w: &World, e: Entity) -> Body {
    w.get::<Body>(e).unwrap()
}

fn steps(w: &mut World, n: u32) {
    for _ in 0..n {
        w.step().unwrap();
    }
}

fn completed(w: &World) -> Vec<TweenCompleted> {
    w.pending_events::<TweenCompleted>().unwrap()
}

#[test]
fn delay_relative_loops_and_yoyo() {
    let (mut w, t, e) = setup();
    // +2 on x after 0.25 s, 0.25 s per cycle, two cycles, yoyo: back to the start.
    t.start(
        Node::Leaf(
            Leaf::new(target(&w, e, "position.x"), vec![2.0])
                .relative(true)
                .delay(0.25)
                .duration(0.25)
                .ease(Ease::Linear)
                .loops(1, true),
        ),
        false,
        0,
    )
    .unwrap();
    steps(&mut w, 14);
    assert_eq!(body(&w, e).position.x, 1.0, "still in its delay");
    steps(&mut w, 16);
    assert!(
        (body(&w, e).position.x - 3.0).abs() < 1e-4,
        "the first cycle reached 1 + 2"
    );
    steps(&mut w, 8);
    let back = body(&w, e).position.x;
    assert!(back < 3.0 && back > 1.0, "yoyo on its way back: {back}");
    steps(&mut w, 7);
    assert_eq!(
        body(&w, e).position.x,
        1.0,
        "the second cycle ends at the start"
    );
    // An integer field rounds; a whole vector takes one value for every lane.
    t.start(
        Node::Leaf(Leaf::new(target(&w, e, "count"), vec![10.0]).duration(0.1)),
        false,
        0,
    )
    .unwrap();
    t.start(
        Node::Leaf(Leaf::new(target(&w, e, "position"), vec![0.0]).duration(0.1)),
        false,
        0,
    )
    .unwrap();
    steps(&mut w, 6);
    assert_eq!(body(&w, e).count, 10);
    assert_eq!(body(&w, e).position, Vec3::ZERO);
}

#[test]
fn sequences_hand_on_their_leftover_time_and_parallels_wait_for_the_last() {
    let (mut w, t, e) = setup();
    let seq = t
        .start(
            Node::group(
                GroupKind::Sequence,
                vec![
                    Leaf::new(target(&w, e, "position.x"), vec![5.0])
                        .duration(0.1)
                        .ease(Ease::Linear),
                    Leaf::new(target(&w, e, "opacity"), vec![0.0])
                        .duration(0.1)
                        .ease(Ease::Linear),
                ],
                0,
            ),
            false,
            1,
        )
        .unwrap();
    steps(&mut w, 3);
    assert!(
        body(&w, e).position.x > 1.0 && body(&w, e).opacity == 1.0,
        "the second waits"
    );
    steps(&mut w, 3);
    assert_eq!(body(&w, e).position.x, 5.0);
    steps(&mut w, 3);
    let o = body(&w, e).opacity;
    assert!(o < 1.0 && o > 0.0, "the second runs: {o}");
    steps(&mut w, 3);
    assert_eq!(body(&w, e).opacity, 0.0);
    assert_eq!(
        completed(&w).iter().map(|c| c.tween).collect::<Vec<_>>(),
        vec![seq]
    );

    // A child ending mid-tick hands the rest of the tick to the next one:
    // 0.11 s is 6.6 ticks, so tick 7 already moves the second child.
    let f = w.spawn((Body::default(),)).unwrap();
    t.start(
        Node::group(
            GroupKind::Sequence,
            vec![
                Leaf::new(target(&w, f, "position.x"), vec![5.0]).duration(0.11),
                Leaf::new(target(&w, f, "opacity"), vec![0.0])
                    .duration(0.1)
                    .ease(Ease::Linear),
            ],
            0,
        ),
        false,
        0,
    )
    .unwrap();
    steps(&mut w, 7);
    let o = body(&w, f).opacity;
    assert!((o - (1.0 - (7.0 / 60.0 - 0.11) / 0.1)).abs() < 1e-3, "{o}");

    let par = t
        .start(
            Node::group(
                GroupKind::Parallel,
                vec![
                    Leaf::new(target(&w, e, "position.z"), vec![9.0]).duration(0.05),
                    Leaf::new(target(&w, e, "opacity"), vec![1.0]).duration(0.2),
                ],
                1,
            ),
            false,
            2,
        )
        .unwrap();
    steps(&mut w, 6);
    assert_eq!(body(&w, e).position.z, 9.0, "the short child is done");
    assert!(t.is_active(par), "the long child is still running");
    steps(&mut w, 12);
    // Run two, 0.1 s in: both children replay their first run's spans.
    assert!(t.is_active(par), "the group runs a second time");
    let o = body(&w, e).opacity;
    assert!(o > 0.0 && o < 1.0, "the opacity replays from 0: {o}");
    steps(&mut w, 6);
    assert!(!t.is_active(par));
    assert_eq!(body(&w, e).opacity, 1.0);
    assert_eq!(
        completed(&w).iter().map(|c| c.tween).collect::<Vec<_>>(),
        vec![par]
    );
}

#[test]
fn a_snapshot_mid_tween_restores_running_tweens() {
    let (mut w, t, e) = setup();
    let start = |w: &World, t: &TweenSystem| {
        t.start(
            Node::group(
                GroupKind::Sequence,
                vec![
                    Leaf::new(target(w, e, "position"), vec![4.0, 5.0, 6.0])
                        .duration(0.4)
                        .ease(Ease::ElasticOut),
                    Leaf::new(target(w, e, "opacity"), vec![0.2])
                        .duration(0.3)
                        .ease(Ease::BounceOut)
                        .loops(2, true),
                ],
                0,
            ),
            false,
            3,
        )
        .unwrap()
    };
    start(&w, &t);
    steps(&mut w, 20);
    let snap = w.snapshot();
    steps(&mut w, 60);
    let (mut r, rt, _) = setup();
    r.restore(&snap).unwrap();
    assert_eq!(rt.count(), 1, "the running group came back");
    steps(&mut r, 60);
    assert_eq!(r.world_hash(), w.world_hash());
    assert_eq!(body(&r, e), body(&w, e));
}
