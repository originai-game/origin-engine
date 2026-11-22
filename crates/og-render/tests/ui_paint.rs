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
//! - a mask measured over the quad instead of its own box, or ignored
//! - a translucent fill blended in linear light instead of in sRGB-encoded
//!   values as CSS blends, on any of the pass's three targets (the output,
//!   its plain view, its own layer), or a layer meeting a float output (a
//!   wide-gamut display's) in linear light
use glam::Vec3;
use og_render::paint::{
    GRADIENT_CONIC, GRADIENT_RADIAL, IMAGE_FILL, PaintGradient, PaintKind, PaintQuad,
};
use og_render::{
    Background, ColorSpace, Mips, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer,
    TextureData, TextureHandle, Tonemap,
};

const W: u32 = 128;

/// Where a frame is drawn: each is another target of the paint pass.
#[derive(Clone, Copy, Debug)]
enum To {
    /// The offscreen sRGB output: its plain view.
    Offscreen,
    /// A plain offscreen output: the output itself.
    Plain,
    /// A caller's sRGB view, which has no plain view: the pass's layer.
    View,
    /// A float offscreen output (a wide-gamut or HDR display's): the
    /// pass's layer, over a copy of what is behind it.
    Float,
}

/// One frame of the quads `quads` makes, at one device px per CSS px.
fn render(quads: impl FnOnce(&mut RenderScene) -> Vec<PaintQuad>) -> Vec<u8> {
    render_to(To::Offscreen, quads)
}

fn render_to(to: To, quads: impl FnOnce(&mut RenderScene) -> Vec<PaintQuad>) -> Vec<u8> {
    use og_render::wgpu;
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
        output_format: match to {
            To::Plain => Some(wgpu::TextureFormat::Rgba8Unorm),
            To::Float => Some(wgpu::TextureFormat::Rgba16Float),
            _ => None,
        },
        ..Default::default()
    })
    .expect("renderer");
    if !matches!(to, To::View) {
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        return renderer.read_offscreen().expect("capture").pixels;
    }
    let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: W,
            height: W,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.output_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
        texture_binding_view_dimension: None,
    });
    let view = texture.create_view(&Default::default());
    renderer
        .render(&scene, &RenderTarget::View(&view))
        .expect("render");
    let (device, queue) = (renderer.device(), renderer.queue());
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (W * W * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                // W is a multiple of the 256-byte row alignment's 64 px.
                bytes_per_row: Some(W * 4),
                rows_per_image: Some(W),
            },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    buffer
        .slice(..)
        .get_mapped_range()
        .expect("mapped")
        .to_vec()
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
        mask: 0,
        filter: 0,
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
        mask: 0,
        filter: 0,
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
        mask: 0,
        filter: 0,
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
            area: [0.0; 4],
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
            area: [0.0; 4],
        };
        vec![gradient_box(scene, [0.0, 32.0, 128.0, 64.0], g)]
    });
    near(at(&px, 64, 64), BLACK, "the centre");
    near(at(&px, 109, 64), grey(128), "half way out, across");
    near(at(&px, 64, 86), grey(128), "half way out, down");
}

#[test]
#[ignore = "requires GPU"]
fn a_mask_fades_what_it_covers_over_its_own_box() {
    // A mask over the left half (x 0..64), opaque to transparent left to
    // right, on two white boxes: one inside it, one past its right edge.
    let px = render(|scene| {
        let mask = PaintGradient {
            angle: std::f32::consts::FRAC_PI_2,
            area: [0.0, 0.0, 64.0, 128.0],
            ..bytemuck::Zeroable::zeroed()
        };
        let mi = scene
            .ui
            .paint
            .shaped_gradient(mask, [(0.0, [0.0, 0.0, 0.0, 1.0]), (1.0, [0.0; 4])]);
        let white = |rect| PaintQuad {
            rect,
            clip: [0.0, 0.0, W as f32, W as f32],
            color: [1.0; 4],
            radius: [0.0; 4],
            p: [0.0; 4],
            kind: PaintKind::Solid as u32,
            a: 0,
            b: 0,
            xf: 0,
            uv: [0.0; 4],
            wipe: [0.0; 4],
            mask: mi + 1,
            filter: 0,
        };
        vec![
            white([16.0, 16.0, 32.0, 96.0]),
            white([80.0, 16.0, 32.0, 96.0]),
        ]
    });
    // At 1/4 and 3/4 of the mask's width: three quarters and a quarter of
    // white over black, blended as encoded.
    near(at(&px, 16, 64), grey(191), "near the mask's opaque edge");
    near(at(&px, 47, 64), grey(64), "a quarter left");
    near(at(&px, 96, 64), BLACK, "past the mask's transparent end");
}

