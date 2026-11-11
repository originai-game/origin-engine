//! The UI pass (`docs/ui.md` U1; `src/ui_scenes.rs`): each scene against its
//! reference frame, and what each primitive must draw read from the frame
//! at the pixels where a wrong implementation differs (a corner cut the
//! wrong way, a gradient mixed in sRGB, a ring without its hole, a clip
//! that leaks, a blur that is not one).

use og_tests::gfx::{self, Rendered};
use og_tests::scenes::{self, GfxScene};

fn render(scene: &str) -> Rendered {
    let mut built: GfxScene = scenes::build(scene).expect("scene");
    gfx::render_built(&mut built).expect("render")
}

/// The sRGB bytes of pixel (x, y) (the overlay is 1280×720 CSS px on a
/// 1280×720 frame: one CSS px a pixel).
fn px(r: &Rendered, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * r.image.width + x) * 4) as usize;
    let p = &r.image.pixels;
    [f32::from(p[i]), f32::from(p[i + 1]), f32::from(p[i + 2])]
}

fn srgb8(linear: [f32; 4]) -> [f32; 3] {
    let e = |c: f32| {
        let c = c.clamp(0.0, 1.0);
        255.0
            * if c <= 0.003_130_8 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
    };
    [e(linear[0]), e(linear[1]), e(linear[2])]
}

fn near(a: [f32; 3], b: [f32; 3], within: f32) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() <= within)
}

/// The cell a primitive sits in (`ui_scenes::cell`).
fn cell(col: u32, row: u32) -> (u32, u32) {
    (40 + col * 200, 40 + row * 170)
}

const BACKING: [f32; 4] = [0.03, 0.035, 0.045, 1.0];

/// sRGB bytes as `px` reads them back (through linear light).
fn ink(rgb: [u8; 3]) -> [f32; 3] {
    let l = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    srgb8([l(rgb[0]), l(rgb[1]), l(rgb[2]), 1.0])
}

/// Corner shapes cut where their geometry says: 12 px in from a 28 px
/// corner a round corner is inside, a bevel's and a notch's cut leave the
/// card; 4 px in, the squircle (radius 48, continuous) is still outside
/// where the round one of 28 would be too. A ring has its hole and an arc
/// stops at its sweep. The whole UI is a handful of draws.
#[test]
#[ignore = "requires GPU"]
fn ui_shapes_cut_their_corners_rings_and_arcs() {
    let r = render("ui-shapes");
    let back = srgb8(BACKING);
    // The shapes' top left corners are 20 px into their cells.
    let corner = |col: u32, d: u32| {
        let (x, y) = cell(col, 0);
        px(&r, x + 20 + d, y + 20 + d)
    };
    assert!(
        near(corner(0, 12), ink([255, 183, 77]), 2.0),
        "round corner: {:?}",
        corner(0, 12)
    );
    assert!(
        near(corner(2, 12), back, 2.0),
        "bevel corner: {:?}",
        corner(2, 12)
    );
    assert!(
        near(corner(3, 12), back, 2.0),
        "notch corner: {:?}",
        corner(3, 12)
    );
    assert!(
        near(corner(1, 4), back, 2.0),
        "squircle corner: {:?}",
        corner(1, 4)
    );
    // Four radii: top left 0 (square), bottom left 60 (cut deep).
    let (x, y) = cell(4, 0);
    assert!(near(px(&r, x + 21, y + 21), ink([239, 83, 80]), 2.0));
    assert!(near(px(&r, x + 24, y + 126), back, 2.0));
    // The ring (cell 2, row 1; 130 px square from (25, 10)) has a hole.
    let (x, y) = cell(2, 1);
    let (cx, cy) = (x + 25 + 65, y + 10 + 65);
    assert!(near(px(&r, cx, cy), back, 2.0), "the ring's hole is drawn");
    assert!(
        near(px(&r, cx + 57, cy), ink([186, 104, 200]), 2.0),
        "the ring"
    );
    // The 120° arc from 12 o'clock: right of centre at 3 o'clock is drawn,
    // left of centre at 9 o'clock is not.
    let (x, y) = cell(4, 1);
    let (cx, cy) = (x + 90, y + 75);
    assert!(
        near(px(&r, cx + 52, cy), ink([239, 83, 80]), 2.0),
        "the arc"
    );
    assert!(near(px(&r, cx - 52, cy), back, 2.0), "past the arc's sweep");
    // The dotted ring (cell 2, row 3; r 65 about (530, 625), dots every
    // 16.1 px from 12 o'clock): its outline is no whole number of periods,
    // so the first dot (x 526..534) overlaps the last (x 520..528). The
    // first dot is round on both sides of the outline's start: x 529 is in
    // it, not cut off at the seam.
    assert!(
        near(px(&r, 529, 560), ink([186, 104, 200]), 2.0),
        "the dot at the dotted ring's start is cut at the seam: {:?}",
        px(&r, 529, 560)
    );
    println!(
        "sprite draws {} sprites {}",
        r.stats.sprite_draws, r.stats.sprites
    );
    assert!(
        r.stats.sprite_draws <= 2,
        "{} UI draws",
        r.stats.sprite_draws
    );
}

