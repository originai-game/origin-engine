//! `ui-paths` (`docs/ui.md` §5.1): run-time vector paths
//! ([`UiGeometry::Path`]), nothing cooked, in the cells of [`super::cell`].
//!
//! - Row 0: a heart of cubics; a ring whose inner circle runs the other
//!   way (a hole by nonzero); a five-point star drawn as one
//!   self-crossing contour, nonzero (solid) and even-odd (its pentagon a
//!   hole); a blob of quadratics under a radial gradient; a 14 px heart
//!   beside a 56 px one.
//! - Row 1: the heart stroked (a band around its outline), with a drop
//!   shadow, with an outer glow; a morph half way from a square to a
//!   circle of the same verbs; the heart squashed to 70 % wide and turned
//!   25° by its transform; a small even-odd star and a star stroked inside.

use super::*;
use og_render::{UiFillRule, UiPath};
use std::sync::Arc;

/// The heart in a 140 px box about the origin, cubics only.
pub fn heart(scale: f32) -> UiPath {
    let s = |x: f32, y: f32| Vec2::new(x, y) * scale / 140.0;
    let mut p = UiPath::new();
    p.move_to(s(0.0, 55.0))
        .cubic_to(s(-30.0, 30.0), s(-70.0, 5.0), s(-70.0, -25.0))
        .cubic_to(s(-70.0, -50.0), s(-50.0, -65.0), s(-32.0, -65.0))
        .cubic_to(s(-15.0, -65.0), s(-5.0, -55.0), s(0.0, -42.0))
        .cubic_to(s(5.0, -55.0), s(15.0, -65.0), s(32.0, -65.0))
        .cubic_to(s(50.0, -65.0), s(70.0, -50.0), s(70.0, -25.0))
        .cubic_to(s(70.0, 5.0), s(30.0, 30.0), s(0.0, 55.0))
        .close();
    p
}

/// A five-point star as one contour through every second vertex.
pub fn star(r: f32, rule: UiFillRule) -> UiPath {
    let v = |k: usize| {
        let a = -std::f32::consts::FRAC_PI_2 + k as f32 * 4.0 * std::f32::consts::PI / 5.0;
        Vec2::from_angle(a) * r
    };
    let mut p = UiPath::new();
    p.move_to(v(0));
    for k in 1..5 {
        p.line_to(v(k));
    }
    p.close();
    p.fill_rule = rule;
    p
}

/// A square of four cubics (straight, so it morphs into [`UiPath::ellipse`]).
fn square_path(h: f32) -> UiPath {
    let mut p = UiPath::new();
    let c = [
        Vec2::new(h, 0.0),
        Vec2::new(h, h),
        Vec2::new(0.0, h),
        Vec2::new(-h, h),
        Vec2::new(-h, 0.0),
        Vec2::new(-h, -h),
        Vec2::new(0.0, -h),
        Vec2::new(h, -h),
    ];
    // Each quarter from an edge's middle round a corner to the next middle,
    // its controls on the corner as a circle's would be near it.
    p.move_to(c[0]);
    for q in 0..4 {
        let (corner, end) = (c[2 * q + 1], c[(2 * q + 2) % 8]);
        p.cubic_to(corner, corner, end);
    }
    p.close();
    p
}

fn path_shape(path: UiPath, r: [f32; 4], paint: UiPaint) -> UiShape {
    shape(UiGeometry::Path(Arc::new(path)), r, paint)
}

