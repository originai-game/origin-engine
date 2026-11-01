use super::*;

/// A UI with the theme and fonts of `previous` (a hot reload keeps them).
pub(crate) fn new_ui(tick: u32, previous: Option<&SharedUi>) -> SharedUi {
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
    ui.set_icons(icons);
    ui.set_style_name(&style_name);
    // A hot reload keeps the documents loaded (and the timelines they define).
    if let Some(p) = previous {
        let p = p.lock();
        for name in p.documents() {
            if let Some(d) = p.document(&name) {
                ui.define_document(d.clone());
            }
        }
    }
    SharedUi::new(ui)
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
                    s.file.clone(),
                    s.atlas,
                    s.fonts,
                    s.icons,
                    s.sounds,
                    s.feedback.clone(),
                ))
            }
            None => None,
        };
        let scheme = match d["scheme"].as_str() {
            Some("light") => Scheme::Light,
            _ => Scheme::Dark,
        };
        let mut theme = match &style {
            Some((file, atlas, _, _, sounds, feedback)) => {
                file.theme_with_feedback(*atlas, *sounds, |id| {
                    feedback.get(id).copied().unwrap_or(0)
                })
            }
            None => Theme::new(scheme),
        };
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
        let style_font = |i: usize| -> Result<Option<UiFont>, String> {
            match style.as_ref().and_then(|s| s.2[i]) {
                None => Ok(None),
                Some(h) => ui_font(&eng, h).map(Some),
            }
        };
        let body = match font("font")? {
            Some(f) => Some(f),
            None => style_font(0)?,
        };
        let fonts = Fonts {
            body: body.unwrap_or_default(),
            display: match font("displayFont")? {
                Some(f) => Some(f),
                None => style_font(1)?,
            },
            mono: match font("monoFont")? {
                Some(f) => Some(f),
                None => style_font(2)?,
            },
            rounded: font("roundedFont")?,
            fallbacks: match d["fallbackFonts"].as_array() {
                Some(list) => list
                    .iter()
                    .filter_map(Value::as_f64)
                    .map(|h| ui_font(&eng, h))
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
            },
            fallback: style_font(3)?,
            cjk: style_font(4)?,
            cjk_display: style_font(5)?,
            // The locale's chain (og.i18n) stays.
            chain: Vec::new(),
        };
        let ui = eng
            .state
            .borrow()
            .ui
            .clone()
            .ok_or("this engine has no UI")?;
        let icons = &d["icons"];
        let atlas = if icons.is_object() {
            let texture = match icons["texture"].as_f64() {
                Some(t) => {
                    let rs = eng.render.borrow();
                    let i = super::assets::index_of(t)
                        .ok_or_else(|| format!("icons.texture: {t} is not a texture"))?;
                    Some(
                        *rs.textures
                            .get(i)
                            .ok_or_else(|| format!("icons.texture: texture {t} is not alive"))?,
                    )
                }
                None => None,
            };
            // An icon set (an `.ogt` of `type icons`, loaded as a font):
            // distance-field glyphs, named by the set.
            let set = match icons["font"].as_f64() {
                Some(h) => {
                    let i = super::assets::index_of(h)
                        .ok_or_else(|| format!("icons.font: {h} is not a font"))?;
                    let names = eng
                        .assets
                        .borrow()
                        .fonts
                        .get(i)
                        .map(|f| f.icons.clone())
                        .ok_or_else(|| format!("icons.font: font {h} is not alive"))?;
                    if names.is_empty() {
                        return Err(format!(
                            "icons.font: font {h} is not an icon set (cook SVGs with an .ogt of `type icons`)"
                        ));
                    }
                    Some((ui_font(&eng, h)?, names))
                }
                None => None,
            };
            match set {
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
                    font: None,
                },
            }
        } else if let Some((file, _, _, Some(texture), _, _)) = &style {
            let (_, columns, rows, names) = file.icons.clone().unwrap_or_default();
            IconAtlas {
                texture: Some(*texture),
                columns,
                rows,
                names,
                font: None,
            }
        } else {
            IconAtlas::default()
        };
        let mut ui = ui.lock();
        ui.set_theme(theme);
        // The style's dressings, its vector fills bound when it loaded
        // (none: the prototype's tokens dress every part).
        ui.set_dressings(
            style
                .as_ref()
                .map_or_else(Vec::new, |s| s.0.dressings.clone()),
        );
        let chain = ui.fonts().chain.clone();
        let fonts = Fonts { chain, ..fonts };
        let fontless = fonts.body.handle.is_none()
            && fonts.display.is_none()
            && fonts.mono.is_none()
            && fonts.rounded.is_none()
            && fonts.fallbacks.is_empty()
            && fonts.fallback.is_none()
            && fonts.chain.is_empty();
        ui.set_fonts(fonts);
        ui.set_icons(atlas);
        ui.set_style_name(style.as_ref().map_or("", |s| s.0.name.as_str()));
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

