//! The palette, dither and print side of Stylise, as pixel assertions with
//! expectations computed here from the inputs, not from a reference frame:
//!
//! - A field already on a palette colour stays that colour under posterize
//!   and a dither (the quantise keeps conformed art), while an off-palette
//!   field still dithers. Old code posterised first: the field became a
//!   checker of two cube corners, each on the palette.
//! - Art the cook conformed (its colours given to the scene as conformed)
//!   stays its colour under the look's posterize and dither with no frame
//!   palette, also a few levels off as compression leaves it, while every
//!   other tone (an unconformed material, light)
//!   still steps and dithers: the frame quantise stays the fallback. The
//!   art is told by its colour before the tone curve, so a tone-mapped
//!   look keeps it too, except in a 1-bit look, whose dither is its only
//!   shading. Old code posterised the art into a checker.
//! - Posterize in tone mode keeps a colour's hue as light moves it: a
//!   grey-green field scaled from 0.5 to 1.25 of its light steps its value
//!   but every frame shows the same tint, and a grey stays grey. Each
//!   channel rounded apart (the channel mode) turns the same field grey at
//!   one light and green at another, as minimal-geometric's ground did
//!   through its moods.
//! - Shade-only halftone prints Ben-Day dots that never touch: the ground
//!   between them is one connected field. Old code reproduced the
//!   shade's full tone, so its dots merged into ink round a grid of
//!   isolated paper holes, a checker. On a surface seen edge-on (a floor
//!   at a grazing angle) the dots fade to flat ink; old code printed them
//!   there too, a checker in the floor's shadows.
//! - The dither in surface space moves with the surface: an orthographic
//!   camera panned by three logical pixels sees the same pattern three
//!   pixels over; in screen space the pattern stays on the screen.
//! - CMYK print of pure cyan prints cyan dots only (no black, magenta or
//!   yellow ink); the one-ink halftone printed black dots.
//! - Colour hatching keeps the colour under its strokes: the darkest
//!   pixels of a hatched red field stay red (Oklab chroma above 0.05),
//!   where one-ink strokes are grey.
//! - Hatching fades in from clean paper: a field just under a stroke set's
//!   threshold carries next to no ink, one well under it is hatched; old
//!   code began every stroke at 0.3 of its ink.

