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
    Antialias, Background, Camera, DirectionalLight, DitherPattern, Exposure, Halftone,
    HalftoneMode, HalftoneRegion, Hatching, Palette, PosterizeMode, Projection, Quantize,
    RenderDesc, RenderScene, RenderTarget, Renderer, StandardMaterial, StyliseSpace, Tonemap,
    shapes,
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
        mode: Default::default(),
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

fn posterized(rgb: [u8; 3]) -> RenderScene {
    let mut scene = field(rgb);
    scene.stylise.quantize = Some(Quantize {
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
    toned.post.tonemap = Tonemap::Aces;
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
    if let Some(q) = &mut one_bit.stylise.quantize {
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
fn tone_posterize_keeps_a_colours_hue_as_its_light_moves() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // The field under light k: its linear colour scaled, as exposure or a
    // dimmer sun scales it, posterized to six tones without a dither.
    let lit = |rgb: [u8; 3], k: f32, mode: PosterizeMode| {
        let mut scene = field(rgb);
        scene.environment.background = Background::Color(Vec3::from(rgb.map(srgb_to_linear)) * k);
        scene.stylise.quantize = Some(Quantize {
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

/// The 4-connected components of the pixels where `inside` holds, and how
/// many of them lie wholly inside the frame (touching none of its edges).
fn components(px: &[u8], inside: impl Fn(&[u8]) -> bool) -> (usize, usize) {
    let mut seen = vec![false; W * H];
    let mut count = 0;
    let mut enclosed = 0;
    for start in 0..W * H {
        if seen[start] || !inside(&px[start * 4..start * 4 + 4]) {
            continue;
        }
        count += 1;
        let mut edge = false;
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(i) = stack.pop() {
            let (x, y) = (i % W, i / W);
            edge |= x == 0 || y == 0 || x + 1 == W || y + 1 == H;
            let mut near = Vec::new();
            if x > 0 {
                near.push(i - 1);
            }
            if x + 1 < W {
                near.push(i + 1);
            }
            if y > 0 {
                near.push(i - W);
            }
            if y + 1 < H {
                near.push(i + W);
            }
            for j in near {
                if !seen[j] && inside(&px[j * 4..j * 4 + 4]) {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        if !edge {
            enclosed += 1;
        }
    }
    (count, enclosed)
}

#[test]
fn shade_halftone_prints_dots_that_never_touch() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    for mode in [HalftoneMode::Ink, HalftoneMode::Cmyk] {
        // A deep shade, well inside the shade region's tones.
        let mut scene = field([46, 40, 70]);
        scene.stylise.halftone = Some(Halftone {
            strength: 1.0,
            size: 6.0,
            angle_degrees: 45.0,
            color: [0.0; 3],
            space: StyliseSpace::Screen,
            mode,
            misregistration: 0.0,
            region: HalftoneRegion::Shade,
        });
        let px = frame(&mut renderer, &scene);
        let luma = |p: &[u8]| u32::from(p[0]) * 3 + u32::from(p[1]) * 6 + u32::from(p[2]);
        let lo = px.chunks(4).map(luma).min().unwrap();
        let hi = px.chunks(4).map(luma).max().unwrap();
        let mid = (lo + hi) / 2;
        // A hole enclosed by touching dots is a ground field inside the
        // frame; a sliver the frame's edge cuts off between dots is not.
        let (ground, enclosed) = components(&px, |p| luma(p) > mid);
        let (dots, _) = components(&px, |p| luma(p) <= mid);
        println!(
            "{mode:?}: luma {lo}..{hi}, {ground} ground fields ({enclosed} enclosed), {dots} dots"
        );
        assert!(dots > 100, "{mode:?}: separate dots ({dots})");
        // One plate's dots never touch. In CMYK two plates' dots at their
        // different angles may overlap round a speck of ground, as a
        // rosette does; old code's merged dots enclosed a hole per dot.
        let most = if mode == HalftoneMode::Ink {
            0
        } else {
            dots / 100
        };
        assert!(
            enclosed <= most,
            "{mode:?}: the dots never touch, so they enclose no ground ({enclosed} enclosed, at most {most})"
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
        mode: Default::default(),
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
            paper: [1.0; 3],
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

#[test]
fn colour_hatching_keeps_conformed_art_in_its_authored_colour() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // A muted earth tone, weak enough that the lit-surface wash takes most
    // of its tint towards paper.
    const SOIL: [u8; 3] = [170, 140, 110];
    let mut ground_chroma = |conformed| {
        let mut scene = field(SOIL);
        scene.stylise.hatching = Some(Hatching {
            strength: 0.5,
            scale: 5.0,
            levels: 1,
            angle_degrees: 45.0,
            color: [0.04, 0.035, 0.03],
            paper: [1.0; 3],
            space: StyliseSpace::Screen,
            depth: 0.0,
            keep_color: true,
        });
        if conformed {
            scene.add_conformed_colors(&[SOIL]);
        }
        let px = frame(&mut renderer, &scene);
        // The lightest half of the pixels: the wash between the strokes.
        let mut by_light: Vec<&[u8]> = px.chunks(4).collect();
        by_light.sort_by_key(|p| p[0] as u32 + p[1] as u32 + p[2] as u32);
        let light = &by_light[by_light.len() / 2..];
        light.iter().map(|p| chroma(p)).sum::<f32>() / light.len() as f32
    };
    let art = ground_chroma(true);
    let lit = ground_chroma(false);
    println!("wash chroma: conformed {art:.4}, lit surface {lit:.4}");
    // Old code washed conformed art as a lit surface, taking the
    // authored colour towards paper with it.
    assert!(
        art > lit * 1.3,
        "conformed art keeps its colour under the wash ({art} against {lit})"
    );
}

/// A dark wall facing the camera, turned `tilt` degrees about x away from
/// it, under shade-only Ben-Day dots.
fn turned_wall(tilt: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.post.tonemap = Tonemap::None;
    let dark = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.08),
        roughness: 1.0,
        ..Default::default()
    });
    let mesh = scene.add_mesh(shapes::plane_xy(60.0, 60.0));
    scene.add_instance(mesh, dark, glam::Mat4::from_rotation_x(-tilt.to_radians()));
    scene.light = DirectionalLight {
        direction_to_light: Vec3::Z,
        intensity: 1.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 10.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.stylise.halftone = Some(Halftone {
        strength: 1.0,
        size: 5.0,
        angle_degrees: 45.0,
        color: [0.0; 3],
        space: StyliseSpace::Screen,
        mode: HalftoneMode::Ink,
        misregistration: 0.0,
        region: HalftoneRegion::Shade,
    });
    scene
}

#[test]
fn shade_dots_print_flat_on_a_surface_seen_edge_on() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // The luma range over the frame's middle rows.
    let range = |px: &[u8]| {
        let luma =
            |i: usize| u32::from(px[i]) * 3 + u32::from(px[i + 1]) * 6 + u32::from(px[i + 2]);
        let (mut lo, mut hi) = (u32::MAX, 0);
        for y in H / 2 - 4..H / 2 + 4 {
            for x in W / 4..W * 3 / 4 {
                let l = luma((y * W + x) * 4);
                lo = lo.min(l);
                hi = hi.max(l);
            }
        }
        hi - lo
    };
    let facing = range(&frame(&mut renderer, &turned_wall(0.0)));
    let edge_on = range(&frame(&mut renderer, &turned_wall(80.0)));
    println!("luma range: facing {facing}, edge-on {edge_on}");
    assert!(
        facing > 300,
        "a wall facing the camera prints its dots ({facing})"
    );
    // Old code printed the same dots on the grazing wall, squeezed into a
    // checker.
    assert!(
        edge_on < 60,
        "a wall seen edge-on keeps flat ink ({edge_on})"
    );
}

#[test]
fn hatching_fades_in_from_clean_paper_at_its_threshold() {
    let mut renderer = Renderer::new_blocking(&desc()).expect("renderer");
    // The spread of the red channel over a grey field `g` under one stroke
    // set (its threshold at tone 0.5).
    let mut spread = |g: u8| {
        let mut scene = field([g, g, g]);
        scene.stylise.hatching = Some(Hatching {
            strength: 1.0,
            scale: 5.0,
            levels: 1,
            angle_degrees: 45.0,
            color: [0.0; 3],
            paper: [1.0; 3],
            space: StyliseSpace::Screen,
            depth: 0.0,
            keep_color: false,
        });
        let px = frame(&mut renderer, &scene);
        let r = px.chunks(4).map(|p| p[0]);
        r.clone().max().unwrap() - r.min().unwrap()
    };
    // Tone 0.48, just under the threshold, and 0.24, well under it.
    let (light, dark) = (spread(122), spread(61));
    println!("stroke spread: near the threshold {light}, in the shade {dark}");
    // Old code started every stroke at 0.3 of its ink: 70 levels at once.
    assert!(
        light <= 8,
        "strokes at their threshold all but vanish ({light})"
    );
    assert!(dark >= 40, "the shade is hatched ({dark})");
}
