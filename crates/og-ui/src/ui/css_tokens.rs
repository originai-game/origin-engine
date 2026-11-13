//! Theme is the widget-facing projection of the root CSS tokens.
use super::*;
use crate::Palette;
use std::collections::BTreeMap;

const TYPE_NAMES: [&str; 21] = [
    "xl2",
    "xl3",
    "xl4",
    "xl5",
    "xl6",
    "xl7",
    "xl8",
    "base",
    "body",
    "caption",
    "display-xl2",
    "display-lg",
    "display-md",
    "display-sm",
    "display-xl",
    "lg",
    "meta",
    "reading",
    "sm",
    "xl",
    "xs",
];

fn step_values(name: &str, step: TypeStep) -> [(String, String); 8] {
    [
        ("size", format!("{}px", step.size)),
        ("line-height", format!("{}px", step.line_height)),
        ("weight", step.weight.to_string()),
        ("tracking", format!("{}em", step.tracking_em)),
        ("cjk-tracking", format!("{}em", step.cjk_tracking_em)),
        ("cjk-line-height", format!("{}px", step.cjk_line_height)),
        (
            "features",
            step.features
                .tags()
                .map(|tag| format!("\"{tag}\""))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        (
            "transform",
            if step.uppercase { "uppercase" } else { "none" }.into(),
        ),
    ]
    .map(|(field, value)| (format!("--type-{name}-{field}"), value))
}

fn typography(theme: &Theme, at: Breakpoint, fonts: &Fonts) -> BTreeMap<String, String> {
    let mut ramp = theme.tokens.typography.ramp(at, 1.0);
    TYPE_NAMES
        .into_iter()
        .zip(ramp.steps_mut())
        .flat_map(|(name, step)| step_values(name, *step))
        .chain(
            crate::tokens::TypeRole::ALL
                .iter()
                .copied()
                .flat_map(|role| {
                    let token = theme.tokens.typography.get(role, at);
                    let family = match token.font {
                        FontRole::Family(family) => {
                            let names = fonts.family_names(family);
                            (!names.is_empty()).then(|| {
                                names
                                    .iter()
                                    .map(FontFamily::css)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                        }
                        _ => None,
                    }
                    .unwrap_or_else(|| {
                        format!(
                            "var(--font-{})",
                            match token.font.base() {
                                FontRole::Display => "display",
                                FontRole::Mono => "mono",
                                FontRole::Rounded => "rounded",
                                _ => "body",
                            }
                        )
                    });
                    step_values(role.name(), token.step(1.0))
                        .into_iter()
                        .chain([(format!("--type-{}-font", role.name()), family)])
                }),
        )
        .collect()
}

fn lengths(theme: &mut Theme, mut visit: impl FnMut(&str, &mut f32)) {
    macro_rules! fields {
        ($group:ident; $($field:ident => $name:literal),* $(,)?) => {
            $(visit($name, &mut theme.$group.$field);)*
        };
    }
    fields!(space; xxs => "space-xxs", xs => "space-xs", sm => "space-sm",
        md => "space-md", lg => "space-lg", xl => "space-xl", xxl => "space-xxl", xl3 => "space-xl3");
    fields!(radius; small => "radius-control", control => "radius-button",
        panel => "radius-panel", bar => "radius-capsule");
    macro_rules! scale {
        ($field:ident, $step:ident) => {
            for step in crate::tokens::$step::ALL {
                visit(
                    &format!("{}-{}", stringify!($field), step.name()),
                    &mut theme.tokens.$field[step.index()],
                );
            }
        };
    }
    scale!(space, SpaceStep);
    scale!(size, SizeStep);
    scale!(radius, RadiusStep);
    scale!(stroke, StrokeWidth);
    macro_rules! steps {
        ($($field:ident => $name:literal),* $(,)?) => {$(
            visit(concat!("type-", $name, "-size"), &mut theme.type_ramp.$field.size);
            visit(concat!("type-", $name, "-line-height"), &mut theme.type_ramp.$field.line_height);
        )*};
    }
    steps!(xl2 => "xl2", xl3 => "xl3", xl4 => "xl4", xl5 => "xl5", xl6 => "xl6",
        xl7 => "xl7", xl8 => "xl8", base => "base", body => "body", caption => "caption",
        display_xl2 => "display-xl2", display_lg => "display-lg", display_md => "display-md",
        display_sm => "display-sm", display_xl => "display-xl", lg => "lg", meta => "meta",
        reading => "reading", sm => "sm", xl => "xl", xs => "xs");
}

/// The unit a root token's number is written in.
#[derive(Clone, Copy)]
enum Unit {
    Px,
    Ms,
    Number,
}

impl Unit {
    fn suffix(self) -> &'static str {
        match self {
            Unit::Px => "px",
            Unit::Ms => "ms",
            Unit::Number => "",
        }
    }

    /// A computed root value as a number of this unit (`calc()` reduced).
    fn read(self, value: &str) -> Option<f32> {
        let value = crate::css::calc::reduce(value).unwrap_or_else(|_| value.to_owned());
        let value = value.trim();
        match self {
            Unit::Ms => value
                .strip_suffix("ms")
                .and_then(|v| v.trim().parse().ok())
                .or_else(|| {
                    value
                        .strip_suffix('s')
                        .and_then(|v| v.trim().parse::<f32>().ok())
                        .map(|s| s * 1000.0)
                }),
            unit => value.strip_suffix(unit.suffix())?.trim().parse().ok(),
        }
    }
}

/// Every number the theme reads from the root CSS tokens, by its variable
/// name without `--`: the lengths, the motion durations, the press scales
/// and control states, each input's densities, the named transitions'
/// springs and the controls' own values.
fn scalars(theme: &mut Theme, mut visit: impl FnMut(&str, Unit, &mut f32)) {
    lengths(theme, |name, value| visit(name, Unit::Px, value));
    let d = &mut theme.durations_ms;
    for (name, value) in [
        ("base", &mut d.base),
        ("bounce", &mut d.bounce),
        ("fast", &mut d.fast),
        ("instant", &mut d.instant),
        ("slow", &mut d.slow),
        ("spring", &mut d.spring),
    ] {
        visit(&format!("motion-duration-{name}"), Unit::Ms, value);
    }
    visit("state-press-scale", Unit::Number, &mut theme.press_scale);
    let s = &mut theme.state;
    visit("state-hover-wash", Unit::Number, &mut s.hover_wash);
    visit(
        "state-control-press-scale",
        Unit::Number,
        &mut s.control_press_scale,
    );
    visit("state-lift", Unit::Px, &mut s.lift);
    visit(
        "state-disabled-opacity",
        Unit::Number,
        &mut s.disabled_opacity,
    );
    visit("touch-target", Unit::Px, &mut s.touch_target);
    let ds = &mut theme.densities;
    for (set, d) in [
        ("compact", &mut ds.mouse),
        ("pad", &mut ds.pad),
        ("touch", &mut ds.touch),
    ] {
        for (field, value) in density_fields(d) {
            visit(&format!("density-{set}-{field}"), Unit::Px, value);
        }
    }
    let m = &mut theme.motion;
    for (name, spring) in [
        ("ui", &mut m.ui),
        ("panel", &mut m.panel),
        ("emphasis", &mut m.emphasis),
        ("playful", &mut m.playful),
    ] {
        visit(
            &format!("motion-named-{name}-duration"),
            Unit::Ms,
            &mut spring.duration_ms,
        );
        visit(
            &format!("motion-named-{name}-bounce"),
            Unit::Number,
            &mut spring.bounce,
        );
    }
    visit("motion-named-enter-duration", Unit::Ms, &mut m.enter_ms);
    visit("motion-named-exit-duration", Unit::Ms, &mut m.exit_ms);
    for (name, unit, value) in theme.controls.fields_mut() {
        let unit = match unit {
            "px" => Unit::Px,
            "ms" => Unit::Ms,
            _ => Unit::Number,
        };
        visit(name, unit, value);
    }
}

/// A density's sizes by their CSS names.
fn density_fields(d: &mut crate::Density) -> [(&'static str, &mut f32); 7] {
    [
        ("control-sm", &mut d.control_sm),
        ("control-md", &mut d.control_md),
        ("control-lg", &mut d.control_lg),
        ("row", &mut d.row),
        ("header", &mut d.header),
        ("gutter", &mut d.gutter),
        ("section", &mut d.section),
    ]
}

/// The eases the theme reads from the root CSS tokens.
fn eases(theme: &mut Theme, mut visit: impl FnMut(&str, &mut crate::motion::Ease)) {
    for (name, bezier) in [
        ("motion-easing-out", &mut theme.ease_out),
        ("motion-easing-exit", &mut theme.ease_exit),
    ] {
        let mut ease = crate::motion::Ease::CubicBezier(*bezier);
        visit(name, &mut ease);
        if let crate::motion::Ease::CubicBezier(b) = ease {
            *bezier = b;
        }
    }
    visit("motion-named-enter-easing", &mut theme.motion.enter_ease);
    visit("motion-named-exit-easing", &mut theme.motion.exit_ease);
}

/// The root tokens of a theme's numbers and eases, `--name` and value, in
/// visiting order (a name two fields read, `space-md` of the v1 scale and
/// the token scale, comes twice).
fn scalar_css(theme: &Theme) -> Vec<(String, String)> {
    let mut theme = *theme;
    let mut out = Vec::new();
    scalars(&mut theme, |name, unit, value| {
        out.push((format!("--{name}"), format!("{value}{}", unit.suffix())));
    });
    eases(&mut theme, |name, ease| {
        out.push((format!("--{name}"), ease.to_text()));
    });
    out
}

impl Theme {
    /// The root tokens that select the density `viewport` takes: each
    /// `--density-active-*` the input's set, which `--control-sm` …
    /// `--section` read.
    pub(crate) fn density_tokens(&self, viewport: &crate::Viewport) -> Vec<(String, String)> {
        let (set, mut d) = self.active_density(viewport);
        density_fields(&mut d)
            .map(|(field, _)| {
                (
                    format!("--density-active-{field}"),
                    format!("var(--density-{set}-{field})"),
                )
            })
            .into()
    }
}

fn palette_css(theme: &Theme) -> BTreeMap<String, String> {
    Palette::NAMES
        .iter()
        .map(|name| (*name, theme.palette.get(name).unwrap()))
        .chain(theme.roles.iter())
        .map(|(name, color)| {
            let [r, g, b] = color.rgb;
            (
                format!("--{}", crate::css::values::kebab(name)),
                format!("rgb({r} {g} {b} / {})", color.alpha),
            )
        })
        .collect()
}

impl Ui {
    /// Read the root CSS tokens into the theme. A frame begins with this,
    /// so the answer is kept: reading again from the same theme under the
    /// same media context, with the root inputs and the fonts unchanged,
    /// cascades nothing.
    pub fn read_theme_tokens(&mut self) {
        let source = ThemeSource {
            theme: self.env.theme,
            configured: self.env.configured,
            media: self.media_context(),
        };
        if let Some(kept) = &self.style.theme_read
            && kept.0 == source
        {
            self.env.theme = kept.1;
            return;
        }
        self.read_root_tokens();
        self.style.theme_read = Some(Box::new((source, self.env.theme)));
    }

    fn read_root_tokens(&mut self) {
        let root = self.css_root_variables();
        let context = crate::css_color::Context::of(&self.env.theme);
        let colors: Vec<_> = Palette::NAMES
            .iter()
            .copied()
            .chain(self.env.configured.roles.iter().map(|(name, _)| name))
            .filter_map(|name| {
                let value = root
                    .get(&format!("--{}", crate::css::values::kebab(name)))?
                    .as_ref()?;
                let color = crate::css_color::parse_str(value).ok()?.resolve(&context);
                Some((name.to_owned(), color))
            })
            .collect();
        for (name, color) in colors {
            if let Some(target) = self.env.theme.palette.get_mut(&name) {
                *target = color;
            } else {
                self.env
                    .theme
                    .roles
                    .set(&name, color)
                    .expect("configured role fits");
            }
        }
        scalars(&mut self.env.theme, |name, unit, target| {
            if let Some(Some(value)) = root.get(&format!("--{name}"))
                && let Some(value) = unit.read(value)
            {
                *target = value;
            }
        });
        eases(&mut self.env.theme, |name, target| {
            if let Some(Some(value)) = root.get(&format!("--{name}"))
                && let Some(ease) = crate::motion::Ease::from_text(value.trim())
                    .or_else(|| crate::motion::Ease::from_css(value))
            {
                *target = ease;
            }
        });
        // `--control-sm` … `--section` are the input's density: a root
        // rule that sets one sets that density's.
        let viewport = self.env.viewport;
        let theme = &mut self.env.theme;
        let (set, _) = theme.active_density(&viewport);
        let active = match set {
            "pad" => &mut theme.densities.pad,
            "touch" => &mut theme.densities.touch,
            _ => &mut theme.densities.mouse,
        };
        for (field, target) in density_fields(active) {
            if let Some(Some(value)) = root.get(&format!("--{field}"))
                && let Some(value) = Unit::Px.read(value)
            {
                *target = value;
            }
        }
        let read_step = |name: &str, step: &mut TypeStep| {
            let value = |field: &str| {
                root.get(&format!("--type-{name}-{field}"))
                    .and_then(Option::as_deref)
            };
            let number = |field: &str, unit: &str| {
                value(field)?.strip_suffix(unit)?.trim().parse::<f32>().ok()
            };
            if let Some(v) = number("size", "px") {
                step.size = v;
            }
            if let Some(v) = number("line-height", "px") {
                step.line_height = v;
            }
            if let Some(v) = number("weight", "") {
                step.weight = v as u16;
            }
            if let Some(v) = number("tracking", "em") {
                step.tracking_em = v;
            }
            if let Some(v) = number("cjk-tracking", "em") {
                step.cjk_tracking_em = v;
            }
            if let Some(v) = number("cjk-line-height", "px") {
                step.cjk_line_height = v;
            }
            if let Some(v) = value("features") {
                step.features = crate::shape::Features::parse(
                    v.split(',')
                        .map(|s| s.trim().trim_matches('"'))
                        .filter(|s| !s.is_empty() && *s != "normal"),
                )
                .unwrap_or_default();
            }
            if let Some(v) = value("transform") {
                step.uppercase = v == "uppercase";
            }
        };
        for (name, step) in TYPE_NAMES
            .into_iter()
            .zip(self.env.theme.type_ramp.steps_mut())
        {
            read_step(name, step);
        }
        let bp = self.breakpoint() as usize;
        for role in crate::tokens::TypeRole::ALL {
            let token = &mut self.env.theme.tokens.typography.steps[role.index()][bp];
            if let Some(Some(value)) = root.get(&format!("--type-{}-font", role.name()))
                && let Ok((names, fallback)) = crate::css::text_css::family_list(value)
            {
                let fallback = fallback
                    .strip_prefix("var(--font-")
                    .and_then(|s| s.strip_suffix(')'))
                    .and_then(FontRole::from_name)
                    .unwrap_or_default();
                token.font = self.env.fonts.family_list(&names, fallback);
            }
            let mut step = token.step(1.0);
            read_step(role.name(), &mut step);
            token.size = step.size;
            token.line_height = step.line_height;
            token.weight = step.weight;
            token.tracking_em = step.tracking_em;
            token.cjk_tracking_em = step.cjk_tracking_em;
            token.features = step.features;
            token.uppercase = step.uppercase;
        }
    }
}

impl crate::ui::Style {
    pub(super) fn write_theme_tokens(&mut self, theme: Theme, env: &crate::ui::Env) {
        let baseline = Theme::new(theme.scheme);
        let mut old = palette_css(&baseline);
        for (name, value) in scalar_css(&baseline) {
            old.entry(name).or_insert(value);
        }
        let mut values = palette_css(&theme);
        values.retain(|name, value| old.get(name) != Some(value));
        // Of a name two fields read, the first that differs is written.
        for (name, value) in scalar_css(&theme) {
            if old.get(&name) != Some(&value) {
                values.entry(name).or_insert(value);
            }
        }
        self.root_tokens = values.into_iter().collect();
        self.root_type_tokens = Breakpoint::ALL.map(|bp| {
            let baseline = typography(&Theme::new(theme.scheme), bp, &env.fonts);
            typography(&theme, bp, &env.fonts)
                .into_iter()
                .filter(|(name, value)| baseline.get(name) != Some(value))
                .collect()
        });
        self.root_inputs_changed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::TypeRole;

    fn frame(ui: &mut Ui) {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.end();
    }

    #[test]
    fn a_root_token_write_is_read_though_the_theme_it_reads_into_is_the_same() {
        let mut ui = Ui::default();
        let radius = |px: f32| format!(":root {{ --radius-button: {px}px; }}");
        let before = ui.theme().radius.control;
        // The same sheet again leaves the theme as it was: the write
        // after it reads into the theme the kept reading started from.
        ui.set_part_css(&radius(before + 4.0)).unwrap();
        ui.set_part_css(&radius(before + 4.0)).unwrap();
        assert_eq!(ui.theme().radius.control, before + 4.0);
        ui.set_part_css(&radius(before + 8.0)).unwrap();
        assert_eq!(ui.theme().radius.control, before + 8.0);
        frame(&mut ui);
        frame(&mut ui);
        assert_eq!(ui.theme().radius.control, before + 8.0);
    }

    #[test]
    fn a_frame_names_the_type_families_among_the_fonts_set_since_the_last() {
        let mut ui = Ui::default();
        ui.set_part_css(":root { --type-button-font: 'Brand', monospace; }")
            .unwrap();
        frame(&mut ui);
        assert_eq!(ui.type_font(TypeRole::Button), FontRole::Mono);
        ui.set_fonts(Fonts {
            families: vec![FamilyFace {
                family: "Brand".into(),
                weight: (400, 400),
                ..FamilyFace::default()
            }],
            ..Fonts::default()
        });
        frame(&mut ui);
        let FontRole::Family(list) = ui.type_font(TypeRole::Button) else {
            panic!("the list keeps the family it now has");
        };
        assert_eq!(ui.env.fonts.family_names(list).len(), 2);
        ui.set_fonts(Fonts::default());
        frame(&mut ui);
        assert_eq!(ui.type_font(TypeRole::Button), FontRole::Mono);
    }
}