use glam::Vec3;
use og_render::{
    Antialias, Background, Camera, DitherPattern, Exposure, PosterizeMode, Quantize, RenderDesc,
    RenderScene, RenderTarget, Renderer, StyliseSpace, Tonemap,
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
    scene.look.post.tonemap = Tonemap::None;
    scene.lighting.environment.background = Background::Color(Vec3::from(rgb.map(srgb_to_linear)));
    scene.frame.camera = Camera {
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

/// Equal-tone near and far panels must not receive equal ink when aerial
/// perspective is enabled. No fog or lighting can explain the reduction.
#[test]
#[ignore = "requires GPU"]
fn hatching_leaves_distant_equal_tones_bare_in_every_pattern_space() {
    use glam::Mat4;
    use og_render::{Hatching, Projection, StandardMaterial, shapes};
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    let mut scene = field([160; 3]);
    scene.frame.camera = Camera {
        position: Vec3::ZERO,
        target: Vec3::NEG_Z,
        projection: Projection::Orthographic { height: 4.0 },
        exposure: Exposure::linear(1.0),
        far: 200.0,
        ..Default::default()
    };
    let mesh = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let material = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.15),
        unlit: 1,
        ..Default::default()
    });
    for (x, z) in [(-1.5, -2.0), (1.5, -82.0)] {
        scene.add_instance(
            mesh,
            material,
            Mat4::from_translation(Vec3::new(x, 0.0, z))
                * Mat4::from_scale(Vec3::new(1.5, 2.5, 0.1)),
        );
    }
    let ink = |px: &[u8], x: usize| {
        (60..120)
            .flat_map(|y| (x..x + 36).map(move |x| (y * W + x) * 4))
            .map(|i| 1.0 - px[i] as f32 / 255.0)
            .sum::<f32>()
            / (60.0 * 36.0)
    };
    for (space, keep_color) in [
        StyliseSpace::Screen,
        StyliseSpace::Surface,
        StyliseSpace::Object,
    ]
    .into_iter()
    .flat_map(|space| [false, true].map(|keep| (space, keep)))
    {
        scene.look.stylise.hatching = Some(Hatching {
            strength: 1.0,
            scale: 6.0,
            levels: 4,
            angle_degrees: 0.0,
            color: [0.0; 3],
            paper: [1.0; 3],
            space,
            depth: 0.0,
            distance: 0.0,
            keep_color,
        });
        let plain = frame(&mut renderer, &scene);
        let (near, far) = (ink(&plain, 75), ink(&plain, 210));
        assert!(
            (near - far).abs() < 0.03 && near > 0.1,
            "{space:?}: equal-tone control {near}/{far}"
        );
        scene.look.stylise.hatching.as_mut().unwrap().distance = 20.0;
        let aerial = frame(&mut renderer, &scene);
        let (near, far) = (ink(&aerial, 75), ink(&aerial, 210));
        assert!(
            far < near * 0.5 && near > 0.1,
            "{space:?}: aerial perspective {near}/{far}"
        );
        let sky: Vec<_> = (0..W).map(|x| aerial[(10 * W + x) * 4]).collect();
        assert!(
            sky.iter().max().unwrap() - sky.iter().min().unwrap() <= 1,
            "the sky keeps its wash without strokes"
        );
    }
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

fn posterized(rgb: [u8; 3]) -> RenderScene {
    let mut scene = field(rgb);
    scene.look.stylise.quantize = Some(Quantize {
        posterize: 4,
        mode: Default::default(),
        palette: None,
        dither: DitherPattern::Bayer2,
        dither_strength: 1.0,
        dither_scale: 1,
        space: StyliseSpace::Screen,
    });
    scene
}

#[test]
#[ignore = "requires GPU"]
fn conformed_art_keeps_its_colour_while_the_rest_still_quantises() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // Without the conformed set the look's quantise steps the art: the
    // fallback a plain material gets.
    let plain = colours(&frame(&mut renderer, &posterized(ART)));
    println!("unconformed: {plain:?}");
    assert!(
        plain.len() >= 2,
        "the art's tone is posterised and dithered: {plain:?}"
    );
    let mut scene = posterized(ART);
    scene.add_conformed_colors(&[ART, [12, 40, 200]]);
    let kept = colours(&frame(&mut renderer, &scene));
    println!("conformed: {kept:?}");
    assert_eq!(kept.len(), 1, "one colour, the art's: {kept:?}");
    assert!(
        kept[0].iter().zip(&ART).all(|(a, b)| a.abs_diff(*b) <= 2),
        "the art's own colour {ART:?}: {kept:?}"
    );
    // A few levels off (block compression, filtering) is still the art:
    // kept as drawn.
    let mut near = posterized([113, 147, 104]);
    near.add_conformed_colors(&[ART]);
    let close = colours(&frame(&mut renderer, &near));
    println!("near: {close:?}");
    assert_eq!(
        close.len(),
        1,
        "a compressed texel of the art is kept: {close:?}"
    );
    // Under a tone curve the art is told by its colour before the curve:
    // kept as the curve draws it, one colour. Old code compared the
    // tone-mapped colour, off the art's by more than the reach, and
    // stepped the art into a dither.
    let mut toned = posterized(ART);
    toned.look.post.tonemap = Tonemap::Aces;
    toned.add_conformed_colors(&[ART]);
    let curved = colours(&frame(&mut renderer, &toned));
    println!("tone-mapped: {curved:?}");
    assert_eq!(
        curved.len(),
        1,
        "the art under ACES keeps one colour: {curved:?}"
    );
    // A 1-bit look (two tones per channel) shades with its dither alone:
    // its conformed art dithers too. Old code kept it one flat colour, a
    // hole through the dither.
    let mut one_bit = posterized(ART);
    if let Some(q) = &mut one_bit.look.stylise.quantize {
        q.posterize = 2;
    }
    one_bit.add_conformed_colors(&[ART]);
    let dithered = colours(&frame(&mut renderer, &one_bit));
    println!("1-bit: {dithered:?}");
    assert!(
        dithered.len() >= 2,
        "conformed art dithers in a 1-bit look: {dithered:?}"
    );
    // A lit tone farther off is not conformed: stepped and dithered.
    let mut lit = posterized([128, 160, 110]);
    lit.add_conformed_colors(&[ART]);
    let stepped = colours(&frame(&mut renderer, &lit));
    println!("lit: {stepped:?}");
    assert!(
        stepped.len() >= 2,
        "a lit tone of the art quantises: {stepped:?}"
    );
}

