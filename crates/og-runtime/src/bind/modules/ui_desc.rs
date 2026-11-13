use super::*;
use std::collections::BTreeMap;

// ── ui: decoding descs ──────────────────────────────────────────────────────

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
pub(super) fn paint(theme: &Theme, p: &wire::Paint) -> Option<Srgba> {
    let alpha = p.alpha.clamp(0.0, 1.0);
    let name = p.token.map(|t| t.as_str()).or(p.role.as_deref());
    let base = match name {
        Some(t) => token(theme, t)?,
        None => match &p.color {
            Some(c) => {
                let ch = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
                Srgba::rgb(ch(c.r), ch(c.g), ch(c.b))
            }
            None => Srgba::WHITE,
        },
    };
    Some(base.fade(alpha))
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

pub fn og_ui_viewport(world: u32) -> crate::host::wire::Viewport {
    // In og-ui units, as the frame lays out (og.ui.frameViewport).
    let css = viewport_of(world);
    let (hud, unit) = engine(world)
        .ok()
        .and_then(|e| e.state.borrow().ui.clone())
        .map_or((og_ui::HUD_MAX_ASPECT, 1.0), |ui| {
            let ui = ui.lock();
            (ui.hud_max_aspect(), ui.unit_for(&css))
        });
    viewport_wire(&css.zoomed(unit), hud)
}

pub fn og_ui_theme(world: u32) -> crate::host::wire::UiTheme {
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
    theme_wire(theme, sizes, &style)
}

/// A viewport as og.ui.viewport answers it, with the HUD band the UI's
/// `hud_max_aspect` makes of it (og-ui's `Ui::hud_area`).
fn viewport_wire(v: &Viewport, hud_max_aspect: f32) -> crate::host::wire::Viewport {
    let [small, large] = v.extremes();
    wire::Viewport {
        width: v.width,
        height: v.height,
        dpr: v.dpr,
        safe_area: wire::Insets {
            left: v.safe_area.left,
            top: v.safe_area.top,
            right: v.safe_area.right,
            bottom: v.safe_area.bottom,
        },
        small_width: Some(small[0]),
        small_height: Some(small[1]),
        large_width: Some(large[0]),
        large_height: Some(large[1]),
        coarse_pointer: v.coarse_pointer,
        reduced_motion: Some(v.reduced_motion),
        high_contrast: Some(v.high_contrast),
        reduced_transparency: Some(v.reduced_transparency),
        mac_keys: Some(v.mac_keys),
        covered: Some(rect_wire(&v.covered)),
        breakpoint: match v.breakpoint() {
            og_ui::Breakpoint::PhonePortrait => wire::Breakpoint::PhonePortrait,
            og_ui::Breakpoint::PhoneLandscape => wire::Breakpoint::PhoneLandscape,
            og_ui::Breakpoint::Tablet => wire::Breakpoint::Tablet,
            og_ui::Breakpoint::Desktop => wire::Breakpoint::Desktop,
        },
        hud: Some(rect_wire(&v.hud_rect(hud_max_aspect))),
    }
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
        let text = wire::text_of(&viewport_wire(&v, hud));
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

pub fn og_ui_frame_pointer() -> crate::host::wire::UiPointer {
    with_ui("og.ui.framePointer", |ui| {
        let i = ui.input();
        wire::UiPointer {
            x: i.pointer.x,
            y: i.pointer.y,
            inside: i.pointer_inside,
            down: i.down,
            pressed: i.pressed,
            released: i.released,
            shift: i.shift,
            ctrl: i.ctrl,
            alt: i.alt,
        }
    })
    .unwrap_or_default()
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
        let text = wire::text_of(&theme_wire(theme, sizes, &style));
        *last = Some((theme, sizes, style, text.clone()));
        text
    })
}

