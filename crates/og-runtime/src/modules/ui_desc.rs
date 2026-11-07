use super::*;

// ── ui: decoding descs ──────────────────────────────────────────────────────

pub(super) fn f(v: &Value, k: &str, d: f32) -> f32 {
    v[k].as_f64().map_or(d, |x| x as f32)
}

/// A `UiRect` (`{ x, y, width, height }`); anything else is the empty one.
pub(super) fn ui_rect(v: &Value) -> og_ui::Rect {
    og_ui::Rect::new(
        f(v, "x", 0.0),
        f(v, "y", 0.0),
        f(v, "width", 0.0),
        f(v, "height", 0.0),
    )
}

pub(super) fn insets(v: &Value) -> Insets {
    Insets {
        left: f(v, "left", 0.0),
        top: f(v, "top", 0.0),
        right: f(v, "right", 0.0),
        bottom: f(v, "bottom", 0.0),
    }
}

pub(super) fn size(v: &Value) -> Option<Size> {
    let range = |v: &Value| (f(v, "min", 0.0), f(v, "max", f32::INFINITY));
    Some(match v["kind"].as_str()? {
        "fit" => {
            let (min, max) = range(v);
            Size::Fit { min, max }
        }
        "grow" => {
            let (min, max) = range(v);
            Size::Grow { min, max }
        }
        "fixed" => Size::Fixed(f(v, "px", 0.0)),
        "percent" => Size::Percent(f(v, "fraction", 0.0)),
        _ => return None,
    })
}

pub(super) fn size_or(v: &Value, d: Size) -> Size {
    size(v).unwrap_or(d)
}

/// A colour role by its name: a ThemeColor, or one the style declares.
fn token(theme: &Theme, name: &str) -> Option<Srgba> {
    theme.color(name)
}

/// Every ThemeColor name, in the IDL's order.
pub const THEME_COLORS: &[&str] = og_ui::Palette::NAMES;

/// A type step by its TypeStepName.
pub(crate) fn step(theme: &Theme, name: &str) -> Option<TypeStep> {
    let r = &theme.type_ramp;
    Some(match name {
        "body" => r.body,
        "xs" => r.xs,
        "meta" => r.meta,
        "caption" => r.caption,
        "sm" => r.sm,
        "reading" => r.reading,
        "base" => r.base,
        "lg" => r.lg,
        "xl" => r.xl,
        "xl2" => r.xl2,
        "xl3" => r.xl3,
        "xl4" => r.xl4,
        "xl5" => r.xl5,
        "xl6" => r.xl6,
        "xl7" => r.xl7,
        "xl8" => r.xl8,
        "displaySm" => r.display_sm,
        "displayMd" => r.display_md,
        "displayLg" => r.display_lg,
        "displayXl" => r.display_xl,
        "displayXl2" | "hero" => r.display_xl2,
        _ => return None,
    })
}

/// Every TypeStepName, in the IDL's order.
pub const TYPE_STEPS: &[&str] = &[
    "body",
    "xs",
    "meta",
    "caption",
    "sm",
    "reading",
    "base",
    "lg",
    "xl",
    "xl2",
    "xl3",
    "xl4",
    "xl5",
    "xl6",
    "xl7",
    "xl8",
    "displaySm",
    "displayMd",
    "displayLg",
    "displayXl",
    "displayXl2",
    "hero",
];

