//! The paint pass (og-ui's new core) drawing images, read back as pixels
//! (no reference frame).
//!
//! What would break:
//! - a nine-slice sampling its corners' texels across the box instead of
//!   keeping them at their slice's on-screen width, or its centre drawn
//!   without the fill flag
//! - a radial wipe showing the wrong quarter, or sweeping the wrong way
//! - a polyline's join drawn by both its segments (twice the alpha), or a
//!   closed one missing its closing side
//! - a conic gradient sweeping counter-clockwise or from the wrong angle, or
//!   a radial one drawn as a circle of its wider radius
use glam::Vec3;
use og_render::paint::{
    GRADIENT_CONIC, GRADIENT_RADIAL, IMAGE_FILL, PaintGradient, PaintKind, PaintQuad,
};
use og_render::{
    Background, ColorSpace, Mips, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer,
    TextureData, TextureHandle, Tonemap,
};

const W: u32 = 128;

/// One frame of the quads `quads` makes, at one device px per CSS px.
fn render(quads: impl FnOnce(&mut RenderScene) -> Vec<PaintQuad>) -> Vec<u8> {
    let mut scene = RenderScene::new();
    scene.ui.overlay = Some(Overlay::new(W as f32, W as f32, 1.0));
    scene.ui.paint.clear();
    let made = quads(&mut scene);
    let p = &mut scene.ui.paint;
    (p.width, p.height) = (W as f32, W as f32);
    p.quads = made;
    p.generation = 1;
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    })
    .expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture").pixels
}

fn at(px: &[u8], x: u32, y: u32) -> [u8; 3] {
    px[((y * W + x) * 4) as usize..][..3].try_into().unwrap()
}

fn texture(
    scene: &mut RenderScene,
    w: u32,
    h: u32,
    rgb: impl Fn(u32, u32) -> [u8; 3],
) -> TextureHandle {
    let rgba = (0..w * h)
        .flat_map(|i| {
            let [r, g, b] = rgb(i % w, i / w);
            [r, g, b, 255]
        })
        .collect();
    scene.add_texture(TextureData {
        width: w,
        height: h,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::Generate,
    })
}

fn image(
    t: TextureHandle,
    rect: [f32; 4],
    screen: [f32; 4],
    slice: [f32; 4],
    flags: u32,
    wipe: [f32; 4],
) -> PaintQuad {
    PaintQuad {
        rect,
        clip: [0.0, 0.0, W as f32, W as f32],
        color: [1.0; 4],
        radius: screen,
        p: slice,
        kind: PaintKind::Image as u32,
        a: t.to_bits(),
        b: flags,
        xf: 0,
        uv: [0.0, 0.0, 1.0, 1.0],
        wipe,
    }
}

const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

fn near(got: [u8; 3], want: [u8; 3], what: &str) {
    let off = got
        .iter()
        .zip(want)
        .map(|(&g, w)| g.abs_diff(w))
        .max()
        .unwrap();
    assert!(off <= 24, "{what}: {got:?}, expected {want:?}");
}

