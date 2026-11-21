//! The paint pass (og-ui's new core) drawing images, read back as pixels
//! (no reference frame).
//!
//! What would break:
//! - a nine-slice sampling its corners' texels across the box instead of
//!   keeping them at their slice's on-screen width, or its centre drawn
//!   without the fill flag
//! - a radial wipe showing the wrong quarter, or sweeping the wrong way
use glam::Vec3;
use og_render::paint::{IMAGE_FILL, PaintKind, PaintQuad};
use og_render::{
    Background, ColorSpace, Mips, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer,
    TextureData, TextureHandle, Tonemap,
};

const W: u32 = 128;

/// One frame of the quads `quads` makes, at one device px per CSS px.
fn render(quads: impl FnOnce(&mut RenderScene) -> Vec<PaintQuad>) -> Vec<u8> {
    let mut scene = RenderScene::new();
    scene.ui.overlay = Some(Overlay::new(W as f32, W as f32, 1.0));
    let made = quads(&mut scene);
    let p = &mut scene.ui.paint;
    p.clear();
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
