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
//! - `ui-text-effects`: text outline in, out and across, a gradient fill
//!   and stacked shadows.
//! - `ui-vector` (native: cooked by og-cook): an SVG emblem tessellated and
//!   drawn at three sizes and turned.

use og_core::glam::{Vec2, Vec3};
use og_render::{
    Background, ColorSpace, FontHandle, Mips, Overlay, RenderScene, Text, TextureData,
    TextureHandle, Tonemap, UiAlign, UiBlend, UiBuiltin, UiCap, UiClip, UiCorner, UiEdge, UiFit,
    UiGeometry, UiGradient, UiGradientKind, UiImage, UiJoin, UiLayer, UiMask, UiMaterial, UiMode,
    UiPaint, UiShape, UiSpread, UiStop, UiStroke, UiTextEffects, UiTextOutline, UiTextShadow,
};

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
fn overlay_scene(build: impl FnOnce(&mut RenderScene, &mut Overlay)) -> RenderScene {
    let mut scene = RenderScene::new();
    let mut o = Overlay::new(W, H, 0.0);
    build(&mut scene, &mut o);
    scene.overlay = Some(o);
    scene.environment.background = Background::Color(Vec3::new(0.012, 0.014, 0.02));
    scene.post.tonemap = Tonemap::None;
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
                tint: [1.0; 4],
                fit,
                focal: Vec2::splat(0.5),
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

/// Bright diagonal stripes over the frame, under the effects (layer 0).
fn stripes(o: &mut Overlay) {
    let colors = [
        rgb(244, 67, 54),
        rgb(255, 193, 7),
        rgb(76, 175, 80),
        rgb(33, 150, 243),
        rgb(156, 39, 176),
    ];
    let (s, c) = 35f32.to_radians().sin_cos();
    for k in 0..14 {
        let x = -200.0 + k as f32 * 120.0;
        o.ui.shapes.push(UiShape {
            transform: [c, s, -s, c],
            ..UiShape::rect([x, -300.0, 40.0, 1400.0], 0.0, colors[k % colors.len()])
        });
    }
}

/// `ui-effects`. See the module documentation.
pub fn effects() -> RenderScene {
    overlay_scene(|_, o| {
        stripes(o);
        // A grey band under rows 0 and 1, so shadows and glows read.
        o.ui.shapes.push(UiShape::rect(
            [20.0, 20.0, 1240.0, 330.0],
            24.0,
            [0.18, 0.19, 0.21, 1.0],
        ));
        let card = |col: usize, row: usize| inset(cell(col, row), 22.0);
        let r = rect([16.0; 4], UiCorner::Round);
        let fill = [0.93, 0.94, 0.96, 1.0];
        let with = |s: UiShape, mode, color| UiShape {
            mode,
            paint: solid(color),
            ..s
        };
        let base = |col, row| shape(r.clone(), card(col, row), solid(fill));
        let layer = |mut s: UiShape, l| {
            s.layer = l;
            s
        };
        // Row 0: two drop shadows (key and ambient), an inner shadow, an
        // outer and an inner glow, and a spread shadow.
        let b = base(0, 0);
        o.ui.shapes.push(layer(
            with(
                b.clone(),
                UiMode::Shadow {
                    offset: Vec2::new(0.0, 2.0),
                    blur: 4.0,
                    spread: 0.0,
                },
                [0.0, 0.0, 0.0, 0.5],
            ),
            1,
        ));
        o.ui.shapes.push(layer(
            with(
                b.clone(),
                UiMode::Shadow {
                    offset: Vec2::new(0.0, 14.0),
                    blur: 28.0,
                    spread: 0.0,
                },
                [0.0, 0.0, 0.0, 0.6],
            ),
            1,
        ));
        o.ui.shapes.push(layer(b, 1));
        o.ui.shapes.push(layer(base(1, 0), 1));
        o.ui.shapes.push(layer(
            with(
                base(1, 0),
                UiMode::InnerShadow {
                    offset: Vec2::new(0.0, 6.0),
                    blur: 12.0,
                    spread: 0.0,
                },
                [0.0, 0.0, 0.0, 0.7],
            ),
            1,
        ));
        let dark = [0.05, 0.06, 0.09, 1.0];
        let cyan = rgb(0, 229, 255);
        let b = shape(r.clone(), card(2, 0), solid(dark));
        o.ui.shapes.push(layer(
            with(
                b.clone(),
                UiMode::Shadow {
                    offset: Vec2::ZERO,
                    blur: 24.0,
                    spread: 2.0,
                },
                cyan,
            ),
            1,
        ));
        o.ui.shapes.push(layer(b, 1));
        let b = shape(r.clone(), card(3, 0), solid(dark));
        o.ui.shapes.push(layer(b.clone(), 1));
        o.ui.shapes.push(layer(
            with(
                b,
                UiMode::InnerShadow {
                    offset: Vec2::ZERO,
                    blur: 20.0,
                    spread: 0.0,
                },
                rgb(255, 64, 129),
            ),
            1,
        ));
        let b = shape(UiGeometry::Ellipse, card(4, 0), solid(rgb(255, 193, 7)));
        o.ui.shapes.push(layer(
            with(
                b.clone(),
                UiMode::Shadow {
                    offset: Vec2::new(8.0, 8.0),
                    blur: 0.0,
                    spread: 6.0,
                },
                [0.0, 0.0, 0.0, 0.8],
            ),
            1,
        ));
        o.ui.shapes.push(layer(b, 1));
        let star = UiGeometry::Polygon {
            sides: 5,
            inner: 0.45,
            rounding: 3.0,
            rotation: 0.0,
        };
        let b = shape(star, card(5, 0), solid(rgb(255, 235, 59)));
        o.ui.shapes.push(layer(
            with(
                b.clone(),
                UiMode::Shadow {
                    offset: Vec2::ZERO,
                    blur: 18.0,
                    spread: 0.0,
                },
                rgb(255, 160, 0),
            ),
            1,
        ));
        o.ui.shapes.push(layer(b, 1));
        // Row 1: blend modes over the stripes (left half over the band's
        // grey, right half over the stripes past its edge) and opacity.
        let modes = [
            UiBlend::Multiply,
            UiBlend::Screen,
            UiBlend::Add,
            UiBlend::Overlay,
            UiBlend::SoftLight,
            UiBlend::ColorDodge,
        ];
        for (col, blend) in modes.into_iter().enumerate() {
            let c = cell(col, 1);
            o.ui.shapes.push(layer(
                UiShape {
                    blend,
                    ..UiShape::rect(
                        [c[0] + 20.0, c[1] + 60.0, 140.0, 160.0],
                        12.0,
                        [0.25, 0.55, 0.95, 1.0],
                    )
                },
                1,
            ));
        }
        // Rows 2 and 3: glass over the stripes, clear and dark, and a half
        // opaque card.
        let glass = |r: [f32; 4], blur, saturation, brightness, tint: [f32; 4]| UiShape {
            mode: UiMode::Backdrop {
                blur,
                saturation,
                brightness,
            },
            paint: solid(tint),
            layer: 2,
            ..UiShape::rect(r, 24.0, [1.0; 4])
        };
        o.ui.shapes.push(glass(
            [60.0, 400.0, 360.0, 260.0],
            16.0,
            1.4,
            1.0,
            [1.0, 1.0, 1.0, 0.12],
        ));
        o.ui.shapes.push(glass(
            [460.0, 400.0, 360.0, 260.0],
            28.0,
            0.6,
            0.45,
            [0.02, 0.03, 0.06, 0.35],
        ));
        o.ui.shapes.push(layer(
            UiShape {
                opacity: 0.5,
                ..UiShape::rect([880.0, 420.0, 340.0, 220.0], 24.0, fill)
            },
            2,
        ));
        // A hairline over the glass's edge: its own layer draws over it.
        o.ui.shapes.push(layer(
            UiShape {
                mode: UiMode::Stroke(UiStroke {
                    width: 1.0,
                    align: UiAlign::Inside,
                    ..UiStroke::default()
                }),
                ..UiShape::rect([60.0, 400.0, 360.0, 260.0], 24.0, [1.0, 1.0, 1.0, 0.5])
            },
            3,
        ));
    })
}

/// A small card for a layer: a gradient ground, a star and a ring.
fn layer_card(o: &mut Overlay, r: [f32; 4], layer: i32) {
    o.ui.shapes.push(UiShape {
        layer,
        ..shape(
            rect([16.0; 4], UiCorner::Round),
            r,
            UiPaint::Gradient(gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 1.0),
                &[(0.0, rgb(98, 0, 234)), (1.0, rgb(0, 191, 165))],
            )),
        )
    });
    let mid = [r[0] + r[2] / 2.0, r[1] + r[3] / 2.0];
    o.ui.shapes.push(UiShape {
        layer: layer + 1,
        ..shape(
            UiGeometry::Polygon {
                sides: 5,
                inner: 0.45,
                rounding: 2.0,
                rotation: 0.0,
            },
            [mid[0] - 40.0, mid[1] - 40.0, 80.0, 80.0],
            solid(rgb(255, 235, 59)),
        )
    });
    o.ui.shapes.push(UiShape {
        layer: layer + 1,
        ..shape(
            UiGeometry::Arc {
                start: 0.0,
                sweep: std::f32::consts::TAU,
                thickness: 5.0,
                cap: UiCap::Butt,
            },
            [mid[0] - 55.0, mid[1] - 55.0, 110.0, 110.0],
            solid([1.0; 4]),
        )
    });
}

