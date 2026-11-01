use super::*;

/// Bright diagonal stripes over the frame, under the effects (layer 0).
pub(super) fn stripes(o: &mut Overlay) {
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
                target: None,
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
