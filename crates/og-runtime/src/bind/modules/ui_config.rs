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
        let style = match u.style.map(|h| h as f64) {
            Some(h) => {
                let i = super::assets::index_of(h)
                    .ok_or_else(|| format!("style: {h} is not a UI style"))?;
                let a = eng.assets.borrow();
                let s = a
                    .styles
                    .get(i)
                    .ok_or_else(|| format!("style: style {h} is not alive"))?;
                Some((s.faces.clone(), s.roles.clone()))
            }
            None => None,
        };
        let font = |h: Option<u64>| -> Result<Option<UiFont>, String> {
            h.map(|h| ui_font(&eng, h as f64)).transpose()
        };
        let fonts = Fonts {
            body: font(u.font)?.unwrap_or_default(),
            display: font(u.display_font)?,
            mono: font(u.mono_font)?,
            rounded: font(u.rounded_font)?,
            fallbacks: u
                .fallback_fonts
                .iter()
                .map(|&h| ui_font(&eng, h as f64))
                .collect::<Result<Vec<_>, _>>()?,
            fallback: None,
            cjk: None,
            cjk_display: None,
            weights: [0; 6],
            faces: Vec::new(),
            chain: Vec::new(),
            cjk_locale: og_ui_next::fonts::CjkLocale::default(),
            families: Vec::new(),
            family_lists: Default::default(),
            backed: Default::default(),
        };
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
                    Some((ui_font(&eng, h)?, names))
                }
                None => None,
            };
            Ok(Some(match set {
                Some((font, names)) => IconAtlas {
                    font: Some(font),
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
        let atlas = atlas_of("icons", &u.icons)?.unwrap_or_default();
        // The locale's chain and forms (og.i18n) and the documents' named
        // families stay; the style's replace the last style's.
        let mut rs = eng.render.borrow_mut();
        let (chain, cjk_locale, mut families) = (
            rs.ui_fonts.chain.clone(),
            rs.ui_fonts.cjk_locale,
            rs.ui_fonts.families.clone(),
        );
        families.retain(|f| !f.document.starts_with("@pack:") && f.document != "@default");
        if let Some((faces, _)) = &style {
            families.extend(faces.clone());
        }
        let fonts = Fonts {
            chain,
            cjk_locale,
            families,
            ..fonts
        };
        let fontless = fonts.body.handle.is_none()
            && fonts.display.is_none()
            && fonts.mono.is_none()
            && fonts.rounded.is_none()
            && fonts.fallbacks.is_empty()
            && fonts.fallback.is_none()
            && fonts.chain.is_empty()
            && fonts.families.is_empty();
        rs.ui_fonts = fonts;
        // A role the game gave no font takes the first of the style's
        // families for it that has a face; one none backs is drawn in the
        // default face (i18n::apply), never in none.
        let roles = style.as_ref().map(|(_, r)| r.clone()).unwrap_or_default();
        rs.ui_fonts.back_roles(roles.map(|names| {
            names
                .into_iter()
                .map(og_ui_next::fonts::FontFamily::Named)
                .collect()
        }));
        // Each face's mixed font: the chain and fallbacks after it.
        rs.ui_fonts.link();
        rs.ui_icons = atlas;
        drop(rs);
        eng.render.borrow_mut().color_filter =
            UiSettings::of(&d, String::new(), (false, false)).color_filter();
        let mut st = eng.state.borrow_mut();
        st.ui_desc = d;
        // i18n::apply draws a role nothing backs in the default face.
        let default_available = eng
            .assets
            .borrow()
            .bundle_for(og_ui_next::fonts::DEFAULT_FONT_ASSET)
            .is_ok_and(|(bundle, _)| bundle.output(og_ui_next::fonts::DEFAULT_FONT_ASSET).is_ok());
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

/// A loaded style directory and the font faces its CSS names.
pub(crate) struct LoadedStyle {
    pub id: String,
    pub faces: Vec<og_ui_next::fonts::FamilyFace>,
    /// Its `fonts`: the font files it ships, loaded (their indices).
    pub fonts: Vec<usize>,
    /// The families its theme names for body, display, mono and rounded
    /// text, first first ([`og_ui_next::pack::Layer::role_families`]).
    pub roles: [Vec<String>; 4],
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
    // A pack's pack.json names its faces (`fonts`) and the families its
    // text roles take (`theme.fontFamily`).
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
    let (faces, fonts) = docs::pack_faces(eng, &layer.fonts, &blob, &format!("@pack:{id}"))?;
    Ok(LoadedStyle {
        id: id.to_owned(),
        faces,
        fonts,
        roles: layer.role_families(),
    })
}

/// A loaded font as the UI lays out and draws with it: its first atlas
/// page's handle and metrics (every page carries every glyph's advance);
/// the renderer draws the other pages from the same layout.
pub(crate) fn ui_font(eng: &Engine, handle: f64) -> Result<UiFont, String> {
    let index = super::assets::index_of(handle).ok_or_else(|| format!("{handle} is not a font"))?;
    let (render, file, pixel) = eng
        .assets
        .borrow()
        .fonts
        .get(index)
        .map(|f| (f.render, f.file.clone(), f.pixel))
        .ok_or_else(|| format!("font {handle} is not alive"))?;
    let mut rs = eng.render.borrow_mut();
    let pages = rs
        .fonts
        .get(render)
        .map(|p| p.pages.clone())
        .ok_or_else(|| format!("font {handle} has no atlas"))?;
    let first = *pages.first().ok_or("the font has no pages")?;
    let msdf = rs
        .scene
        .font(first)
        .cloned()
        .ok_or("the font's first page is missing")?;
    rs.ui_pages.insert(first, pages);
    let mut font = UiFont::new(first, &msdf);
    font.metrics = font.metrics.with_file(file).with_pixel(pixel);
    Ok(font)
}