/// A loaded UI style: its file and the handles of the assets it names.
pub(crate) struct LoadedStyle {
    pub id: String,
    pub file: og_ui::StyleFile,
    pub atlas: Option<og_render::TextureHandle>,
    /// Font asset handles: body, display, numeric, fallback, cjk, cjkDisplay.
    pub fonts: [Option<f64>; 6],
    pub icons: Option<og_render::TextureHandle>,
    pub sounds: [u64; 6],
    /// The sound handle of each asset id its feedback sets name.
    pub feedback: std::collections::BTreeMap<String, u64>,
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
    (|| -> Result<usize, String> {
        let id = og_bundle::runtime::asset_id(path).to_string();
        if let Some(i) = eng.assets.borrow().styles.iter().position(|s| s.id == id) {
            return Ok(i);
        }
        let text = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .style_text(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let mut file = og_ui::parse_style(&text).map_err(|e| format!("{id}: {e}"))?;
        // Ids relative to the style file first, as the cook resolved them.
        file.resolve(&id, |r| eng.assets.borrow().bundle_for(r).is_ok());
        let texture = |p: &str| -> Result<og_render::TextureHandle, String> {
            let i = super::assets::load_texture(eng, p)?;
            eng.render
                .borrow()
                .textures
                .get(i)
                .copied()
                .ok_or_else(|| format!("{id}: the texture {p} did not load"))
        };
        let atlas = file.atlas.as_deref().map(texture).transpose()?;
        let icons = file.icons.as_ref().map(|i| texture(&i.0)).transpose()?;
        let mut fonts = [None; 6];
        for (k, p) in file.fonts.iter().enumerate() {
            // A Chinese companion is in the bundle only when the game shows Chinese.
            if k >= 4
                && let Some(p) = p
                && eng.assets.borrow().bundle_for(p).is_err()
            {
                continue;
            }
            if let Some(p) = p {
                let i = super::assets::load_font(eng, p)?;
                fonts[k] = Some(super::assets::handle(i) as f64);
            }
        }
        let mut sounds = [0u64; 6];
        for (k, p) in file.sounds.iter().enumerate() {
            if let Some(p) = p {
                let i = super::assets::load_sound(eng, p)?;
                sounds[k] = eng.assets.borrow().sound_handles[i];
            }
        }
        let mut feedback = std::collections::BTreeMap::new();
        for p in file.feedback_sounds.iter().flatten() {
            if !feedback.contains_key(p) {
                let i = super::assets::load_sound(eng, p)?;
                feedback.insert(p.clone(), eng.assets.borrow().sound_handles[i]);
            }
        }
        // The dressings' vector fills, bound to the loaded vectors.
        let mut vectors = std::collections::BTreeMap::new();
        for v in file.vector_assets() {
            vectors.insert(v.clone(), super::ui_draw::load_vector(eng, &v)?.1);
        }
        file.bind_vectors(|v| vectors.get(v).copied());
        let mut a = eng.assets.borrow_mut();
        a.styles.push(LoadedStyle {
            id,
            file,
            atlas,
            fonts,
            icons,
            sounds,
            feedback,
        });
        Ok(a.styles.len() - 1)
    })()
}

/// A loaded font as the UI lays out and draws with it: its first atlas
/// page's handle and metrics (every page carries every glyph's advance);
/// the renderer draws the other pages from the same layout.
pub(crate) fn ui_font(eng: &Engine, handle: f64) -> Result<UiFont, String> {
    let index = super::assets::index_of(handle).ok_or_else(|| format!("{handle} is not a font"))?;
    let (render, file) = eng
        .assets
        .borrow()
        .fonts
        .get(index)
        .map(|f| (f.render, f.file.clone()))
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
    font.metrics = font.metrics.with_file(file);
    Ok(font)
}
