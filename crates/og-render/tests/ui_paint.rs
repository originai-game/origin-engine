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
//! - a glyph's raster read from the wrong place in the glyph atlas, or its
//!   coverage drawn as anything but Skia's preblend of it (each test below
//!   says more)
use glam::Vec3;
use og_render::glyphs::{GlyphKey, GlyphSlot, GlyphTier, UNUSED_FRAMES};
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
        round: 0,
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
        round: 0,
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
        round: 0,
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
            round: 0,
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
fn a_round_clip_cuts_an_image_round_and_an_inverted_one_spares_its_hole() {
    // Left: a white image across a clip's 20 px top left corner (its box
    // 8..56, the arc's centre at 28, 28). Right: a red shade over x 64..128
    // cut by an inverted round of the box 80..120 × 8..48, radius 20 (a
    // spotlight's hole round a round button). A gradient and a colour
    // matrix come first, so the rounds' rows sit after theirs.
    let px = render(|scene| {
        let ground = gradient_box(scene, [0.0, 100.0, 4.0, 4.0], bytemuck::Zeroable::zeroed());
        let p = &mut scene.ui.paint;
        p.filters.push(og_render::paint::COLOR_IDENTITY);
        let round = |rect, invert| og_render::paint::PaintRound {
            rect,
            radius: [20.0; 4],
            xf: 0,
            invert,
            _pad: [0; 2],
        };
        p.rounds.push(round([8.0, 8.0, 56.0, 56.0], 0));
        p.rounds.push(round([80.0, 8.0, 120.0, 48.0], 1));
        let white = texture(scene, 4, 4, |_, _| WHITE);
        let mut shade = solid([64.0, 0.0, 64.0, 64.0], RED, 1.0);
        shade.round = 2;
        vec![
            ground,
            PaintQuad {
                round: 1,
                ..image(
                    white,
                    [0.0, 0.0, 64.0, 64.0],
                    [0.0; 4],
                    [0.0; 4],
                    IMAGE_FILL,
                    [0.0; 4],
                )
            },
            shade,
        ]
    });
    near(at(&px, 10, 10), BLACK, "the image outside the arc");
    near(at(&px, 20, 20), WHITE, "inside the arc");
    near(at(&px, 50, 30), WHITE, "along the box's edge");
    near(at(&px, 60, 30), BLACK, "past the box");
    near(
        at(&px, 82, 10),
        RED,
        "the shade in the hole's corner, outside the arc",
    );
    near(at(&px, 100, 28), BLACK, "none in the hole");
    near(at(&px, 92, 20), BLACK, "none inside the arc");
    near(at(&px, 100, 56), RED, "the shade below the hole");
}

/// A group's composite quad (`group` of the paint's), over `rect`.
fn composite(group: u32, rect: [f32; 4]) -> PaintQuad {
    PaintQuad {
        kind: PaintKind::Group as u32,
        a: group,
        ..solid(rect, WHITE, 1.0)
    }
}