/// `ui-paths`. See the module documentation.
pub fn paths() -> RenderScene {
    overlay_scene(|_, o| {
        let ink = [
            rgb(239, 83, 80),
            rgb(77, 208, 225),
            rgb(255, 183, 77),
            rgb(186, 104, 200),
            rgb(129, 199, 132),
            rgb(255, 241, 118),
        ];
        for row in 0..2 {
            for col in 0..6 {
                backing(o, cell(col, row));
            }
        }
        // Row 0.
        o.ui.shapes
            .push(path_shape(heart(140.0), cell(0, 0), solid(ink[0])));
        let mut ring = UiPath::new();
        ring.ellipse(Vec2::ZERO, Vec2::splat(60.0));
        // The inner circle the other way round: a hole by nonzero.
        let inner = {
            let mut c = UiPath::new();
            c.ellipse(Vec2::ZERO, Vec2::splat(32.0));
            c.map(|p| Vec2::new(p.x, -p.y))
        };
        ring.verbs.extend(inner.verbs);
        ring.points.extend(inner.points);
        o.ui.shapes
            .push(path_shape(ring, cell(1, 0), solid(ink[1])));
        o.ui.shapes.push(path_shape(
            star(64.0, UiFillRule::NonZero),
            cell(2, 0),
            solid(ink[2]),
        ));
        o.ui.shapes.push(path_shape(
            star(64.0, UiFillRule::EvenOdd),
            cell(3, 0),
            solid(ink[3]),
        ));
        let mut blob = UiPath::new();
        blob.move_to(Vec2::new(0.0, -60.0))
            .quad_to(Vec2::new(80.0, -60.0), Vec2::new(55.0, 10.0))
            .quad_to(Vec2::new(30.0, 75.0), Vec2::new(-20.0, 55.0))
            .quad_to(Vec2::new(-85.0, 30.0), Vec2::new(-50.0, -20.0))
            .quad_to(Vec2::new(-30.0, -60.0), Vec2::new(0.0, -60.0))
            .close();
        o.ui.shapes.push(path_shape(
            blob,
            cell(4, 0),
            UiPaint::Gradient(gradient(
                UiGradientKind::Radial,
                Vec2::new(0.4, 0.35),
                Vec2::new(0.6, 0.6),
                &[(0.0, rgb(255, 241, 118)), (1.0, rgb(186, 104, 200))],
            )),
        ));
        // A heart of 14 px beside one of 56.
        let c = cell(5, 0);
        o.ui.shapes.push(path_shape(
            heart(14.0),
            [c[0] + 20.0, c[1] + 20.0, 14.0, 14.0],
            solid(ink[5]),
        ));
        o.ui.shapes.push(path_shape(
            heart(56.0),
            [c[0] + 80.0, c[1] + 60.0, 56.0, 56.0],
            solid(ink[5]),
        ));
        // Row 1.
        o.ui.shapes.push(UiShape {
            mode: UiMode::Stroke(UiStroke {
                width: 6.0,
                ..Default::default()
            }),
            ..path_shape(heart(120.0), cell(0, 1), solid(ink[0]))
        });
        o.ui.shapes.push(UiShape {
            mode: UiMode::Shadow {
                offset: Vec2::new(6.0, 8.0),
                blur: 10.0,
                spread: 0.0,
            },
            ..path_shape(heart(110.0), cell(1, 1), solid([0.0, 0.0, 0.0, 0.9]))
        });
        o.ui.shapes
            .push(path_shape(heart(110.0), cell(1, 1), solid(ink[1])));
        o.ui.shapes.push(UiShape {
            mode: UiMode::Shadow {
                offset: Vec2::ZERO,
                blur: 16.0,
                spread: 2.0,
            },
            ..path_shape(heart(100.0), cell(2, 1), solid(ink[2]))
        });
        o.ui.shapes.push(path_shape(
            heart(100.0),
            cell(2, 1),
            solid([0.02, 0.02, 0.03, 1.0]),
        ));
        let mut circle = UiPath::new();
        circle.ellipse(Vec2::ZERO, Vec2::splat(60.0));
        let morph = square_path(60.0)
            .lerp(&circle, 0.5)
            .expect("the same verbs");
        o.ui.shapes
            .push(path_shape(morph, cell(3, 1), solid(ink[3])));
        let (s, co) = 25f32.to_radians().sin_cos();
        o.ui.shapes.push(UiShape {
            // Squashed to 70 % wide, then turned 25° clockwise.
            transform: [co * 0.7, s * 0.7, -s, co],
            ..path_shape(heart(130.0), cell(4, 1), solid(ink[4]))
        });
        let c = cell(5, 1);
        o.ui.shapes.push(path_shape(
            star(30.0, UiFillRule::EvenOdd),
            [c[0] + 10.0, c[1] + 10.0, 80.0, 80.0],
            solid(ink[5]),
        ));
        o.ui.shapes.push(UiShape {
            mode: UiMode::Stroke(UiStroke {
                width: 2.0,
                align: UiAlign::Inside,
                ..Default::default()
            }),
            ..path_shape(
                star(30.0, UiFillRule::NonZero),
                [c[0] + 90.0, c[1] + 60.0, 80.0, 80.0],
                solid(ink[5]),
            )
        });
    })
}
