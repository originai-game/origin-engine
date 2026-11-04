//! The art contract at run time (docs/art.md §2.4): what a look's art
//! resources (`ArtLine`, `ArtPixels`, `ArtPalette`, `ArtMotion`,
//! `ArtCamera`) default on the frame and on the sprites, once the project
//! names its look in project.ogt (`look "looks/<id>.ogt"`, the same line
//! the cook conforms its assets by). Without that line nothing here
//! applies, so a game that only applies looks (GameDesc.look, applyLook)
//! draws as it did before the contract.
//!
//! - `ArtPixels.rotation cleanEdge`: pixel-art textures (those sampled
//!   nearest) draw through og-render's `TextureFilter::CleanEdge`.
//! - `ArtLine.width`: the default of `Sprite.outline` for a sprite whose
//!   texture the cook did not line, where the contract sets
//!   `ArtPixels.unit` (a logical pixel to measure the line in); and the
//!   meshes' outline (`MaterialStyle.outlineWidth`, its colour, its
//!   distance falloff and `outlinePlace` from `ArtLine.place`) where the
//!   frame's MaterialStyle sets no outline of its own ([`mesh_line`]).
//! - `ArtPalette.colors` with `strict`: the UI's icons tinted to the
//!   nearest palette colour (an icon atlas is a distance field, which the
//!   cook does not conform).
//! - `ArtCamera`: the camera's projection, angle and field of view written
//!   when a look is applied ([`apply_camera`]); `pixelPerfect` snaps a 2D
//!   camera by whole logical pixels (`fx::snap_to_grid`).
//! - `ArtMotion.stepFps`: og-anim's default step ([`step_default`]).
//!
//! Everything but the camera and the step is extraction: it reads the
//! world and never writes it.

use super::codec::{Layout, RecordKind};
use super::engine::Engine;
use super::extract::RenderState;
use super::fx::{color, f, handle_index, on, resource, u, variant};
use glam::{Quat, Vec3};
use og_ecs::World;
use og_render::{TextureFilter, TextureHandle};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// What this frame takes from the contract (all zero: nothing).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ArtFrame {
    /// `ArtLine.width` and `color`, where `ArtPixels.unit` is set.
    pub line: Option<(f32, [f32; 3])>,
    /// `ArtLine` round meshes: its width, colour, place and distance.
    pub mesh_line: Option<MeshLine>,
    /// `ArtPixels.unit`: logical pixels per world unit (0 none).
    pub unit: f32,
    /// `ArtPixels.logicalHeight`.
    pub logical_height: u32,
    /// `ArtPixels.rotation cleanEdge`.
    pub clean_edge: bool,
    /// `ArtCamera.pixelPerfect`.
    pub pixel_perfect: bool,
    /// `ArtPalette.colors` when `strict` (sRGB).
    pub strict: Vec<[u8; 3]>,
}

/// `ArtLine` as the meshes' outline takes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshLine {
    pub width: f32,
    pub color: [f32; 3],
    pub place: og_render::OutlinePlace,
    pub distance: f32,
}

/// The meshes' outline under the contract: `ArtLine` fills the frame's
/// material style where the style sets no outline width of its own (a
/// look's MaterialStyle outline, or a game's, wins whole: its width, colour,
/// distance and place).
pub(crate) fn mesh_line(art: &ArtFrame, style: &mut og_render::MaterialStyle) {
    let Some(l) = art.mesh_line else { return };
    if style.outline_width > 0.0 {
        return;
    }
    style.outline_width = l.width.min(31.0);
    style.outline_color = l.color;
    style.outline_distance = l.distance;
    style.outline_place = l.place;
}

/// The contract's drawing state, kept on the render state.
#[derive(Default)]
pub(crate) struct ArtState {
    /// The project names its look: the defaults apply.
    pub contract: bool,
    pub frame: ArtFrame,
    /// A texture's index texture (the cook's `<id>#index`), by the render
    /// handle of the texture; an index texture loaded by its own id maps
    /// to itself.
    pub index_of: HashMap<TextureHandle, TextureHandle>,
    /// Textures drawn with a palette but without an index texture, each
    /// warned of once.
    pub warned: BTreeSet<TextureHandle>,
}