#[test]
#[ignore = "requires GPU"]
fn a_group_blurs_what_it_holds_and_a_drop_shadow_lands_below_it() {
    use og_render::paint::{PaintEffect, PaintGroup};
    // Left: a white 32 px box at 16, 16 blurred by a deviation of 4 px.
    // Right: a white 16 px box at 88, 16 with a red drop shadow 24 px
    // below it, sharp (a deviation of half a pixel).
    let px = render(|scene| {
        let p = &mut scene.ui.paint;
        p.effects = vec![
            PaintEffect::Blur(4.0),
            PaintEffect::DropShadow {
                offset: [0.0, 24.0],
                sigma: 0.5,
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];
        let group = |quad, first| PaintGroup {
            quad,
            end: quad + 2,
            first,
            count: 1,
            xf: 0,
            backdrop: false,
        };
        p.groups = vec![group(0, 0), group(2, 1)];
        vec![
            composite(0, [0.0, 0.0, 64.0, 64.0]),
            solid([16.0, 16.0, 32.0, 32.0], WHITE, 1.0),
            composite(1, [80.0, 8.0, 32.0, 64.0]),
            solid([88.0, 16.0, 16.0, 16.0], WHITE, 1.0),
        ]
    });
    // Across the blurred box's left edge at mid height, in encoded values
    // as CSS blurs: half at the edge, 0.5 erfc(1/√2) = 0.159 a deviation
    // out, 0.841 a deviation in, the middle untouched.
    near(at(&px, 32, 32), WHITE, "the middle");
    near(at(&px, 16, 32), grey(128), "the edge");
    near(at(&px, 12, 32), grey(41), "a deviation out");
    near(at(&px, 20, 32), grey(214), "a deviation in");
    near(at(&px, 2, 32), BLACK, "past three deviations");
    near(at(&px, 96, 24), WHITE, "the shadowed box over its shadow");
    near(at(&px, 96, 48), RED, "its shadow 24 px below");
    near(at(&px, 96, 36), BLACK, "between them");
}

/// A wide blur runs on its layer halved (twice for a deviation of 12
/// px) and is scaled back: it must still be the Gaussian asked for, where
/// it was. Ways this could fail: the halved image read at the wrong scale
/// or shifted by a texel of a level, the deviation left at its full-size
/// value or not divided by the halving, or the scale back skipped.
#[test]
#[ignore = "requires GPU"]
fn a_wide_blur_runs_halved_and_lands_as_the_gaussian_asked_for() {
    use og_render::paint::{PaintEffect, PaintGroup};
    // A white half plane from x = 64 (the layer's right edge 64 px, five
    // deviations, past it), blurred by a deviation of 12 px.
    let px = render(|scene| {
        let p = &mut scene.ui.paint;
        p.effects = vec![PaintEffect::Blur(12.0)];
        p.groups = vec![PaintGroup {
            quad: 0,
            end: 2,
            first: 0,
            count: 1,
            xf: 0,
            backdrop: false,
        }];
        vec![
            composite(0, [0.0, 0.0, W as f32, W as f32]),
            solid([64.0, -64.0, 256.0, 256.0], WHITE, 1.0),
        ]
    });
    // In encoded values, as CSS blurs: half at the edge, 0.5 erfc(1/√2)
    // a deviation either side, 0.023 two out.
    let close = |x: u32, want: u8, what: &str| {
        let got = at(&px, x, 64)[0];
        assert!(
            got.abs_diff(want) <= 8,
            "{what} (x {x}): {got}, expected {want}"
        );
    };
    close(64, 128, "the edge");
    close(52, 41, "a deviation out");
    close(76, 214, "a deviation in");
    close(40, 6, "two deviations out");
    close(30, 0, "three deviations out");
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
        round: 0,
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

#[test]
#[ignore = "requires GPU"]
fn a_backdrop_filters_what_is_under_it_on_every_target() {
    use og_render::paint::{GROUP_SHAPED, PaintEffect, PaintGroup};
    // White over the left half (x < 64), the black frame on the right; a
    // backdrop over 32..96 rounded by 16, blurred by a deviation of 4 then
    // halved (brightness 0.5), in encoded values.
    for to in [To::Offscreen, To::Plain, To::View, To::Float] {
        let px = render_to(to, |scene| {
            let p = &mut scene.ui.paint;
            p.filters.push([
                [0.5, 0.0, 0.0, 0.0],
                [0.0, 0.5, 0.0, 0.0],
                [0.0, 0.0, 0.5, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0; 4],
            ]);
            p.effects = vec![PaintEffect::Blur(4.0), PaintEffect::Matrix(0)];
            p.groups = vec![PaintGroup {
                quad: 1,
                end: 2,
                first: 0,
                count: 2,
                xf: 0,
                backdrop: true,
            }];
            vec![
                solid([0.0, 0.0, 64.0, 128.0], WHITE, 1.0),
                PaintQuad {
                    radius: [16.0; 4],
                    b: GROUP_SHAPED,
                    ..composite(0, [32.0, 32.0, 64.0, 64.0])
                },
            ]
        });
        let what = |w: &str| format!("{to:?}: {w}");
        near(at(&px, 40, 64), grey(128), &what("white, halved"));
        near(at(&px, 88, 64), BLACK, &what("black"));
        // 0.5 erfc(0.5 / (4 √2)) = 0.45 of white half a pixel past it.
        near(at(&px, 64, 64), grey(57), &what("across the edge, blurred"));
        near(at(&px, 16, 64), WHITE, &what("beside it"));
        near(at(&px, 34, 34), WHITE, &what("its corner, outside the arc"));
    }
}

/// A wide backdrop blur (a deviation of 12 px) captures what is under it
/// at a quarter of its size, each texel the box under it, blurs there and
/// is read back bilinearly: it must still be the Gaussian asked for, then
/// its colour matrix. Ways this could fail: the capture's boxes off by a
/// texel or reading one texel each, the blur not told its input is
/// already halved, or the matrix run at the wrong level's size.
#[test]
#[ignore = "requires GPU"]
fn a_wide_backdrop_blur_captures_halved_and_lands_as_asked() {
    use og_render::paint::{GROUP_SHAPED, PaintEffect, PaintGroup};
    for to in [To::Offscreen, To::Plain] {
        let px = render_to(to, |scene| {
            let p = &mut scene.ui.paint;
            p.filters.push([
                [0.5, 0.0, 0.0, 0.0],
                [0.0, 0.5, 0.0, 0.0],
                [0.0, 0.0, 0.5, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0; 4],
            ]);
            p.effects = vec![PaintEffect::Blur(12.0), PaintEffect::Matrix(0)];
            p.groups = vec![PaintGroup {
                quad: 1,
                end: 2,
                first: 0,
                count: 2,
                xf: 0,
                backdrop: true,
            }];
            vec![
                solid([0.0, 0.0, 64.0, 128.0], WHITE, 1.0),
                PaintQuad {
                    radius: [16.0; 4],
                    b: GROUP_SHAPED,
                    ..composite(0, [8.0, 8.0, 112.0, 112.0])
                },
            ]
        });
        // In encoded values, halved: a quarter at the edge, 0.5 erfc of
        // ∓1/√2 of a half a deviation either side.
        let close = |x: u32, want: u8, what: &str| {
            let got = at(&px, x, 64)[0];
            assert!(
                got.abs_diff(want) <= 8,
                "{to:?}: {what} (x {x}): {got}, expected {want}"
            );
        };
        close(64, 64, "the edge");
        close(52, 107, "a deviation into the white");
        close(76, 20, "a deviation into the black");
        close(28, 127, "deep in the white");
    }
}

fn raster(
    tier: GlyphTier,
    slot: GlyphSlot,
    at: [f32; 2],
    color: [f32; 4],
    p: [f32; 4],
) -> PaintQuad {
    let (x, y) = (f32::from(slot.x), f32::from(slot.y));
    let (w, h) = (f32::from(slot.width), f32::from(slot.height));
    PaintQuad {
        rect: [at[0], at[1], w, h],
        clip: [0.0, 0.0, W as f32, W as f32],
        color,
        radius: [0.0; 4],
        p,
        kind: PaintKind::Raster as u32,
        a: tier as u32,
        b: u32::from(slot.page),
        xf: 0,
        uv: [x, y, x + w, y + h],
        wipe: [0.0; 4],
        mask: 0,
        filter: 0,
        round: 0,
    }
}

/// A 4×4 mask, each pixel its own coverage: 15, 31, … 255 row by row.
fn ramp() -> Vec<u8> {
    (0..16u32).map(|i| (15 + 16 * i) as u8).collect()
}

fn close(got: [u8; 3], want: [u8; 3], what: &str) {
    let off = got.iter().zip(want).map(|(&g, w)| g.abs_diff(w)).max();
    assert!(off.unwrap() <= 2, "{what}: {got:?}, expected {want:?}");
}

/// What would break:
/// - a coverage glyph read from the wrong place in its page (a plot past
///   the first uploaded at the page's origin, a texel off, or filtered
///   with its neighbour)
/// - coverage drawn as it is, not through Skia's preblend as Chrome on
///   Linux draws text (white ink at half coverage is 143 there, not 128),
///   or blended with the ground in linear light (186), on any of the
///   pass's targets
#[test]
#[ignore = "requires GPU"]
fn a_coverage_glyph_is_preblended_then_blended_as_css_blends() {
    // SkTMaskGamma's row for white ink (gamma 1.2, no contrast there) at
    // the ramp's coverages, worked from Skia's formula.
    const WANT: [u8; 16] = [
        24, 44, 62, 80, 96, 112, 128, 143, 157, 172, 186, 200, 214, 228, 242, 255,
    ];
    let key = |g| GlyphKey::new(GlyphTier::Coverage, 0, 0, g, 16 * 64);
    for to in [To::Offscreen, To::Plain, To::View] {
        let px = render_to(to, |scene| {
            let mut atlas = scene.ui.glyphs.lock().unwrap();
            // The first fills plot 0, so the ramp lies in plot 1.
            atlas
                .insert(key(1), (0, 0), (255, 255), &[255; 255 * 255])
                .unwrap();
            let slot = atlas.insert(key(2), (0, 0), (4, 4), &ramp()).unwrap();
            assert_eq!((slot.x, slot.y), (256, 0));
            vec![
                solid([0.0, 0.0, W as f32, W as f32], BLACK, 1.0),
                raster(GlyphTier::Coverage, slot, [10.0, 20.0], [1.0; 4], [0.0; 4]),
            ]
        });
        for (i, want) in (0..16u32).zip(WANT) {
            let (x, y) = (10 + i % 4, 20 + i / 4);
            close(at(&px, x, y), [want; 3], &format!("{to:?}, pixel {i}"));
        }
        // Its neighbours in the plot are clear: nothing past its edge.
        for (x, y) in [(9, 20), (14, 20), (10, 19), (10, 24)] {
            close(at(&px, x, y), [0; 3], "beside the glyph");
        }
    }
}

/// What would break:
/// - a colour glyph tinted by the text colour, or its premultiplied
///   colour drawn as straight (a half-clear pixel half as bright again)
/// - a distance field's edge at the wrong value, or its range not scaled
///   to the screen
#[test]
#[ignore = "requires GPU"]
fn colour_glyphs_keep_their_colours_and_a_field_draws_its_inside() {
    let px = render(|scene| {
        let mut atlas = scene.ui.glyphs.lock().unwrap();
        // Opaque, then the same colour half clear (premultiplied).
        let color = atlas
            .insert(
                GlyphKey::new(GlyphTier::Color, 0, 0, 1, 16 * 64),
                (0, 0),
                (2, 1),
                &[200, 100, 50, 255, 100, 50, 25, 128],
            )
            .unwrap();
        // Inside (every channel full) on the left, outside on the right.
        let field: Vec<u8> = (0..8 * 8 * 4)
            .map(|i| if (i / 4) % 8 < 4 { 255 } else { 0 })
            .collect();
        let field = atlas
            .insert(
                GlyphKey::new(GlyphTier::Field, 0, 0, 1, 0),
                (0, 0),
                (8, 8),
                &field,
            )
            .unwrap();
        vec![
            // Green ink: a colour glyph draws in its own colours.
            raster(
                GlyphTier::Color,
                color,
                [10.0, 10.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0; 4],
            ),
            raster(
                GlyphTier::Field,
                field,
                [40.0, 40.0],
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 12.0, 0.0],
            ),
        ]
    });
    close(at(&px, 10, 10), [200, 100, 50], "an opaque colour pixel");
    // Half of it over black, blended as encoded: its premultiplied colour.
    close(at(&px, 11, 10), [100, 50, 25], "a half-clear colour pixel");
    for y in [40, 47] {
        close(at(&px, 41, y), [255; 3], "inside the field");
        close(at(&px, 46, y), [0; 3], "outside the field");
    }
}

/// What would break: a plot emptied and filled again keeping the pixels
/// it was first uploaded with.
#[test]
#[ignore = "requires GPU"]
fn a_refilled_plot_draws_its_new_glyph() {
    let mut scene = RenderScene::new();
    scene.ui.overlay = Some(Overlay::new(W as f32, W as f32, 1.0));
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    scene.ui.paint.clear();
    (scene.ui.paint.width, scene.ui.paint.height) = (W as f32, W as f32);
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: W,
        height: W,
        ..Default::default()
    })
    .expect("renderer");
    let key = |g| GlyphKey::new(GlyphTier::Coverage, 0, 0, g, 16 * 64);
    let mut draw = |scene: &mut RenderScene, glyph: u32, mask: [u8; 4]| {
        let slot = scene
            .ui
            .glyphs
            .lock()
            .unwrap()
            .insert(key(glyph), (0, 0), (2, 2), &mask)
            .unwrap();
        assert_eq!((slot.x, slot.y), (0, 0));
        scene.ui.paint.quads = vec![raster(
            GlyphTier::Coverage,
            slot,
            [8.0, 8.0],
            [1.0; 4],
            [0.0; 4],
        )];
        scene.ui.paint.generation += 1;
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
        let px = renderer.read_offscreen().expect("capture").pixels;
        [(8, 8), (9, 8), (8, 9), (9, 9)].map(|(x, y)| at(&px, x, y)[0])
    };
    assert_eq!(draw(&mut scene, 1, [255, 0, 0, 255]), [255, 0, 0, 255]);
    // Unused for 64 frames, its plot is emptied; the next glyph takes
    // the same place.
    for _ in 0..UNUSED_FRAMES {
        scene.ui.glyphs.lock().unwrap().advance();
    }
    assert_eq!(scene.ui.glyphs.lock().unwrap().stats().evicted, 1);
    assert_eq!(draw(&mut scene, 2, [0, 255, 255, 0]), [0, 255, 255, 0]);
}