/// A Paint: a token, else an sRGB colour as authored (0–1), else white.
pub(super) fn paint(theme: &Theme, v: &Value) -> Option<Srgba> {
    if !v.is_object() {
        return None;
    }
    let alpha = f(v, "alpha", 1.0).clamp(0.0, 1.0);
    let base = match v["token"].as_str().or(v["role"].as_str()) {
        Some(t) => token(theme, t)?,
        None => {
            let c = &v["color"];
            if c.is_object() {
                let ch = |k: &str| (f(c, k, 1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
                Srgba::rgb(ch("r"), ch("g"), ch("b"))
            } else {
                Srgba::WHITE
            }
        }
    };
    Some(base.fade(alpha))
}

pub(super) fn element(theme: &Theme, d: &Value) -> Element {
    Element {
        direction: match d["direction"].as_str() {
            Some("column") => Direction::Column,
            _ => Direction::Row,
        },
        width: size_or(&d["width"], Size::fit()),
        height: size_or(&d["height"], Size::fit()),
        padding: insets(&d["padding"]),
        gap: f(d, "gap", 0.0),
        align: align(&d["align"]),
        cross: cross(&d["cross"]),
        wrap: d["wrap"].as_bool().unwrap_or(false),
        scroll: d["scroll"].as_bool().unwrap_or(false),
        scroll_x: d["scrollX"].as_bool().unwrap_or(false),
        hide_bars: false,
        fade: 0.0,
        fade_into: None,
        interactive: d["interactive"].as_bool().unwrap_or(false),
        focusable: d["focusable"].as_bool().unwrap_or(false),
        adjustable: d["adjustable"].as_bool().unwrap_or(false),
        own_ring: d["ownRing"].as_bool().unwrap_or(false),
        arrows: {
            let a = &d["arrows"];
            [
                ("left", og_ui::ARROW_LEFT),
                ("right", og_ui::ARROW_RIGHT),
                ("up", og_ui::ARROW_UP),
                ("down", og_ui::ARROW_DOWN),
            ]
            .into_iter()
            .filter(|(k, _)| a[*k].as_bool().unwrap_or(false))
            .fold(0, |m, (_, bit)| m | bit)
        },
        background: paint(theme, &d["background"]),
        radius: f(d, "radius", 0.0),
        frame: frame_part(&d["frame"]),
        // No partClass: a decoration, which takes nothing from the tokens
        // (no outline, corner or elevation); a card or panel names its class.
        part: d["part"].as_str().and_then(|sel| {
            let class = d["partClass"]
                .as_str()
                .and_then(og_ui::tokens::ElementClass::from_name);
            og_ui::dress::PartRef::parse(sel, class).ok()
        }),
        aspect: f(d, "aspect", 0.0).max(0.0),
    }
}

pub(super) fn frame_part(v: &Value) -> Option<StylePart> {
    StylePart::from_name(v.as_str()?)
}

pub(super) fn align(v: &Value) -> Align {
    match v.as_str() {
        Some("center") => Align::Center,
        Some("end") => Align::End,
        Some("spaceBetween") => Align::SpaceBetween,
        _ => Align::Start,
    }
}

pub(super) fn cross(v: &Value) -> Cross {
    match v.as_str() {
        Some("center") => Cross::Center,
        Some("end") => Cross::End,
        Some("stretch") => Cross::Stretch,
        Some("baseline") => Cross::Baseline,
        _ => Cross::Start,
    }
}

pub(super) fn response(r: Response) -> Value {
    json!({
        "hovered": r.hovered, "pressed": r.pressed, "clicked": r.clicked,
        "changed": r.changed, "focused": r.focused, "submitted": r.submitted,
        "dropped": r.dropped, "pad": r.pad, "focusedBy": r.focused_by, "adjust": r.adjust, "adjustY": r.adjust_y, "ring": r.ring,
        "tapped": r.tapped, "doubleTapped": r.double_tapped, "longPressed": r.long_pressed,
        "longPress": r.long_press,
    })
}

// ── ui: the calls ───────────────────────────────────────────────────────────

/// Runs `f` on the UI whose frame is open; outside a frame, logs a warning
/// on the engine that opened one last and answers `None`.
pub(crate) fn with_ui<T>(call: &str, f: impl FnOnce(&mut Ui) -> T) -> Option<T> {
    let building = BUILDING.with(Cell::get);
    let target = building.or_else(|| LAST.with(Cell::get));
    let eng = target.and_then(|id| engine(id).ok())?;
    if building.is_none() {
        if let Ok(mut st) = eng.state.try_borrow_mut() {
            let text = format!(
                "{call} was called outside a system: og.ui calls build the frame the engine opens before the game's systems and closes after them, so they do nothing anywhere else"
            );
            if st.ui_warned.insert(text.clone()) {
                st.log
                    .push(json!({ "tick": 0, "warning": true, "text": text }));
            }
        }
        return None;
    }
    let ui = eng.state.borrow().ui.clone()?;
    let mut guard = ui.lock();
    Some(f(&mut guard))
}

pub fn og_ui_viewport(world: u32) -> String {
    // In og-ui units, as the frame lays out (og.ui.frameViewport).
    let css = viewport_of(world);
    let (hud, unit) = engine(world)
        .ok()
        .and_then(|e| e.state.borrow().ui.clone())
        .map_or((og_ui::HUD_MAX_ASPECT, 1.0), |ui| {
            let ui = ui.lock();
            (ui.hud_max_aspect(), ui.unit_for(&css))
        });
    viewport_json(&css.zoomed(unit), hud).to_string()
}

pub fn og_ui_theme(world: u32) -> String {
    let (theme, sizes, style) = engine(world)
        .ok()
        .and_then(|e| e.state.borrow().ui.clone())
        .map(|ui| {
            let ui = ui.lock();
            (*ui.theme(), ui.sizes(), ui.style_name().to_string())
        })
        .unwrap_or_else(|| {
            let t = Theme::default();
            let s = t.sizes(&Viewport::default());
            (t, s, String::new())
        });
    theme_json(theme, sizes, &style)
}

pub fn og_ui_frame_viewport() -> String {
    let (v, hud) = with_ui("og.ui.frameViewport", |ui| {
        (ui.viewport(), ui.hud_max_aspect())
    })
    .unwrap_or((Viewport::default(), og_ui::HUD_MAX_ASPECT));
    // The UI kit asks many times a tick (each component that adapts to the
    // surface) and the viewport rarely changes: the last answer is kept with
    // what it was made from, as og_ui_frame_theme's is. Built each call, it
    // was a fifth of a kit frame in the dev build.
    type Made = (Viewport, f32, String);
    thread_local!(static MADE: std::cell::RefCell<Option<Made>> = const { std::cell::RefCell::new(None) });
    MADE.with(|last| {
        let mut last = last.borrow_mut();
        if let Some((lv, lh, text)) = last.as_ref()
            && *lv == v
            && lh.to_bits() == hud.to_bits()
        {
            return text.clone();
        }
        let text = viewport_json(&v, hud).to_string();
        *last = Some((v, hud, text.clone()));
        text
    })
}

pub fn og_ui_frame_cursor() -> String {
    // The last completed frame's: the one the host shows now.
    let c = with_ui("og.ui.frameCursor", |ui| ui.frame().cursor).unwrap_or_default();
    match c {
        og_ui::Cursor::Auto => og_ui::Cursor::Default,
        c => c,
    }
    .name()
    .to_string()
}

pub fn og_ui_frame_pointer() -> String {
    with_ui("og.ui.framePointer", |ui| {
        let i = ui.input();
        json!({
            "x": i.pointer.x, "y": i.pointer.y, "inside": i.pointer_inside, "down": i.down,
            "pressed": i.pressed, "released": i.released, "shift": i.shift, "ctrl": i.ctrl, "alt": i.alt,
        })
    })
    .unwrap_or_else(|| {
        json!({ "x": 0.0, "y": 0.0, "inside": false, "down": false, "pressed": false, "released": false, "shift": false, "ctrl": false, "alt": false })
    })
    .to_string()
}

pub fn og_ui_frame_direction() -> String {
    let rtl = with_ui("og.ui.frameDirection", |ui| {
        ui.direction() == og_i18n::Direction::Rtl
    })
    .unwrap_or(false);
    if rtl { "rtl" } else { "ltr" }.to_owned()
}

pub fn og_ui_set_focus(path: String) {
    with_ui("og.ui.setFocus", |ui| ui.set_focus(&path));
}

pub fn og_ui_scroll_into_view(path: String, center: bool) {
    with_ui("og.ui.scrollIntoView", |ui| {
        ui.scroll_into_view_path(&path, center)
    });
}

pub fn og_ui_last_path() -> String {
    with_ui("og.ui.lastPath", |ui| ui.last_path()).unwrap_or_default()
}

pub fn og_ui_has_icon(name: String) -> bool {
    with_ui("og.ui.hasIcon", |ui| ui.has_icon(&name)).unwrap_or(false)
}

pub fn og_ui_seconds() -> f64 {
    with_ui("og.ui.seconds", |ui| ui.seconds()).unwrap_or(0.0)
}

pub fn og_ui_frame_theme() -> String {
    let (theme, sizes, style) = with_ui("og.ui.frameTheme", |ui| {
        (*ui.theme(), ui.sizes(), ui.style_name().to_string())
    })
    .unwrap_or_else(|| {
        let t = Theme::default();
        let s = t.sizes(&Viewport::default());
        (t, s, String::new())
    });
    // The UI kit asks once a tick and the theme rarely changes: the last
    // answer is kept with what it was made from.
    type Made = (Theme, og_ui::Density, String, String);
    thread_local!(static MADE: std::cell::RefCell<Option<Made>> = const { std::cell::RefCell::new(None) });
    MADE.with(|last| {
        let mut last = last.borrow_mut();
        if let Some((t, d, k, text)) = last.as_ref()
            && *t == theme
            && *d == sizes
            && *k == style
        {
            return text.clone();
        }
        let text = theme_json(theme, sizes, &style);
        *last = Some((theme, sizes, style, text.clone()));
        text
    })
}

fn theme_json(theme: Theme, sizes: og_ui::Density, style: &str) -> String {
    let hex = |c: Srgba| {
        let [r, g, b] = c.rgb;
        if c.alpha >= 1.0 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!(
                "#{r:02x}{g:02x}{b:02x}{:02x}",
                (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8
            )
        }
    };
    let mut palette = Map::new();
    for name in THEME_COLORS {
        if let Some(c) = token(&theme, name) {
            palette.insert(name.to_string(), json!(hex(c)));
        }
    }
    for (name, c) in theme.roles.iter() {
        palette.insert(name.to_string(), json!(hex(c)));
    }
    let mut ramp = Map::new();
    for name in TYPE_STEPS {
        if let Some(s) = step(&theme, name) {
            ramp.insert(
                name.to_string(),
                json!({ "size": s.size, "lineHeight": s.line_height, "trackingEm": s.tracking_em, "cjkLineHeight": s.cjk_line_height, "cjkTrackingEm": s.cjk_tracking_em, "weight": s.weight, "uppercase": s.uppercase, "features": s.features.tags().collect::<Vec<_>>() }),
            );
        }
    }
    let sp = theme.space;
    let r = theme.radius;
    let d = theme.durations_ms;
    json!({
        "scheme": if theme.scheme == Scheme::Light { "light" } else { "dark" },
        "palette": palette,
        "space": { "xxs": sp.xxs, "xs": sp.xs, "sm": sp.sm, "md": sp.md, "lg": sp.lg, "xl": sp.xl, "xxl": sp.xxl, "xl3": sp.xl3 },
        "radius": { "small": r.small, "control": r.control, "bar": r.bar, "panel": r.panel },
        "typeRamp": ramp,
        "sizes": { "controlSm": sizes.control_sm, "controlMd": sizes.control_md, "controlLg": sizes.control_lg, "gutter": sizes.gutter, "header": sizes.header, "row": sizes.row, "section": sizes.section },
        "durationsMs": { "instant": d.instant, "fast": d.fast, "base": d.base, "slow": d.slow, "spring": d.spring, "bounce": d.bounce },
        "style": style,
        "styleVersion": if theme.art.legacy { 1 } else { 2 },
        "contrast": if theme.contrast == og_ui::Contrast::High { "high" } else { "standard" },
        "reducedMotion": theme.reduced_motion,
        "reducedTransparency": theme.reduced_transparency,
        "textScale": theme.text_scale,
        "unit": theme.unit,
        "tokens": tokens_json(&theme.tokens),
    })
    .to_string()
}

/// The style's tokens (docs/ui.md §12.2) as og.ui.theme answers them
/// (`UiTheme.tokens`): type by role and breakpoint, the space, size,
/// radius and stroke scales, elevation, the shape language per element
/// class, the springs, the rarity, team and series colours, and each UI
/// event's feedback set.
fn tokens_json(t: &og_ui::StyleTokens) -> Value {
    use og_ui::tokens::*;
    let font = |f: og_ui::FontRole| match f {
        og_ui::FontRole::Body => "body",
        og_ui::FontRole::Display => "display",
        og_ui::FontRole::Mono => "mono",
        og_ui::FontRole::Rounded => "rounded",
    };
    let mut typography = Map::new();
    for &role in TypeRole::ALL {
        let mut by = Map::new();
        for bp in og_ui::Breakpoint::ALL {
            let k = t.typography.get(role, bp);
            by.insert(
                bp.name().to_string(),
                json!({ "size": k.size, "lineHeight": k.line_height, "trackingEm": k.tracking_em, "cjkTrackingEm": k.cjk_tracking_em, "weight": k.weight, "font": font(k.font), "uppercase": k.uppercase, "features": k.features.tags().collect::<Vec<_>>() }),
            );
        }
        typography.insert(role.name().to_string(), Value::Object(by));
    }
    let scale = |names: &[&str], v: &[f32]| -> Value {
        Value::Object(
            names
                .iter()
                .zip(v)
                .map(|(n, v)| (n.to_string(), json!(v)))
                .collect(),
        )
    };
    let names = |all: &[&'static str]| all.to_vec();
    let space = scale(
        &names(&SpaceStep::ALL.iter().map(|s| s.name()).collect::<Vec<_>>()),
        &t.space,
    );
    let size = scale(
        &names(&SizeStep::ALL.iter().map(|s| s.name()).collect::<Vec<_>>()),
        &t.size,
    );
    let radius = scale(
        &names(&RadiusStep::ALL.iter().map(|s| s.name()).collect::<Vec<_>>()),
        &t.radius,
    );
    let stroke = scale(
        &names(
            &StrokeWidth::ALL
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>(),
        ),
        &t.stroke,
    );
    let elevation: Map<String, Value> = Level::ALL
        .iter()
        .map(|&l| {
            let shadows: Vec<Value> = t
                .elevation(l)
                .shadows()
                .iter()
                .map(|s| json!({ "dx": s.dx, "dy": s.dy, "blur": s.blur, "spread": s.spread, "color": hex(s.color) }))
                .collect();
            (l.name().to_string(), json!(shadows))
        })
        .collect();
    let shape: Map<String, Value> = ElementClass::ALL
        .iter()
        .map(|&c| {
            let k = t.shape(c);
            (
                c.name().to_string(),
                json!({ "corner": k.corner.name(), "radius": k.radius }),
            )
        })
        .collect();
    let springs: Map<String, Value> = SpringName::ALL
        .iter()
        .map(|&n| {
            let k = t.spring(n);
            (
                n.name().to_string(),
                json!({ "durationMs": k.duration_ms, "bounce": k.bounce }),
            )
        })
        .collect();
    let swatches = |v: &[Swatch]| -> Vec<Value> { v.iter().map(|s| json!(hex(s.color))).collect() };
    let rarity: Map<String, Value> = Rarity::ALL
        .iter()
        .map(|&r| (r.name().to_string(), json!(hex(t.rarity(r).color))))
        .collect();
    let feedback: Map<String, Value> = SoundEvent::ALL
        .iter()
        .map(|&e| {
            let f = t.feedback(e);
            let pulses: Vec<[f32; 4]> = f.haptic.pulses().iter().map(|p| [p.start_ms, p.ms, p.low, p.high]).collect();
            (
                e.name().to_string(),
                json!({ "sounds": f.count, "volume": f.volume, "pitch": f.pitch, "haptic": pulses }),
            )
        })
        .collect();
    json!({
        "typography": typography,
        "space": space,
        "size": size,
        "radius": radius,
        "stroke": stroke,
        "elevation": elevation,
        "shape": shape,
        "springs": springs,
        "staggerMs": t.motion.stagger_ms,
        "overshoot": t.motion.overshoot,
        "rarity": rarity,
        "team": swatches(&t.colors.team),
        "series": t.colors.series.iter().map(|c| hex(*c)).collect::<Vec<_>>(),
        "feedback": feedback,
    })
}

/// An `og.ui.Customize` desc (null: not customisable).
pub(super) fn customize(v: &Value) -> Option<og_ui::Customize> {
    v.is_object().then(|| og_ui::Customize {
        key: v["key"].as_str().unwrap_or_default().to_owned(),
        lock_move: v["lockMove"].as_bool().unwrap_or(false),
        lock_scale: v["lockScale"].as_bool().unwrap_or(false),
        lock_opacity: v["lockOpacity"].as_bool().unwrap_or(false),
        min_scale: f(v, "minScale", 0.0).max(0.0),
        max_scale: f(v, "maxScale", 0.0).max(0.0),
        min_opacity: f(v, "minOpacity", 0.0).clamp(0.0, 1.0),
    })
}

pub(super) fn opt_f(v: &Value, k: &str) -> Option<f32> {
    v[k].as_f64().map(|x| x as f32)
}

pub(super) fn tone(v: &Value) -> Tone {
    match v.as_str() {
        Some("success") => Tone::Success,
        Some("warn") => Tone::Warn,
        Some("danger") => Tone::Danger,
        Some("info") => Tone::Info,
        Some("primary") => Tone::Primary,
        Some("inverse") => Tone::Inverse,
        _ => Tone::Neutral,
    }
}

pub(crate) fn anchor_of(v: &Value) -> Option<Anchor> {
    Some(match v.as_str()? {
        "topLeft" => Anchor::TopLeft,
        "top" => Anchor::Top,
        "topRight" => Anchor::TopRight,
        "left" => Anchor::Left,
        "center" => Anchor::Center,
        "right" => Anchor::Right,
        "bottomLeft" => Anchor::BottomLeft,
        "bottom" => Anchor::Bottom,
        "bottomRight" => Anchor::BottomRight,
        _ => return None,
    })
}

pub(super) fn transition_kind(v: &Value) -> Option<TransitionKind> {
    Some(match v.as_str()? {
        "fade" => TransitionKind::Fade,
        "slideUp" => TransitionKind::SlideUp,
        "slideDown" => TransitionKind::SlideDown,
        "slideLeft" => TransitionKind::SlideLeft,
        "slideRight" => TransitionKind::SlideRight,
        "scale" => TransitionKind::Scale,
        "pop" => TransitionKind::Pop,
        "glitch" => TransitionKind::Glitch,
        _ => return None,
    })
}

pub(super) fn idx(v: &Value) -> Option<u32> {
    v.as_u64().map(|n| n as u32)
}

pub(super) fn opt(v: Option<u32>) -> Value {
    v.map_or(Value::Null, |n| json!(n))
}

pub(super) fn rect_json(r: &og_ui::Rect) -> Value {
    json!({ "x": r.x, "y": r.y, "width": r.width, "height": r.height })
}

/// The engine whose UI frame is open, else the one opened last.
pub(crate) fn ui_engine() -> Option<u32> {
    BUILDING.with(Cell::get).or_else(|| LAST.with(Cell::get))
}

pub(crate) fn texture_of(v: &Value) -> Option<og_render::TextureHandle> {
    let t = v.as_f64()?;
    let eng = engine(ui_engine()?).ok()?;
    let rs = eng.render.borrow();
    super::assets::index_of(t).and_then(|i| rs.textures.get(i).copied())
}

pub(super) fn uv_of(v: &Value) -> Option<[f32; 4]> {
    v.as_array().filter(|a| a.len() == 4).map(|a| {
        let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
        [n(0), n(1), n(2), n(3)]
    })
}

#[cfg(test)]
mod tests {
    use og_ui::tokens::ElementClass;
    use serde_json::json;

    /// A part with no partClass is a decoration (no class, so nothing from
    /// the tokens); one that names its class keeps it. Defaulting to card
    /// drew every undressed scrim, sweep and node as a hairline card.
    #[test]
    fn a_part_without_a_class_is_a_decoration() {
        let theme = og_ui::Theme::dark();
        let el = super::element(&theme, &json!({ "part": "UnitFrame.scrim" }));
        assert_eq!(el.part.map(|p| p.class), Some(None));
        let el = super::element(&theme, &json!({ "part": "Shop.card", "partClass": "card" }));
        assert_eq!(el.part.map(|p| p.class), Some(Some(ElementClass::Card)));
    }
}
