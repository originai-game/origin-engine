use super::*;
use crate::host::wire;

/// The UiDesc a configure call makes: the last one's fields with the ones
/// this call gives (`null` clears a field; one left out keeps its value).
fn merged_ui_desc(last: &Value, given: Value) -> Result<Value, String> {
    let mut d = match last {
        Value::Object(o) => o.clone(),
        _ => Map::new(),
    };
    match given {
        Value::Object(o) => d.extend(o),
        Value::Null => {}
        other => return Err(format!("desc: a UiDesc is an object, not {other}")),
    }
    Ok(Value::Object(d))
}

/// The UI settings in effect, as og.ui.theme answers them: the configured
/// UiDesc's (og.ui.configure), with the page's reduced-motion and contrast
/// preferences (the host's viewport) over them. Made from the desc each
/// time it is asked: nothing but the desc holds them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiSettings {
    pub dark: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub text_scale: f32,
    /// The UI unit in CSS px (UiDesc.unitScale, else auto by the surface).
    pub unit: f32,
    pub color_vision: wire::ColorVision,
    pub color_vision_filter: wire::ColorVisionFilter,
    pub color_vision_strength: f32,
    /// The configured style's id; empty for the default style.
    pub style: String,
}

impl UiSettings {
    /// `desc` (a UiDesc, null for none) under a page that prefers reduced
    /// motion or more contrast when `page` says so.
    pub(crate) fn of(desc: &Value, style: String, page: (bool, bool)) -> UiSettings {
        let u: wire::UiDesc = wire::value(desc, "desc").unwrap_or_default();
        UiSettings {
            dark: u.scheme == wire::Scheme::Dark,
            high_contrast: u.contrast == wire::Contrast::High || page.1,
            reduced_motion: u.reduced_motion || page.0,
            text_scale: u.text_scale.clamp(1.0, 1.5),
            unit: 1.0,
            color_vision: u.color_vision,
            color_vision_filter: u.color_vision_filter,
            color_vision_strength: u.color_vision_strength.clamp(0.0, 1.0),
            style,
        }
    }

    /// The colour vision filter over the frame; none for typical vision.
    pub(crate) fn color_filter(&self) -> Option<og_render::ColorFilter> {
        og_render::ColorFilter::new(
            og_render::ColorVision::from_name(self.color_vision.as_str()).unwrap_or_default(),
            match self.color_vision_filter {
                wire::ColorVisionFilter::Simulate => og_render::ColorVisionFilter::Simulate,
                wire::ColorVisionFilter::Correct => og_render::ColorVisionFilter::Correct,
            },
            self.color_vision_strength,
        )
    }
}

/// Engine `eng`'s UI settings in effect, under the page preferences of
/// viewport `v`, with the UI unit `unit` laid out in.
pub(crate) fn settings(eng: &Engine, v: &Viewport, unit: f32) -> UiSettings {
    let style = crate::present::ui_next::style_id(eng.id).unwrap_or_default();
    UiSettings {
        unit,
        ..UiSettings::of(
            &eng.state.borrow().ui_desc,
            style,
            (v.reduced_motion, v.high_contrast),
        )
    }
}

