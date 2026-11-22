//! The test font and the text scene.

use super::*;
use og_render::TextGlyph;
use og_render::ui::mass::{NUMBER_SYMBOLS, UiMassGlyph, UiMassString};
use og_text::fonts::Collection;
use og_text::layout::{Style, Text as Layout};
use og_text::strike::Strikes;
use std::cell::RefCell;

/// The faces every text scene draws, as the default pack orders them:
/// Instrument Sans for Latin, then Noto Sans SC (templates/fonts) for Han.
const FAMILY: &str = r#""Instrument Sans", "Noto Sans SC""#;

#[cfg(not(target_arch = "wasm32"))]
fn faces() -> Result<[Vec<u8>; 2], String> {
    let read = |rel: &str| {
        let path = format!("{}/../../templates/{rel}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(&path)
            .map_err(|e| format!("{path}: {e} (bun scripts/assets.ts ensure templates)"))
    };
    Ok([
        read("styles/shared/InstrumentSans-Regular.ttf")?,
        read("fonts/chinese-simplified/NotoSansSC-Regular.ttf")?,
    ])
}

#[cfg(target_arch = "wasm32")]
fn faces() -> Result<[Vec<u8>; 2], String> {
    Ok([
        include_bytes!("../../../../templates/styles/shared/InstrumentSans-Regular.ttf").to_vec(),
        include_bytes!("../../../../templates/fonts/chinese-simplified/NotoSansSC-Regular.ttf")
            .to_vec(),
    ])
}

struct Layouts {
    fonts: Collection,
    strikes: Strikes,
    layout: Layout,
}

thread_local! {
    static LAYOUTS: RefCell<Option<Layouts>> = const { RefCell::new(None) };
}

/// The test font of a scene: its texts are laid out once and their glyphs
/// put into the scene's glyph atlas.
#[derive(Clone)]
pub struct TestFont {
    atlas: og_render::glyphs::SharedAtlas,
}

impl TestFont {
    /// White text of em size 1 at the origin, lines broken at `\n`.
    pub fn text(&self, s: &str) -> Text {
        self.aligned(s, 0.0)
    }

    /// [`TestFont::text`] with each line moved back by `shift` of its
    /// width (0.5 centred on the position).
    pub fn aligned(&self, s: &str, shift: f32) -> Text {
        let mut glyphs = Vec::new();
        self.laid(s, shift, &mut glyphs);
        Text {
            glyphs,
            size: 1.0,
            color: [1.0; 4],
            ..Default::default()
        }
    }

    /// Lays `s` out in the test faces at the field em size (one line,
    /// aligned by `shift` of its width) and puts its glyphs into the
    /// scene's glyph atlas.
    fn laid(&self, s: &str, shift: f32, out: &mut Vec<TextGlyph>) {
        LAYOUTS.with(|l| {
            let mut l = l.borrow_mut();
            let l = l.as_mut().expect("loaded by load_font");
            let style = Style {
                family: FAMILY,
                size: og_render::glyphs::FIELD_EM_PX,
                wrap: false,
                align: match shift {
                    s if s >= 0.75 => og_text::layout::Alignment::End,
                    s if s >= 0.25 => og_text::layout::Alignment::Center,
                    _ => og_text::layout::Alignment::Start,
                },
                ..Default::default()
            };
            let mut atlas = self.atlas.lock().unwrap();
            for _ in 0..8 {
                if og_text::strike::text_glyphs(
                    &mut l.strikes,
                    &mut l.fonts,
                    &mut atlas,
                    &mut l.layout,
                    s,
                    &style,
                    shift,
                    [1.0; 3],
                    out,
                ) {
                    return;
                }
            }
            panic!("the glyph atlas never made room for {s:?}");
        });
    }

    /// A mass template's string: laid out and rastered into the atlas, its
    /// pen in each glyph's quad, and its width (em) for the template's
    /// pivot to divide.
    pub fn mass_string(&self, s: &str) -> Result<UiMassString, String> {
        let mut glyphs = Vec::new();
        let mut width = 0.0;
        LAYOUTS.with(|l| {
            let mut l = l.borrow_mut();
            let l = l.as_mut().expect("loaded by load_font");
            let style = Style {
                family: FAMILY,
                size: og_render::glyphs::FIELD_EM_PX,
                wrap: false,
                align: og_text::layout::Alignment::Start,
                ..Default::default()
            };
            let mut atlas = self.atlas.lock().unwrap();
            for _ in 0..8 {
                if og_text::strike::text_glyphs(
                    &mut l.strikes,
                    &mut l.fonts,
                    &mut atlas,
                    &mut l.layout,
                    s,
                    &style,
                    0.0,
                    [1.0; 3],
                    &mut glyphs,
                ) {
                    break;
                }
            }
            // The glyphs' advances, in the laid-out order. The layout of a
            // one-line string is what the whole call laid.
            let mut advances = Vec::new();
            l.layout.runs(&l.fonts, |run| {
                advances
                    .extend(run.glyphs.iter().filter(|g| g.id != 0).map(|g| g.advance));
            });
            let em = style.size.max(1e-3);
            width = l.layout.width() / em;
            if glyphs.len() != advances.len() {
                return Err(format!("mass string {s:?}: the glyph atlas has no room"));
            }
            Ok(UiMassString {
                glyphs: glyphs
                    .iter()
                    .zip(advances)
                    .map(|(g, a)| UiMassGlyph {
                        glyph: *g,
                        advance: a / em,
                    })
                    .collect(),
                width,
            })
        })
    }

    /// A mass template's number symbols: one per shader symbol code
    /// ([`og_draw::ui::mass::NUMBER_SYMBOLS`]), each laid alone (its quad
    /// from its own origin, the shader places it by its advance) and
    /// rastered into the atlas.
    pub fn mass_symbols(&self) -> Result<Vec<UiMassGlyph>, String> {
        LAYOUTS.with(|l| {
            let mut l = l.borrow_mut();
            let l = l.as_mut().expect("loaded by load_font");
            let style = Style {
                family: FAMILY,
                size: og_render::glyphs::FIELD_EM_PX,
                wrap: false,
                align: og_text::layout::Alignment::Start,
                ..Default::default()
            };
            let mut atlas = self.atlas.lock().unwrap();
            NUMBER_SYMBOLS
                .iter()
                .map(|ch| {
                    let mut glyphs = Vec::new();
                    for _ in 0..8 {
                        if og_text::strike::text_glyphs(
                            &mut l.strikes,
                            &mut l.fonts,
                            &mut atlas,
                            &mut l.layout,
                            &ch.to_string(),
                            &style,
                            0.0,
                            [1.0; 3],
                            &mut glyphs,
                        ) {
                            break;
                        }
                    }
                    // The pen's move past the one glyph: its string's width.
                    let advance = l.layout.width() / style.size;
                    Ok(UiMassGlyph {
                        glyph: glyphs.first().cloned().unwrap_or_default(),
                        advance,
                    })
                })
                .collect()
        })
    }
}

/// The test font for `scene`'s glyph atlas.
pub fn load_font(scene: &mut RenderScene) -> Result<TestFont, String> {
    LAYOUTS.with(|l| -> Result<(), String> {
        let mut l = l.borrow_mut();
        if l.is_none() {
            let mut fonts = Collection::new();
            for face in faces()? {
                fonts.add(std::sync::Arc::new(face))?;
            }
            *l = Some(Layouts {
                fonts,
                strikes: Strikes::new(),
                layout: Layout::new(),
            });
        }
        Ok(())
    })?;
    Ok(TestFont {
        atlas: scene.ui.glyphs.clone(),
    })
}

/// The non-whitespace characters of the text scene, for glyph counts.
pub const TEXT_LINES: &[(&str, f32, f32, f32, [f32; 4])] = &[
    // (text, x, y, em size, colour)
    ("Origin Engine", -9.2, 3.2, 2.4, [1.0, 1.0, 1.0, 1.0]),
    ("原点引擎 文字渲染", -9.2, 0.4, 1.5, [1.0, 0.8, 0.3, 1.0]),
    (
        "Field text: sharp at any size. 0123456789 !?@#&\nThe quick brown fox jumps over the lazy dog.",
        -9.2,
        -1.6,
        0.55,
        [0.6, 0.85, 1.0, 1.0],
    ),
    (
        "tiny 12px-ish text stays legible",
        -9.2,
        -4.2,
        0.32,
        [0.8, 0.8, 0.8, 1.0],
    ),
];

pub(super) fn text_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let font = load_font(&mut scene)?;
    for (text, x, y, size, color) in TEXT_LINES {
        scene.sprite.texts.push(Text {
            position: Vec3::new(*x, *y, 0.0),
            size: *size,
            color: *color,
            ..font.text(text)
        });
    }
    scene.sprite.texts.push(Text {
        position: Vec3::new(1.0, -4.5, 0.0),
        size: 0.9,
        rotation: 0.35,
        color: [1.0, 0.4, 0.5, 1.0],
        layer: 1,
        ..font.text("rotated")
    });
    // A huge glyph: the distance field is magnified ~10x.
    scene.sprite.texts.push(Text {
        position: Vec3::new(6.0, -3.0, 0.0),
        size: 4.5,
        color: [0.35, 0.9, 0.5, 1.0],
        ..font.text("&")
    });
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 10.5);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.015, 0.02, 0.035));
    scene.look.post.tonemap = Tonemap::None;
    Ok(scene)
}
