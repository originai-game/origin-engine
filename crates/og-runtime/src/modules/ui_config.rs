use super::*;

/// A UI with the theme and fonts of `previous` (a hot reload keeps them).
pub(crate) fn new_ui(
    tick: u32,
    seed: u64,
    previous: Option<&SharedUi>,
) -> Result<SharedUi, String> {
    let (theme, fonts, icons, style_name) = match previous {
        Some(p) => {
            let ui = p.lock();
            (
                *ui.theme(),
                ui.fonts().clone(),
                ui.icons().clone(),
                ui.style_name().to_string(),
            )
        }
        None => (
            Theme::default(),
            Fonts::default(),
            IconAtlas::default(),
            String::new(),
        ),
    };
    let mut ui = Ui::new(UiDesc {
        theme,
        ticks_per_second: tick,
        fonts,
    });
    ui.seed_feedback(seed);
    ui.set_icons(icons);
    if let Some(p) = previous {
        ui.set_images(p.lock().images().clone());
        ui.inherit_part_css(&p.lock());
    }
    ui.set_style_name(&style_name);
    // A hot reload keeps the documents loaded (and the timelines they define).
    if let Some(p) = previous {
        let p = p.lock();
        ui.inherit_store_registrations(&p);
        for name in p.documents() {
            if let Some(d) = p.document(&name) {
                ui.define_document(d.clone());
            }
        }
        ui.restore_state(&p.save_state())
            .map_err(|e| e.reason().to_owned())?;
        ui.restore_world_state(&p.save_world_state())
            .map_err(|e| e.reason().to_owned())?;
    }
    Ok(SharedUi::new(ui))
}

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

