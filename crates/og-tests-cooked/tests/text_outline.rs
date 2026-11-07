//! A text outline reaches the width it asks for on small text: the UI
//! kit's body and display faces, cooked with their own import settings
//! (tests/fixtures/ui-text/assets/fonts), draw a 2 px outline around 16 px text
//! about 2 px wide.
//!
//! og-render's MSDF text bounds an outline by the distance field's reach,
//! `0.5 * px_range * size / em_px - 0.5` CSS px (overlay.wgsl): at the
//! `px_range 4` the kit's faces were cooked with, 16 px text took at most
//! 0.5 px (em 32) or 0.17 px (em 48) whatever width a style asked, which
//! left the HUD's party names hazy on a pale ground (U6c r18 and r19). The
//! faces now take a third of their em (11 at em 32, 16 at em 48).
//!
//! The reading: a white "I" on white with a black outline outside it, so
//! only the outline darkens the frame; across each row of the stem's
//! straight middle the outline's coverage (`1 - linear luminance`, black
//! over white blended in linear light) sums to twice its width, one band
//! on each side.

use og_cook::runtime::{Bundle, CookedFont};
use og_cook::{CookOptions, cook_project};
use og_core::glam::Vec3;
use og_render::{
    Background, ColorSpace, FontHandle, Glyph, Mips, MsdfFont, Overlay, RenderDesc, RenderScene,
    RenderTarget, Renderer, Text, TextureData, Tonemap, UiAlign, UiTextEffects, UiTextOutline,
};
use std::path::{Path, PathBuf};

const KIT_FONTS: [&str; 2] = ["NotoSansSC-Regular-subset.ttf", "Inter-Display.ttf"];
const SIZE: f32 = 16.0;
const WIDTH: f32 = 2.0;

fn kit_fonts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ui-text/assets/fonts")
}

/// The atlas page of `f` that holds `c`, as a scene font (og-runtime adds
/// one per page; a face cooked with a wider range may take two).
fn page_font(scene: &mut RenderScene, f: &CookedFont, c: char) -> FontHandle {
    let page = f
        .glyphs
        .iter()
        .find(|g| g.codepoint == c as u32)
        .unwrap_or_else(|| panic!("{}: no {c:?}", f.id))
        .page;
    let atlas = scene.add_texture(TextureData {
        mips: Mips::None,
        color_space: ColorSpace::Linear,
        ..f.pages[page as usize].clone()
    });
    let m = &f.meta;
    scene.add_font(MsdfFont {
        atlas,
        distance_range: m.px_range,
        foreground: false,
        line_height: m.ascender - m.descender + m.line_gap,
        glyphs: f
            .glyphs
            .iter()
            .filter(|g| g.page == page)
            .filter_map(|g| {
                Some(Glyph {
                    key: g.key()?,
                    advance: g.advance,
                    plane_bounds: [g.plane_left, g.plane_bottom, g.plane_right, g.plane_top],
                    atlas_bounds: [
                        g.atlas_x as f32,
                        g.atlas_y as f32,
                        (g.atlas_x + g.atlas_w) as f32,
                        (g.atlas_y + g.atlas_h) as f32,
                    ],
                })
            })
            .collect(),
        kerning: Vec::new(),
    })
}

fn lin(byte: u8) -> f32 {
    let c = f32::from(byte) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// The outline's width, CSS px, around `font`'s 16 px "I".
fn measured_outline(font: &str) -> f32 {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("og-text-outline-{font}"));
    let _ = std::fs::remove_dir_all(&root);
    let dst = root.join("assets/fonts");
    std::fs::create_dir_all(&dst).expect("dir");
    for f in [font.to_string(), format!("{font}.ogt")] {
        std::fs::copy(kit_fonts_dir().join(&f), dst.join(&f))
            .unwrap_or_else(|e| panic!("{f}: {e}"));
    }
    let report = cook_project(&root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).expect("bundle");
    let cooked = bundle
        .load_font(&format!("fonts/{font}"))
        .unwrap_or_else(|e| panic!("{font}: {e:?}"));

    let (w, h) = (96u32, 64u32);
    let mut scene = RenderScene::new();
    let handle = page_font(&mut scene, &cooked, 'I');
    let mut o = Overlay::new(w as f32, h as f32, 1.0);
    o.texts.push(Text {
        position: Vec3::new(40.0, 40.0, 0.0),
        size: SIZE,
        color: [1.0; 4],
        ..Text::new(handle, "I")
    });
    o.ui.text_effects.push(UiTextEffects {
        text: 0,
        outline: Some(UiTextOutline {
            width: WIDTH,
            color: [0.0, 0.0, 0.0, 1.0],
            align: UiAlign::Outside,
        }),
        ..UiTextEffects::default()
    });
    scene.overlay = Some(o);
    scene.environment.background = Background::Color(Vec3::ONE);
    scene.post.tonemap = Tonemap::None;
    let desc = RenderDesc {
        width: w,
        height: h,
        ..RenderDesc::default()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");

    // Each row's summed coverage; the stem's straight middle is the rows
    // within the middle half of those the glyph darkens.
    let rows: Vec<f32> = (0..h)
        .map(|y| {
            (0..w)
                .map(|x| 1.0 - lin(image.pixels[((y * w + x) * 4) as usize]))
                .sum()
        })
        .collect();
    let inked: Vec<usize> = (0..rows.len()).filter(|&y| rows[y] > 0.05).collect();
    assert!(
        inked.len() >= 8,
        "{font}: the outline darkens {} rows",
        inked.len()
    );
    let (first, last) = (inked[0], inked[inked.len() - 1]);
    let quarter = (last - first) / 4;
    let mut middle: Vec<f32> = rows[first + quarter..=last - quarter].to_vec();
    middle.sort_by(f32::total_cmp);
    middle[middle.len() / 2] / 2.0
}

#[test]
#[ignore = "requires GPU"]
fn a_2_px_outline_on_16_px_text_is_about_2_px_wide_in_the_kit_faces() {
    let mut failing = Vec::new();
    for font in KIT_FONTS {
        let got = measured_outline(font);
        println!("{font}: a {WIDTH} px outline on {SIZE} px text measures {got:.2} px");
        if (got - WIDTH).abs() > 0.35 {
            failing.push(format!("{font}: {got:.2} px, not about {WIDTH}"));
        }
    }
    assert!(failing.is_empty(), "{}", failing.join("\n"));
}
