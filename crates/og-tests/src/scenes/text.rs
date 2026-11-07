//! MSDF font loading and the text scene.

use super::*;

/// The committed MSDF test atlas (`assets/fonts/noto-sans-sc-msdf.*`, see
/// `crates/og-render/README.md` §Testing): the atlas PNG plus the glyph CSV
/// and metrics JSON `msdf-atlas-gen` wrote with `-yorigin top`.
pub fn load_font(scene: &mut RenderScene) -> Result<og_render::FontHandle, String> {
    let png = "fonts/noto-sans-sc-msdf.png";
    let img = image::load_from_memory(&asset(png)?)
        .map_err(|e| format!("{png}: {e}"))?
        .to_rgba8();
    let (width, height) = img.dimensions();
    let atlas = scene.add_texture(TextureData {
        width,
        height,
        rgba: img.into_raw(),
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    let json_path = "fonts/noto-sans-sc-msdf.json";
    let json = asset_text(json_path)?;
    let number = |key: &str| -> Result<f32, String> {
        let at = json
            .find(&format!("\"{key}\":"))
            .ok_or_else(|| format!("{json_path}: no {key}"))?;
        let rest = &json[at + key.len() + 3..];
        let end = rest.find([',', '}']).unwrap_or(rest.len());
        rest[..end]
            .trim()
            .parse()
            .map_err(|e| format!("{json_path}: {key}: {e}"))
    };
    let csv_path = "fonts/noto-sans-sc-msdf.csv";
    let csv = asset_text(csv_path)?;
    let mut glyphs = Vec::new();
    for line in csv.lines().filter(|l| !l.trim().is_empty()) {
        let f: Vec<f32> = line
            .split(',')
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<_, _>>()
            .map_err(|e| format!("{csv_path}: {line}: {e}"))?;
        if f.len() != 10 {
            return Err(format!("{csv_path}: {line}: 10 columns expected"));
        }
        let codepoint = char::from_u32(f[0] as u32)
            .ok_or_else(|| format!("{csv_path}: bad codepoint {}", f[0]))?;
        // msdf-atlas-gen with -yorigin top: plane bounds left, top, right,
        // bottom with y down; atlas bounds left, top, right, bottom in pixels.
        glyphs.push(Glyph {
            key: codepoint.into(),
            advance: f[1],
            plane_bounds: [f[2], -f[5], f[4], -f[3]],
            atlas_bounds: [f[6], f[7], f[8], f[9]],
        });
    }
    Ok(scene.add_font(MsdfFont {
        atlas,
        distance_range: number("distanceRange")?,
        foreground: false,
        silhouette: false,
        line_height: number("lineHeight")?,
        glyphs,
        kerning: Vec::new(),
    }))
}

/// The non-whitespace characters of the text scene, for glyph counts.
pub const TEXT_LINES: &[(&str, f32, f32, f32, [f32; 4])] = &[
    // (text, x, y, em size, colour)
    ("Origin Engine", -9.2, 3.2, 2.4, [1.0, 1.0, 1.0, 1.0]),
    ("原点引擎 文字渲染", -9.2, 0.4, 1.5, [1.0, 0.8, 0.3, 1.0]),
    (
        "MSDF text: sharp at any size. 0123456789 !?@#&\nThe quick brown fox jumps over the lazy dog.",
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
        scene.texts.push(Text {
            position: Vec3::new(*x, *y, 0.0),
            size: *size,
            color: *color,
            ..Text::new(font, *text)
        });
    }
    scene.texts.push(Text {
        position: Vec3::new(1.0, -4.5, 0.0),
        size: 0.9,
        rotation: 0.35,
        color: [1.0, 0.4, 0.5, 1.0],
        layer: 1,
        ..Text::new(font, "rotated")
    });
    // A huge glyph: the distance field is magnified ~10x.
    scene.texts.push(Text {
        position: Vec3::new(6.0, -3.0, 0.0),
        size: 4.5,
        color: [0.35, 0.9, 0.5, 1.0],
        ..Text::new(font, "&")
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.5);
    scene.environment.background = Background::Color(Vec3::new(0.015, 0.02, 0.035));
    scene.post.tonemap = Tonemap::None;
    Ok(scene)
}