/// Whether the engine's project names its look (project.ogt's `look`).
pub(crate) fn contract(eng: &Engine) -> bool {
    eng.state
        .borrow()
        .project
        .as_ref()
        .is_some_and(|p| p.look.is_some())
}

/// og-anim's default step under the contract: `ArtMotion.stepFps`, read by
/// og.anim every tick (none without the contract).
pub(crate) fn step_default(
    contract: bool,
    layouts: &BTreeMap<String, Layout>,
) -> Option<og_anim::StepDefault> {
    if !contract {
        return None;
    }
    let (_, offset) = layouts.get("og.ArtMotion")?.slot("stepFps")?;
    Some(og_anim::StepDefault {
        resource: "og.ArtMotion".into(),
        offset,
    })
}

/// Reads this frame's contract from the world's art resources and sets the
/// pixel-art textures' filter by it (`ArtPixels.rotation`).
pub(crate) fn frame(world: &World, rs: &mut RenderState) {
    let frame = if rs.art.contract {
        read(world, rs)
    } else {
        ArtFrame::default()
    };
    // Pixel-art textures (sampled nearest) through cleanEdge while the
    // contract asks for it, back to nearest when it no longer does.
    let scene = &mut rs.scene;
    let filters = rs
        .textures
        .iter()
        .chain(rs.art.index_of.values())
        .copied()
        .collect::<Vec<_>>();
    for t in filters {
        match (scene.texture_filter(t), frame.clean_edge) {
            (TextureFilter::Nearest, true) => {
                scene.set_texture_filter(t, TextureFilter::CleanEdge);
            }
            (TextureFilter::CleanEdge, false) => {
                scene.set_texture_filter(t, TextureFilter::Nearest);
            }
            _ => {}
        }
    }
    rs.art.frame = frame;
}

