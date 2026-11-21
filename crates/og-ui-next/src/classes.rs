//! Tailwind class strings to [`Record`]s (CONVENTIONS.md §Classes and the
//! Record). NSH's resolver (`nsh-tailwind`) parses a class string into
//! rules once per distinct string and applies them for an element's
//! environment into NSH's `Style`; this module maps that `Style` onto the
//! record the arena lays out and paint draws. There is no cascade: later
//! utilities win, variants after plain utilities.

use crate::motion;
use crate::pack::NineSlice;
use crate::style::{GRADIENT_STOPS, Gradient, Record, Rgba, SHADOWS, Shadow, Slice, Text, Tint};
use nsh_tailwind::effects::{Filter, GradientKind, Keyframes};
use nsh_tailwind::geometry::{DefiniteLength, Points, Size};
use nsh_tailwind::style::{Color, Style, StyleValue, SvgPaint, TextAlign, TextTransform};
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
    lists: FxHashMap<Box<str>, Parsed>,
    /// The pack's variant class strings by `Component.slot[.variant]`.
    variants: std::collections::BTreeMap<String, String>,
    /// The pack's nine-slice images by name, in the scheme ([`nine_slice`]).
    images: std::collections::BTreeMap<String, Slice>,
    /// A class string it parsed reads the window's exact size.
    sized: bool,
}

/// A class string parsed: its Tailwind classes, and the names of its
/// `nine-<name>` classes in order.
struct Parsed {
    list: ClassList,
    nine: Vec<Box<str>>,
    /// The same `nine-` classes as `z-[<n>]` with their variants (`n`
    /// standing for name `n`), so NSH's own variant and order rules pick
    /// the one that holds.
    nine_list: Option<ClassList>,
}