fn theme_wire(theme: Theme, sizes: og_ui::Density, style: &str) -> crate::host::wire::UiTheme {
    let mut palette = BTreeMap::new();
    for name in THEME_COLORS {
        if let Some(c) = token(&theme, name) {
            palette.insert(name.to_string(), hex(c));
        }
    }
    for (name, c) in theme.roles.iter() {
        palette.insert(name.to_string(), hex(c));
    }
    let mut type_ramp = BTreeMap::new();
    for name in TYPE_STEPS {
        if let Some(s) = step(&theme, name) {
            type_ramp.insert(
                name.to_string(),
                wire::TypeStep {
                    size: s.size,
                    line_height: s.line_height,
                    tracking_em: s.tracking_em,
                    cjk_line_height: s.cjk_line_height,
                    cjk_tracking_em: s.cjk_tracking_em,
                    weight: s.weight.into(),
                    uppercase: s.uppercase,
                    features: s.features.tags().map(str::to_string).collect(),
                },
            );
        }
    }
    let named = |v: &[(&str, f32)]| -> BTreeMap<String, f32> {
        v.iter().map(|&(k, x)| (k.to_string(), x)).collect()
    };
    let sp = theme.space;
    let r = theme.radius;
    let d = theme.durations_ms;
    wire::UiTheme {
        scheme: match theme.scheme {
            Scheme::Light => wire::Scheme::Light,
            Scheme::Dark => wire::Scheme::Dark,
        },
        palette,
        space: named(&[
            ("xxs", sp.xxs),
            ("xs", sp.xs),
            ("sm", sp.sm),
            ("md", sp.md),
            ("lg", sp.lg),
            ("xl", sp.xl),
            ("xxl", sp.xxl),
            ("xl3", sp.xl3),
        ]),
        radius: named(&[
            ("small", r.small),
            ("control", r.control),
            ("bar", r.bar),
            ("panel", r.panel),
        ]),
        type_ramp,
        sizes: wire::ControlSizes {
            control_sm: sizes.control_sm,
            control_md: sizes.control_md,
            control_lg: sizes.control_lg,
            gutter: sizes.gutter,
            header: sizes.header,
            row: sizes.row,
            section: sizes.section,
        },
        durations_ms: named(&[
            ("instant", d.instant),
            ("fast", d.fast),
            ("base", d.base),
            ("slow", d.slow),
            ("spring", d.spring),
            ("bounce", d.bounce),
        ]),
        style: style.to_string(),
        style_version: if theme.art.legacy { 1 } else { 2 },
        contrast: match theme.contrast {
            og_ui::Contrast::Standard => wire::Contrast::Standard,
            og_ui::Contrast::High => wire::Contrast::High,
        },
        reduced_motion: theme.reduced_motion,
        reduced_transparency: theme.reduced_transparency,
        color_vision: match theme.color_vision {
            og_render::ColorVision::Typical => wire::ColorVision::None,
            og_render::ColorVision::Protanopia => wire::ColorVision::Protanopia,
            og_render::ColorVision::Deuteranopia => wire::ColorVision::Deuteranopia,
            og_render::ColorVision::Tritanopia => wire::ColorVision::Tritanopia,
        },
        color_vision_filter: match theme.color_vision_filter {
            og_render::ColorVisionFilter::Correct => wire::ColorVisionFilter::Correct,
            og_render::ColorVisionFilter::Simulate => wire::ColorVisionFilter::Simulate,
        },
        color_vision_strength: theme.color_vision_strength,
        text_scale: theme.text_scale,
        unit: theme.unit,
        tokens: tokens_json(&theme.tokens),
    }
}

/// A colour as `#rrggbb`, or `#rrggbbaa` when it is translucent.
fn hex(c: Srgba) -> String {
    let [r, g, b] = c.rgb;
    if c.alpha >= 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!(
            "#{r:02x}{g:02x}{b:02x}{:02x}",
            (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8
        )
    }
}

/// The style's tokens (docs/ui.md §3.1) as og.ui.theme answers them
/// (`UiTheme.tokens`): type by role and breakpoint, the space, size,
/// radius and stroke scales, elevation, the shape language per element
/// class, the springs, the rarity, team and series colours, and each UI
/// event's feedback set.
fn tokens_json(t: &og_ui::StyleTokens) -> Value {
    use og_ui::tokens::*;
    // A token's type names a role, never a document's family.
    let font = |f: og_ui::FontRole| match f.base() {
        og_ui::FontRole::Display => "display",
        og_ui::FontRole::Mono => "mono",
        og_ui::FontRole::Rounded => "rounded",
        _ => "body",
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
    // What carries each colour token's meaning besides its hue (docs/ui.md
    // §3.2): an icon (its index in the style's icons), a pattern or pips.
    let cues: Map<String, Value> = colour_token_names()
        .into_iter()
        .filter_map(|name| {
            let v = match t.colors.cue(&name)? {
                Cue::None => return None,
                Cue::Icon(i) => json!({ "icon": i }),
                Cue::Pattern(p) => json!({ "pattern": p.name() }),
                Cue::Pips(n) => json!({ "pips": n }),
            };
            Some((name, v))
        })
        .collect();
    let feedback: Map<String, Value> = SoundEvent::ALL
        .iter()
        .map(|&e| {
            let f = t.feedback(e);
            let pulses: Vec<[f32; 4]> = f.haptic.pulses().iter().map(|p| [p.start_ms, p.ms, p.low, p.high]).collect();
            (
                e.name().to_string(),
                json!({ "sounds": f.count, "volume": f.volume, "playbackRate": f.pitch, "haptic": pulses }),
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
        "cues": cues,
        "team": swatches(&t.colors.team),
        "series": t.colors.series.iter().map(|c| hex(*c)).collect::<Vec<_>>(),
        "feedback": feedback,
    })
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

/// A rectangle as the IDL's `UiRect`.
pub(super) fn rect_wire(r: &og_ui::Rect) -> crate::host::wire::UiRect {
    wire::UiRect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

/// The engine whose UI frame is open, else the one opened last.
pub(crate) fn ui_engine() -> Option<u32> {
    BUILDING.with(Cell::get).or_else(|| LAST.with(Cell::get))
}

pub(crate) fn texture_of(t: Option<u64>) -> Option<og_render::TextureHandle> {
    let t = t? as f64;
    let eng = engine(ui_engine()?).ok()?;
    let rs = eng.render.borrow();
    super::assets::index_of(t).and_then(|i| rs.textures.get(i).copied())
}