/// A 9×9 frame: a 3-texel red border round a green centre.
fn frame(x: u32, y: u32) -> [u8; 3] {
    if (3..6).contains(&x) && (3..6).contains(&y) {
        GREEN
    } else {
        RED
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_nine_slice_keeps_its_slices_and_draws_its_centre_only_when_filled() {
    for fill in [true, false] {
        let px = render(|scene| {
            let t = texture(scene, 9, 9, frame);
            let flags = if fill { IMAGE_FILL } else { 0 };
            vec![image(
                t,
                [16.0, 16.0, 96.0, 48.0],
                [12.0; 4],
                [3.0; 4],
                flags,
                [0.0; 4],
            )]
        });
        // Each 3-texel slice is 12 px wide on screen: red all along it.
        near(at(&px, 20, 20), RED, "top left corner");
        near(at(&px, 21, 40), RED, "left edge");
        near(at(&px, 64, 21), RED, "top edge");
        near(at(&px, 107, 59), RED, "bottom right corner");
        near(at(&px, 64, 40), if fill { GREEN } else { BLACK }, "centre");
        near(
            at(&px, 34, 40),
            if fill { GREEN } else { BLACK },
            "centre, near its left",
        );
        near(at(&px, 8, 8), BLACK, "outside");
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_radial_wipe_shows_its_share_from_the_top_in_its_direction() {
    for ccw in [false, true] {
        let px = render(|scene| {
            let t = texture(scene, 4, 4, |_, _| WHITE);
            let wipe = [0.75, 0.0, f32::from(u8::from(ccw)), 0.0];
            vec![image(
                t,
                [24.0, 24.0, 80.0, 80.0],
                [0.0; 4],
                [0.0; 4],
                IMAGE_FILL,
                wipe,
            )]
        });
        // A quarter from 12 o'clock: the top right clockwise, the top left
        // counter-clockwise.
        let (shown, hidden) = if ccw {
            ((44, 44), (84, 44))
        } else {
            ((84, 44), (44, 44))
        };
        near(at(&px, shown.0, shown.1), WHITE, "the shown quarter");
        near(at(&px, hidden.0, hidden.1), BLACK, "the next quarter");
        near(at(&px, 84, 84), BLACK, "the bottom right");
        near(at(&px, 44, 84), BLACK, "the bottom left");
    }
}

fn line(points: &mut Vec<[f32; 2]>, pts: &[[f32; 2]], width: f32, closed: bool) -> PaintQuad {
    let first = points.len() as u32;
    points.extend_from_slice(pts);
    PaintQuad {
        rect: [10.0, 10.0, 100.0, 100.0],
        clip: [0.0, 0.0, W as f32, W as f32],
        color: [1.0, 1.0, 1.0, 0.5],
        radius: [0.0; 4],
        p: [width, f32::from(u8::from(closed)), 0.0, 0.0],
        kind: PaintKind::Line as u32,
        a: first,
        b: pts.len() as u32,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_polyline_draws_its_joins_once_and_closes_when_closed() {
    let mut points = Vec::new();
    let quads = vec![
        // A V, its join at (60, 90) in the box (70, 100 on screen).
        line(
            &mut points,
            &[[10.0, 10.0], [60.0, 90.0], [100.0, 10.0]],
            8.0,
            false,
        ),
        // A closed square: its fourth side, (10, 100) back to (10, 60).
        line(
            &mut points,
            &[[0.0, 50.0], [40.0, 50.0], [40.0, 90.0], [0.0, 90.0]],
            4.0,
            true,
        ),
    ];
    let px = render(|scene| {
        scene.ui.paint.points = points;
        quads
    });
    let mid = at(&px, 45, 66);
    assert!(mid[0] > 60, "the stroke draws: {mid:?}");
    let join = at(&px, 70, 99);
    let off = mid
        .iter()
        .zip(join)
        .map(|(&m, j)| m.abs_diff(j))
        .max()
        .unwrap();
    assert!(
        off <= 6,
        "the join as bright as the stroke, not twice: {join:?} vs {mid:?}"
    );
    near(at(&px, 70, 108), BLACK, "outside the round join");
    let closing = at(&px, 10, 80);
    assert!(closing[0] > 60, "the closing side: {closing:?}");
    near(at(&px, 30, 80), BLACK, "inside the square");
}

/// A box filled by gradient `g`, black to white.
fn gradient_box(scene: &mut RenderScene, rect: [f32; 4], g: PaintGradient) -> PaintQuad {
    let gi = scene
        .ui
        .paint
        .shaped_gradient(g, [(0.0, [0.0, 0.0, 0.0, 1.0]), (1.0, [1.0; 4])]);
    PaintQuad {
        rect,
        clip: [0.0, 0.0, W as f32, W as f32],
        color: [1.0; 4],
        radius: [0.0; 4],
        p: [0.0; 4],
        kind: PaintKind::Gradient as u32,
        a: gi,
        b: 0,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
    }
}

fn grey(v: u8) -> [u8; 3] {
    [v; 3]
}

#[test]
#[ignore = "requires GPU"]
fn conic_and_radial_gradients_draw_their_shapes() {
    // From 3 o'clock, clockwise: a quarter turn at 6, a half at 9, three
    // quarters at 12 (mixed in sRGB, as CSS mixes).
    let px = render(|scene| {
        let g = PaintGradient {
            angle: std::f32::consts::FRAC_PI_2,
            shape: GRADIENT_CONIC,
            center: [0.0, 0.0],
            radii: [0.0; 2],
            first: 0,
            count: 0,
        };
        vec![gradient_box(scene, [0.0, 0.0, 128.0, 128.0], g)]
    });
    near(at(&px, 64, 104), grey(64), "a quarter turn, below");
    near(at(&px, 24, 64), grey(128), "half a turn, left");
    near(at(&px, 64, 24), grey(191), "three quarters, above");
    // An ellipse twice as wide as tall about the box's centre: half way
    // out is 45 px across and 22.5 px down.
    let px = render(|scene| {
        let r = 45.25 * 2.0;
        let g = PaintGradient {
            angle: 0.0,
            shape: GRADIENT_RADIAL,
            center: [0.0, 0.0],
            radii: [r, r / 2.0],
            first: 0,
            count: 0,
        };
        vec![gradient_box(scene, [0.0, 32.0, 128.0, 64.0], g)]
    });
    near(at(&px, 64, 64), BLACK, "the centre");
    near(at(&px, 109, 64), grey(128), "half way out, across");
    near(at(&px, 64, 86), grey(128), "half way out, down");
}