fn read(world: &World, rs: &RenderState) -> ArtFrame {
    let l = &rs.draw.fx;
    let pixels = resource(world, l.art_pixels.as_ref()).unwrap_or_default();
    let line = resource(world, l.art_line.as_ref()).unwrap_or_default();
    let camera = resource(world, l.art_camera.as_ref()).unwrap_or_default();
    let palette = resource(world, l.art_palette.as_ref()).unwrap_or_default();
    let unit = f(&pixels, "unit", 0.0).max(0.0);
    let width = f(&line, "width", 0.0).max(0.0);
    let strict = if on(&palette, "strict", false) {
        handle_index(&palette["colors"])
            .and_then(|i| rs.fx.look.palettes.get(i)?.1)
            .and_then(|h| rs.scene.palette(h))
            .map(|p| p.colors.clone())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let line_color = rgb(&line["color"], Vec3::from(OUTLINE_COLOR)).to_array();
    ArtFrame {
        line: (width > 0.0 && unit > 0.0).then_some((width, line_color)),
        mesh_line: (width > 0.0).then(|| MeshLine {
            width,
            color: line_color,
            // ArtLinePlace's order: outside (its zero value), inside, both.
            place: match variant(&line["place"], &["outside", "inside", "both"]) {
                1 => og_render::OutlinePlace::Inside,
                2 => og_render::OutlinePlace::Both,
                _ => og_render::OutlinePlace::Outside,
            },
            distance: f(&line, "distanceFalloff", 0.0).max(0.0),
        }),
        unit,
        logical_height: u(&pixels, "logicalHeight", 0),
        clean_edge: variant(
            &pixels["rotation"],
            &["free", "nearest", "cleanEdge", "prerotated"],
        ) == 2,
        pixel_perfect: on(&camera, "pixelPerfect", false),
        strict,
    }
}

/// A colour field as decoded: `{r, g, b}` or `[r, g, b]`.
fn rgb(v: &Value, default: Vec3) -> Vec3 {
    match v.as_array() {
        Some(a) if a.len() >= 3 => {
            let at = |i: usize| a[i].as_f64().map_or(0.0, |x| x as f32);
            Vec3::new(at(0), at(1), at(2))
        }
        _ => color(v, default),
    }
}

/// `Sprite.outline`'s default colour: a sprite at it takes the contract's.
pub(crate) const OUTLINE_COLOR: [f32; 3] = [0.02, 0.02, 0.03];

/// A world sprite's pixel outline (`Sprite.outline`, `outlineColor`): its
/// own width, else the contract's line for a texture the cook did not line
/// (one not sampled as pixel art); a logical pixel is 1 / `ArtPixels.unit`
/// world units under the contract, else one texel. `region` is the
/// sprite's uv region, `size` its drawn size in world units.
pub(crate) fn outline(
    art: &ArtFrame,
    scene: &og_render::RenderScene,
    texture: Option<TextureHandle>,
    own: (f32, [f32; 3]),
    region: [f32; 4],
    size: glam::Vec2,
) -> Option<(f32, f32, [f32; 3])> {
    let t = texture?;
    let (mut width, mut rgb) = own;
    if width <= 0.0 {
        let (w, c) = art.line?;
        if scene.texture_filter(t).is_nearest() {
            return None;
        }
        width = w;
        if rgb == OUTLINE_COLOR {
            rgb = c;
        }
    } else if rgb == OUTLINE_COLOR
        && let Some((_, c)) = art.line
    {
        rgb = c;
    }
    let pixel = if art.unit > 0.0 && size.x.abs() > 0.0 {
        let (tw, _) = scene.texture_size(t)?;
        let texels = (region[2] - region[0]).abs() * tw as f32;
        texels / (size.x.abs() * art.unit)
    } else {
        1.0
    };
    Some((width, pixel, rgb))
}

/// The texture a sprite or tile layer drawing `texture` shows through
/// palette `palette`'s row `row` (`Sprite.palette`, `Tilemap.palette`): its
/// index texture through the row, or `texture` itself, warned of once,
/// when it has none.
pub(crate) fn paletted(
    scene: &mut og_render::RenderScene,
    (art, notes): (&mut ArtState, &mut Vec<String>),
    texture: TextureHandle,
    palette: TextureHandle,
    row: u32,
) -> TextureHandle {
    let Some(&index) = art.index_of.get(&texture) else {
        if art.warned.insert(texture) {
            notes.push(format!(
                "palette swap: texture {} has no index texture (the cook writes `<id>#index` beside a texture the art contract maps to a palette): drawn as it is",
                texture.index()
            ));
        }
        return texture;
    };
    match scene.paletted_texture(index, palette, row) {
        Ok(t) => t,
        Err(e) => {
            if art.warned.insert(texture) {
                notes.push(format!("{e}: drawn without its palette"));
            }
            texture
        }
    }
}

/// The UI's icons on a strict palette: each icon's colour moved to the
/// palette's nearest (by Oklab distance), over the overlay's sprites from
/// `sprites` on and texts from `texts` on.
pub(crate) fn palette_icons(
    art: &ArtFrame,
    overlay: &mut og_render::Overlay,
    (sprites, texts, shapes): (usize, usize, usize),
    icons: (&[TextureHandle], &[og_render::FontHandle]),
) {
    if art.strict.is_empty() {
        return;
    }
    let to = |c: &mut [f32; 4]| {
        let rgb = nearest(&art.strict, [c[0], c[1], c[2]]);
        c[..3].copy_from_slice(&rgb);
    };
    for s in &mut overlay.sprites[sprites..] {
        if s.texture.is_some_and(|t| icons.0.contains(&t)) {
            to(&mut s.color);
        }
    }
    for t in &mut overlay.texts[texts..] {
        if icons.1.contains(&t.font) {
            to(&mut t.color);
        }
    }
    for s in &mut overlay.ui.shapes[shapes..] {
        if let og_render::UiPaint::Image(im) = &mut s.paint
            && icons.0.contains(&im.texture)
        {
            to(&mut im.tint);
        }
    }
}

/// The palette colour (sRGB bytes) nearest a linear colour, by Oklab
/// distance, as a linear colour.
fn nearest(palette: &[[u8; 3]], linear: [f32; 3]) -> [f32; 3] {
    let target = oklab(linear);
    let mut best = (f32::MAX, linear);
    for c in palette {
        let lin = c.map(|v| srgb_to_linear(v as f32 / 255.0));
        let p = oklab(lin);
        let d = (0..3)
            .map(|i| (p[i] - target[i]) * (p[i] - target[i]))
            .sum::<f32>();
        if d < best.0 {
            best = (d, lin);
        }
    }
    best.1
}

// Rendering only: the UI's icon tint, never read by the tick.
#[allow(clippy::disallowed_methods)]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Björn Ottosson's Oklab of a linear sRGB colour.
// Rendering only: the UI's icon tint, never read by the tick.
#[allow(clippy::disallowed_methods)]
fn oklab(c: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = c;
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// A look's `ArtCamera` on the main camera, as it is applied under the
/// contract: `projection` sets og.Camera.orthographic, `fixedAngle` its
/// rotation from `pitch` (below the horizon) and `yaw` (about +Y), and
/// `fovMin`/`fovMax` clamp its field of view. Fields at zero leave the
/// camera's own. Inside a system that did not declare og.Camera among its
/// writes, the camera is left as it is (as an undeclared art resource is).
pub(crate) fn apply_camera(eng: &Engine, look: &Map<String, Value>) -> Result<(), String> {
    if !contract(eng) {
        return Ok(());
    }
    if let Some(f) = super::engine::running(eng.id)
        && !eng.state.borrow().systems[f.system]
            .writes
            .iter()
            .any(|w| w == "og.Camera")
    {
        return Ok(());
    }
    let art = Value::Object(look.clone());
    let mut set = Map::new();
    match variant(&art["projection"], &["any", "perspective", "orthographic"]) {
        1 => {
            set.insert("orthographic".into(), json!(false));
        }
        2 => {
            set.insert("orthographic".into(), json!(true));
        }
        _ => {}
    }
    if on(&art, "fixedAngle", false) {
        let q = angle(f(&art, "pitch", 0.0), f(&art, "yaw", 0.0));
        set.insert(
            "quaternion".into(),
            json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w }),
        );
    }
    let (lo, hi) = (f(&art, "fovMin", 0.0), f(&art, "fovMax", 0.0));
    if lo > 0.0 || hi > 0.0 {
        let fov = camera_fov(eng).unwrap_or(50.0);
        let mut v = fov;
        if lo > 0.0 {
            v = v.max(lo);
        }
        if hi > 0.0 {
            v = v.min(hi);
        }
        if v != fov {
            set.insert("fov".into(), json!(v));
        }
    }
    if set.is_empty() {
        return Ok(());
    }
    super::engine::write_resource_value(eng.id, "og.Camera", &Value::Object(set))
        .map_err(|e| format!("ArtCamera: {e}"))
}