/// A pack image (`Pack::images`) as the box image it draws, its texture
/// the slot its `src` loaded into (0: none yet). A tinted one is a shape
/// the box's fill colours.
pub fn nine_slice(image: &NineSlice, texture: u32) -> Slice {
    Slice {
        texture,
        slice: image.slice,
        width: image.width.unwrap_or([0.0; 4]),
        fill: image.fill,
        tile: image.tile,
        tint: if image.tint { Tint::Fill } else { Tint::Own },
        ..Slice::default()
    }
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
                variants: Default::default(),
                images: Default::default(),
                sized: false,
            },
            ignored,
        ))
    }

    /// The pack's variants (`Pack::variants`), which `@Component.slot` and
    /// `@Component.slot.variant` tokens in a class string stand for
    /// (`packClass` at runtime): the slot's classes, then the variant's.
    pub fn set_variants(&mut self, variants: std::collections::BTreeMap<String, String>) {
        self.variants = variants;
        self.lists.clear();
    }

    /// The pack's nine-slice images by name ([`nine_slice`]): a
    /// `nine-<name>` class draws its image as the box's, under any variant
    /// (the variant is not read: a pack splits its images by scheme
    /// itself); the last one named wins.
    pub fn set_images(&mut self, images: std::collections::BTreeMap<String, Slice>) {
        self.images = images;
    }

    /// `classes` with its `@…` tokens replaced by the variants they name; a
    /// token naming no variant stays, so it is reported unknown.
    fn expand(&self, classes: &str) -> Option<String> {
        if !classes.contains('@') {
            return None;
        }
        let mut out = String::with_capacity(classes.len() * 4);
        let mut push = |s: &str| {
            out.push_str(s);
            out.push(' ');
        };
        for class in classes.split_ascii_whitespace() {
            let Some(key) = class.strip_prefix('@') else {
                push(class);
                continue;
            };
            let end = key.match_indices('.').nth(1).map_or(key.len(), |(i, _)| i);
            match (self.variants.get(&key[..end]), self.variants.get(key)) {
                (Some(slot), own) => {
                    push(slot);
                    if end < key.len()
                        && let Some(own) = own
                    {
                        push(own);
                    }
                }
                (None, Some(own)) => push(own),
                (None, None) => push(class),
            }
        }
        Some(out)
    }

    fn parsed(&mut self, classes: &str) -> &Parsed {
        if !self.lists.contains_key(classes) {
            let expanded = self.expand(classes);
            let (mut rest, mut nine, mut stand) = (String::new(), Vec::new(), String::new());
            for class in expanded
                .as_deref()
                .unwrap_or(classes)
                .split_ascii_whitespace()
            {
                let (variants, utility) = class.rsplit_once(':').unwrap_or(("", class));
                let Some(name) = utility.strip_prefix("nine-") else {
                    rest.push_str(class);
                    rest.push(' ');
                    continue;
                };
                nine.push(Box::from(name));
                if !variants.is_empty() {
                    stand.push_str(variants);
                    stand.push(':');
                }
                stand.push_str(&format!("z-[{}] ", nine.len()));
            }
            let list = tailwind::parse(&self.theme, &rest);
            self.sized |= list.deps() & tailwind::deps::SIZE != 0;
            let nine_list = (!nine.is_empty()).then(|| tailwind::parse(&self.theme, &stand));
            self.lists.insert(
                classes.into(),
                Parsed {
                    list,
                    nine,
                    nine_list,
                },
            );
        }
        &self.lists[classes]
    }

    /// Whether a class string it resolved reads the window's exact size
    /// (`w-screen`, `h-[16vh]`): its records change with every resize, not
    /// only at a breakpoint.
    pub fn sized(&self) -> bool {
        self.sized
    }

    /// The classes of `classes` that are no known utility, as written, and
    /// its `nine-<name>` classes that name no image of the pack.
    pub fn unknown(&mut self, classes: &str) -> Vec<String> {
        let images = std::mem::take(&mut self.images);
        let p = self.parsed(classes);
        let mut out: Vec<String> = p.list.unknown().iter().map(|c| c.to_string()).collect();
        out.extend(
            p.nine
                .iter()
                .filter(|n| !images.contains_key(&***n))
                .map(|n| format!("nine-{n}")),
        );
        self.images = images;
        out
    }

    /// The state bits `classes` reads (`hover:` …): an element whose
    /// classes read none keeps one record.
    pub fn tracked(&mut self, classes: &str) -> u16 {
        let p = self.parsed(classes);
        p.list.tracked_states() | p.nine_list.as_ref().map_or(0, ClassList::tracked_states)
    }

    /// The record `classes` give an element in `env`.
    pub fn record(&mut self, classes: &str, env: &Env) -> Record {
        self.record_styled(classes, "", env)
    }

    /// The record `classes` and the inline declarations `style` give an
    /// element in `env`: `style` is CSS text as the renderer's `setStyle`
    /// sends it (`width: 40%; opacity: 0.5`), applied over the classes
    /// through NSH's own declarations, so it takes what a style prop
    /// takes. A declaration NSH does not know is skipped.
    pub fn record_styled(&mut self, classes: &str, style: &str, env: &Env) -> Record {
        self.record_data(classes, style, env, "")
    }

    /// [`Classes::record_styled`] for an element with `data-*` attributes:
    /// `data` names them, separated by spaces, the element's own as
    /// `name` and its nearest ancestor group's as `group/name` (`/name`
    /// for an unnamed `group`), which `data-<name>:` and
    /// `group-data-<name>[/group]:` read.
    pub fn record_data(&mut self, classes: &str, style: &str, env: &Env, data: &str) -> Record {
        let has = |group: Option<&str>, name: &str| {
            data.split(' ').any(|t| match group {
                None => t == name,
                Some(g) => t.strip_prefix(g).and_then(|r| r.strip_prefix('/')) == Some(name),
            })
        };
        let env = *env;
        let parsed = self.parsed(classes);
        let (list, nine, nine_list) = (
            parsed.list.clone(),
            parsed.nine.clone(),
            parsed.nine_list.clone(),
        );
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
        let mut out = Style::default();
        let mut props = ClassProps::default();
        list.apply_data(&cx, &has, &self.theme, &mut out, &mut props, |_| {});
        for (name, value) in declarations(style) {
            let value = match value.parse::<f32>() {
                Ok(n) => StyleValue::Number(n),
                Err(_) => StyleValue::Str(value),
            };
            let _ = out.set(&camel(name), value);
        }
        let mut record = record_of(&out, &props);
        // The `nine-<name>` that holds in `env`, the last one written.
        if let Some(nine_list) = nine_list {
            let mut stand = Style::default();
            nine_list.apply_data(
                &cx,
                &has,
                &self.theme,
                &mut stand,
                &mut ClassProps::default(),
                |_| {},
            );
            let held = usize::try_from(stand.z_index - 1)
                .ok()
                .and_then(|i| nine.get(i));
            if let Some(image) = held.and_then(|n| self.images.get(&**n)) {
                record.skin.image = *image;
            }
        }
        record
    }
}