/// Run-time paths (`ui_scenes::paths`): the nonzero ring keeps its
/// reversed inner circle as a hole; one self-crossing star is solid by
/// nonzero and hollow by even-odd, its spikes drawn by both; the heart's
/// cubics leave the notch between its lobes; a stroke draws the outline,
/// not the inside; a drop shadow darkens past the heart's edge; the morph
/// half way is neither the square nor the circle; a turned, squashed
/// heart keeps its point; a 14 px heart draws.
/// A layer draped over uneven terrain as a decal (`ui-ground`, a
/// document's `<ground>`): at screen points where the checker cell the
/// terrain's surface falls in differs from the cell the flat plane would
/// show there, the frame shows the surface's cell, inverted in the first
/// two columns by a backdrop inside the ground's layer; the pillar's face
/// shows no checker; the sky past the rect shows none.
///
/// Ways it could fail: the layer laid on the flat plane (the old quad
/// map), the depth read at the wrong pixel or as the wrong depth (cells
/// shifted), the plane's axes swapped (columns for rows), a surface that
/// stands across the plane painted, the layer drawn where the scene
/// has nothing, or the backdrop planned on the screen's pixels (its rect is
/// far off the screen, so it draws nothing).
#[test]
#[ignore = "requires GPU"]
fn ui_ground_drapes_a_layer_over_the_terrain() {
    use og_core::glam::{Vec3, Vec4};
    use og_tests::ui_scenes::{ground_height, ground_pillar, ground_view};
    let r = render("ui-ground");
    let inv = ground_view().inverse();
    let ray = |x: f32, y: f32| {
        let n = |z: f32| {
            let p = inv * Vec4::new(x / 640.0 - 1.0, 1.0 - y / 360.0, z, 1.0);
            p.truncate() / p.w
        };
        (n(0.0), (n(1.0) - n(0.0)).normalize())
    };
    let (lo, hi) = ground_pillar();
    let hits_pillar = |o: Vec3, d: Vec3| {
        let (t0, t1) = ((lo - o) / d, (hi - o) / d);
        let near = t0.min(t1).max_element();
        let far = t0.max(t1).min_element();
        near <= far && far > 0.0
    };
    // The checker's cell (gold or not) at a world point, and how far it
    // lies from a cell's edge or the ring, in cells.
    let cell = |p: Vec3| {
        let (u, v) = ((p.x + 3.0) / 6.0 * 8.0, (p.z + 3.0) / 6.0 * 8.0);
        let ring = (((u - 4.0).powi(2) + (v - 4.0).powi(2)).sqrt() - 2.0).abs() - 0.2;
        let edge = (u - u.round()).abs().min((v - v.round()).abs()).min(ring);
        let inside = (0.0..8.0).contains(&u) && (0.0..8.0).contains(&v);
        (((u.floor() + v.floor()) as i32 % 2 == 0), edge, inside)
    };
    let gold = ink([240, 180, 60]);
    let blue = ink([40, 90, 200]);
    let mut differ = 0;
    let mut inverted = 0;
    for y in (150..700).step_by(9) {
        for x in (150..1130).step_by(9) {
            let (o, d) = ray(x as f32 + 0.5, y as f32 + 0.5);
            if hits_pillar(o, d) {
                continue;
            }
            // The terrain's surface: march, then halve.
            let (mut a, mut b) = (0.0f32, 0.0f32);
            let mut t = 0.0;
            while t < 40.0 {
                let p = o + d * t;
                if p.y < ground_height(p.x, p.z) {
                    b = t;
                    break;
                }
                a = t;
                t += 0.02;
            }
            if b == 0.0 {
                continue;
            }
            for _ in 0..24 {
                let m = 0.5 * (a + b);
                let p = o + d * m;
                if p.y < ground_height(p.x, p.z) {
                    b = m
                } else {
                    a = m
                }
            }
            let surface = o + d * b;
            let flat = o + d * (-o.y / d.y);
            let (s_gold, s_edge, s_in) = cell(surface);
            let (f_gold, f_edge, f_in) = cell(flat);
            // Points well inside cells, facing up, where the two differ.
            let slope = Vec3::new(
                ground_height(surface.x - 0.01, surface.z)
                    - ground_height(surface.x + 0.01, surface.z),
                0.02,
                0.0,
            )
            .normalize()
            .y;
            if !(s_in && f_in && s_edge > 0.25 && f_edge > 0.25 && slope > 0.7 && s_gold != f_gold)
            {
                continue;
            }
            let p = px(&r, x, y);
            // The first two columns are under a backdrop that inverts them.
            let u = (surface.x + 3.0) / 6.0 * 8.0;
            let fu = (flat.x + 3.0) / 6.0 * 8.0;
            if (u - 2.0).abs() < 0.25 || (u < 2.0) != (fu < 2.0) {
                continue;
            }
            let (gold, blue) = if u < 2.0 {
                inverted += 1;
                (ink([15, 75, 195]), ink([215, 165, 55]))
            } else {
                (gold, blue)
            };
            let (want, not) = if s_gold { (gold, blue) } else { (blue, gold) };
            assert!(
                near(p, want, 30.0) && !near(p, not, 60.0),
                "({x}, {y}): surface {surface:?} wants {want:?}, the flat plane {not:?}; got {p:?}"
            );
            differ += 1;
        }
    }
    assert!(
        differ >= 20,
        "only {differ} points tell the drape from the plane"
    );
    assert!(
        inverted >= 5,
        "only {inverted} points under the ground's backdrop"
    );
    // The pillar's face, within the reach, takes no checker; the sky past
    // the rect shows its own colour.
    let face = (lo + hi) / 2.0 + Vec3::new(0.0, -0.6, 0.26);
    let c = ground_view().project_point3(face);
    let (fx, fy) = (((c.x + 1.0) * 640.0) as u32, ((1.0 - c.y) * 360.0) as u32);
    let p = px(&r, fx, fy);
    assert!(
        !near(p, gold, 40.0) && !near(p, blue, 40.0),
        "pillar face ({fx}, {fy}): {p:?}"
    );
    let sky = px(&r, 640, 8);
    assert!(near(sky, px(&r, 20, 8), 3.0), "sky: {sky:?}");
}

