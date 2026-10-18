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
fn every_ease_starts_at_0_and_ends_at_1() {
    for e in EASES {
        assert_eq!(e.apply(0.0), 0.0, "{}", e.name());
        assert!(
            (e.apply(1.0) - 1.0).abs() < 1e-6,
            "{} {}",
            e.name(),
            e.apply(1.0)
        );
        assert_eq!(Ease::from_name(e.name()), Some(e));
    }
    assert!(Ease::BackOut.apply(0.6) > 1.0, "back overshoots");
    assert!(Ease::BackIn.apply(0.3) < 0.0, "back anticipates");
    assert!((0..=100).all(|i| (0.0..=1.0).contains(&Ease::BounceOut.apply(i as f32 / 100.0))));
    assert!(
        Ease::ElasticOut.apply(0.2) > 1.0,
        "elastic rings past the end"
    );
    assert!(Ease::QuadIn.apply(0.5) < 0.5 && Ease::QuadOut.apply(0.5) > 0.5);
}

#[test]
fn a_leaf_eases_one_lane_and_reports_once() {
    let (mut w, t, e) = setup();
    let h = t
        .start(
            Node::Leaf(
                Leaf::new(target(&w, e, "position.y"), vec![4.0])
                    .duration(0.5)
                    .ease(Ease::Linear),
            ),
            false,
            7,
        )
        .unwrap();
    steps(&mut w, 15);
    let b = body(&w, e);
    assert!(
        (b.position.y - 3.0).abs() < 1e-4,
        "halfway from 2 to 4: {}",
        b.position.y
    );
    assert_eq!(
        (b.position.x, b.position.z, b.opacity),
        (1.0, 3.0, 1.0),
        "only the lane moves"
    );
    steps(&mut w, 15);
    assert_eq!(body(&w, e).position.y, 4.0);
    assert!(!t.is_active(h));
    assert_eq!(
        completed(&w),
        vec![TweenCompleted {
            tween: h,
            entity: e.to_bits(),
            tag: 7
        }]
    );
    steps(&mut w, 1);
    assert!(completed(&w).is_empty(), "once");
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
fn time_scale_and_hit_stop_slow_and_hold_scaled_tweens_only() {
    let (mut w, t, e) = setup();
    t.start(
        Node::Leaf(
            Leaf::new(target(&w, e, "position.x"), vec![61.0])
                .duration(1.0)
                .ease(Ease::Linear),
        ),
        false,
        0,
    )
    .unwrap();
    t.start(
        Node::Leaf(
            Leaf::new(target(&w, e, "position.y"), vec![62.0])
                .duration(1.0)
                .ease(Ease::Linear),
        ),
        true,
        0,
    )
    .unwrap();
    w.set_time_scale(0.5).unwrap();
    steps(&mut w, 12);
    let b = body(&w, e);
    assert!(
        (b.position.x - 7.0).abs() < 1e-3,
        "scaled: 6 of 60 ticks' worth: {}",
        b.position.x
    );
    assert!(
        (b.position.y - 14.0).abs() < 1e-3,
        "unscaled: 12: {}",
        b.position.y
    );
    w.hit_stop(6).unwrap();
    steps(&mut w, 6);
    let c = body(&w, e);
    assert_eq!(c.position.x, b.position.x, "held through the hit stop");
    assert!(
        (c.position.y - 20.0).abs() < 1e-3,
        "the unscaled one kept going"
    );
}

#[test]
fn cancelled_and_orphaned_tweens_end_without_an_event() {
    let (mut w, t, e) = setup();
    let a = t
        .start(
            Node::Leaf(Leaf::new(target(&w, e, "opacity"), vec![0.0]).duration(0.1)),
            false,
            0,
        )
        .unwrap();
    let other = w.spawn((Body::default(),)).unwrap();
    t.start(
        Node::Leaf(Leaf::new(target(&w, other, "opacity"), vec![0.0]).duration(0.1)),
        false,
        0,
    )
    .unwrap();
    steps(&mut w, 2);
    assert!(t.cancel(a));
    assert!(!t.cancel(a));
    w.despawn(other).unwrap();
    steps(&mut w, 10);
    assert_eq!(t.count(), 0);
    let all: usize = (0..3)
        .map(|_| {
            let n = completed(&w).len();
            w.step().unwrap();
            n
        })
        .sum();
    assert_eq!(all, 0);
    let o = body(&w, e).opacity;
    assert!(o < 1.0 && o > 0.0, "stopped where it was: {o}");
    t.start(
        Node::Leaf(Leaf::new(target(&w, e, "opacity"), vec![0.0])),
        false,
        0,
    )
    .unwrap();
    t.start(
        Node::Leaf(Leaf::new(target(&w, e, "position.x"), vec![0.0])),
        false,
        0,
    )
    .unwrap();
    assert_eq!(t.cancel_entity(e.to_bits()), 2);
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

#[test]
fn targets_must_be_numeric_fields_and_values_must_fit() {
    let (w, t, e) = setup();
    let id = w.id::<Body>().unwrap();
    let schema = w.schema(id).unwrap();
    let refuse = |p: &str| {
        Target::resolve(schema, id.index(), e.to_bits(), p)
            .unwrap_err()
            .reason()
            .to_string()
    };
    assert!(
        refuse("on").contains("tweens animate numbers"),
        "{}",
        refuse("on")
    );
    assert!(refuse("size").contains("no field 'size'"));
    assert!(refuse("opacity.x").contains("no lane"));
    assert!(refuse("position.w").contains("no lane"));
    let v = target(&w, e, "position");
    assert!(
        t.start(Node::Leaf(Leaf::new(v, vec![1.0, 2.0])), false, 0)
            .is_err()
    );
    assert!(
        t.start(Node::Leaf(Leaf::new(v, vec![f32::NAN])), false, 0)
            .is_err()
    );
    assert!(
        t.start(Node::Leaf(Leaf::new(v, vec![1.0]).duration(-1.0)), false, 0)
            .is_err()
    );
    assert!(
        t.start(Node::group(GroupKind::Parallel, vec![], 0), false, 0)
            .is_err()
    );
}
