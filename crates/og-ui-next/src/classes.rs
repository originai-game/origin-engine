//! Tailwind class strings to [`Record`]s (CONVENTIONS.md §Classes and the
//! Record). NSH's resolver (`nsh-tailwind`) parses a class string into
//! rules once per distinct string and applies them for an element's
//! environment into NSH's `Style`; this module maps that `Style` onto the
//! record the arena lays out and paint draws. There is no cascade: later
//! utilities win, variants after plain utilities.

use crate::style::{GRADIENT_STOPS, Gradient, Record, Rgba, SHADOWS, Shadow, Text};
use nsh_tailwind::effects::{Filter, GradientKind};
use nsh_tailwind::geometry::{DefiniteLength, Points, Size};
use nsh_tailwind::style::{Color, Style, TextAlign, TextTransform};
use nsh_tailwind::tailwind::{self, ClassEnvironment, ClassList, ClassProps, Context, Theme};
use nsh_tailwind::transform::TransformKind;
use rune_taffy::CompactStyle;
use rustc_hash::FxHashMap;

pub use nsh_tailwind::tailwind::state;

/// What an element's classes are resolved against.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Env {
    /// The UI's box, CSS px (breakpoints).
    pub window: [f32; 2],
    pub dark: bool,
    pub rtl: bool,
    /// The player touches the screen (`pointer-coarse:`).
    pub touch: bool,
    /// The element's [`state`] bits (`hover:`, `active:`, `focus-visible:` …).
    pub states: u16,
}

/// A theme and the class strings resolved against it, each parsed once.
pub struct Classes {
    theme: Theme,
    lists: FxHashMap<Box<str>, ClassList>,
}

impl Classes {
    /// Tailwind's default theme extended by `theme_json` (NSH's
    /// `Theme::from_json` keys, one scheme's: the pack layer splits
    /// `{ light, dark }` before it reaches here). Answers the theme and the
    /// entries it left out.
    pub fn new(theme_json: Option<&str>) -> Result<(Classes, Vec<String>), String> {
        let (theme, ignored) = match theme_json {
            Some(json) => Theme::from_json_lenient(json).map_err(|e| e.0)?,
            None => (Theme::default(), Vec::new()),
        };
        Ok((
            Classes {
                theme,
                lists: FxHashMap::default(),
            },
            ignored,
        ))
    }

    fn list(&mut self, classes: &str) -> &ClassList {
        if !self.lists.contains_key(classes) {
            let list = tailwind::parse(&self.theme, classes);
            self.lists.insert(classes.into(), list);
        }
        &self.lists[classes]
    }

    /// The classes of `classes` that are no known utility, as written.
    pub fn unknown(&mut self, classes: &str) -> Vec<String> {
        self.list(classes)
            .unknown()
            .iter()
            .map(|c| c.to_string())
            .collect()
    }

    /// The state bits `classes` reads (`hover:` …): an element whose
    /// classes read none keeps one record.
    pub fn tracked(&mut self, classes: &str) -> u16 {
        self.list(classes).tracked_states()
    }

    /// The record `classes` give an element in `env`.
    pub fn record(&mut self, classes: &str, env: &Env) -> Record {
        let env = *env;
        let list = self.list(classes).clone();
        let cx = Context {
            env: ClassEnvironment {
                dark: env.dark,
                pointer: if env.touch {
                    tailwind::Pointer::Coarse
                } else {
                    tailwind::Pointer::Fine
                },
                ..ClassEnvironment::default()
            },
            window: Size::new(Points(env.window[0]), Points(env.window[1])),
            rtl: env.rtl,
            states: env.states,
            position: 0,
            group: &|_| 0,
        };
        let mut style = Style::default();
        let mut props = ClassProps::default();
        list.apply(&cx, &self.theme, &mut style, &mut props, |_| {});
        record_of(&style, &props)
    }
}

/// A packed sRGB colour (`0xRRGGBBAA`) as linear RGBA, straight alpha.
fn linear(c: Color) -> Rgba {
    let [r, g, b, a] = c.0.to_be_bytes();
    og_css::Srgba {
        rgb: [r, g, b],
        alpha: f32::from(a) / 255.0,
        p3: None,
    }
    .linear()
}

fn fraction(l: &DefiniteLength) -> Option<f32> {
    match l {
        DefiniteLength::Fraction(p) => Some(p.0),
        DefiniteLength::Absolute(_) => None,
    }
}