#[test]
#[ignore = "requires GPU"]
fn ui_paths_fill_by_their_rule_and_take_every_mode() {
    let r = render("ui-paths");
    let back = srgb8(BACKING);
    let centre = |col: u32, row: u32| {
        let (x, y) = cell(col, row);
        (x + 90, y + 75)
    };
    let at = |col: u32, row: u32, dx: i32, dy: i32| {
        let (x, y) = centre(col, row);
        px(&r, (x as i32 + dx) as u32, (y as i32 + dy) as u32)
    };
    let check = |what: &str, got: [f32; 3], want: [f32; 3]| {
        assert!(near(got, want, 3.0), "{what}: {got:?}, not {want:?}");
    };
    let red = ink([239, 83, 80]);
    let cyan = ink([77, 208, 225]);
    let amber = ink([255, 183, 77]);
    let violet = ink([186, 104, 200]);
    // The heart (140 px): inside, in a lobe, and the notch between them.
    check("the heart", at(0, 0, 0, 0), red);
    check("the heart's left lobe", at(0, 0, -32, -55), red);
    check("the notch between the lobes", at(0, 0, 0, -50), back);
    // The ring: its hole by nonzero (the inner circle runs the other way).
    check("the ring's hole", at(1, 0, 0, 0), back);
    check("the ring", at(1, 0, 46, 0), cyan);
    // The star: nonzero fills the pentagon, even-odd leaves it; both draw
    // the spikes (50 px up, inside the top spike).
    check("the nonzero star's pentagon", at(2, 0, 0, 0), amber);
    check("the even-odd star's pentagon", at(3, 0, 0, 0), back);
    check("the nonzero star's spike", at(2, 0, 0, -50), amber);
    check("the even-odd star's spike", at(3, 0, 0, -50), violet);
    // On the lines that cross the nonzero star's inside (its pentagon's
    // top edge, 64·cos 72° above the centre, and a lower one): no seam.
    check("the nonzero star's inner line", px(&r, 530, 95), amber);
    check(
        "the nonzero star's inner line, left",
        at(2, 0, -12, -16),
        amber,
    );
    // The stroked heart (120 px): its outline at the left lobe, not its inside.
    check("the stroke's inside", at(0, 1, 0, 0), back);
    check("the stroke", at(0, 1, -60, -21), red);
    // The drop shadow (6, 8) past the heart's right edge.
    let shadow = at(1, 1, 58, 4);
    assert!(
        shadow.iter().zip(back).all(|(s, b)| *s < b - 4.0),
        "the path's drop shadow: {shadow:?} over {back:?}"
    );
    check("the shadowed heart", at(1, 1, 0, 0), cyan);
    // The morph half way: at 45° the square's edge is at 52.5 px on each
    // axis, the circle's at 42.4, the morph's at 47.5.
    check(
        "inside the morph, outside the circle",
        at(3, 1, 45, 45),
        violet,
    );
    check(
        "outside the morph, inside the square",
        at(3, 1, 50, 50),
        back,
    );
    // The heart squashed to 70 % wide and turned 25°: its point, local
    // (0, 51), lies left of and below the centre; just inside it is drawn,
    // just past it is not.
    let (sn, cs) = 25f32.to_radians().sin_cos();
    let turned = |y: f32| at(4, 1, (-sn * y).round() as i32, (cs * y).round() as i32);
    let green = ink([129, 199, 132]);
    check("inside the turned heart's point", turned(44.0), green);
    check("at the turned heart's point", turned(49.0), green);
    check("past the turned heart's point", turned(56.0), back);
    // The 14 px heart at (1060, 60).
    check("the 14 px heart", px(&r, 1067, 67), ink([255, 241, 118]));
}