/// `ui-layers`. See the module documentation.
pub fn layers() -> RenderScene {
    overlay_scene(|scene, o| {
        for row in 0..3 {
            for col in 0..6 {
                backing(o, cell(col, row));
            }
        }
        // Row 0, cells 0 and 1: content cut to a circle (antialiased), and
        // rows of a list cut to a rounded panel and its rectangle.
        let c = cell(0, 0);
        let rect_clip = o.add_clip(c);
        let circle = [c[0] + 20.0, c[1] + 5.0, 140.0, 140.0];
        o.ui.clips.push(UiClip {
            rect: rect_clip,
            geometry: UiGeometry::Ellipse,
            center: Vec2::new(circle[0] + 70.0, circle[1] + 70.0),
            half: Vec2::splat(70.0),
            transform: [1.0, 0.0, 0.0, 1.0],
            parent: None,
        });
        let clip0 = o.ui.clips.len() as u32 - 1;
        for k in 0..6 {
            o.ui.shapes.push(UiShape {
                clip: Some(clip0),
                layer: 1,
                ..UiShape::rect(
                    [c[0], c[1] + k as f32 * 25.0, 180.0, 25.0],
                    0.0,
                    if k % 2 == 0 {
                        rgb(255, 112, 67)
                    } else {
                        rgb(255, 204, 128)
                    },
                )
            });
        }
        let c = cell(1, 0);
        let panel_clip = o.add_clip(c);
        o.ui.clips.push(UiClip {
            rect: panel_clip,
            geometry: rect([28.0; 4], UiCorner::Squircle(4.0)),
            center: Vec2::new(c[0] + 90.0, c[1] + 75.0),
            half: Vec2::new(90.0, 75.0),
            transform: [1.0, 0.0, 0.0, 1.0],
            parent: None,
        });
        let clip1 = o.ui.clips.len() as u32 - 1;
        for k in 0..5 {
            o.ui.shapes.push(UiShape {
                clip: Some(clip1),
                layer: 1,
                ..UiShape::rect(
                    [c[0] - 10.0, c[1] - 12.0 + k as f32 * 36.0, 200.0, 30.0],
                    6.0,
                    rgb(77, 208, 225),
                )
            });
        }
        // The layers: each card captures its own two overlay layers and
        // composites at layer 50 in its cell.
        let add = |o: &mut Overlay,
                   k: usize,
                   cell_r: [f32; 4],
                   material: UiMaterial,
                   params: [[f32; 4]; og_render::UI_MATERIAL_PARAMS],
                   mask: Option<UiMask>| {
            let base = 100 + 10 * k as i32;
            let r = inset(cell_r, 12.0);
            layer_card(o, r, base);
            o.ui.layers.push(UiLayer {
                capture: [base, base + 9],
                rect: inset(cell_r, -8.0),
                material,
                params,
                mask,
            });
            let n = o.ui.layers.len() as u32 - 1;
            o.ui.shapes.push(UiShape {
                layer: 50,
                ..shape(
                    rect([0.0; 4], UiCorner::Round),
                    inset(cell_r, -8.0),
                    UiPaint::Layer(n),
                )
            });
        };
        let none = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
        let p = |a: [f32; 4], b: [f32; 4]| {
            let mut out = none;
            out[0] = a;
            out[1] = b;
            out
        };
        let fade = gradient(
            UiGradientKind::Linear,
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            &[
                (0.0, [1.0; 4]),
                (0.6, [1.0; 4]),
                (1.0, [1.0, 1.0, 1.0, 0.0]),
            ],
        );
        let duotone = scene.add_ui_material(og_render::example_ui_material());
        let cases: [(usize, usize, UiMaterial, _, Option<UiMask>); 12] = [
            (2, 0, UiMaterial::Plain, none, Some(UiMask::Gradient(fade))),
            (
                3,
                0,
                UiMaterial::Builtin(UiBuiltin::Dissolve),
                p([0.45, 0.06, 18.0, 0.0], rgb(255, 145, 0)),
                None,
            ),
            (
                4,
                0,
                UiMaterial::Builtin(UiBuiltin::Hologram),
                p([4.0, 0.0, 2.0, 0.5], rgb(0, 229, 255)),
                None,
            ),
            (
                5,
                0,
                UiMaterial::Builtin(UiBuiltin::Pixelate),
                p([8.0, 0.0, 0.0, 0.0], [0.0; 4]),
                None,
            ),
            (
                0,
                1,
                UiMaterial::Builtin(UiBuiltin::Outline),
                p([3.0, 0.0, 0.0, 0.0], [1.0; 4]),
                None,
            ),
            (
                1,
                1,
                UiMaterial::Builtin(UiBuiltin::Blur),
                p([6.0, 0.0, 0.0, 0.0], [0.0; 4]),
                None,
            ),
            (
                2,
                1,
                UiMaterial::Builtin(UiBuiltin::Shine),
                p([0.5, 0.15, 0.8, 0.9], [1.0; 4]),
                None,
            ),
            (
                3,
                1,
                UiMaterial::Builtin(UiBuiltin::ColorAdjust),
                p([0.0, 0.0, 1.0, 1.0], [0.0; 4]),
                None,
            ),
            (
                4,
                1,
                UiMaterial::Builtin(UiBuiltin::Ripple),
                p([3.0, 24.0, 0.0, 0.0], [0.5, 0.5, 0.0, 0.0]),
                None,
            ),
            (
                5,
                1,
                UiMaterial::Builtin(UiBuiltin::Glitch),
                p([0.8, 10.0, 6.0, 0.0], [0.0; 4]),
                None,
            ),
            (
                0,
                2,
                UiMaterial::Custom(duotone),
                {
                    let m = og_render::example_ui_material();
                    m.defaults()
                },
                None,
            ),
            (
                1,
                2,
                UiMaterial::Builtin(UiBuiltin::Dissolve),
                p([0.45, 0.06, 18.0, 0.0], rgb(255, 145, 0)),
                Some(UiMask::Gradient(gradient(
                    UiGradientKind::Radial,
                    Vec2::new(0.5, 0.5),
                    Vec2::new(0.5, 0.5),
                    &[
                        (0.0, [1.0; 4]),
                        (0.8, [1.0; 4]),
                        (1.0, [1.0, 1.0, 1.0, 0.0]),
                    ],
                ))),
            ),
        ];
        for (k, (col, row, material, params, mask)) in cases.into_iter().enumerate() {
            add(o, k, cell(col, row), material, params, mask);
        }
        o.ui.time = 1.25;
    })
}

