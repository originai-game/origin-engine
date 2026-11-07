//! The Rive import on rive-runtime's own test files (MIT,
//! `tests/fixtures/rive/`), each expectation read off the file's objects.

use super::convert::artboards;
use super::read::*;
use og_render::UiPaint;
use og_ui::rive::{Artboard, Player};
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn fixture(name: &str) -> Vec<u8> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/rive/assets")
        .join(name);
    std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn convert(name: &str) -> Result<Vec<Artboard>, String> {
    artboards(&read(&fixture(name))?)
}

#[test]
fn a_shape_draws_where_its_transforms_put_it() {
    let a = convert("shapetest.riv").expect("converts");
    let p = Player::new(Arc::new(a[0].clone()), None);
    // The shape at (139.2, 31.7), its path at (65.9, 65.9) through four
    // cubic vertices 65.9 from it: a circle of radius 65.9 about
    // (205.1, 97.7) in a 400 × 200 artboard drawn at its own size.
    let e = p.elements([0.0, 0.0, 400.0, 200.0]);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].id, "New Artboard/Ellipse");
    let [x, y, w, h] = e[0].bounds;
    let (cx, cy, r) = (139.203_13 + 65.941_41, 31.730_469 + 65.941_41, 65.941_41);
    for (got, want) in [(x, cx - r), (y, cy - r), (w, 2.0 * r), (h, 2.0 * r)] {
        assert!(
            (got - want).abs() < 0.05,
            "{:?} vs centre ({cx}, {cy}) r {r}",
            e[0].bounds
        );
    }
    // Under it, the artboard's background (0xFF313131); the circle's fill
    // has no colour of its own, so Rive's default grey (0xFF747474).
    let shapes = p.draw([0.0, 0.0, 400.0, 200.0], 1.0);
    let solid = |s: &og_render::UiShape| match &s.paint {
        UiPaint::Solid(c) => *c,
        _ => panic!("solid"),
    };
    let grey = |v: u8| {
        og_ui::Srgba {
            rgb: [v; 3],
            alpha: 1.0,
        }
        .linear()
    };
    assert_eq!(shapes.len(), 2);
    assert_eq!(solid(&shapes[0]), grey(0x31));
    assert_eq!(solid(&shapes[1]), grey(0x74));
}

#[test]
fn a_nested_artboard_draws_at_its_node_with_its_opacity() {
    let a = convert("nested_artboard_opacity.riv").expect("converts");
    let p = Player::new(Arc::new(a[0].clone()), None);
    let shapes = p.draw([0.0, 0.0, 500.0, 500.0], 1.0);
    // The nested 250 px artboard's red background at (125, 125), at the
    // container's opacity 0.3275, over the parent's own background.
    let red = shapes
        .iter()
        .find(|s| matches!(s.paint, UiPaint::Solid(c) if c[0] > 0.9 && c[1] < 0.1))
        .expect("the nested artboard's red");
    let UiPaint::Solid(c) = red.paint else {
        unreachable!()
    };
    assert!((c[3] - 0.3275).abs() < 1e-4, "alpha {}", c[3]);
    assert!((red.center - og_core::glam::Vec2::splat(250.0)).length() < 0.01);
    assert!((red.half - og_core::glam::Vec2::splat(125.0)).length() < 0.01);
    let ids: Vec<String> = p
        .elements([0.0, 0.0, 500.0, 500.0])
        .into_iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(ids, ["Parent Artboard/Nested artboard container"]);
}

#[test]
fn keys_play_on_the_tick_with_their_interpolators() {
    let a = convert("cubic_value_test.riv").expect("converts");
    // Its state machine's entry plays "Timeline 1", over its work area
    // (frames 5 to 53 at 60 fps), keying grey_rectangle's x.
    let ab = Arc::new(a[0].clone());
    assert_eq!(ab.machines, [("State Machine 1".to_string(), Some(0))]);
    let node = ab
        .nodes
        .iter()
        .position(|n| n.name == "grey_rectangle")
        .expect("node");
    let mut p = Player::new(ab.clone(), None);
    let x = |p: &Player| p.pose()[node].x;
    // At frame 10 and 24 the keys' own values.
    p.advance(5.0 / 60.0);
    assert!((x(&p) - 435.974_15).abs() < 1e-3, "{}", x(&p));
    p.advance(14.0 / 60.0);
    assert!((x(&p) - 333.980_83).abs() < 1e-3, "{}", x(&p));
    // Between 10 and 24 the value interpolator's curve runs through its
    // own controls (y1 = 239.7 here, not the straight line's 384.98 at
    // its first third): by frame 14 it is well under the line.
    let mut q = Player::new(ab.clone(), None);
    q.advance(9.0 / 60.0);
    let line = 435.974_15 + (333.980_83 - 435.974_15) * (4.0 / 14.0);
    assert!(x(&q) < line - 5.0, "{} vs the line's {line}", x(&q));
    // Two runs of the same steps agree to the bit.
    let mut r = Player::new(ab, None);
    r.advance(9.0 / 60.0);
    assert_eq!(q, r);
}

#[test]
fn what_phase_1_does_not_convert_is_an_error_naming_it() {
    let e = convert("light_switch.riv").expect_err("a state machine input");
    assert!(
        e.contains("StateMachineBool") && e.contains("phase 2"),
        "{e}"
    );

    // Another major version.
    let mut b = fixture("shapetest.riv");
    b[4] = 6;
    let e = read(&b).expect_err("format 6");
    assert!(e.contains("format 6") && e.contains("re-export"), "{e}");

    // A Luau script, here a scripted drawable in the artboard.
    let mut f = read(&fixture("shapetest.riv")).expect("reads");
    let script = super::defs::TYPES
        .iter()
        .find(|t| t.1 == "ScriptedDrawable")
        .expect("in the tables")
        .0;
    f.objects.push(Object {
        ty: script,
        props: vec![(4, Prop::String("Wobble".into())), (5, Prop::Uint(0))],
    });
    let e = artboards(&f).expect_err("Luau");
    assert!(
        e.contains("Luau") && e.contains("ScriptedDrawable 'Wobble'"),
        "{e}"
    );
}

#[test]
fn an_artboard_survives_its_codec() {
    for a in convert("cubic_value_test.riv")
        .into_iter()
        .chain(convert("nested_artboard_opacity.riv"))
        .flatten()
    {
        assert_eq!(og_ui::rive::decode(&og_ui::rive::encode(&a)), Ok(a));
    }
}