/// CSS text's declarations as (name, value), trimmed, empty ones left out.
fn declarations(css: &str) -> impl Iterator<Item = (&str, &str)> {
    css.split(';').filter_map(|d| {
        let (n, v) = d.split_once(':')?;
        let (n, v) = (n.trim(), v.trim());
        (!n.is_empty() && !v.is_empty()).then_some((n, v))
    })
}

/// A CSS property name in React Native's camel case (`border-top-width`
/// is `borderTopWidth`), the names NSH's declarations take.
fn camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut up = false;
    for c in name.chars() {
        match c {
            '-' => up = !out.is_empty(),
            c if up => {
                out.push(c.to_ascii_uppercase());
                up = false;
            }
            c => out.push(c),
        }
    }
    out
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
    // `pt-[calc(env(hud-inset-top)+16px)]`: padding that adds the HUD's
    // safe area, on the side whose own inset it reads.
    for (k, var) in style.env_padding().into_iter().enumerate() {
        use nsh_tailwind::style::EnvVar::*;
        let side = match var {
            Some(HudInsetTop) => 0,
            Some(HudInsetRight) => 1,
            Some(HudInsetBottom) => 2,
            Some(HudInsetLeft) => 3,
            _ => continue,
        };
        if side == k {
            r.safe[1] |= 1 << k;
        }
    }
    // A `<Line>`'s stroke: `stroke-accent`, `stroke-2`.
    match &style.paint.stroke {
        Some(SvgPaint::Color(c)) => r.stroke = Some(linear(*c)),
        Some(SvgPaint::None) => r.stroke = Some([0.0; 4]),
        Some(SvgPaint::CurrentColor) | None => {}
    }
    if let Some(w) = style.paint.stroke_width {
        r.stroke_width = w.0;
    }
    let (view, text, skin) = (&style.view, &style.text, &mut r.skin);
    skin.fill = linear(view.background_color);
    skin.opacity = view.opacity;
    let c = &view.corner_radius;
    let all = view.border_radius.0;
    skin.radius =
        [c.top_left, c.top_right, c.bottom_right, c.bottom_left].map(|v| v.map_or(all, |p| p.0));
    let w = &view.borders.widths;
    skin.border = [w.top.0, w.right.0, w.bottom.0, w.left.0];
    // The commonest side colour is the border's; the sides of the first
    // other one take it (a third colour draws as the second); a tie goes to the
    // top's.
    let cs = view.borders.colors;
    let base = *cs
        .iter()
        .rev()
        .max_by_key(|c| cs.iter().filter(|d| d == c).count())
        .expect("four sides");
    skin.border_color = linear(base);
    skin.border_sides = 0;
    if let Some(&side) = cs.iter().find(|&&c| c != base) {
        skin.border_side = linear(side);
        skin.border_sides = (0..4).filter(|&k| cs[k] != base).fold(0, |m, k| m | 1 << k);
    }
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
    // The outline (`text-stroke-*`), its colour the text's by default.
    if let Some(w) = text.stroke_width {
        skin.text_stroke.width = w.0;
        skin.text_stroke.color = text.stroke_color.map_or(skin.ink, linear);
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
    r.motion = motion_of(style);
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

/// What `transition-*`, `duration-*`, `delay-*`, `ease-*` and `animate-*`
/// ask of motion; NSH's milliseconds become seconds.
fn motion_of(style: &Style) -> motion::Spec {
    let mut m = motion::Spec::default();
    if let Some(t) = &style.view.transition {
        // NSH's groups are motion's bits one to one, `LAYOUT` (ours) too.
        m.props = t.properties.0;
        m.duration = t.duration / 1000.0;
        m.delay = t.delay / 1000.0;
        m.ease = match t.spring {
            Some(s) => motion::Ease::Spring {
                stiffness: s.stiffness,
                damping: s.damping,
                mass: s.mass,
            },
            None => motion::Ease::Bezier(t.easing),
        };
    }
    m.animate = style.view.animation.map(|a| motion::Loop {
        keyframes: match a.keyframes {
            Keyframes::Spin => motion::Keyframes::Spin,
            Keyframes::Ping => motion::Keyframes::Ping,
            Keyframes::Pulse => motion::Keyframes::Pulse,
            Keyframes::Bounce => motion::Keyframes::Bounce,
            Keyframes::Pop => motion::Keyframes::Pop,
            Keyframes::Stamp => motion::Keyframes::Stamp,
            Keyframes::Beat => motion::Keyframes::Beat,
            Keyframes::Float => motion::Keyframes::Float,
            Keyframes::Shimmer => motion::Keyframes::Shimmer,
        },
        duration: a.duration / 1000.0,
        delay: a.delay / 1000.0,
        ease: a.easing,
        iterations: a.iterations,
    });
    m
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

    #[test]
    fn motion_classes_resolve_to_the_motion_spec_and_presence_states() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env::default();
        let r = c.record(
            "transition-colors transition-layout duration-300 delay-75 ease-in",
            &env,
        );
        let m = r.motion;
        assert_eq!(
            m.props,
            motion::COLORS | motion::LAYOUT,
            "layout stays beside colours"
        );
        assert_eq!((m.duration, m.delay), (0.3, 0.075));
        assert_eq!(m.ease, motion::Ease::Bezier([0.4, 0.0, 1.0, 1.0]));
        assert_eq!(
            c.record("transition-all", &env).motion.props & motion::LAYOUT,
            0
        );
        assert_eq!(
            c.record("transition duration-200 transition-none", &env)
                .motion
                .props,
            0
        );
        let beat = c.record("animate-beat", &env).motion.animate.unwrap();
        assert_eq!(
            (beat.keyframes, beat.duration, beat.iterations),
            (motion::Keyframes::Beat, 0.4, Some(1))
        );
        assert_eq!(
            c.record("animate-shimmer", &env)
                .motion
                .animate
                .unwrap()
                .iterations,
            None
        );
        assert!(
            c.unknown("animate-pop animate-stamp animate-float animate-wobble")
                == ["animate-wobble"]
        );
        // `starting:` and `exit:` give the entering and leaving records.
        let s = "opacity-100 starting:opacity-0 exit:opacity-0 exit:translate-y-2";
        assert_eq!(
            c.tracked(s) & (state::STARTING | state::EXIT),
            state::STARTING | state::EXIT
        );
        assert_eq!(c.record(s, &env).skin.opacity, 1.0);
        let start = Env {
            states: state::STARTING,
            ..env
        };
        assert_eq!(c.record(s, &start).skin.opacity, 0.0);
        let exit = c.record(
            s,
            &Env {
                states: state::EXIT,
                ..env
            },
        );
        assert_eq!(
            (exit.skin.opacity, exit.skin.transform.translate[1]),
            (0.0, 8.0)
        );
    }

    #[test]
    fn overflow_classes_set_each_axis_as_css() {
        use taffy::Overflow::*;
        let (mut c, _) = Classes::new(None).unwrap();
        for (class, x, y) in [
            ("overflow-auto", Scroll, Scroll),
            ("overflow-hidden", Hidden, Hidden),
            ("overflow-x-auto", Scroll, Visible),
            ("overflow-y-scroll", Visible, Scroll),
            ("overflow-x-hidden overflow-y-auto", Hidden, Scroll),
            ("overflow-clip", Clip, Clip),
        ] {
            let r = c.record(class, &Env::default());
            let o = taffy::CoreStyle::overflow(&r.layout);
            assert_eq!((o.x, o.y), (x, y), "{class}");
        }
    }

    /// `order-*` is the layout's `order`; CSS math in a size resolves
    /// against the window, which a change of it resolves again.
    #[test]
    fn order_and_css_math_classes_resolve() {
        let (mut c, _) = Classes::new(None).unwrap();
        for (class, want) in [
            ("order-2", 2),
            ("-order-1", -1),
            ("order-first", -9999),
            ("order-[5]", 5),
        ] {
            let r = c.record(class, &Env::default());
            assert_eq!(taffy::CoreStyle::order(&r.layout), want, "{class}");
        }
        let size = |c: &mut Classes, class: &str, w: f32, h: f32| {
            let env = Env {
                window: [w, h],
                ..Env::default()
            };
            let s = taffy::CoreStyle::size(&c.record(class, &env).layout);
            (s.width.value(), s.height.value())
        };
        let math = "size-[max(4rem,min(8.286rem,16vh))]";
        // 16vh of 600 is 96: between 4rem and 8.286rem.
        assert_eq!(size(&mut c, math, 800.0, 600.0), (96.0, 96.0));
        // 16vh of 200 is 32, under 4rem: the floor, as `size-[4rem]` is.
        let floor = size(&mut c, "size-[4rem]", 800.0, 200.0);
        assert!(floor.0 > 32.0);
        assert_eq!(size(&mut c, math, 800.0, 200.0), floor);
    }

    #[test]
    fn hud_insets_add_to_the_padding_side_that_reads_them() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env {
            window: [800.0, 600.0],
            ..Env::default()
        };
        let r = c.record(
            "pt-[calc(env(hud-inset-top)+16px)] pl-[env(hud-inset-left)] pr-4",
            &env,
        );
        assert_eq!(r.safe, [0, 0b1001]);
        let s = taffy::Style::from(&r.layout_in([5.0, 7.0, 0.0, 30.0]));
        assert_eq!(s.padding.top, taffy::LengthPercentage::length(21.0));
        assert_eq!(s.padding.left, taffy::LengthPercentage::length(30.0));
        assert_eq!(s.padding.right, taffy::LengthPercentage::length(16.0));
    }

    #[test]
    fn an_inline_style_folds_over_its_classes() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env::default();
        let class = "w-4 opacity-50 bg-red-500";
        let base = c.record(class, &env);
        let r = c.record_styled(
            class,
            "width: 40%; opacity:0.75 ;; background-color: #0000ff",
            &env,
        );
        let w = |r: &Record| taffy::CoreStyle::size(&r.layout).width;
        assert_eq!(w(&base), taffy::Dimension::length(16.0), "w-4");
        assert_eq!(
            w(&r),
            taffy::Dimension::percent(0.4),
            "the style wins over w-4"
        );
        assert_eq!(r.skin.opacity, 0.75);
        assert_eq!(r.skin.fill, linear(Color(0x0000ffff)));
        // What it does not name stays the class's; an unknown one is skipped.
        let r = c.record_styled(class, "bogus-thing: 3", &env);
        assert_eq!((w(&r), r.skin.opacity), (w(&base), 0.5));
        assert_eq!(camel("border-top-left-radius"), "borderTopLeftRadius");
    }

    #[test]
    fn pack_tokens_stand_for_their_variants() {
        let (mut c, _) = Classes::new(None).unwrap();
        c.set_variants(
            [
                ("Panel.frame".to_owned(), "p-2 opacity-50".to_owned()),
                ("Panel.frame.hud".to_owned(), "opacity-75".to_owned()),
            ]
            .into(),
        );
        let env = Env::default();
        let pad = |r: &Record| taffy::Style::from(&r.layout).padding.top;
        // The slot alone; the slot then its variant, the variant winning.
        let r = c.record("@Panel.frame", &env);
        assert_eq!(
            (pad(&r), r.skin.opacity),
            (taffy::LengthPercentage::length(8.0), 0.5)
        );
        let r = c.record("p-1 @Panel.frame.hud", &env);
        assert_eq!(
            (pad(&r), r.skin.opacity),
            (taffy::LengthPercentage::length(8.0), 0.75)
        );
        // A variant the pack lacks keeps the slot's; a slot it lacks is unknown.
        assert_eq!(c.record("@Panel.frame.menu", &env).skin.opacity, 0.5);
        assert_eq!(c.unknown("@Card.body"), ["@Card.body"]);
    }

    #[test]
    fn a_nine_class_draws_the_packs_image_as_the_box() {
        let (mut c, _) = Classes::new(None).unwrap();
        let image = |tint| NineSlice {
            src: "styles/x/plate.png".into(),
            slice: [12.0, 16.0, 12.0, 16.0],
            width: None,
            fill: false,
            tile: true,
            tint,
        };
        c.set_images(
            [
                ("plate".to_owned(), nine_slice(&image(true), 7)),
                ("frame".to_owned(), nine_slice(&image(false), 9)),
            ]
            .into(),
        );
        let env = Env::default();
        let r = c.record("p-2 nine-plate bg-red-500", &env);
        let im = r.skin.image;
        assert_eq!(
            (im.texture, im.slice, im.width),
            (7, [12.0, 16.0, 12.0, 16.0], [0.0; 4])
        );
        assert_eq!(
            (im.fill, im.tile, im.tint, im.uv),
            (false, true, Tint::Fill, [0.0, 0.0, 1.0, 1.0])
        );
        assert!(r.skin.fill[3] > 0.0, "the classes still apply");
        // A variant's holds only in its state or window, then winning.
        let hover = Env {
            states: state::HOVERED,
            ..env
        };
        let class = "hover:nine-frame nine-plate";
        assert_ne!(c.tracked(class), 0, "it reads the hover state");
        assert_eq!(c.record(class, &env).skin.image.texture, 7);
        let im = c.record(class, &hover).skin.image;
        assert_eq!((im.texture, im.tint), (9, Tint::Own));
        assert_eq!(
            c.record("nine-frame nine-plate", &env).skin.image.texture,
            7,
            "the last one"
        );
        // A name the pack lacks draws nothing and is reported.
        assert_eq!(c.record("nine-gone", &env).skin.image, Slice::default());
        assert_eq!(c.unknown("p-2 nine-gone nine-plate"), ["nine-gone"]);
    }

    /// A side's colour over the border's (a spinner's arc,
    /// `border-rim border-t-accent`): the commonest colour is the border's,
    /// the other sides are marked; one colour marks none.
    #[test]
    fn a_side_colour_marks_its_side() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env::default();
        let s = c
            .record("border-4 border-[#ff0000] border-t-[#0000ff]", &env)
            .skin;
        assert_eq!((s.border_color[0], s.border_color[2]), (1.0, 0.0));
        assert_eq!((s.border_side[0], s.border_side[2]), (0.0, 1.0));
        assert_eq!(s.border_sides, 1, "top only");
        let s = c
            .record("border-4 border-[#ff0000] border-x-[#0000ff]", &env)
            .skin;
        assert_eq!(s.border_sides, 0b1010, "right and left");
        assert_eq!(
            c.record("border-4 border-[#ff0000]", &env)
                .skin
                .border_sides,
            0
        );
    }

    #[test]
    fn data_variants_read_the_element_and_its_nearest_group() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env::default();
        let class =
            "opacity-50 data-on:opacity-75 group-data-armed/booster:p-2 group-data-open:pt-1";
        let pad = |r: &Record| taffy::Style::from(&r.layout).padding.top;
        let r = c.record_data(class, "", &env, "");
        assert_eq!(
            (r.skin.opacity, pad(&r)),
            (0.5, taffy::LengthPercentage::length(0.0))
        );
        let r = c.record_data(class, "", &env, "on booster/armed");
        assert_eq!(
            (r.skin.opacity, pad(&r)),
            (0.75, taffy::LengthPercentage::length(8.0))
        );
        // Another group's, or its own, `armed` is not the booster group's.
        let r = c.record_data(class, "", &env, "armed boost/armed /open");
        assert_eq!(
            (r.skin.opacity, pad(&r)),
            (0.5, taffy::LengthPercentage::length(4.0))
        );
    }

    #[test]
    fn a_text_stroke_class_outlines_the_text() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env::default();
        let r = c.record("text-red-500 text-stroke-2 text-stroke-black", &env);
        assert_eq!(r.skin.text_stroke.width, 2.0);
        assert_eq!(r.skin.text_stroke.color, linear(Color(0x000000ff)));
        assert_ne!(r.skin.ink, r.skin.text_stroke.color, "the fill stays red");
        // Its colour is the text's unless named; a colour alone draws none.
        let r = c.record("text-red-500 text-stroke-[1.5px]", &env);
        assert_eq!(
            (r.skin.text_stroke.width, r.skin.text_stroke.color),
            (1.5, r.skin.ink)
        );
        assert_eq!(
            c.record("text-stroke-black", &env).skin.text_stroke.width,
            0.0
        );
        assert!(c.unknown("text-stroke-2 text-stroke-white").is_empty());
    }

    #[test]
    fn a_stroke_class_sets_a_lines_colour_and_width() {
        let (mut c, _) = Classes::new(None).unwrap();
        let env = Env::default();
        let r = c.record("stroke-red-500 stroke-3", &env);
        assert_eq!(r.stroke_width, 3.0);
        assert!(r.stroke.is_some_and(|s| s[0] > s[1]), "red: {:?}", r.stroke);
        let r = c.record("stroke-current", &env);
        assert_eq!((r.stroke, r.stroke_width), (None, 1.0), "the ink, 1 px");
    }
}