#[test]
#[ignore = "requires GPU"]
fn tone_posterize_keeps_a_colours_hue_as_its_light_moves() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // The field under light k: its linear colour scaled, as exposure or a
    // dimmer sun scales it, posterized to six tones without a dither.
    let lit = |rgb: [u8; 3], k: f32, mode: PosterizeMode| {
        let mut scene = field(rgb);
        scene.lighting.environment.background =
            Background::Color(Vec3::from(rgb.map(srgb_to_linear)) * k);
        scene.look.stylise.quantize = Some(Quantize {
            posterize: 6,
            mode,
            palette: None,
            dither: DitherPattern::None,
            dither_strength: 1.0,
            dither_scale: 1,
            space: StyliseSpace::Screen,
        });
        scene
    };
    // The tint of a frame's one colour: its channels over the largest.
    let tint = |renderer: &mut Renderer, scene: &RenderScene| {
        let c = colours(&frame(renderer, scene));
        assert_eq!(c.len(), 1, "a flat field stays one colour: {c:?}");
        let v = f32::from(*c[0].iter().max().expect("three channels")).max(1.0);
        (c[0], c[0].map(|x| f32::from(x) / v))
    };
    const GREY_GREEN: [u8; 3] = [140, 165, 140];
    let lights = [0.5, 0.7, 0.85, 1.0, 1.25];
    let toned: Vec<_> = lights
        .iter()
        .map(|&k| tint(&mut renderer, &lit(GREY_GREEN, k, PosterizeMode::Tone)))
        .collect();
    println!("tone: {toned:?}");
    for (c, t) in &toned {
        assert!(
            t.iter().zip(toned[0].1).all(|(a, b)| (a - b).abs() < 0.02),
            "every light shows the first one's tint {:?}: {c:?} is {t:?}",
            toned[0].1
        );
    }
    assert!(
        toned.windows(2).any(|w| w[0].0 != w[1].0),
        "the value still steps with the light: {toned:?}"
    );
    // The same field rounded per channel changes hue across the same
    // lights: the defect the tone mode exists for.
    let channel: Vec<_> = lights
        .iter()
        .map(|&k| tint(&mut renderer, &lit(GREY_GREEN, k, PosterizeMode::Channel)).1)
        .collect();
    println!("channel: {channel:?}");
    assert!(
        channel
            .iter()
            .any(|t| t.iter().zip(channel[0]).any(|(a, b)| (a - b).abs() > 0.1)),
        "per channel, the field's tint moves with the light: {channel:?}"
    );
    // A grey stays grey at every light.
    for k in lights {
        let (c, _) = tint(&mut renderer, &lit([128, 128, 128], k, PosterizeMode::Tone));
        assert!(c[0] == c[1] && c[1] == c[2], "a grey at light {k} is {c:?}");
    }
}