/// `ui-text-effects`: outline out, in and across; a gradient fill; stacked
/// shadows; all four together.
pub fn text_effects(
    font: impl FnOnce(&mut RenderScene) -> Result<FontHandle, String>,
) -> Result<RenderScene, String> {
    let mut err = None;
    let scene = overlay_scene(|scene, o| {
        let font = match font(scene) {
            Ok(f) => f,
            Err(e) => {
                err = Some(e);
                return;
            }
        };
        stripes(o);
        o.ui.shapes.push(UiShape::rect(
            [30.0, 30.0, 1220.0, 660.0],
            24.0,
            [0.06, 0.065, 0.08, 0.92],
        ));
        let line = |o: &mut Overlay, s: &str, y: f32, color: [f32; 4]| -> u32 {
            o.texts.push(Text {
                position: Vec3::new(70.0, y, 0.0),
                size: 72.0,
                color,
                layer: 1,
                ..Text::new(font, s)
            });
            o.texts.len() as u32 - 1
        };
        let white = [1.0; 4];
        let orange = rgb(255, 109, 0);
        let outline = |width, align| UiTextOutline {
            width,
            color: rgb(20, 20, 40),
            align,
        };
        let t = line(o, "Outline out 原点", 120.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            outline: Some(UiTextOutline {
                color: orange,
                ..outline(4.0, UiAlign::Outside)
            }),
            ..UiTextEffects::default()
        });
        let t = line(o, "Outline in", 230.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            outline: Some(UiTextOutline {
                color: orange,
                ..outline(1.5, UiAlign::Inside)
            }),
            ..UiTextEffects::default()
        });
        let t = line(o, "Gradient fill", 340.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            fill: Some(gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
                &[(0.0, rgb(255, 241, 118)), (1.0, rgb(255, 61, 0))],
            )),
            fill_box: [70.0, 340.0 - 60.0, 700.0, 80.0],
            ..UiTextEffects::default()
        });
        let t = line(o, "Two shadows", 450.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            shadows: vec![
                UiTextShadow {
                    offset: Vec2::new(4.0, 4.0),
                    blur: 0.0,
                    color: rgb(213, 0, 249),
                },
                UiTextShadow {
                    offset: Vec2::new(0.0, 10.0),
                    blur: 14.0,
                    color: [0.0, 0.0, 0.0, 0.9],
                },
            ],
            ..UiTextEffects::default()
        });
        let t = line(o, "VICTORY 原点", 590.0, white);
        o.ui.text_effects.push(UiTextEffects {
            text: t,
            shadows: vec![UiTextShadow {
                offset: Vec2::new(0.0, 6.0),
                blur: 10.0,
                color: [0.0, 0.0, 0.0, 0.8],
            }],
            outline: Some(UiTextOutline {
                width: 3.0,
                color: rgb(62, 39, 35),
                align: UiAlign::Outside,
            }),
            fill: Some(gradient(
                UiGradientKind::Linear,
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
                &[
                    (0.0, rgb(255, 245, 157)),
                    (0.55, rgb(255, 193, 7)),
                    (1.0, rgb(230, 81, 0)),
                ],
            )),
            fill_box: [70.0, 590.0 - 60.0, 700.0, 80.0],
            ..UiTextEffects::default()
        });
    });
    match err {
        Some(e) => Err(e),
        None => Ok(scene),
    }
}