#[test]
#[ignore = "requires GPU"]
fn a_colour_matrix_filters_the_encoded_colour() {
    // A gradient first, so the matrices' rows sit after its three.
    // brightness(0.5) on (200, 40, 40) halves its code values, as CSS's
    // filter does (halving it in linear light gives 146 red); invert gives
    // (55, 215, 215); opacity(0.5) of white over black is encoded grey.
    let px = render(|scene| {
        let paint = &mut scene.ui.paint;
        paint.gradient(0.0, [(0.0, [1.0; 4]), (1.0, [1.0; 4])]);
        let scale = |s: f32, alpha: f32, offset: f32| {
            [
                [s, 0.0, 0.0, 0.0],
                [0.0, s, 0.0, 0.0],
                [0.0, 0.0, s, 0.0],
                [0.0, 0.0, 0.0, alpha],
                [offset, offset, offset, 0.0],
            ]
        };
        paint.filters.extend([
            scale(0.5, 1.0, 0.0),
            scale(-1.0, 1.0, 1.0),
            scale(1.0, 0.5, 0.0),
        ]);
        let with = |q: PaintQuad, filter| PaintQuad { filter, ..q };
        vec![
            with(solid([0.0, 0.0, 32.0, 128.0], [200, 40, 40], 1.0), 1),
            with(solid([32.0, 0.0, 32.0, 128.0], [200, 40, 40], 1.0), 2),
            with(solid([64.0, 0.0, 32.0, 128.0], WHITE, 1.0), 3),
            solid([96.0, 0.0, 32.0, 128.0], [200, 40, 40], 1.0),
        ]
    });
    let exact = |x, want: [u8; 3], what: &str| {
        let got = at(&px, x, 64);
        let off = got.iter().zip(want).map(|(&g, w)| g.abs_diff(w)).max();
        assert!(off <= Some(2), "{what}: {got:?}, expected {want:?}");
    };
    exact(16, [100, 20, 20], "brightness(0.5)");
    exact(48, [55, 215, 215], "invert");
    exact(80, [128, 128, 128], "opacity(0.5) over black");
    exact(112, [200, 40, 40], "no filter");
}

/// sRGB's decoding of one channel.
fn linear(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn solid(rect: [f32; 4], [r, g, b]: [u8; 3], alpha: f32) -> PaintQuad {
    PaintQuad {
        rect,
        clip: [0.0, 0.0, W as f32, W as f32],
        color: [linear(r), linear(g), linear(b), alpha],
        radius: [0.0; 4],
        p: [0.0; 4],
        kind: PaintKind::Solid as u32,
        a: 0,
        b: 0,
        xf: 0,
        uv: [0.0; 4],
        wipe: [0.0; 4],
        mask: 0,
        filter: 0,
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_translucent_fill_blends_in_encoded_srgb_as_css_does() {
    // A quarter of white over a dark red ground, and the ground alone.
    // CSS: 0.75 × (200, 40, 40) + 0.25 × 255 = (214, 94, 94); a blend in
    // linear light puts the green at 141.
    for to in [To::Offscreen, To::Plain, To::View, To::Float] {
        let px = render_to(to, |_| {
            vec![
                solid([0.0, 0.0, 128.0, 64.0], [200, 40, 40], 1.0),
                solid([32.0, 0.0, 64.0, 128.0], WHITE, 0.25),
            ]
        });
        let exact = |x, y, want: [u8; 3], what: &str| {
            let got = at(&px, x, y);
            let off = got.iter().zip(want).map(|(&g, w)| g.abs_diff(w)).max();
            assert!(off <= Some(2), "{to:?}, {what}: {got:?}, expected {want:?}");
        };
        exact(8, 32, [200, 40, 40], "the ground");
        exact(64, 32, [214, 94, 94], "white over the ground");
        // Over the frame itself (black): a quarter of white. A layer over a
        // caller's view meets it in the view's light instead (ui/paint.rs).
        if !matches!(to, To::View) {
            exact(64, 96, grey(64), "white over the frame");
        }
    }
}