/// The camera's rotation looking `pitch` degrees below the horizon, turned
/// `yaw` degrees about +Y (the camera looks down its local -Z).
// Rendering setup, not the tick: applied with a look.
#[allow(clippy::disallowed_methods)]
pub(crate) fn angle(pitch: f32, yaw: f32) -> Quat {
    Quat::from_rotation_y(yaw.to_radians()) * Quat::from_rotation_x(-pitch.to_radians())
}

/// og.tween.squash's default impact under the contract's
/// `ArtMotion.squash` (`amount`, 0 to 1; 0: no constraint, the preset's
/// own (1.25, 0.8)): the preset's squash scaled so 0.5 is the preset and
/// 1 rubber hose, (1.5, 0.6); z stays 0 (at rest).
pub(crate) fn squash_impact(amount: f32) -> Vec3 {
    let base = Vec3::new(1.25, 0.8, 0.0);
    if amount <= 0.0 {
        return base;
    }
    let k = amount.min(1.0) / 0.5;
    Vec3::new(1.0 + (base.x - 1.0) * k, 1.0 + (base.y - 1.0) * k, 0.0)
}

/// The contract's `ArtMotion.squash` (0 without the contract or the
/// resource), read as the camera's field of view is: from the running
/// system's frame or the world, whether or not the system declared it
/// (a default, never a value the tick depends on beyond the call).
pub(crate) fn squash_amount(eng: &Engine) -> f32 {
    if !contract(eng) {
        return 0.0;
    }
    resource_field(eng, "og.ArtMotion", "squash", 0.0)
}

fn camera_fov(eng: &Engine) -> Option<f32> {
    Some(resource_field(eng, "og.Camera", "fov", 50.0))
}

/// A resource's number field, `default` when the resource is absent.
fn resource_field(eng: &Engine, name: &str, field: &str, default: f32) -> f32 {
    resource_json(eng, name).map_or(default, |v| f(&v, field, default))
}

