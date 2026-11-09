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
                glass: Default::default(),
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
                ground: None,
                filter: None,
                backdrop: false,
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

/// A 640×720 texture of independent random texels: the finest detail
/// there is, the worst case of a photograph for a blur.
fn noise_texture(scene: &mut RenderScene) -> TextureHandle {
    let (w, h) = (640u32, 720u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    let mut s = 0x2545_f491_u32;
    for _ in 0..w * h {
        for _ in 0..3 {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            rgba.push((s >> 24) as u8);
        }
        rgba.push(255);
    }
    scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    })
}

/// Blurs of the finest detail: the left half a layer of random texels
/// through the blur material of `radius` CSS px, the right half the same
/// texels under a glass backdrop of `2 × radius` (the backdrop's reach for
/// a CSS blur of `radius`), whose bottom quarter is a hard edge, black to
/// the left of x 960 and white to its right. A correct blur of either is
/// smooth, and the edge a smooth ramp without kinks.
pub fn blur_detail(radius: f32) -> RenderScene {
    overlay_scene(|scene, o| {
        let noise = noise_texture(scene);
        let image = |r: [f32; 4], layer: i32| UiShape {
            layer,
            ..shape(
                rect([0.0; 4], UiCorner::Round),
                r,
                UiPaint::Image(UiImage {
                    texture: noise,
                    uv: [0.0, 0.0, 1.0, 1.0],
                    size: Vec2::new(640.0, 720.0),
                    tint: [1.0; 4],
                    fit: UiFit::Fill,
                    focal: Vec2::splat(0.5),
                    luminance: false,
                    offset: Default::default(),
                    nearest: false,
                }),
            )
        };
        let left = [0.0, 0.0, 640.0, 720.0];
        o.ui.shapes.push(image(left, 10));
        let mut params = [[0.0; 4]; og_render::UI_MATERIAL_PARAMS];
        params[0][0] = radius;
        o.ui.layers.push(UiLayer {
            capture: [10, 10],
            rect: left,
            material: UiMaterial::Builtin(UiBuiltin::Blur),
            params,
            mask: None,
            target: None,
            ground: None,
            filter: None,
            backdrop: false,
        });
        o.ui.shapes.push(UiShape {
            layer: 50,
            ..shape(rect([0.0; 4], UiCorner::Round), left, UiPaint::Layer(0))
        });
        let right = [640.0, 0.0, 640.0, 720.0];
        o.ui.shapes.push(image(right, 0));
        o.ui.shapes.push(UiShape::rect(
            [640.0, 540.0, 320.0, 180.0],
            0.0,
            [0.0, 0.0, 0.0, 1.0],
        ));
        o.ui.shapes
            .push(UiShape::rect([960.0, 540.0, 320.0, 180.0], 0.0, [1.0; 4]));
        o.ui.shapes.push(UiShape {
            mode: UiMode::Backdrop {
                blur: 2.0 * radius,
                saturation: 1.0,
                brightness: 1.0,
                glass: Default::default(),
            },
            paint: solid([0.0; 4]),
            layer: 1,
            ..UiShape::rect(right, 0.0, [1.0; 4])
        });
    })
}

/// `ui-ground`'s camera: at [`GROUND_EYE`] looking at the origin, 45°
/// vertically on the 1280×720 frame, 0.1 to 200 m.
pub const GROUND_EYE: Vec3 = Vec3::new(0.0, 5.5, 8.5);

