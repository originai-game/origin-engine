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

fn palette_css(theme: &Theme) -> BTreeMap<String, String> {
    Palette::NAMES
        .iter()
        .map(|name| (*name, theme.palette.get(name).unwrap()))
        .chain(theme.roles.iter())
        .map(|(name, color)| {
            let [r, g, b] = color.rgb;
            (
                format!("--{}", crate::doc::css::kebab(name)),
                format!("rgb({r} {g} {b} / {})", color.alpha),
            )
        })
        .collect()
}

impl Ui {
    pub(crate) fn read_theme_tokens(&mut self) {
        let root = self.css_root_variables();
        let context = crate::css_color::Context::of(&self.env.theme);
        let colors: Vec<_> = Palette::NAMES
            .iter()
            .copied()
            .chain(self.env.configured.roles.iter().map(|(name, _)| name))
            .filter_map(|name| {
                let value = root
                    .get(&format!("--{}", crate::doc::css::kebab(name)))?
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
        lengths(&mut self.env.theme, |name, target| {
            if let Some(Some(value)) = root.get(&format!("--{name}"))
                && let Ok(value) = crate::doc::calc::reduce(value)
                && let Some(value) = value
                    .strip_suffix("px")
                    .and_then(|v| v.trim().parse::<f32>().ok())
            {
                *target = value;
            }
        });
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
                && let Ok((names, fallback)) = crate::doc::text_css::family_list(value)
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
        let mut baseline = Theme::new(theme.scheme);
        let mut old = palette_css(&baseline);
        lengths(&mut baseline, |name, value| {
            old.insert(format!("--{name}"), format!("{value}px"));
        });
        let mut values = palette_css(&theme);
        let mut theme = theme;
        lengths(&mut theme, |name, value| {
            let name = format!("--{name}");
            let value = format!("{value}px");
            if old.get(&name) != Some(&value) {
                values.insert(name, value);
            }
        });
        values.retain(|name, value| old.get(name) != Some(value));
        self.root_tokens = values.into_iter().collect();
        self.root_type_tokens = Breakpoint::ALL.map(|bp| {
            let baseline = typography(&Theme::new(theme.scheme), bp, &env.fonts);
            typography(&theme, bp, &env.fonts)
                .into_iter()
                .filter(|(name, value)| baseline.get(name) != Some(value))
                .collect()
        });
    }
}