/// An emblem in the SVG subset: a gradient shield with a stroked rim, a
/// star with round joins, a dashed ring of arcs, and an even-odd hole.
pub const EMBLEM_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120">
  <defs>
    <linearGradient id="metal" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffe082"/>
      <stop offset="1" stop-color="#ff6f00"/>
    </linearGradient>
    <radialGradient id="glow" cx="60" cy="56" r="40" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#4fc3f7"/>
      <stop offset="1" stop-color="#1a237e"/>
    </radialGradient>
  </defs>
  <path d="M60 6 L108 24 C108 70 90 98 60 114 C30 98 12 70 12 24 Z" fill="url(#metal)" stroke="#3e2723" stroke-width="4" stroke-linejoin="round"/>
  <path fill-rule="evenodd" fill="url(#glow)" d="M60 22 a34 34 0 1 0 0.01 0 Z M60 38 a18 18 0 1 1 -0.01 0 Z"/>
  <polygon points="60,40 65,53 79,53 68,61 72,75 60,67 48,75 52,61 41,53 55,53" fill="#fffde7" stroke="#ff6f00" stroke-width="2" stroke-linejoin="round"/>
  <circle cx="60" cy="56" r="44" fill="none" stroke="#fff" stroke-opacity="0.7" stroke-width="2" stroke-dasharray="6 4"/>
  <g transform="translate(60 102) rotate(45)"><rect x="-5" y="-5" width="10" height="10" rx="2" fill="#3e2723"/></g>
