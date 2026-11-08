use super::*;

pub(super) fn desc_of(desc: Option<String>) -> Value {
    parse(desc.as_deref()).unwrap_or(Value::Null)
}

pub fn og_ui_open(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.open", |ui| {
        let el = element(ui.theme(), &d);
        ui.open(id.as_str(), el)
    });
}

pub fn og_ui_open_scroll(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.openScroll", |ui| {
        let s = ScrollDesc {
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            padding: insets(&d["padding"]),
            gap: f(&d, "gap", 0.0),
            axes: match d["axes"].as_str() {
                Some("horizontal") => og_ui::ScrollAxes::Horizontal,
                Some("both") => og_ui::ScrollAxes::Both,
                _ => og_ui::ScrollAxes::Vertical,
            },
            hide_bars: d["hideBars"].as_bool().unwrap_or(false),
            fade: f(&d, "fade", 0.0),
            fade_into: d["fadeInto"].as_str().and_then(|n| ui.theme().color(n)),
        };
        ui.open_scroll(id.as_str(), s)
    });
}

pub fn og_ui_close() -> String {
    response(with_ui("og.ui.close", |ui| ui.close()).unwrap_or_default()).to_string()
}

pub fn og_ui_label(id: String, text: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.label", |ui| {
        let t = *ui.theme();
        let l = LabelDesc {
            step: d["step"].as_str().and_then(|s| step(&t, s)),
            color: paint(&t, &d["color"]),
            on_scene: d["onScene"].as_bool().unwrap_or(false),
            uppercase: d["uppercase"].as_bool().unwrap_or(false),
            align: match d["align"].as_str() {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            nowrap: d["nowrap"].as_bool().unwrap_or(false),
            width: size_or(&d["width"], Size::fit()),
            font: d["font"]
                .as_str()
                .and_then(FontRole::from_name)
                .unwrap_or(FontRole::Body),
            heading: d["heading"].as_bool().unwrap_or(false),
            ellipsis: d["ellipsis"].as_bool().unwrap_or(false),
            max_lines: d["maxLines"].as_u64().unwrap_or(0) as u32,
            fit_min: d["fitMin"].as_str().and_then(|s| step(&t, s)),
        };
        ui.label(id.as_str(), &text, l)
    });
}

pub fn og_ui_rich_text(id: String, markup: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.richText", |ui| {
        let t = *ui.theme();
        let desc = og_ui::RichTextDesc {
            step: d["step"].as_str().and_then(|s| step(&t, s)),
            color: paint(&t, &d["color"]),
            align: match d["align"].as_str() {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            nowrap: d["nowrap"].as_bool().unwrap_or(false),
            width: size_or(&d["width"], Size::fit()),
            font: d["font"]
                .as_str()
                .and_then(FontRole::from_name)
                .unwrap_or(FontRole::Body),
        };
        ui.rich_text(id.as_str(), &markup, desc)
    })
    .unwrap_or_default();
    json!({ "response": response(r.response), "link": r.link, "hovered": r.hovered }).to_string()
}

pub fn og_ui_button(id: String, text: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.button", |ui| {
        let b = ButtonDesc {
            variant: match d["variant"].as_str() {
                Some("primary") => ButtonVariant::Primary,
                Some("quiet") => ButtonVariant::Quiet,
                Some("danger") => ButtonVariant::Danger,
                Some("ghost") => ButtonVariant::Ghost,
                _ => ButtonVariant::Secondary,
            },
            size: match d["size"].as_str() {
                Some("small") => ControlSize::Small,
                Some("large") => ControlSize::Large,
                _ => ControlSize::Medium,
            },
            disabled: d["disabled"].as_bool().unwrap_or(false),
            width: size(&d["width"]),
            autofocus: d["autofocus"].as_bool().unwrap_or(false),
            square: d["square"].as_bool().unwrap_or(false),
        };
        ui.button(id.as_str(), &text, b)
    });
    response(r.unwrap_or_default()).to_string()
}

pub fn og_ui_image(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    let texture = d["texture"].as_f64().and_then(|t| {
        let eng = engine(BUILDING.with(Cell::get)?).ok()?;
        let rs = eng.render.borrow();
        super::assets::index_of(t).and_then(|i| rs.textures.get(i).copied())
    });
    with_ui("og.ui.image", |ui| {
        let t = *ui.theme();
        let uv = d["uv"].as_array().filter(|a| a.len() == 4).map(|a| {
            let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
            [n(0), n(1), n(2), n(3)]
        });
        let i = ImageDesc {
            // A texture-less image is a solid rectangle.
            texture,
            uv,
            tint: paint(&t, &d["tint"]),
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            aspect: f(&d, "aspect", 0.0).max(0.0),
            label: d["label"].as_str().unwrap_or_default().to_string(),
        };
        ui.image(id.as_str(), i)
    });
}