/// World-space UI (`docs/ui.md` §1, `ui_scenes::world`): occluders hide a
/// panel behind the wall and keep the one against the sky; the bars draw
/// their back, ghost and fill where their values put them; the mass
/// elements' numbers and names draw at all.
#[test]
#[ignore = "requires GPU"]
fn ui_world_occludes_and_draws_mass_elements() {
    let r = render("ui-world");
    let pink = srgb8([0.9, 0.3, 0.6, 1.0]);
    // The left panel: only sky behind it at 20 m.
    assert!(near(px(&r, 320, 360), pink, 6.0), "{:?}", px(&r, 320, 360));
    // The right one: the wall (12 m) is in front of 20 m.
    assert!(
        !near(px(&r, 640, 360), pink, 40.0),
        "{:?}",
        px(&r, 640, 360)
    );
    // Bars from x = 80, 120 px: the first's fill to 0.5 (green), its
    // ghost to 0.75 (pale), its back past it (dark).
    let fill = px(&r, 110, 86);
    let ghost = px(&r, 160, 86);
    let back = px(&r, 190, 86);
    assert!(fill[1] > fill[0] + 60.0, "fill {fill:?}");
    assert!(ghost[0] > 200.0 && ghost[1] > 180.0, "ghost {ghost:?}");
    // 85 % opaque over the sky (136, 155, 197).
    assert!(back.iter().all(|&c| c < 110.0), "back {back:?}");
    // The third bar's fill reaches 0.9 (x 188) and has no ghost.
    assert!(px(&r, 185, 134)[1] > 120.0, "{:?}", px(&r, 185, 134));
    // The score's glyphs: gold ink in its band at the top centre.
    let gold = (40..90)
        .flat_map(|y| (560..720).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&r, x, y);
            // (1, 0.85, 0.3) linear: about (255, 237, 149) in sRGB.
            p[0] > 220.0 && p[1] > 200.0 && p[2] < 180.0
        })
        .count();
    assert!(gold > 200, "{gold} gold pixels in the score");
    // Damage numbers: white, gold and red ink over the lower half.
    let red = (360..720)
        .flat_map(|y| (0..1280).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(&r, x, y);
            p[0] > 200.0 && p[1] < 120.0 && p[2] < 120.0
        })
        .count();
    assert!(red > 500, "{red} red pixels of damage numbers");
    // Confetti: cyan particles flung around (300, 200), none at its
    // centre any more and none past the farthest travel (about 180 px).
    let cyan = |x0: u32, x1: u32, y0: u32, y1: u32| {
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let p = px(&r, x, y);
                p[0] < 150.0 && p[1] > 220.0 && p[2] > 230.0
            })
            .count()
    };
    let ring = cyan(150, 450, 60, 360);
    assert!(ring > 1500, "{ring} cyan confetti pixels");
    assert!(
        cyan(290, 310, 190, 210) < 20,
        "confetti still at the centre"
    );
}

