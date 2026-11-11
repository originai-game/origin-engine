//! The UI pass's scenes (`docs/ui.md` U1, §5.1–§5.7): every primitive of
//! og-render's `ui` module in the overlay over a flat ground, 1280×720 CSS
//! px fitted to the output, each in a cell of its own so a frame reads as a
//! table of what the pass draws.
//!
//! - `ui-shapes`: rectangles with each corner shape and four radii, a
//!   capsule, circle, ellipse, rings and arcs with their caps, polygons and
//!   stars, polylines with joins and caps, a Bézier, dashed strokes and a
//!   rotated rectangle.
//! - `ui-paints`: the four gradient kinds in OKLab with spreads, image fits,
//!   nine-slices, a pattern, strokes with their own paint and alignment.
//! - `ui-effects`: drop shadows, inner shadow, glows, backdrop glass over
//!   stripes, the blend modes and opacity.
//! - `ui-layers`: a clip to a circle and to a rounded panel, a gradient
//!   mask, the built-in materials and a custom one.
//! - `ui-ground`: a layer draped as a decal over a heightfield, a pillar
//!   standing in it (a document's `<ground>`).
//! - `ui-text-effects`: text outline in, out and across, a gradient fill
//!   and stacked shadows.
//! - `ui-glass-*` (§5.3): flat glass's weights, hairline, shadows, the
//!   scrim and the fallbacks over bright, dark and busy backdrops
//!   (`glass.rs`).
//! - `ui-paths`: run-time vector paths, filled by both rules, under a
//!   gradient, stroked, shadowed, morphed and transformed (`paths.rs`).
//! - `ui-world` (`docs/ui.md` §1): mass elements (damage numbers,
//!   nameplates faded behind a wall, bars with a ghost, icons, a compact
//!   score) and two occluded panels over a lit 3D scene.

use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, FontHandle, Mips, Overlay, RenderScene, Text, TextureData,
    TextureHandle, Tonemap, UiAlign, UiBlend, UiBuiltin, UiCap, UiClip, UiCorner, UiEdge, UiFit,
    UiGeometry, UiGradient, UiGradientKind, UiImage, UiJoin, UiLayer, UiMask, UiMaterial, UiMode,
    UiPaint, UiShape, UiSpread, UiStop, UiStroke, UiTextEffects, UiTextOutline, UiTextShadow,
};

mod effects_layers;
mod glass;
mod paths;
mod text_world;

pub use effects_layers::{
    GROUND_EYE, blur_detail, effects, ground, ground_height, ground_pillar, ground_view, layers,
};
pub use glass::{GlassInk, GlassScene, glass_scene};
pub use paths::{heart, paths, star};
pub use text_world::*;

const W: f32 = 1280.0;
const H: f32 = 720.0;

/// The cells: 6 columns of 180×150 CSS px from (40, 40), 200 and 170 apart.
fn cell(col: usize, row: usize) -> [f32; 4] {
    [
        40.0 + col as f32 * 200.0,
        40.0 + row as f32 * 170.0,
        180.0,
        150.0,
    ]
}

/// `r` shrunk by `by` on every side.
fn inset(r: [f32; 4], by: f32) -> [f32; 4] {
    [r[0] + by, r[1] + by, r[2] - 2.0 * by, r[3] - 2.0 * by]
}

fn shape(geometry: UiGeometry, r: [f32; 4], paint: UiPaint) -> UiShape {
    UiShape {
        geometry,
        paint,
        ..UiShape::rect(r, 0.0, [1.0; 4])
    }
}

fn rect(radii: [f32; 4], corner: UiCorner) -> UiGeometry {
    UiGeometry::Rect { radii, corner }
}

fn solid(c: [f32; 4]) -> UiPaint {
    UiPaint::Solid(c)
}

/// sRGB bytes as linear RGBA.
fn rgb(r: u8, g: u8, b: u8) -> [f32; 4] {
    let l = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    [l(r), l(g), l(b), 1.0]
}

fn gradient(kind: UiGradientKind, from: Vec2, to: Vec2, stops: &[(f32, [f32; 4])]) -> UiGradient {
    UiGradient {
        kind,
        from,
        to,
        stops: stops
            .iter()
            .map(|&(offset, color)| UiStop { offset, color })
            .collect(),
        ..UiGradient::default()
    }
}