fn record_of(style: &Style, props: &ClassProps) -> Record {
    let mut r = Record {
        layout: CompactStyle::from(&style.layout),
        ..Record::default()
    };
    let (view, text, skin) = (&style.view, &style.text, &mut r.skin);
    skin.fill = linear(view.background_color);
    skin.opacity = view.opacity;
    let c = &view.corner_radius;
    let all = view.border_radius.0;
    skin.radius =
        [c.top_left, c.top_right, c.bottom_right, c.bottom_left].map(|v| v.map_or(all, |p| p.0));
    let w = &view.borders.widths;
    skin.border = [w.top.0, w.right.0, w.bottom.0, w.left.0];
    skin.border_color = linear(view.borders.colors[0]);
    r.layout.set_border(skin.border);
    skin.outline.width = view.outline.width.0;
    skin.outline.offset = view.outline.offset.0;
    skin.outline.color = linear(view.outline.color);
    // Translate, scale and rotate in their own fields; a percentage
    // translate, skew and 3D need the box or a matrix and are left out.
    for op in view.transform.ops() {
        let (t, v) = (&mut skin.transform, op.value);
        match op.kind {
            TransformKind::TranslateX => t.translate[0] += v,
            TransformKind::TranslateY => t.translate[1] += v,
            TransformKind::Scale => t.scale = [t.scale[0] * v, t.scale[1] * v],
            TransformKind::ScaleX => t.scale[0] *= v,
            TransformKind::ScaleY => t.scale[1] *= v,
            TransformKind::Rotate => t.rotate += v.to_radians(),
            _ => {}
        }
    }
    skin.transform.origin = [
        fraction(&view.transform_origin.x).unwrap_or(0.5),
        fraction(&view.transform_origin.y).unwrap_or(0.5),
    ];
    if let Some(fx) = &view.effects {
        for (k, s) in fx.box_shadows.iter().take(SHADOWS).enumerate() {
            skin.shadows[k] = Shadow {
                offset: [s.offset_x.0, s.offset_y.0],
                blur: s.blur.0,
                spread: s.spread.0,
                color: linear(s.color),
                inset: s.inset,
            };
            skin.shadow_count = k as u8 + 1;
        }
        skin.gradient = fx.background_images.iter().find_map(|g| {
            let (angle, corner) = match g.kind {
                GradientKind::Linear { angle } => (angle.0, [0, 0]),
                GradientKind::LinearCorner { right, bottom } => (
                    0.0,
                    [if right { 1 } else { -1 }, if bottom { 1 } else { -1 }],
                ),
                _ => return None,
            };
            let n = g.stops.len().min(GRADIENT_STOPS);
            let mut out = Gradient {
                angle,
                corner,
                count: n as u8,
                stops: [(0.0, [0.0; 4]); GRADIENT_STOPS],
            };
            for (k, s) in g.stops.iter().take(n).enumerate() {
                // An unplaced stop spreads evenly between its neighbours.
                let even = if n > 1 {
                    k as f32 / (n - 1) as f32
                } else {
                    0.0
                };
                let at = s.position.as_ref().and_then(fraction).unwrap_or(even);
                out.stops[k] = (at, linear(s.color));
            }
            Some(out)
        });
        skin.backdrop_blur = fx
            .backdrop_filters
            .iter()
            .find_map(|f| match f {
                Filter::Blur(p) => Some(p.0),
                _ => None,
            })
            .unwrap_or(0.0);
    }
    if let Some(c) = text.color {
        skin.ink = linear(c);
        r.inked = true;
    }
    if let (Some(c), Some(o)) = (text.shadow_color, text.shadow_offset) {
        skin.text_shadow = Shadow {
            offset: [o.x.0, o.y.0],
            blur: text.shadow_radius.map_or(0.0, |p| p.0),
            spread: 0.0,
            color: linear(c),
            inset: false,
        };
    }
    if let Some(d) = text.decoration {
        skin.decoration.line = u8::from(d.underline) | (u8::from(d.line_through) << 2);
        skin.decoration.color = text.decoration_color.map_or(skin.ink, linear);
        skin.decoration.thickness = text.decoration_thickness.map_or(0.0, |p| p.0);
    }
    let t = &mut r.text;
    if let Some(s) = text.font_size {
        r.font_size = s.0;
        t.sized = true;
    }
    if let Some(w) = text.font_weight {
        t.weight = w;
        t.set |= Text::WEIGHT;
    }
    if let Some(m) = text.line_height_multiple.or(text.line_height_em) {
        (t.line, t.line_px) = (m, false);
        t.set |= Text::LINE;
    } else if let Some(p) = text.line_height {
        (t.line, t.line_px) = (p.0, true);
        t.set |= Text::LINE;
    }
    if let Some(em) = text.letter_spacing_em {
        (t.tracking_em, t.tracking_px) = (em, 0.0);
        t.set |= Text::TRACKING;
    } else if let Some(p) = text.letter_spacing {
        (t.tracking_em, t.tracking_px) = (0.0, p.0);
        t.set |= Text::TRACKING;
    }
    if let Some(a) = text.text_align {
        t.align = match a {
            TextAlign::Center => 1,
            TextAlign::Right => 2,
            _ => 0,
        };
        t.set |= Text::ALIGN;
    }
    if let Some(x) = text.transform {
        t.transform = match x {
            TextTransform::Uppercase => 1,
            TextTransform::Lowercase => 2,
            TextTransform::Capitalize => 3,
            _ => 0,
        };
        t.set |= Text::TRANSFORM;
    }
    if let Some(nowrap) = props.no_wrap {
        t.nowrap = nowrap;
        t.set |= Text::NOWRAP;
    }
    if props.ellipsize.is_some() {
        t.overflow = 1;
        t.set |= Text::OVERFLOW;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_resolve_to_records() {
        let (mut c, ignored) = Classes::new(Some(
            r##"{ "colors": { "plate": "#336699" }, "radius": { "card": 10 } }"##,
        ))
        .unwrap();
        assert!(ignored.is_empty(), "{ignored:?}");
        let env = Env {
            window: [1280.0, 720.0],
            ..Env::default()
        };
        // Later utilities win; a theme colour and radius; per-corner radius.
        let r = c.record(
            "bg-red-500 bg-plate rounded-card rounded-tl-none p-4 border-2 opacity-50",
            &env,
        );
        let plate = linear(Color(0x336699ff));
        assert_eq!(r.skin.fill, plate);
        assert_eq!(r.skin.radius, [0.0, 10.0, 10.0, 10.0]);
        assert_eq!(r.skin.border, [2.0; 4]);
        assert_eq!(r.skin.opacity, 0.5);
        assert_eq!(
            r.layout.padding().top,
            taffy::LengthPercentage::length(16.0)
        );
        // A variant holds only in its state; breakpoints read the UI's box.
        let hover = "bg-black hover:bg-white sm:p-2";
        assert_eq!(c.record(hover, &env).skin.fill, linear(Color(0x000000ff)));
        let hovered = Env {
            states: state::HOVERED,
            ..env
        };
        assert_eq!(
            c.record(hover, &hovered).skin.fill,
            linear(Color(0xffffffff))
        );
        assert_ne!(c.tracked(hover) & state::HOVERED, 0);
        let phone = Env {
            window: [390.0, 844.0],
            ..env
        };
        assert_eq!(
            c.record(hover, &phone).layout.padding().top,
            taffy::LengthPercentage::length(0.0)
        );
        assert_eq!(
            c.record(hover, &env).layout.padding().top,
            taffy::LengthPercentage::length(8.0)
        );
        // Shadows, ring and text.
        let r = c.record(
            "shadow-md ring-2 ring-white text-lg font-bold uppercase text-center",
            &env,
        );
        assert!(r.skin.shadow_count >= 2, "{:?}", r.skin.shadows);
        assert_eq!((r.text.weight, r.text.transform, r.text.align), (700, 1, 1));
        assert_eq!(r.font_size, 18.0);
        assert_eq!(c.unknown("p-4 not-a-class"), ["not-a-class"]);
        // The UI's size classes: narrow is under 600 wide, short under 500
        // tall, compact either; 844x390 is short and compact, not narrow.
        let size = "narrow:p-1 short:m-1 compact:gap-1";
        let side = Env {
            window: [844.0, 390.0],
            ..env
        };
        let r = c.record(size, &side);
        let px = |v: f32| taffy::LengthPercentage::length(v);
        assert_eq!(r.layout.padding().top, px(0.0));
        assert_eq!(r.layout.gap().width, px(4.0));
        let up = Env {
            window: [390.0, 844.0],
            ..env
        };
        let r = c.record(size, &up);
        assert_eq!(
            (r.layout.padding().top, r.layout.gap().width),
            (px(4.0), px(4.0))
        );
        let wide = Env {
            window: [600.0, 500.0],
            ..env
        };
        assert_eq!(c.record(size, &wide).layout.gap().width, px(0.0));
        // Motion's states.
        let enter = "opacity-100 starting:opacity-0 exit:opacity-25";
        assert_eq!(c.record(enter, &env).skin.opacity, 1.0);
        let starting = Env {
            states: state::STARTING,
            ..env
        };
        assert_eq!(c.record(enter, &starting).skin.opacity, 0.0);
        let exit = Env {
            states: state::EXIT,
            ..env
        };
        assert_eq!(c.record(enter, &exit).skin.opacity, 0.25);
    }
}
