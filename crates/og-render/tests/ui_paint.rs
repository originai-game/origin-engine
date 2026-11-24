//! The paint pass (og-ui's new core) drawing on each of its targets, read
//! back as pixels (no reference frame): a translucent fill blended as CSS
//! blends, on the output, its plain view, a caller's view and a float
//! output.
use glam::Vec3;
use og_render::paint::{PaintKind, PaintQuad};
use og_render::{Background, Overlay, RenderDesc, RenderScene, RenderTarget, Renderer, Tonemap};

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

const WHITE: [u8; 3] = [255, 255, 255];

fn grey(v: u8) -> [u8; 3] {
    [v; 3]
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
        blend: 0,
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