/// `ui-ground`'s projection times view, as og-render's camera makes it.
pub fn ground_view() -> og_core::glam::Mat4 {
    use og_core::glam::{Mat4, Vec4};
    // look_at_rh and perspective_rh by hand (og_core's maths, not libm).
    let (eye, near, far) = (GROUND_EYE, 0.1, 200.0);
    let f = (-eye).normalize();
    let side = f.cross(Vec3::Y).normalize();
    let u = side.cross(f);
    let view = Mat4::from_cols(
        Vec4::new(side.x, u.x, -f.x, 0.0),
        Vec4::new(side.y, u.y, -f.y, 0.0),
        Vec4::new(side.z, u.z, -f.z, 0.0),
        Vec4::new(-side.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
    );
    let (s, c) = og_core::math::sin_cos(22.5f32.to_radians());
    let k = c / s;
    let r = far / (near - far);
    let proj = Mat4::from_cols(
        Vec4::new(k / (W / H), 0.0, 0.0, 0.0),
        Vec4::new(0.0, k, 0.0, 0.0),
        Vec4::new(0.0, 0.0, r, -1.0),
        Vec4::new(0.0, 0.0, r * near, 0.0),
    );
    proj * view
}

/// The terrain under `ui-ground`'s decal at (x, z): waves of ±0.5 m and a
/// 0.45 m mound right of centre, within the decal's 1.5 m reach.
pub fn ground_height(x: f32, z: f32) -> f32 {
    0.5 * (0.8 * x).sin() * (0.6 * z).cos()
        + 0.45 * (-((x - 1.0).powi(2) + (z + 0.5).powi(2)) / 1.5).exp()
}

/// `ui-ground`'s pillar: 0.5 × 2.4 × 0.5 m standing at (1.8, _, 1.4) on
/// the terrain's height there, its min and max corners.
pub fn ground_pillar() -> (Vec3, Vec3) {
    let base = ground_height(1.8, 1.4);
    (
        Vec3::new(1.55, base - 0.3, 1.15),
        Vec3::new(2.05, base + 2.4, 1.65),
    )
}

/// `ui-ground` (docs/ui.md §1): a layer drawn off the screen (a checker of
/// 8×8 squares, 50 px each, and a ring) laid as a document's `<ground>` is
/// on a 6 × 6 m horizontal plane at the origin, its top edge the far one,
/// over the waves of [`ground_height`] and a pillar standing in it: the
/// checker follows the terrain's rise and fall (a decal against the
/// depth prepass, [`og_render::UiGround`]), the pillar hides what is
/// behind it and its sides take none, and the sky shows past the rect.
pub fn ground() -> RenderScene {
    use og_render::{Camera, Material, Projection, Vertex, shapes};
    let mut scene = RenderScene::new();
    // The terrain: a 20 m heightfield, 0.2 m cells.
    let n = 100u32;
    let at = |i: u32, j: u32| {
        let (x, z) = (
            -10.0 + 20.0 * i as f32 / n as f32,
            -10.0 + 20.0 * j as f32 / n as f32,
        );
        Vec3::new(x, ground_height(x, z), z)
    };
    let mut mesh = og_render::MeshData {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    for j in 0..=n {
        for i in 0..=n {
            let p = at(i, j);
            let e = 0.01;
            let normal = Vec3::new(
                ground_height(p.x - e, p.z) - ground_height(p.x + e, p.z),
                2.0 * e,
                ground_height(p.x, p.z - e) - ground_height(p.x, p.z + e),
            )
            .normalize();
            mesh.vertices.push(Vertex {
                position: p.into(),
                normal: normal.into(),
                uv: [p.x / 2.0, p.z / 2.0],
                color: [1.0; 4],
                tangent: [0.0; 4],
            });
        }
    }
    for j in 0..n {
        for i in 0..n {
            let a = j * (n + 1) + i;
            let b = a + n + 1;
            mesh.indices
                .extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let terrain = scene.add_mesh(mesh);
    let grass = scene.add_material(Material {
        base_color: [0.32, 0.42, 0.28, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(terrain, grass, og_core::glam::Mat4::IDENTITY);
    let (lo, hi) = ground_pillar();
    let pillar = scene.add_mesh(shapes::cuboid(hi - lo));
    let stone = scene.add_material(Material {
        base_color: [0.6, 0.55, 0.5, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(
        pillar,
        stone,
        og_core::glam::Mat4::from_translation((lo + hi) / 2.0),
    );
    scene.camera = Camera {
        position: GROUND_EYE,
        target: Vec3::ZERO,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 45.0,
        },
        near: 0.1,
        far: 200.0,
        ..Default::default()
    };
    scene.environment.background = Background::Color(Vec3::new(0.25, 0.35, 0.55));
    scene.post.tonemap = Tonemap::None;
    let mut o = Overlay::new(W, H, 0.0);
    let (x0, y0) = (100_000.0, 0.0);
    for j in 0..8 {
        for i in 0..8 {
            let c = if (i + j) % 2 == 0 {
                rgb(240, 180, 60)
            } else {
                rgb(40, 90, 200)
            };
            o.ui.shapes.push(UiShape {
                layer: 10,
                ..UiShape::rect(
                    [x0 + i as f32 * 50.0, y0 + j as f32 * 50.0, 50.0, 50.0],
                    0.0,
                    c,
                )
            });
        }
    }
    o.ui.shapes.push(UiShape {
        layer: 11,
        ..shape(
            UiGeometry::Arc {
                start: 0.0,
                sweep: std::f32::consts::TAU,
                thickness: 20.0,
                cap: UiCap::Round,
            },
            [x0 + 100.0, y0 + 100.0, 200.0, 200.0],
            solid(rgb(250, 250, 250)),
        )
    });
    let view = ground_view();
    let corners = [
        Vec3::new(-3.0, 0.0, -3.0),
        Vec3::new(3.0, 0.0, -3.0),
        Vec3::new(3.0, 0.0, 3.0),
        Vec3::new(-3.0, 0.0, 3.0),
    ];
    let seen = corners.map(|p| {
        let c = view.project_point3(p);
        Vec2::new((c.x + 1.0) * 0.5 * W, (1.0 - c.y) * 0.5 * H)
    });
    o.ui.layers.push(UiLayer {
        capture: [10, 19],
        rect: [x0, y0, 400.0, 400.0],
        material: UiMaterial::Plain,
        params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
        mask: None,
        target: None,
        filter: None,
        backdrop: false,
        ground: Some(og_render::UiGround {
            corners: Some(seen),
            drape: og_render::UiDrape::new(
                view,
                GROUND_EYE,
                (-GROUND_EYE).normalize(),
                Vec2::new(W, H),
                [corners[0], corners[1], corners[3]],
                [x0, y0, 400.0, 400.0],
                1.5,
            )
            .expect("a plane in view"),
            plane: false,
        }),
    });
    // A backdrop inside it over its first two columns: its filter
    // (sRGB inversion) runs over what the ground's layer drew under it,
    // in that layer's own pixels (its rect is far off the screen).
    let backdrop = [x0, y0, 100.0, 400.0];
    o.ui.layers.push(UiLayer {
        capture: [20, 29],
        rect: backdrop,
        material: UiMaterial::Plain,
        params: [[0.0; 4]; og_render::UI_MATERIAL_PARAMS],
        mask: None,
        target: None,
        filter: Some(og_render::UiFilter {
            primitives: vec![og_render::UiFilterPrimitive {
                op: og_render::UiFilterOp::ColorMatrix([
                    -1.0, 0.0, 0.0, 0.0, 1.0, //
                    0.0, -1.0, 0.0, 0.0, 1.0, //
                    0.0, 0.0, -1.0, 0.0, 1.0, //
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ]),
                inputs: Default::default(),
                subregion: [-1.0e6, -1.0e6, 2.0e6, 2.0e6],
                srgb: true,
            }],
            ..Default::default()
        }),
        backdrop: true,
        ground: None,
    });
    o.ui.shapes.push(UiShape {
        layer: 12,
        ..shape(rect([0.0; 4], UiCorner::Round), backdrop, UiPaint::Layer(1))
    });
    // The composite covers the screen; the drape cuts it.
    o.ui.shapes.push(UiShape {
        layer: 50,
        ..shape(
            rect([0.0; 4], UiCorner::Round),
            [0.0, 0.0, W, H],
            UiPaint::Layer(0),
        )
    });
    scene.overlay = Some(o);
    scene
}
