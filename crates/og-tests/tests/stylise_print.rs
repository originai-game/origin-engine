//! The palette, dither and print side of Stylise, as pixel assertions with
//! expectations computed here from the inputs, not from a reference frame:
//!
//! - A field already on a palette colour stays that colour under posterize
//!   and a dither (the quantise keeps conformed art), while an off-palette
//!   field still dithers. Old code posterised first: the field became a
//!   checker of two cube corners, each on the palette.
//! - The dither in surface space moves with the surface: an orthographic
//!   camera panned by three logical pixels sees the same pattern three
//!   pixels over; in screen space the pattern stays on the screen.
//! - CMYK print of pure cyan prints cyan dots only (no black, magenta or
//!   yellow ink); the one-ink halftone printed black dots.
//! - Colour hatching keeps the colour under its strokes: the darkest
//!   pixels of a hatched red field stay red (Oklab chroma above 0.05),
//!   where one-ink strokes are grey.

use glam::Vec3;
use og_render::{
    Antialias, Background, Camera, DirectionalLight, DitherPattern, Exposure, Halftone,
    HalftoneMode, HalftoneRegion, Hatching, Palette, Projection, Quantize, RenderDesc, RenderScene,
    RenderTarget, Renderer, StandardMaterial, StyliseSpace, Tonemap, shapes,
};
use og_tests::scenes;

const W: usize = 320;
const H: usize = 180;

fn desc() -> RenderDesc {
    RenderDesc {
        width: W as u32,
        height: H as u32,
        antialias: Antialias::Off,
        ..scenes::base_desc()
    }
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// An empty scene whose background is the sRGB colour `rgb`, untonemapped,
/// so the frame shows `rgb` exactly.
fn field(rgb: [u8; 3]) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.post.tonemap = Tonemap::None;
    scene.environment.background = Background::Color(Vec3::from(rgb.map(srgb_to_linear)));
    scene.camera = Camera {
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

fn frame(renderer: &mut Renderer, scene: &RenderScene) -> Vec<u8> {
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("frame");
    renderer.read_offscreen().expect("capture").pixels
}

/// The distinct colours of a frame (colours within 2 of 255 on every
/// channel count as one).
fn colours(px: &[u8]) -> Vec<[u8; 3]> {
    let mut out: Vec<[u8; 3]> = Vec::new();
    for p in px.chunks(4) {
        let c = [p[0], p[1], p[2]];
        if !out
            .iter()
            .any(|o| o.iter().zip(&c).all(|(a, b)| a.abs_diff(*b) <= 2))
        {
            out.push(c);
        }
    }
    out
}

/// A colour off the 4-tone posterize grid (0, 85, 170, 255 per channel)
/// with the eight grid corners round it: the palette where old code's
/// posterize landed every pixel on a corner, each a palette colour.
const ART: [u8; 3] = [110, 150, 100];

fn corner_palette() -> Palette {
    let mut colors = vec![ART];
    for r in [85, 170] {
        for g in [85, 170] {
            for b in [85, 170] {
                colors.push([r, g, b]);
            }
        }
    }
    Palette { colors }
}

fn quantized(rgb: [u8; 3], scene_palette: &Palette) -> RenderScene {
    let mut scene = field(rgb);
    let palette = scene.add_palette(scene_palette.clone());
    scene.stylise.quantize = Some(Quantize {
        posterize: 4,
        palette: Some(palette),
        dither: DitherPattern::Bayer2,
        dither_strength: 1.0,
        dither_scale: 1,
        space: StyliseSpace::Screen,
    });
    scene
}

#[test]
fn a_field_on_a_palette_colour_stays_flat_under_posterize_and_dither() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let pal = corner_palette();
    let kept = colours(&frame(&mut renderer, &quantized(ART, &pal)));
    println!("on the palette: {kept:?}");
    assert_eq!(kept.len(), 1, "one colour, the art's: {kept:?}");
    assert!(
        kept[0].iter().zip(&ART).all(|(a, b)| a.abs_diff(*b) <= 2),
        "the art's own colour {ART:?}: {kept:?}"
    );
    // A tone between the palette's colours (as light or fog make) is still
    // stepped and dithered: two or more colours, each on the palette.
    let lit = [128, 128, 128];
    let dithered = colours(&frame(&mut renderer, &quantized(lit, &pal)));
    println!("off the palette: {dithered:?}");
    assert!(
        dithered.len() >= 2,
        "an off-palette tone dithers: {dithered:?}"
    );
    for c in &dithered {
        assert!(
            pal.colors
                .iter()
                .any(|p| p.iter().zip(c).all(|(a, b)| a.abs_diff(*b) <= 2)),
            "{c:?} is a palette colour"
        );
    }
}

/// A grey wall square to an orthographic camera at x = `pan`, its pixels
/// exactly 1/32 m wide, posterised to two tones through a 4x4 Bayer
/// dither laid in `space`.
fn wall(pan: f32, space: StyliseSpace) -> RenderScene {
    let mut scene = RenderScene::new();
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.5),
        roughness: 1.0,
        ..Default::default()
    });
    let mesh = scene.add_mesh(shapes::plane_xy(60.0, 60.0));
    scene.add_instance(mesh, grey, glam::Mat4::IDENTITY);
    scene.light = DirectionalLight {
        direction_to_light: Vec3::Z,
        intensity: 1.2,
        cast_shadows: false,
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(pan, 0.0, 10.0),
        target: Vec3::new(pan, 0.0, 0.0),
        projection: Projection::Orthographic {
            height: H as f32 / 32.0,
        },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.stylise.quantize = Some(Quantize {
        posterize: 2,
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
        space,
    });
    scene
}

