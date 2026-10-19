//! og.Cloth self-collision (`thickness`): a square sheet hanging from its
//! top row is lowered onto the floor, so it folds and piles up on itself.
//!
//! Ways this could fail, each caught below: the pairs held apart are the
//! wrong ones (the sheet's own links fight them: it buckles or explodes, or
//! near neighbours are pushed apart and it stretches); the spatial hash
//! misses pairs across a cell boundary (points pass through each other
//! there); points tunnel through a layer in one step (the per-step limit of
//! half the thickness); a cloth without `thickness` is changed (it must lie
//! flat in one layer, passing through itself); the result depends on more
//! than the inputs (two runs differ). A layer passing through another shows
//! as two points of the sheet that are not near neighbours coming closer
//! than the thickness allows, at any tick of the fall.

mod support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

const SPACING: f32 = 0.05;
const N: u32 = 20;

/// Where the top row is at `tick`: lowered from 1.1 m to 0.3 m over two
/// seconds, swaying along z, so the 0.95 m sheet folds onto the floor
/// under it.
fn top(tick: u32) -> Transform {
    let s = (tick as f32 / 120.0).min(1.0);
    let sway = 0.12 * og_core::math::sin(tick as f32 * 0.08);
    at(-0.5, 1.1 - 0.8 * s, sway)
}

fn drop(thickness: f32) -> (World, PhysicsSystem, Entity) {
    let (mut w, p) = world(Physics::default(), 1);
    floor(&mut w);
    let sheet = w
        .spawn((
            top(0),
            Cloth {
                columns: N,
                rows: N,
                spacing: SPACING,
                pins: PIN_TOP,
                iterations: 6,
                substeps: 4,
                gust: 0.0,
                radius: 0.02,
                friction: 0.9,
                thickness,
                ..Cloth::default()
            },
        ))
        .unwrap();
    (w, p, sheet)
}

/// The closest two points of the sheet that are not near neighbours (rest
/// distance at least `thickness`), as a fraction of `thickness`.
fn closest(points: &[Vec3], thickness: f32) -> f32 {
    let mut best = f32::MAX;
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let (dx, dy) = (
                (i as i32 % N as i32 - j as i32 % N as i32) as f32,
                (i as i32 / N as i32 - j as i32 / N as i32) as f32,
            );
            if SPACING * (dx * dx + dy * dy).sqrt() < thickness {
                continue;
            }
            best = best.min((points[i] - points[j]).length());
        }
    }
    best / thickness
}

/// Points in the floor layer (below 3 cm) and in the pile's layers above it
/// (6 to 25 cm; the sheet's top row hangs at 30 cm).
fn layers(points: &[Vec3]) -> (usize, usize) {
    let on = points.iter().filter(|p| p.y < 0.03).count();
    let above = points.iter().filter(|p| p.y > 0.06 && p.y < 0.25).count();
    (on, above)
}

#[test]
fn a_sheet_that_falls_and_folds_never_passes_through_itself() {
    let thickness = SPACING;
    let (mut w, p, sheet) = drop(thickness);
    let mut worst = f32::MAX;
    for tick in 0..180 {
        w.insert(sheet, top(tick)).unwrap();
        w.step().unwrap();
        let c = p.cloth_points(sheet).unwrap();
        worst = worst.min(closest(&c.points, thickness));
    }
    let c = p.cloth_points(sheet).unwrap();
    let low = c.points.iter().map(|q| q.y).fold(f32::MAX, f32::min);
    let (floor_layer, above) = layers(&c.points);
    eprintln!(
        "self-collision: closest pair {worst:.3} × thickness over the fall; {floor_layer} points on the floor, {above} on the layers above"
    );
    assert!(
        worst >= 0.7,
        "two layers came within {worst} × the thickness: one passed through the other"
    );
    assert!(
        above >= 60,
        "the sheet piled up in layers ({above} points above the floor layer)"
    );
    assert!(low >= 0.0, "nothing went through the floor ({low} m)");
    // The links still hold: the sheet neither tore nor stretched.
    for y in 0..N {
        for x in 0..N - 1 {
            let i = (y * N + x) as usize;
            let d = (c.points[i + 1] - c.points[i]).length();
            assert!(d < SPACING * 1.3, "link ({x}, {y}) stretched to {d}");
        }
    }

    // The same sheet without self-collision lies in one layer, through itself.
    let (mut w, p, flat) = drop(0.0);
    let mut through = f32::MAX;
    for tick in 0..180 {
        w.insert(flat, top(tick)).unwrap();
        w.step().unwrap();
        through = through.min(closest(&p.cloth_points(flat).unwrap().points, thickness));
    }
    let (floor0, above0) = layers(&p.cloth_points(flat).unwrap().points);
    eprintln!(
        "without: closest pair {through:.3} × thickness; {floor0} points on the floor, {above0} above"
    );
    assert!(
        through < 0.3,
        "without self-collision layers meet ({through})"
    );
    assert!(
        above0 * 3 < above,
        "without self-collision it lies in one layer ({above0} points above it)"
    );

    // Twice the same.
    let (mut w2, p2, sheet2) = drop(thickness);
    for tick in 0..180 {
        w2.insert(sheet2, top(tick)).unwrap();
        w2.step().unwrap();
    }
    assert_eq!(p2.cloth_points(sheet2), Some(c));
}