/// A scene with an overlay of `ui` over a dark slate ground, untonemapped.
pub fn overlay_scene(build: impl FnOnce(&mut RenderScene, &mut Overlay)) -> RenderScene {
    let mut scene = RenderScene::new();
    let mut o = Overlay::new(W, H, 0.0);
    build(&mut scene, &mut o);
    scene.ui.overlay = Some(o);
    scene.environment.background = Background::Color(Vec3::new(0.012, 0.014, 0.02));
    scene.look.post.tonemap = Tonemap::None;
    scene
}

/// A cell's backing card, so each primitive reads against a ground.
fn backing(o: &mut Overlay, r: [f32; 4]) {
    o.ui.shapes
        .push(UiShape::rect(r, 10.0, [0.03, 0.035, 0.045, 1.0]));
}

/// `ui-shapes`. See the module documentation.
pub fn shapes() -> RenderScene {
    overlay_scene(|_, o| {
        let ink = [
            rgb(255, 183, 77),
            rgb(77, 208, 225),
            rgb(186, 104, 200),
            rgb(129, 199, 132),
            rgb(239, 83, 80),
            rgb(255, 241, 118),
        ];
        for row in 0..4 {
            for col in 0..6 {
                backing(o, cell(col, row));
            }
        }
        let tau = std::f32::consts::TAU;
        // Row 0: corner shapes, four radii, a capsule.
        let corners = [
            rect([28.0; 4], UiCorner::Round),
            rect([48.0; 4], UiCorner::Squircle(4.0)),
            rect([28.0; 4], UiCorner::Bevel),
            rect([28.0; 4], UiCorner::Notch),
            rect([0.0, 20.0, 40.0, 60.0], UiCorner::Round),
        ];
        for (col, g) in corners.into_iter().enumerate() {
            o.ui.shapes
                .push(shape(g, inset(cell(col, 0), 20.0), solid(ink[col])));
        }
        let c = cell(5, 0);
        o.ui.shapes.push(shape(
            rect([30.0; 4], UiCorner::Round),
            [c[0] + 20.0, c[1] + 45.0, 140.0, 60.0],
            solid(ink[5]),
        ));
        // Row 1: circle, ellipse, ring, arcs with round and butt caps, a pie.
        let square = |col: usize, row: usize| {
            let c = cell(col, row);
            [c[0] + 25.0, c[1] + 10.0, 130.0, 130.0]
        };
        o.ui.shapes
            .push(shape(UiGeometry::Ellipse, square(0, 1), solid(ink[0])));
        o.ui.shapes.push(shape(
            UiGeometry::Ellipse,
            inset(cell(1, 1), 15.0),
            solid(ink[1]),
        ));
        let arc = |start: f32, sweep: f32, thickness: f32, cap: UiCap| UiGeometry::Arc {
            start,
            sweep,
            thickness,
            cap,
        };
        o.ui.shapes.push(shape(
            arc(0.0, tau, 16.0, UiCap::Butt),
            square(2, 1),
            solid(ink[2]),
        ));
        o.ui.shapes.push(shape(
            arc(-tau / 4.0, 0.75 * tau, 18.0, UiCap::Round),
            square(3, 1),
            solid(ink[3]),
        ));
        o.ui.shapes.push(shape(
            arc(-tau / 4.0, tau / 3.0, 26.0, UiCap::Butt),
            square(4, 1),
            solid(ink[4]),
        ));
        o.ui.shapes.push(shape(
            arc(-tau / 4.0, 0.7 * tau, 65.0, UiCap::Butt),
            square(5, 1),
            solid(ink[5]),
        ));
        // Row 2: polygons and stars, polylines with joins and caps.
        let polygon = |sides: u32, inner: f32, rounding: f32| UiGeometry::Polygon {
            sides,
            inner,
            rounding,
            rotation: 0.0,
        };
        let polys = [
            polygon(6, 1.0, 0.0),
            polygon(5, 0.45, 0.0),
            polygon(3, 1.0, 12.0),
            polygon(8, 0.6, 4.0),
        ];
        for (col, g) in polys.into_iter().enumerate() {
            o.ui.shapes.push(shape(g, square(col, 2), solid(ink[col])));
        }
        let zigzag: Vec<Vec2> = (0..5)
            .map(|i| {
                Vec2::new(
                    -60.0 + 30.0 * i as f32,
                    if i % 2 == 0 { 35.0 } else { -35.0 },
                )
            })
            .collect();
        for (col, cap, join) in [
            (4, UiCap::Round, UiJoin::Round),
            (5, UiCap::Square, UiJoin::Miter(4.0)),
        ] {
            o.ui.shapes.push(shape(
                UiGeometry::Polyline {
                    points: zigzag.clone(),
                    closed: false,
                    filled: false,
                    width: 12.0,
                    cap,
                    join,
                },
                cell(col, 2),
                solid(ink[col]),
            ));
        }
        // Row 3: a Bézier, dashed strokes, strokes by alignment, a turned
        // rectangle, a closed bevelled outline.
        let bezier = og_render::flatten_cubic(
            Vec2::new(-70.0, 50.0),
            Vec2::new(-40.0, -90.0),
            Vec2::new(40.0, 90.0),
            Vec2::new(70.0, -50.0),
            0.25,
        );
        o.ui.shapes.push(shape(
            UiGeometry::Polyline {
                points: bezier,
                closed: false,
                filled: false,
                width: 6.0,
                cap: UiCap::Round,
                join: UiJoin::Round,
            },
            cell(0, 3),
            solid(ink[0]),
        ));
        let stroke = |width: f32, align: UiAlign, dash: [f32; 2], cap: UiCap| {
            UiMode::Stroke(UiStroke {
                width,
                align,
                dash,
                dash_offset: 0.0,
                dash_cap: cap,
            })
        };
        o.ui.shapes.push(UiShape {
            mode: stroke(4.0, UiAlign::Center, [14.0, 8.0], UiCap::Butt),
            ..shape(
                rect([18.0; 4], UiCorner::Round),
                inset(cell(1, 3), 22.0),
                solid(ink[1]),
            )
        });
        o.ui.shapes.push(UiShape {
            mode: stroke(8.0, UiAlign::Center, [0.1, 16.0], UiCap::Round),
            ..shape(UiGeometry::Ellipse, square(2, 3), solid(ink[2]))
        });
        // Inside, across and outside the same outline (a hairline marks it).
        let c = cell(3, 3);
        for (k, align) in [UiAlign::Inside, UiAlign::Center, UiAlign::Outside]
            .into_iter()
            .enumerate()
        {
            let r = [c[0] + 18.0 + 52.0 * k as f32, c[1] + 50.0, 40.0, 50.0];
            o.ui.shapes.push(UiShape {
                mode: stroke(8.0, align, [0.0, 0.0], UiCap::Butt),
                ..shape(rect([6.0; 4], UiCorner::Round), r, solid(ink[3]))
            });
            o.ui.shapes.push(UiShape {
                mode: stroke(1.0, UiAlign::Center, [0.0, 0.0], UiCap::Butt),
                ..shape(rect([6.0; 4], UiCorner::Round), r, solid([1.0; 4]))
            });
        }
        let (s, co) = 30f32.to_radians().sin_cos();
        o.ui.shapes.push(UiShape {
            transform: [co, s, -s, co],
            ..shape(
                rect([12.0; 4], UiCorner::Squircle(5.0)),
                inset(cell(4, 3), 35.0),
                solid(ink[4]),
            )
        });
        o.ui.shapes.push(shape(
            UiGeometry::Polyline {
                points: vec![
                    Vec2::new(0.0, -55.0),
                    Vec2::new(60.0, 45.0),
                    Vec2::new(-60.0, 45.0),
                ],
                closed: true,
                filled: false,
                width: 10.0,
                cap: UiCap::Butt,
                join: UiJoin::Bevel,
            },
            cell(5, 3),
            solid(ink[5]),
        ));
    })
}