pub fn og_ui_zoom_view(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let texture = texture_of(&d["texture"]);
    let r = with_ui("og.ui.zoomView", |ui| {
        let t = *ui.theme();
        ui.zoom_view(
            id.as_str(),
            &og_ui::ZoomViewDesc {
                texture,
                uv: uv_of(&d["uv"]),
                tint: paint(&t, &d["tint"]),
                width: size_or(&d["width"], Size::fit()),
                height: size_or(&d["height"], Size::fit()),
                aspect: f(&d, "aspect", 0.0).max(0.0),
                min_zoom: f(&d, "minZoom", 0.0),
                max_zoom: f(&d, "maxZoom", 0.0),
            },
        )
    });
    let (zoom, x, y) = r.map_or((1.0, 0.5, 0.5), |r| (r.zoom, r.center.x, r.center.y));
    json!({ "zoom": zoom, "centerX": x, "centerY": y }).to_string()
}

pub fn og_ui_text_field(id: String, value: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let mut text = value;
    let r = with_ui("og.ui.textField", |ui| {
        let fd = FieldDesc {
            placeholder: d["placeholder"].as_str().unwrap_or_default().to_string(),
            width: size(&d["width"]),
            max_chars: d["maxChars"].as_u64().unwrap_or(0) as u32,
            invalid: d["invalid"].as_bool().unwrap_or(false),
            bare: d["bare"].as_bool().unwrap_or(false),
            tight: d["tight"].as_bool().unwrap_or(false),
            component: d["component"].as_str().map(str::to_string),
            input_type: d["type"]
                .as_str()
                .and_then(og_ui::InputType::from_name)
                .unwrap_or_default(),
            inputmode: d["inputmode"]
                .as_str()
                .and_then(og_ui::InputMode::from_name),
            autocomplete: d["autocomplete"].as_str().unwrap_or_default().to_string(),
            // readonly and the constraints are document attributes only.
            ..FieldDesc::default()
        };
        ui.text_field(id.as_str(), &mut text, &fd)
    });
    json!({ "response": response(r.unwrap_or_default()), "value": text }).to_string()
}

pub fn og_ui_text_area(id: String, value: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let mut text = value;
    let r = with_ui("og.ui.textArea", |ui| {
        let ad = og_ui::TextAreaDesc {
            placeholder: d["placeholder"].as_str().unwrap_or_default().to_string(),
            width: size(&d["width"]),
            rows: d["rows"].as_u64().unwrap_or(0) as u32,
            max_chars: d["maxChars"].as_u64().unwrap_or(0) as u32,
            invalid: d["invalid"].as_bool().unwrap_or(false),
            bare: d["bare"].as_bool().unwrap_or(false),
            tight: d["tight"].as_bool().unwrap_or(false),
            component: d["component"].as_str().map(str::to_string),
            ..og_ui::TextAreaDesc::default()
        };
        ui.text_area(id.as_str(), &mut text, &ad)
    });
    json!({ "response": response(r.unwrap_or_default()), "value": text }).to_string()
}

pub fn og_ui_counter(id: String, value: f64, desc: Option<String>) -> f64 {
    let d = desc_of(desc);
    with_ui("og.ui.counter", |ui| {
        let t = *ui.theme();
        let c = og_ui::CounterDesc {
            step: d["step"].as_str().and_then(|s| step(&t, s)),
            font: d["font"]
                .as_str()
                .and_then(FontRole::from_name)
                .unwrap_or(FontRole::Body),
            color: paint(&t, &d["color"]),
            align: match d["align"].as_str() {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            width: size_or(&d["width"], Size::fit()),
            duration_ms: f(&d, "durationMs", 0.0),
            from: d["from"].as_f64(),
            decimals: d["decimals"].as_u64().unwrap_or(0).min(9) as u32,
            prefix: d["prefix"].as_str().unwrap_or_default().to_string(),
            suffix: d["suffix"].as_str().unwrap_or_default().to_string(),
        };
        ui.counter(id.as_str(), value, &c)
    })
    .unwrap_or(value)
}

pub fn og_ui_burst(desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.burst", |ui| {
        let t = *ui.theme();
        ui.burst(og_ui::BurstDesc {
            count: d["count"].as_u64().unwrap_or(0).min(64) as u32,
            color: paint(&t, &d["color"]),
            radius: f(&d, "radius", 0.0),
            duration_ms: f(&d, "durationMs", 0.0),
            delay_ms: f(&d, "delayMs", 0.0),
        })
    });
}