fn resource_json(eng: &Engine, name: &str) -> Option<Value> {
    let st = eng.state.borrow();
    let layout = st
        .layouts
        .get(name)
        .filter(|l| l.kind == RecordKind::Resource)?
        .clone();
    let id = st
        .names_by_id
        .iter()
        .position(|n| n == name)
        .map(|i| og_ecs::SchemaId::from_index(i as u32))?;
    drop(st);
    let bytes = match super::engine::running(eng.id) {
        Some(fr) => {
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { fr.ctx() };
            ctx.resource_bytes(id).ok()?
        }
        None => eng.world().ok()?.resource_bytes(id).ok()?,
    };
    Some(layout.decode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_strict_palette_takes_the_nearest_colour_by_oklab() {
        // A red heart on the handheld greens: by lightness alone it would
        // take the mid green (48, 98, 48); with its chroma counted, the
        // yellow-green (139, 172, 15) is nearer (worked out in Oklab in
        // Python: 0.084 against 0.099 squared). White the lightest, black the
        // darkest.
        let greens = [[15, 56, 15], [48, 98, 48], [139, 172, 15], [155, 188, 15]];
        let lin = |c: [u8; 3]| c.map(|v| srgb_to_linear(v as f32 / 255.0));
        let red = nearest(&greens, [0.8, 0.05, 0.05]);
        assert_eq!(red, lin([139, 172, 15]));
        assert_eq!(nearest(&greens, [1.0, 1.0, 1.0]), lin([155, 188, 15]));
        assert_eq!(nearest(&greens, [0.0, 0.0, 0.0]), lin([15, 56, 15]));
    }

    /// ArtLine lines the meshes where the frame's MaterialStyle sets no
    /// outline: its width, colour, falloff and place (outside, ArtLine's
    /// zero value, is not the style's inside); a style's own outline wins
    /// whole, its place included.
    #[test]
    fn the_contract_s_line_outlines_meshes_where_the_style_does_not() {
        let line = MeshLine {
            width: 2.5,
            color: [0.1, 0.0, 0.2],
            place: og_render::OutlinePlace::Outside,
            distance: 12.0,
        };
        let art = ArtFrame {
            mesh_line: Some(line),
            ..ArtFrame::default()
        };
        let mut style = og_render::MaterialStyle::default();
        mesh_line(&art, &mut style);
        assert_eq!(style.outline_width, 2.5);
        assert_eq!(style.outline_color, [0.1, 0.0, 0.2]);
        assert_eq!(style.outline_distance, 12.0);
        assert_eq!(style.outline_place, og_render::OutlinePlace::Outside);
        let mut own = og_render::MaterialStyle {
            outline_width: 1.0,
            ..Default::default()
        };
        mesh_line(&art, &mut own);
        assert_eq!(own.outline_width, 1.0);
        assert_eq!(own.outline_place, og_render::OutlinePlace::Inside);
        let mut none = og_render::MaterialStyle::default();
        mesh_line(&ArtFrame::default(), &mut none);
        assert_eq!(none, og_render::MaterialStyle::default());
    }

    /// ArtMotion.squash sets og.tween.squash's default impact: 0 is the
    /// preset's own, 0.5 the same, 0.8 more (1.4, 0.68), 1 rubber hose.
    #[test]
    fn the_contract_s_squash_scales_the_default_impact() {
        let close = |a: Vec3, b: Vec3| (a - b).length() < 1e-5;
        assert!(close(squash_impact(0.0), Vec3::new(1.25, 0.8, 0.0)));
        assert!(close(squash_impact(0.5), Vec3::new(1.25, 0.8, 0.0)));
        assert!(close(squash_impact(0.8), Vec3::new(1.4, 0.68, 0.0)));
        assert!(close(squash_impact(1.0), Vec3::new(1.5, 0.6, 0.0)));
        assert!(close(squash_impact(0.1), Vec3::new(1.05, 0.96, 0.0)));
    }

    #[test]
    fn the_isometric_angle_looks_down_and_across() {
        // 30 degrees below the horizon, turned 45: the view (-Z) points
        // down by sin 30 and equally along -X and -Z.
        let d = angle(30.0, 45.0) * Vec3::NEG_Z;
        assert!((d.y + 0.5).abs() < 1e-5, "{d}");
        assert!((d.x - d.z).abs() < 1e-5 && d.x < 0.0, "{d}");
        // Zero is the camera's rest: looking down -Z.
        assert!((angle(0.0, 0.0) * Vec3::NEG_Z - Vec3::NEG_Z).length() < 1e-6);
    }
}