/// A 64×64 test image: a 16 px frame with orange corners (a white dot in
/// each) and teal edges round a navy and blue checker, so a fit, a crop and
/// a nine-slice show where each part went.
fn frame_image(scene: &mut RenderScene) -> TextureHandle {
    let mut rgba = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64u32 {
        for x in 0..64u32 {
            let edge_x = !(16..48).contains(&x);
            let edge_y = !(16..48).contains(&y);
            let c: [u8; 3] = if edge_x && edge_y {
                let (cx, cy) = (if x < 16 { 8 } else { 56 }, if y < 16 { 8 } else { 56 });
                let d2 = (x as i32 - cx).pow(2) + (y as i32 - cy).pow(2);
                if d2 < 16 {
                    [255, 255, 255]
                } else {
                    [245, 124, 0]
                }
            } else if edge_x || edge_y {
                [0, 150, 136]
            } else if ((x / 8) + (y / 8)) % 2 == 0 {
                [26, 35, 126]
            } else {
                [66, 165, 245]
            };
            rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    scene.add_texture(TextureData {
        width: 64,
        height: 64,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

/// A 128×64 picture: a sky gradient, a sun left of centre and a hill.
fn picture(scene: &mut RenderScene) -> TextureHandle {
    let mut rgba = Vec::with_capacity(128 * 64 * 4);
    for y in 0..64u32 {
        for x in 0..128u32 {
            let t = y as f32 / 63.0;
            let mut c = [
                (40.0 + 150.0 * t) as u8,
                (90.0 + 100.0 * t) as u8,
                (200.0 - 40.0 * t) as u8,
            ];
            let (dx, dy) = (x as f32 - 40.0, y as f32 - 22.0);
            if dx * dx + dy * dy < 100.0 {
                c = [255, 214, 79];
            }
            let hill = 50.0 - 14.0 * ((x as f32 - 90.0) / 30.0).cos().max(0.0);
            if y as f32 > hill {
                c = [56, 142, 60];
            }
            rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    scene.add_texture(TextureData {
        width: 128,
        height: 64,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

/// `ui-paints`. See the module documentation.
pub fn paints() -> RenderScene {
    overlay_scene(|scene, o| {
        let frame = frame_image(scene);
        let pic = picture(scene);
        for row in 0..3 {
            for col in 0..6 {
                backing(o, cell(col, row));
            }
        }
        let blue = rgb(33, 66, 255);
        let yellow = rgb(255, 230, 0);
        let card = |col: usize, row: usize| inset(cell(col, row), 14.0);
        let round = rect([14.0; 4], UiCorner::Round);
        // Row 0: blue to yellow through OKLab (no grey middle), a rainbow,
        // radial, conic, diamond, and a repeating reflected band.
        let rainbow = [
            (0.0, rgb(229, 57, 53)),
            (0.2, rgb(251, 140, 0)),
            (0.4, rgb(253, 216, 53)),
            (0.6, rgb(67, 160, 71)),
            (0.8, rgb(30, 136, 229)),
            (1.0, rgb(142, 36, 170)),
        ];
        let grads = [
            gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
                &[(0.0, blue), (1.0, yellow)],
            ),
            gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 1.0),
                &rainbow,
            ),
            gradient(
                UiGradientKind::Radial,
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 0.5),
                &[
                    (0.0, [1.0; 4]),
                    (0.5, rgb(255, 112, 67)),
                    (1.0, rgb(40, 20, 60)),
                ],
            ),
            gradient(
                UiGradientKind::Conic,
                Vec2::new(0.5, 0.5),
                Vec2::new(-std::f32::consts::FRAC_PI_2, 0.0),
                &rainbow,
            ),
            gradient(
                UiGradientKind::Diamond,
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 0.5),
                &[(0.0, yellow), (1.0, rgb(0, 121, 107))],
            ),
            UiGradient {
                spread: UiSpread::Reflect,
                ..gradient(
                    UiGradientKind::Linear,
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.25, 0.0),
                    &[(0.0, rgb(255, 64, 129)), (1.0, rgb(48, 63, 159))],
                )
            },
        ];
        for (col, g) in grads.into_iter().enumerate() {
            o.ui.shapes
                .push(shape(round.clone(), card(col, 0), UiPaint::Gradient(g)));
        }
        // Row 1: fill, cover, contain (focal left), none, nine-slices.
        let image = |texture, size: Vec2, fit| {
            UiPaint::Image(UiImage {
                texture,
                uv: [0.0, 0.0, 1.0, 1.0],
                size,
                density: 1.0,
                tint: [1.0; 4],
                fit,
                focal: Vec2::splat(0.5),
                luminance: false,
                offset: Default::default(),
                nearest: false,
            })
        };
        let pic_size = Vec2::new(128.0, 64.0);
        o.ui.shapes.push(shape(
            round.clone(),
            card(0, 1),
            image(pic, pic_size, UiFit::Fill),
        ));
        o.ui.shapes.push(shape(
            round.clone(),
            card(1, 1),
            image(pic, pic_size, UiFit::Cover),
        ));
        let mut contain = image(pic, pic_size, UiFit::Contain);
        if let UiPaint::Image(im) = &mut contain {
            im.focal = Vec2::new(0.5, 0.0);
        }
        o.ui.shapes.push(shape(round.clone(), card(2, 1), contain));
        o.ui.shapes.push(shape(
            round.clone(),
            card(3, 1),
            image(pic, pic_size, UiFit::None),
        ));
        let slice = |edges, center| UiFit::NineSlice {
            source: [16.0; 4],
            dest: [24.0; 4],
            edges,
            center,
        };
        let s64 = Vec2::splat(64.0);
        o.ui.shapes.push(shape(
            rect([0.0; 4], UiCorner::Round),
            card(4, 1),
            image(frame, s64, slice(UiEdge::Stretch, true)),
        ));
        o.ui.shapes.push(shape(
            rect([0.0; 4], UiCorner::Round),
            card(5, 1),
            image(frame, s64, slice(UiEdge::Round, false)),
        ));
        // Row 2: a pattern, strokes with their own paint and alignment.
        o.ui.shapes.push(shape(
            UiGeometry::Ellipse,
            card(0, 2),
            image(
                frame,
                s64,
                UiFit::Pattern {
                    tile: Vec2::splat(40.0),
                    rotation: 0.4,
                    offset: Vec2::new(5.0, 0.0),
                    scroll: Vec2::new(30.0, 0.0),
                },
            ),
        ));
        let stroke = |width: f32, align: UiAlign| {
            UiMode::Stroke(UiStroke {
                width,
                align,
                ..UiStroke::default()
            })
        };
        let border = gradient(
            UiGradientKind::Linear,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            &[(0.0, rgb(0, 229, 255)), (1.0, rgb(213, 0, 249))],
        );
        o.ui.shapes
            .push(UiShape::rect(card(1, 2), 18.0, [0.06, 0.07, 0.1, 1.0]));
        o.ui.shapes.push(UiShape {
            mode: stroke(6.0, UiAlign::Inside),
            ..shape(
                rect([18.0; 4], UiCorner::Round),
                card(1, 2),
                UiPaint::Gradient(border.clone()),
            )
        });
        for (col, align) in [
            (2, UiAlign::Inside),
            (3, UiAlign::Center),
            (4, UiAlign::Outside),
        ] {
            let r = inset(cell(col, 2), 30.0);
            o.ui.shapes.push(UiShape {
                mode: stroke(12.0, align),
                ..shape(
                    rect([20.0; 4], UiCorner::Squircle(4.0)),
                    r,
                    UiPaint::Gradient(border.clone()),
                )
            });
            o.ui.shapes.push(UiShape {
                mode: stroke(1.0, UiAlign::Center),
                ..shape(rect([20.0; 4], UiCorner::Squircle(4.0)), r, solid([1.0; 4]))
            });
        }
        o.ui.shapes.push(UiShape {
            mode: UiMode::Stroke(UiStroke {
                width: 10.0,
                align: UiAlign::Inside,
                dash: [18.0, 10.0],
                dash_offset: 0.0,
                dash_cap: UiCap::Round,
            }),
            ..shape(
                UiGeometry::Ellipse,
                card(5, 2),
                UiPaint::Gradient(gradient(
                    UiGradientKind::Conic,
                    Vec2::new(0.5, 0.5),
                    Vec2::ZERO,
                    &[(0.0, yellow), (0.5, rgb(255, 61, 0)), (1.0, yellow)],
                )),
            )
        });
    })
}