pub fn og_ui_configure(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let eng = engine(world)?;
        let d = merged_ui_desc(&eng.state.borrow().ui_desc, parse(desc.as_deref())?)?;
        let style = match d["style"].as_f64() {
            Some(h) => {
                let i = super::assets::index_of(h)
                    .ok_or_else(|| format!("style: {h} is not a UI style"))?;
                let a = eng.assets.borrow();
                let s = a
                    .styles
                    .get(i)
                    .ok_or_else(|| format!("style: style {h} is not alive"))?;
                Some((
                    s.pack.clone(),
                    s.faces.clone(),
                    s.textures.clone(),
                    s.sounds.clone(),
                ))
            }
            None => None,
        };
        let scheme = match d["scheme"].as_str() {
            Some("light") => Scheme::Light,
            _ => Scheme::Dark,
        };
        let mut theme = Theme::new(scheme);
        if d["contrast"].as_str() == Some("high") {
            theme = theme.to_high_contrast();
        }
        if d["primaryColor"].is_object() {
            let c = &d["primaryColor"];
            let ch = |k: &str| (f(c, k, 1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
            theme = theme.with_primary(Srgba::rgb(ch("r"), ch("g"), ch("b")));
        }
        theme.reduced_motion = d["reducedMotion"].as_bool().unwrap_or(false);
        theme.reduced_transparency = d["reducedTransparency"].as_bool().unwrap_or(false);
        theme.color_vision = d["colorVision"]
            .as_str()
            .and_then(og_render::ColorVision::from_name)
            .unwrap_or_default();
        theme.color_vision_filter = match d["colorVisionFilter"].as_str() {
            Some("simulate") => og_render::ColorVisionFilter::Simulate,
            _ => og_render::ColorVisionFilter::Correct,
        };
        theme.color_vision_strength = d["colorVisionStrength"]
            .as_f64()
            .map_or(1.0, |v| v.clamp(0.0, 1.0) as f32);
        theme.text_scale = d["textScale"].as_f64().map_or(1.0, |v| v as f32);
        theme.unit_pin = d["unitScale"].as_f64().map_or(0.0, |v| {
            if v > 0.0 {
                v.clamp(1.0, 2.0) as f32
            } else {
                0.0
            }
        });
        theme.density = d["density"]
            .as_str()
            .and_then(DensityMode::from_name)
            .unwrap_or_default();
        let sounds = &d["sounds"];
        if sounds.is_object() {
            for e in og_ui::UiSound::ALL {
                if let Some(h) = sounds[e.name()].as_f64() {
                    theme.art.sounds[e as usize] = h as u64;
                    let f = &mut theme.tokens.feedback[e.event().index()];
                    f.sounds = [0; og_ui::tokens::SET_SOUNDS];
                    f.sounds[0] = h as u64;
                    f.count = u8::from(h != 0.0);
                }
            }
        }
        let font = |key: &str| -> Result<Option<UiFont>, String> {
            match d[key].as_f64() {
                None => Ok(None),
                Some(h) => ui_font(&eng, h).map(Some),
            }
        };
        let fonts = Fonts {
            body: font("font")?.unwrap_or_default(),
            display: font("displayFont")?,
            mono: font("monoFont")?,
            rounded: font("roundedFont")?,
            fallbacks: match d["fallbackFonts"].as_array() {
                Some(list) => list
                    .iter()
                    .filter_map(Value::as_f64)
                    .map(|h| ui_font(&eng, h))
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
            },
            fallback: None,
            cjk: None,
            cjk_display: None,
            weights: [0; 6],
            faces: Vec::new(),
            chain: Vec::new(),
            cjk_locale: og_ui::CjkLocale::default(),
            families: Vec::new(),
            family_lists: Default::default(),
        };
        let ui = eng
            .state
            .borrow()
            .ui
            .clone()
            .ok_or("this engine has no UI")?;
        // A game's atlas (`icons`), else the style's; under either, the
        // set that draws the names it lacks (`fallbackIcons`).
        let atlas_of = |field: &str| -> Result<Option<IconAtlas>, String> {
            let icons = &d[field];
            if !icons.is_object() {
                return Ok(None);
            }
            let texture = match icons["texture"].as_f64() {
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
            let set = match icons["font"].as_f64() {
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
                    columns: icons["columns"].as_u64().unwrap_or(1).max(1) as u32,
                    rows: icons["rows"].as_u64().unwrap_or(1).max(1) as u32,
                    names: icons["names"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .map(|n| n.as_str().unwrap_or_default().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                    ..IconAtlas::default()
                },
            }))
        };
        let mut atlas = atlas_of("icons")?.unwrap_or_default();
        atlas.fallback = atlas_of("fallbackIcons")?.map(Box::new);
        // Pictures rich text draws inline by name ([img=…]).
        let mut images = std::collections::BTreeMap::new();
        for im in d["images"].as_array().into_iter().flatten() {
            let name = im["name"].as_str().unwrap_or_default();
            let Some(t) = im["texture"].as_f64() else {
                continue;
            };
            if name.is_empty() {
                return Err("images: an image without a name".into());
            }
            let rs = eng.render.borrow();
            let i = super::assets::index_of(t)
                .ok_or_else(|| format!("images.{name}.texture: {t} is not a texture"))?;
            let texture = *rs
                .textures
                .get(i)
                .ok_or_else(|| format!("images.{name}.texture: texture {t} is not alive"))?;
            let uv = match im["uv"].as_array() {
                Some(a) if a.len() == 4 => {
                    let v = |k: usize| a[k].as_f64().unwrap_or(0.0) as f32;
                    [v(0), v(1), v(2), v(3)]
                }
                _ => [0.0, 0.0, 1.0, 1.0],
            };
            let aspect = im["aspect"].as_f64().map_or(1.0, |a| a as f32);
            images.insert(
                name.to_string(),
                og_ui::InlineImage {
                    texture,
                    uv,
                    aspect: if aspect > 0.0 { aspect } else { 1.0 },
                },
            );
        }
        let mut ui = ui.lock();
        ui.set_theme(theme);
        // The locale's chain and forms (og.i18n) and the documents' named
        // families (`@font-face`) stay.
        let (chain, cjk_locale, mut families) = (
            ui.fonts().chain.clone(),
            ui.fonts().cjk_locale,
            ui.fonts().families.clone(),
        );
        families.retain(|f| !f.document.starts_with("@pack:"));
        if let Some((_, faces, _, _)) = &style {
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
        ui.set_fonts(fonts);
        ui.set_icons(atlas);
        ui.set_images(images);
        if let Some((pack, _, textures, sounds)) = &style {
            ui.set_style_pack(pack)?;
            ui.set_css_sounds(sounds.clone());
            for (path, texture) in textures {
                ui.define_texture(path, *texture);
            }
        } else {
            ui.style_documents(None)?;
            ui.set_part_css(og_ui::DEFAULT_CSS)?;
            ui.set_css_sounds(Default::default());
            ui.set_style_name("");
        }
        ui.set_toast_anchor(anchor_of(&d["toastAnchor"]).unwrap_or(Anchor::Bottom));
        ui.set_hud_max_aspect(
            d["hudMaxAspect"]
                .as_f64()
                .map_or(og_ui::HUD_MAX_ASPECT, |a| a as f32),
        );
        drop(ui);
        let mut st = eng.state.borrow_mut();
        st.ui_desc = d;
        if fontless {
            let text = "og.ui.configure: the UI has no font now, so it draws no text (every label, button and field is blank): give `font` (og.load.font) or a `style` (og.load.style, whose fonts it brings), in this call or an earlier one; a field set to null is cleared, a field left out keeps its value".to_string();
            if st.ui_warned.insert(text.clone()) {
                let tick = eng.world().map_or(0, |w| w.time().tick);
                st.log
                    .push(json!({ "tick": tick, "warning": true, "text": text }));
            }
        }
        Ok(())
    })();
    match answer {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// A loaded CSS directory and the assets its stylesheets reference.
pub(crate) struct LoadedStyle {
    pub id: String,
    pub pack: og_ui::style_pack::StylePack,
    pub faces: Vec<og_ui::FamilyFace>,
    pub textures: Vec<(String, og_render::TextureHandle)>,
    pub sounds: std::collections::BTreeMap<String, u64>,
}

/// `og.load.style`: a cooked UI style, its atlas, fonts, icons and sounds loaded with it.
pub fn og_load_style(world: u32, path: String) -> String {
    match engine(world).and_then(|eng| {
        let i = load_style(&eng, &path)?;
        docs::note_styles(&eng);
        Ok(i)
    }) {
        Ok(i) => ok(json!(super::assets::handle(i) as f64)),
        Err(e) => refuse(e),
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
        return Err("style.ogt is removed; og.load.style takes the directory containing the pack's CSS files".into());
    }
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle
            .style_text(id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let files = serde_json::from_str(&text).map_err(|e| format!("{id}: CSS pack sources: {e}"))?;
    let pack = og_ui::style_pack::StylePack::parse(id, files)?;
    let faces = docs::sheet_faces(eng, &pack.sheet, id, &format!("@pack:{id}"))?;
    let texture = |p: &str| -> Result<og_render::TextureHandle, String> {
        let i = super::assets::load_texture(eng, p)?;
        eng.render
            .borrow()
            .textures
            .get(i)
            .copied()
            .ok_or_else(|| format!("{id}: the texture {p} did not load"))
    };
    let mut textures = Vec::new();
    let mut sounds = std::collections::BTreeMap::new();
    for asset in &pack.assets {
        let ty = {
            let assets = eng.assets.borrow();
            let (_, bundle) = assets.blob(asset)?;
            bundle.output(asset).map_err(|e| e.to_string())?.ty.clone()
        };
        match ty.as_str() {
            "og.Font" => {}
            "og.Sound" => {
                let index = super::assets::load_sound(eng, asset)?;
                sounds.insert(asset.clone(), eng.assets.borrow().sound_handles[index]);
            }
            "og.Texture" => textures.push((asset.clone(), texture(asset)?)),
            _ => return Err(format!("{id}: CSS asset {asset} has unsupported type {ty}")),
        }
    }
    Ok(LoadedStyle {
        id: id.to_owned(),
        pack,
        faces,
        textures,
        sounds,
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