pub fn og_ui_configure(world: u32, desc: Option<String>) -> Result<(), String> {
    let answer = (|| -> Result<_, String> {
        let eng = engine(world)?;
        let d = merged_ui_desc(&eng.state.borrow().ui_desc, parse(desc.as_deref())?)?;
        // The merged desc (patch semantics are the merge's), read whole.
        let u: wire::UiDesc = wire::value(&d, "desc")?;
        // A game's atlas (`icons`), else the style's; under either, the
        // set that draws the names it lacks (`fallbackIcons`).
        let atlas_of = |field: &str,
                        icons: &Option<wire::IconAtlasDesc>|
         -> Result<Option<IconAtlas>, String> {
            let Some(icons) = icons else {
                return Ok(None);
            };
            let texture = match icons.texture.map(|t| t as f64) {
                Some(t) => {
                    let rs = eng.render.borrow();
                    let i = super::assets::index_of(t)
                        .ok_or_else(|| format!("{field}.texture: {t} is not a texture"))?;
                    Some(
                        *rs.textures
                            .get(i)
                            .ok_or_else(|| format!("{field}.texture: texture {t} is not alive"))?,
                    )
                }
                None => None,
            };
            // An icon set (an `.ogt` of `type icons`, loaded as a font):
            // distance-field glyphs, named by the set.
            let set = match icons.font.map(|h| h as f64) {
                Some(h) => {
                    let i = super::assets::index_of(h)
                        .ok_or_else(|| format!("{field}.font: {h} is not a font"))?;
                    let names = eng
                        .assets
                        .borrow()
                        .fonts
                        .get(i)
                        .map(|f| f.icons.clone())
                        .ok_or_else(|| format!("{field}.font: font {h} is not alive"))?;
                    if names.is_empty() {
                        return Err(format!(
                            "{field}.font: font {h} is not an icon set (cook SVGs with an .ogt of `type icons`)"
                        ));
                    }
                    Some((
                        eng.assets
                            .borrow()
                            .fonts
                            .get(i)
                            .and_then(|f| f.face.clone()),
                        names,
                    ))
                }
                None => None,
            };
            Ok(Some(match set {
                Some((face, names)) => IconAtlas {
                    face,
                    names,
                    ..IconAtlas::default()
                },
                None => IconAtlas {
                    texture,
                    columns: icons.columns.max(1),
                    rows: icons.rows.max(1),
                    names: icons.names.clone(),
                    pictures: icons.pictures,
                    ..IconAtlas::default()
                },
            }))
        };
        let mut atlas = atlas_of("icons", &u.icons)?.unwrap_or_default();
        let mut rs = eng.render.borrow_mut();
        // The pack's icon sets stay under the game's (`pack_images`).
        atlas.under = rs.ui_icons.under.take();
        rs.ui_icons = atlas;
        drop(rs);
        eng.render.borrow_mut().color_filter =
            UiSettings::of(&d, String::new(), (false, false)).color_filter();
        let mut st = eng.state.borrow_mut();
        st.ui_desc = d;
        // i18n::apply draws a role nothing backs in the default face.
        let fontless = u.font.is_none()
            && u.display_font.is_none()
            && u.mono_font.is_none()
            && u.rounded_font.is_none()
            && u.fallback_fonts.is_empty()
            && u.style.is_none();
        let default_available = eng
            .assets
            .borrow()
            .bundle_for(og_ui_next::pack::DEFAULT_FONT_ASSET)
            .is_ok_and(|(bundle, _)| bundle.output(og_ui_next::pack::DEFAULT_FONT_ASSET).is_ok());
        if fontless && !default_available {
            let text = "og.ui.configure: the UI has no font now, so it draws no text (every label, button and field is blank): give `font` (og.load.font) or a `style` (og.load.style, whose fonts it brings), in this call or an earlier one; a field set to null is cleared, a field left out keeps its value".to_string();
            if st.ui_warned.insert(text.clone()) {
                let tick = eng.world().map_or(0, |w| w.time().tick);
                st.log
                    .push(json!({ "tick": tick, "warning": true, "text": text }));
            }
        }
        drop(st);
        crate::bind::i18n::apply(&eng)?;
        Ok(())
    })();
    match answer {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

/// A loaded style directory and the font files it ships.
pub(crate) struct LoadedStyle {
    pub id: String,
    /// Its `fonts`: the font files it ships, first first. Each is loaded
    /// once it is fetched (the web reads them after the boot,
    /// `bind::i18n::apply`).
    pub files: Vec<String>,
}

/// `og.load.style`: a cooked UI style, its atlas, fonts, icons and sounds loaded with it.
pub fn og_load_style(world: u32, path: String) -> Result<serde_json::Value, String> {
    match engine(world).and_then(|eng| {
        let i = load_style(&eng, &path)?;
        docs::note_styles(&eng);
        Ok(i)
    }) {
        Ok(i) => Ok(json!(super::assets::handle(i) as f64)),
        Err(e) => Err(e),
    }
}

/// `og.load.style`: the style's load index, its atlas, fonts, icons and
/// sounds loaded with it (once per path).
pub(crate) fn load_style(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path)
        .trim_end_matches('/')
        .to_string();
    if let Some(i) = eng.assets.borrow().styles.iter().position(|s| s.id == id) {
        return Ok(i);
    }
    let style = read_style(eng, &id)?;
    let mut a = eng.assets.borrow_mut();
    a.styles.push(style);
    Ok(a.styles.len() - 1)
}

pub(super) fn read_style(eng: &Engine, id: &str) -> Result<LoadedStyle, String> {
    if id.ends_with(".ogt") {
        return Err("style.ogt is removed; og.load.style takes the directory containing the pack's pack.json".into());
    }
    // A pack's pack.json names its faces (`fonts`); the families its
    // text roles take (`theme.fontFamily`) went with the TSX font state.
    let blob = format!("{id}/pack.json");
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets
            .blob(&blob)
            .map_err(|e| format!("{id}: a style pack is a directory with a pack.json: {e}"))?;
        bundle
            .pack_text(&blob, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let layer = og_ui_next::pack::Layer::parse(&text).map_err(|e| format!("{blob}: {e}"))?;
    Ok(LoadedStyle {
        id: id.to_owned(),
        files: layer.fonts,
    })
}