#[test]
#[ignore = "requires GPU"]
fn mass_timing_luts_draw_at_the_shared_curves_positions() {
    use og_core::glam::{Vec2, Vec3};
    use og_render::{
        Overlay, RenderDesc, RenderScene, RenderTarget, Renderer, UiMassBatch, UiMassInstance,
        UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate, UiParticleShape,
    };
    use og_tween::{Ease, StepPosition};
    use std::sync::Arc;
    let curves = [
        Ease::Linear,
        Ease::QuadOut,
        Ease::CubicBezier([0.17, 0.85, 0.63, 0.08]),
        Ease::Steps {
            count: 4,
            position: StepPosition::JumpStart,
        },
    ];
    let desc = RenderDesc {
        width: 480,
        height: 280,
        ..Default::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).unwrap();
    for tick in [1, 3] {
        let mut scene = RenderScene::new();
        scene.environment.background = og_render::Background::Color(Vec3::splat(0.015));
        let mut overlay = Overlay::new(480.0, 280.0, 0.0);
        overlay.ui.tick = tick;
        let mut centers = Vec::new();
        for (row, ease) in curves.iter().copied().enumerate() {
            let y = 50.0 + row as f32 * 60.0;
            let x = 40.0 + 360.0 * ease.apply(tick as f32 / 8.0);
            centers.push((x.round() as u32, y as u32));
            for (width, height, at, color, layer) in [
                (360.0, 4.0, 220.0, [0.1, 0.12, 0.15, 1.0], 0),
                (2.0, 40.0, x, [1.0, 0.6, 0.1, 1.0], 1),
            ] {
                overlay.ui.mass.push(UiMassBatch {
                    template: Arc::new(UiMassTemplate {
                        kind: UiMassKind::Particle {
                            shape: UiParticleShape::Square,
                        },
                        size: Vec2::new(width, height),
                        ..Default::default()
                    }),
                    space: UiMassSpace::Screen,
                    instances: vec![UiMassInstance {
                        position: [at, y, 0.0],
                        color,
                        scale: 1.0,
                        ..Default::default()
                    }],
                    layer,
                    clip: None,
                });
            }
            overlay.ui.mass.push(UiMassBatch {
                template: Arc::new(UiMassTemplate {
                    kind: UiMassKind::Particle {
                        shape: UiParticleShape::Square,
                    },
                    size: Vec2::new(16.0, 24.0),
                    lifetime: 8,
                    keys: vec![
                        UiMassKey::default(),
                        UiMassKey {
                            t: 1.0,
                            offset: Vec2::new(360.0, 0.0),
                            ease,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                space: UiMassSpace::Screen,
                instances: vec![UiMassInstance {
                    position: [40.0, y, 0.0],
                    color: [0.05, 0.85, 1.0, 1.0],
                    scale: 1.0,
                    ..Default::default()
                }],
                layer: 2,
                clip: None,
            });
        }
        scene.ui.overlay = Some(overlay);
        renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
        let image = renderer.read_offscreen().unwrap();
        for (x, y) in centers {
            let at = ((y * image.width + x) * 4) as usize;
            assert!(
                image.pixels[at + 1] > 200 && image.pixels[at + 2] > 240,
                "GPU square misses CPU marker at {x},{y}"
            );
            let away = ((y * image.width + x + 16) * 4) as usize;
            assert!(
                image.pixels[away + 2] < 180,
                "square has wrong size or offset"
            );
        }
        if let Some(dir) = std::env::var_os("OG_TIMING_FRAMES") {
            gfx::save_png(
                &std::path::PathBuf::from(dir).join(format!("mass-timing-{tick}.png")),
                &image,
            )
            .unwrap();
        }
    }
}