pub fn og_ui_screen_transition(id: String, screen: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let texture = d["texture"].as_f64().and_then(|h| {
        let i = super::assets::index_of(h)?;
        let target = BUILDING.with(Cell::get)?;
        let eng = engine(target).ok()?;

        eng.render.borrow().textures.get(i).copied()
    });
    let r = with_ui("og.ui.screenTransition", |ui| {
        let t = *ui.theme();
        let desc = og_ui::ScreenTransitionDesc {
            kind: match d["kind"].as_str() {
                Some("wipe") => og_ui::ScreenWipe::Wipe,
                Some("iris") => og_ui::ScreenWipe::Iris,
                Some("dissolve") => og_ui::ScreenWipe::Dissolve,
                Some("slide") => og_ui::ScreenWipe::Slide,
                _ => og_ui::ScreenWipe::Fade,
            },
            duration_ms: f(&d, "durationMs", 0.0),
            color: paint(&t, &d["color"]),
            texture: texture.map(|t| (t, [0.0, 0.0, 1.0, 1.0])),
            from: match d["from"].as_str() {
                Some("right") => og_ui::Side::Right,
                Some("top") => og_ui::Side::Top,
                Some("bottom") => og_ui::Side::Bottom,
                _ => og_ui::Side::Left,
            },
        };
        ui.screen_transition(id.as_str(), &screen, &desc)
    })
    .unwrap_or(og_ui::ScreenTransition {
        shown: screen.clone(),
        ..Default::default()
    });
    json!({ "shown": r.shown, "covering": r.covering, "revealing": r.revealing, "cover": r.cover })
        .to_string()
}

pub fn og_ui_tooltip(text: String) {
    with_ui("og.ui.tooltip", |ui| ui.tooltip(&text));
}

// ── ui: drawing over the frame ──────────────────────────────────────────────

/// Hands the UI's last frame to the render scene's overlay: screen-space
/// sprites and text in CSS pixels over the finished frame (after every post
/// effect, never tone mapped or blurred), at any camera.
/// `shifts`: how far each anchored element moves on a frame drawn between
/// ticks, by key (`crate::interpolate::ui_shifts`).
pub(crate) fn emit(rs: &mut super::extract::RenderState, shifts: &og_core::IdMap<u64, glam::Vec2>) {
    let Some(ui) = rs.ui.clone() else {
        return;
    };
    let ui = ui.lock();
    // The player's colour vision filter over the whole frame (docs/ui.md
    // §3.2), whether or not the UI draws anything this frame.
    rs.scene.color_filter = ui.theme().color_filter();
    let frame = ui.frame();
    let world_frame = ui.world_frame();
    if frame.items.is_empty() && world_frame.batches.is_empty() {
        return;
    }
    // The frame lays out in og-ui units and emits CSS px (docs/ui.md
    // §15.2): the overlay is the CSS surface, the unit times the frame's.
    let (w, h) = (
        frame.viewport.width * frame.unit,
        frame.viewport.height * frame.unit,
    );
    // Scale 0 fits the viewport's height to the target (as a capture at
    // another size needs).
    let overlay = rs
        .scene
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(w.max(1.0), h.max(1.0), 0.0));
    let first_text = overlay.texts.len();
    let first_sprite = overlay.sprites.len();
    let first_shape = overlay.ui.shapes.len();
    frame.emit_shifted(
        &og_ui::EmitDesc {
            // Over the game's screen layers up to og_render::UI_LAYER, under
            // those above it (og_render::screen_layer).
            layer: og_render::UI_OVERLAY_LAYER,
            time: ui.seconds() as f32,
        },
        overlay,
        &|key| shifts.get(&key).copied(),
    );
    // Mass elements no element owns (docs/ui.md §1), under the UI.
    world_frame.emit(og_render::UI_OVERLAY_LAYER, overlay);
    crate::extract::draw_other_pages(overlay, first_text, &rs.ui_pages);
    // Icons on a strict art palette (super::art): the icon atlas is a
    // distance field the cook leaves as it is, so its tint is moved to
    // the palette here.
    let icons = ui.icons();
    let icon_textures: Vec<og_render::TextureHandle> =
        icons.layers().filter_map(|a| a.texture).collect();
    let icon_fonts: Vec<og_render::FontHandle> = icons
        .layers()
        .filter_map(|a| a.font.as_ref().and_then(|f| f.handle))
        .flat_map(|h| rs.ui_pages.get(&h).cloned().unwrap_or_else(|| vec![h]))
        .collect();
    if let Some(overlay) = rs.scene.overlay.as_mut() {
        super::art::palette_icons(
            &rs.art.frame,
            overlay,
            (first_sprite, first_text, first_shape),
            (&icon_textures, &icon_fonts),
        );
    }
}