/// The share of pixels (away from the edges) where `a` at x + `shift`
/// differs from `b` at x by more than 8 of 255.
fn shifted_differs(a: &[u8], b: &[u8], shift: usize) -> f64 {
    let (mut n, mut all) = (0usize, 0usize);
    for y in 8..H - 8 {
        for x in 8..W - 8 - shift {
            let i = (y * W + x + shift) * 4;
            let j = (y * W + x) * 4;
            all += 1;
            if (0..3).any(|k| a[i + k].abs_diff(b[j + k]) > 8) {
                n += 1;
            }
        }
    }
    n as f64 / all as f64
}

#[test]
fn a_surface_dither_moves_with_the_wall_as_the_camera_pans() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // Three logical pixels: off the 4x4 pattern's period, so a pattern that
    // stays on the screen no longer lines up once shifted.
    let pan = 3.0 / 32.0;
    let mut moved = |space| {
        let a = frame(&mut renderer, &wall(0.0, space));
        let b = frame(&mut renderer, &wall(pan, space));
        assert!(colours(&a).len() >= 2, "the wall dithers");
        // The camera moved right: the wall moved three pixels left.
        shifted_differs(&a, &b, 3)
    };
    let surface = moved(StyliseSpace::Surface);
    let screen = moved(StyliseSpace::Screen);
    println!("pixels that differ from the wall moved: surface {surface:.4}, screen {screen:.4}");
    assert!(
        surface < 0.01,
        "on the surface the pattern moves with it ({surface})"
    );
    assert!(
        screen > 0.2,
        "on the screen the pattern stays put ({screen})"
    );
}

#[test]
fn cmyk_prints_pure_cyan_in_cyan_ink_only() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let mut print = |mode| {
        let mut scene = field([0, 255, 255]);
        scene.stylise.halftone = Some(Halftone {
            strength: 1.0,
            size: 8.0,
            angle_degrees: 45.0,
            color: [0.0; 3],
            space: StyliseSpace::Screen,
            mode,
            misregistration: 1.5,
            region: HalftoneRegion::All,
        });
        frame(&mut renderer, &scene)
    };
    let px = print(HalftoneMode::Cmyk);
    // Cyan ink takes red only: green and blue stay at paper white, and
    // red goes dark in the dots.
    let other_ink = px.chunks(4).filter(|p| p[1] < 245 || p[2] < 245).count();
    let cyan = px.chunks(4).filter(|p| p[0] < 64).count();
    println!("cmyk: {cyan} cyan pixels, {other_ink} with another ink");
    assert_eq!(other_ink, 0, "no magenta, yellow or black ink");
    assert!(cyan > px.len() / 4 / 10, "cyan dots ({cyan})");
    // The one-ink halftone of the same field prints black-ish dots.
    let ink = print(HalftoneMode::Ink);
    assert!(
        ink.chunks(4).any(|p| p[1] < 64),
        "one ink darkens green too"
    );
}

/// Oklab chroma of an sRGB colour.
fn chroma(p: &[u8]) -> f32 {
    let [r, g, b] = [p[0], p[1], p[2]].map(|c| f64::from(srgb_to_linear(c)));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    let a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s;
    let bb = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s;
    (a * a + bb * bb).sqrt() as f32
}

#[test]
fn colour_hatching_keeps_the_hue_under_its_strokes() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let mut stroke_chroma = |keep_color| {
        let mut scene = field([230, 60, 50]);
        scene.stylise.hatching = Some(Hatching {
            strength: 0.9,
            scale: 5.0,
            levels: 4,
            angle_degrees: 45.0,
            color: [0.04, 0.035, 0.03],
            space: StyliseSpace::Screen,
            depth: 0.0,
            keep_color,
        });
        let px = frame(&mut renderer, &scene);
        // The darkest tenth of the pixels: the strokes.
        let mut by_light: Vec<&[u8]> = px.chunks(4).collect();
        by_light.sort_by_key(|p| p[0] as u32 + p[1] as u32 + p[2] as u32);
        let dark = &by_light[..by_light.len() / 10];
        dark.iter().map(|p| chroma(p)).sum::<f32>() / dark.len() as f32
    };
    let kept = stroke_chroma(true);
    let grey = stroke_chroma(false);
    println!("stroke chroma: kept {kept:.4}, one ink {grey:.4}");
    assert!(kept > 0.05, "the strokes stay red ({kept})");
    assert!(grey < 0.03, "one-ink strokes are grey ({grey})");
}