</svg>"##;

/// `ui-vector`: the emblem at 64, 160 and 360 CSS px, turned 20° at 160,
/// and tinted at 96.
#[cfg(not(target_arch = "wasm32"))]
pub fn vector() -> Result<RenderScene, String> {
    let data = og_cook::cook::vector::tessellate(EMBLEM_SVG, None)?;
    let mut err = None;
    let scene = overlay_scene(|scene, o| {
        let v = scene.add_ui_vector(data);
        let draw = |o: &mut Overlay, rect: [f32; 4], transform: [f32; 4], tint: [f32; 4]| {
            o.ui.vectors.push(og_render::UiVectorDraw {
                vector: v,
                rect,
                transform,
                tint,
                layer: 0,
                clip: None,
            });
        };
        let id = [1.0, 0.0, 0.0, 1.0];
        o.ui.shapes.push(UiShape::rect(
            [20.0, 20.0, 1240.0, 680.0],
            24.0,
            [0.08, 0.09, 0.12, 1.0],
        ));
        draw(o, [60.0, 60.0, 64.0, 64.0], id, [1.0; 4]);
        draw(o, [160.0, 60.0, 160.0, 160.0], id, [1.0; 4]);
        let (s, c) = 20f32.to_radians().sin_cos();
        draw(o, [360.0, 60.0, 160.0, 160.0], [c, s, -s, c], [1.0; 4]);
        draw(o, [60.0, 260.0, 96.0, 96.0], id, [0.4, 0.9, 1.0, 1.0]);
        draw(o, [600.0, 40.0, 640.0, 640.0], id, [1.0; 4]);
        if o.ui.vectors.is_empty() {
            err = Some("no vector drawn".to_string());
        }
    });
    match err {
        Some(e) => Err(e),
        None => Ok(scene),
    }
}
