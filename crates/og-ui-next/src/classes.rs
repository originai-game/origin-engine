//! Tailwind class strings to [`Record`]s (CONVENTIONS.md §Classes and the
//! Record). NSH's resolver (`nsh-tailwind`) parses a class string into
//! rules once per distinct string and applies them for an element's
//! environment into NSH's `Style`; this module maps that `Style` onto the
//! record the arena lays out and paint draws. There is no cascade: later
//! utilities win, variants after plain utilities.

use crate::motion;
use crate::pack::NineSlice;
use crate::record::{
    GRADIENT_STOPS, Gradient, Record, Rgba, SHADOWS, Shadow, Shape, Slice, Text, TextShadows, Tint,
};
use nsh_tailwind::effects::{Filter, GradientKind, Keyframes};
use nsh_tailwind::geometry::{DefiniteLength, Points, Size};
use nsh_tailwind::style::{Color, Style, StyleValue, SvgPaint, TextAlign, TextTransform};
use nsh_tailwind::tailwind::{self, ClassEnvironment, ClassList, ClassProps, Context, Theme};
use nsh_tailwind::transform::TransformKind;
use og_draw::paint::PaintEffect;
use rune_taffy::CompactStyle;
use rustc_hash::FxHashMap;

pub use nsh_tailwind::tailwind::state;

/// The [`Env::states`] bit that asks for an element's record in the other
/// scheme than the page's: it or an ancestor carries `scheme-dark` or
/// `scheme-light` and the page is in the other one. NSH's state bits stop
/// below it; a resolver strips it before NSH sees the states.
pub const OTHER_SCHEME: u16 = 1 << 15;

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
    /// The placeholder ink a field without its own takes, per scheme
    /// (light, dark): the theme's `fg-muted`, which the ink solver holds
    /// on every ground. `None` until first read.
    placeholder: [Option<Option<Rgba>>; 2],
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
    /// Its last `scheme-dark` (true) or `scheme-light`.
    scheme: Option<bool>,
    /// Its last `stagger-<step>[/<cap>]`, in seconds.
    stagger: Option<[f32; 2]>,
    /// It has `transition-count`.
    count: bool,
    /// Its last `glide-<name>`.
    glide: Option<Box<str>>,
}

/// `stagger-<step>` or `stagger-<step>/<cap>` in ms (`stagger-40/400`):
/// the step between the children entering together and the most it
/// spreads them over, in seconds (no cap: ten steps).
fn stagger(class: &str) -> Option<[f32; 2]> {
    let v = class.strip_prefix("stagger-")?;
    let (step, cap) = v.split_once('/').unwrap_or((v, ""));
    let step = step
        .parse::<f32>()
        .ok()
        .filter(|s| s.is_finite() && *s >= 0.0)?;
    let cap = match cap {
        "" => step * 10.0,
        c => c
            .parse::<f32>()
            .ok()
            .filter(|c| c.is_finite() && *c >= 0.0)?,
    };
    Some([step / 1000.0, cap / 1000.0])
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
                placeholder: [None; 2],
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
            let (mut scheme, mut stag, mut count) = (None, None, false);
            let mut glide = None;
            for class in expanded
                .as_deref()
                .unwrap_or(classes)
                .split_ascii_whitespace()
            {
                let (variants, utility) = class.rsplit_once(':').unwrap_or(("", class));
                // The scheme a subtree's art is drawn in (Tailwind's
                // `color-scheme` utilities), unconditional.
                match class {
                    "scheme-dark" => {
                        scheme = Some(true);
                        continue;
                    }
                    "scheme-light" => {
                        scheme = Some(false);
                        continue;
                    }
                    // The scheme of the element's parent again, over one
                    // a slot or an earlier class named (`color-scheme:
                    // normal`).
                    "scheme-normal" => {
                        scheme = None;
                        continue;
                    }
                    // A number in its text rolls to a new one on its
                    // transition's curve (the tree's counts).
                    "transition-count" => {
                        count = true;
                        continue;
                    }
                    // A selection's indicator slides from the one it
                    // replaces (the tree's glides).
                    c if c.starts_with("glide-") && c.len() > 6 => {
                        glide = Some(Box::from(&c[6..]));
                        continue;
                    }
                    c if stagger(c).is_some() => {
                        stag = stagger(c);
                        continue;
                    }
                    _ => {}
                }
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
                    scheme,
                    stagger: stag,
                    count,
                    glide,
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
                .filter(|n| &***n != crate::pack::NINE_NONE && !images.contains_key(&***n))
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
        let env = Env {
            states: env.states & !OTHER_SCHEME,
            ..*env
        };
        let parsed = self.parsed(classes);
        let (list, nine, nine_list, scheme) = (
            parsed.list.clone(),
            parsed.nine.clone(),
            parsed.nine_list.clone(),
            parsed.scheme,
        );
        let (stag, count, glide) = (parsed.stagger, parsed.count, parsed.glide.clone());
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
            // A group's states, as `tree::ops` writes them in the
            // data: `name/@bits` (`/@bits` for an unnamed group).
            group: &|name| {
                data.split(' ')
                    .find_map(|t| t.strip_prefix(name)?.strip_prefix("/@")?.parse().ok())
                    .unwrap_or(0)
            },
        };
        // CSS's initial flex values under the classes: an item is `flex: 0
        // 1 auto` with the automatic minimum, so it shrinks with its
        // siblings but not below its content (NSH's defaults are React
        // Native's: no shrink, minimum 0).
        let mut out = Style::default();
        out.layout.flex_shrink = 1.0;
        out.layout.min_size = taffy::Size::auto();
        let mut props = ClassProps::default();
        list.apply_data(&cx, &has, &self.theme, &mut out, &mut props, |_| {});
        for (name, value) in declarations(style) {
            let value = self.theme.rem_lengths(value);
            let value = match value.parse::<f32>() {
                Ok(n) => StyleValue::Number(n),
                Err(_) => StyleValue::Str(&value),
            };
            let _ = out.set(&camel(name), value);
        }
        // What the classes set of the node's own props (`line-clamp-*`,
        // `pointer-events-*`, `text-ellipsis`) NSH keeps in the style.
        let mut own = ClassProps::default();
        out.node_props(&mut own);
        props.number_of_lines = own.number_of_lines.or(props.number_of_lines);
        props.ellipsize = own.ellipsize.or(props.ellipsize);
        props.selectable = own.selectable.or(props.selectable);
        props.pointer_events = own.pointer_events.or(props.pointer_events);
        // `ps-4`, `me-2`, `rounded-s-lg`: the start and end sides on the
        // page's direction's (NSH's core places them; this tree has none).
        // Only the sides: the layout's direction stays what it was.
        let (direction, logical) = (out.layout.direction, out.logical);
        nsh_tailwind::style::apply_direction(&mut out.layout, &mut out.view, &logical, env.rtl);
        out.layout.direction = direction;
        let mut record = record_of(&out, &props, &self.theme);
        // `space-x-*` ends each child but the last with a margin, the
        // start one with `space-x-reverse`: physical sides by direction.
        if let Some(space) = &props.space {
            let margin = |key, v: &Option<nsh_tailwind::style::PropertyValue>| {
                let mut s = Style::default();
                s.apply_value(key, v.as_ref()?).ok()?;
                Some(s.layout.margin)
            };
            use nsh_tailwind::style::StyleKey as K;
            let left = space.reverse_x != env.rtl;
            record.space.x = margin(K::MarginRight, &space.x).map(|m| (m.right, left));
            record.space.y = margin(K::MarginBottom, &space.y).map(|m| (m.bottom, space.reverse_y));
        }
        record.scheme = scheme;
        record.stagger = stag;
        record.count = count;
        record.glide = glide;
        record.dark = env.dark;
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
            let held = stand
                .z_index
                .and_then(|z| usize::try_from(z - 1).ok())
                .and_then(|i| nine.get(i));
            // `nine-none` holds no art: the box draws its fill alone.
            if let Some(image) = held
                .filter(|n| &***n != crate::pack::NINE_NONE)
                .and_then(|n| self.images.get(&**n))
            {
                record.skin.image = *image;
            }
        }
        if record.skin.placeholder.is_none() {
            record.skin.placeholder = self.placeholder(&env);
        }
        record
    }

    /// The theme's placeholder ink in `env`'s scheme: `fg-muted`, held on
    /// every ground, where half the field's ink alpha read too faint on
    /// dark art.
    fn placeholder(&mut self, env: &Env) -> Option<Rgba> {
        let i = usize::from(env.dark);
        if let Some(c) = self.placeholder[i] {
            return c;
        }
        self.placeholder[i] = Some(None);
        let env = Env { states: 0, ..*env };
        let c = self
            .record_styled("placeholder:text-fg-muted", "", &env)
            .skin
            .placeholder;
        self.placeholder[i] = Some(c);
        c
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

/// What an inline `style` declaration's property does here
/// ([`style_property`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleProperty {
    /// NSH reads it and the tree draws or lays it out.
    Drawn,
    /// NSH reads it into a field the tree does not draw (the reason).
    NotDrawn(&'static str),
    /// No NSH style property has the name: the declaration is skipped.
    Unknown,
}

/// The style properties NSH reads into a field this tree does not draw,
/// with why: each is a field `every_nsh_style_field_is_read_or_named_not_drawn`
/// names as not drawn.
const NOT_DRAWN: [(&str, &str); 10] = [
    ("glassEffect", "a platform material"),
    ("glassTintColor", "a platform material"),
    ("tintColor", "a native image's tint"),
    ("backfaceVisibility", "3D: the tree draws 2D transforms"),
    ("accentColor", "an accent colour is not drawn yet"),
    ("direction", "the tree takes the page's direction"),
    ("writingDirection", "the tree takes the page's direction"),
    ("textAlignVertical", "vertical alignment inside a text"),
    ("verticalAlign", "the tree lays out no inline boxes"),
    ("strokeDasharray", "a line's dashes are not drawn"),
];

/// What the inline style property `name` does, by the name the renderer's
/// `setStyle` sends it under (`background-color`, or `--…`). A custom
/// property (`--x`) is not a style property NSH reads.
pub fn style_property(name: &str) -> StyleProperty {
    let key = camel(name.trim());
    if nsh_tailwind::style::StyleKey::parse(&key).is_none() {
        return StyleProperty::Unknown;
    }
    match NOT_DRAWN.iter().find(|(k, _)| *k == key) {
        Some((_, why)) => StyleProperty::NotDrawn(why),
        None => StyleProperty::Drawn,
    }
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

/// A colour filter function as its matrix (Filter Effects 1 §13, on the
/// encoded colour); `None` for one that needs the pixels around it (blur,
/// drop-shadow).
fn color_matrix(f: &Filter) -> Option<og_draw::paint::ColorMatrix> {
    type M3 = [[f32; 3]; 3];
    let rows = |m: M3, offset: f32, alpha: f32| {
        let [r, g, b] = m.map(|[x, y, z]| [x, y, z, 0.0]);
        [
            r,
            g,
            b,
            [0.0, 0.0, 0.0, alpha],
            [offset, offset, offset, 0.0],
        ]
    };
    let diag = |s: f32| [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]];
    // `base` moved `t` of the way to `toward`, entry by entry.
    let mix = |base: M3, toward: M3, t: f32| -> M3 {
        std::array::from_fn(|i| {
            std::array::from_fn(|k| base[i][k] + (toward[i][k] - base[i][k]) * t)
        })
    };
    let id = diag(1.0);
    Some(match *f {
        Filter::Brightness(s) => rows(diag(s), 0.0, 1.0),
        Filter::Contrast(s) => rows(diag(s), 0.5 - 0.5 * s, 1.0),
        Filter::Grayscale(s) => {
            let gray = [[0.2126, 0.7152, 0.0722]; 3];
            rows(mix(id, gray, s.clamp(0.0, 1.0)), 0.0, 1.0)
        }
        Filter::Sepia(s) => {
            let sepia = [
                [0.393, 0.769, 0.189],
                [0.349, 0.686, 0.168],
                [0.272, 0.534, 0.131],
            ];
            rows(mix(id, sepia, s.clamp(0.0, 1.0)), 0.0, 1.0)
        }
        // From the luminance (0) through the colour (1), past it above 1.
        Filter::Saturate(s) => rows(mix([[0.213, 0.715, 0.072]; 3], id, s.max(0.0)), 0.0, 1.0),
        Filter::HueRotate(a) => {
            let (sin, cos) = og_core::math::sin_cos(a.0);
            let m = [
                [
                    0.213 + cos * 0.787 - sin * 0.213,
                    0.715 - cos * 0.715 - sin * 0.715,
                    0.072 - cos * 0.072 + sin * 0.928,
                ],
                [
                    0.213 - cos * 0.213 + sin * 0.143,
                    0.715 + cos * 0.285 + sin * 0.140,
                    0.072 - cos * 0.072 - sin * 0.283,
                ],
                [
                    0.213 - cos * 0.213 - sin * 0.787,
                    0.715 - cos * 0.715 + sin * 0.715,
                    0.072 + cos * 0.928 + sin * 0.072,
                ],
            ];
            rows(m, 0.0, 1.0)
        }
        Filter::Invert(s) => {
            let s = s.clamp(0.0, 1.0);
            rows(diag(1.0 - 2.0 * s), s, 1.0)
        }
        Filter::Opacity(s) => rows(id, 0.0, s.clamp(0.0, 1.0)),
        Filter::Blur(_) | Filter::DropShadow { .. } => return None,
    })
}

/// An NSH gradient as a record's: its shape, angle, centre and up to
/// [`GRADIENT_STOPS`] stops.
fn gradient_of(g: &nsh_tailwind::effects::Gradient) -> Gradient {
    let mid = [0.5; 2];
    let (shape, angle, corner, center) = match g.kind {
        GradientKind::Linear { angle } => (Shape::Linear, angle.0, [0, 0], mid),
        GradientKind::LinearCorner { right, bottom } => (
            Shape::Linear,
            0.0,
            [if right { 1 } else { -1 }, if bottom { 1 } else { -1 }],
            mid,
        ),
        GradientKind::Radial { circle, size, x, y } => {
            (Shape::Radial { circle, size }, 0.0, [0, 0], [x.0, y.0])
        }
        GradientKind::Conic { from, x, y } => (Shape::Conic, from.0, [0, 0], [x.0, y.0]),
    };
    let n = g.stops.len().min(GRADIENT_STOPS);
    let mut out = Gradient {
        shape,
        angle,
        corner,
        center,
        count: n as u8,
        stops: [(0.0, [0.0; 4]); GRADIENT_STOPS],
    };
    let all = g.stops.len();
    for (k, slot) in out.stops.iter_mut().take(n).enumerate() {
        // Past the cap, every stop's place is sampled evenly, so
        // the last stop is always kept.
        let i = if n > 1 {
            (k * (all - 1) + (n - 1) / 2) / (n - 1)
        } else {
            0
        };
        let s = &g.stops[i];
        // An unplaced stop spreads evenly between its neighbours.
        let even = if all > 1 {
            i as f32 / (all - 1) as f32
        } else {
            0.0
        };
        let at = s.position.as_ref().and_then(fraction).unwrap_or(even);
        *slot = (at, linear(s.color));
    }
    out
}

fn record_of(style: &Style, props: &ClassProps, theme: &Theme) -> Record {
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
    r.fill = style.paint.fill.as_ref().map(|f| match f {
        SvgPaint::Color(c) => Some(linear(*c)),
        SvgPaint::None => Some([0.0; 4]),
        SvgPaint::CurrentColor => None,
    });
    if let Some(w) = style.paint.stroke_width {
        r.stroke_width = w.0;
    }
    r.cursor = style.view.cursor;
    r.z_index = style.z_index;
    r.hidden = style.view.hidden;
    r.object = style
        .object_fit()
        .map(|fit| (fit, style.view.object_position));
    r.touch = props
        .pointer_events
        .map(|p| p == nsh_tailwind::props::PointerEvents::Auto);
    let (view, text, skin) = (&style.view, &style.text, &mut r.skin);
    skin.fill = linear(view.background_color);
    skin.opacity = view.opacity;
    skin.blend = match view.blend_mode {
        nsh_tailwind::effects::BlendMode::PlusLighter => 1,
        nsh_tailwind::effects::BlendMode::Screen => 2,
        nsh_tailwind::effects::BlendMode::Multiply => 3,
        _ => 0,
    };
    let c = &view.corner_radius;
    let all = view.border_radius.0;
    skin.radius =
        [c.top_left, c.top_right, c.bottom_right, c.bottom_left].map(|v| v.map_or(all, |p| p.0));
    let w = &view.borders.widths;
    skin.border = [w.top.0, w.right.0, w.bottom.0, w.left.0];
    // The commonest side colour is the border's; the sides of the first
    // other one take it (a third colour draws as the second); a tie goes to
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
    // Translate (a fraction of the box's own size resolved at paint),
    // scale, rotate and 2D skew; 3D transforms are left out.
    for op in view.transform.ops() {
        let (t, v) = (&mut skin.transform, op.value);
        match op.kind {
            TransformKind::TranslateX => t.translate[0] += v,
            TransformKind::TranslateY => t.translate[1] += v,
            TransformKind::TranslateXFraction => t.percent[0] += v,
            TransformKind::TranslateYFraction => t.percent[1] += v,
            TransformKind::Scale => t.scale = [t.scale[0] * v, t.scale[1] * v],
            TransformKind::ScaleX => t.scale[0] *= v,
            TransformKind::ScaleY => t.scale[1] *= v,
            TransformKind::Rotate => t.rotate += v.to_radians(),
            TransformKind::SkewX => t.skew[0] += v.to_radians(),
            TransformKind::SkewY => t.skew[1] += v.to_radians(),
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
        skin.gradient = fx.background_images.first().map(gradient_of);
        r.mask = fx.mask_images.first().map(gradient_of);
        r.filter = fx
            .filters
            .iter()
            .filter_map(color_matrix)
            .reduce(|m, next| og_draw::paint::color_then(&m, &next));
        let effects = fx
            .filters
            .iter()
            .filter_map(|f| match *f {
                Filter::Blur(p) if p.0 > 0.0 => Some(PaintEffect::Blur(p.0)),
                Filter::DropShadow {
                    offset_x,
                    offset_y,
                    blur,
                    color,
                    // Its blur is a radius, as a box shadow's is: a Gaussian
                    // of half its deviation, as browsers draw it.
                } if color.0 & 0xff != 0 => Some(PaintEffect::DropShadow {
                    offset: [offset_x.0, offset_y.0],
                    sigma: 0.5 * blur.0.max(0.0),
                    color: linear(color),
                }),
                _ => None,
            })
            .take(crate::record::EFFECTS);
        r.effect_count = 0;
        for (k, e) in effects.enumerate() {
            r.effects[k] = e;
            r.effect_count = k as u8 + 1;
        }
        // Blurs in a list add up (a Gaussian of their deviations'
        // root sum of squares).
        skin.backdrop_blur = fx
            .backdrop_filters
            .iter()
            .filter_map(|f| match f {
                Filter::Blur(p) => Some(p.0 * p.0),
                _ => None,
            })
            .sum::<f32>()
            .sqrt();
        r.backdrop_filter = fx
            .backdrop_filters
            .iter()
            .filter_map(color_matrix)
            .reduce(|m, next| og_draw::paint::color_then(&m, &next));
    }
    if let Some(c) = text.color {
        skin.ink = linear(c);
        r.inked = true;
    }
    // Every layer; `text-shadow-none` sets none, so a child of a shadowed
    // parent draws none instead of inheriting it.
    if let Some(list) = text.shadows {
        let t = &mut skin.text_shadows;
        *t = TextShadows::default();
        for (k, l) in list.layers().iter().enumerate() {
            t.layers[k] = Shadow {
                offset: [l.offset_x.0, l.offset_y.0],
                blur: l.radius.0.max(0.0),
                spread: 0.0,
                color: linear(l.color),
                inset: false,
            };
            t.ink[k] = l.ink;
        }
        t.count = list.layers().len() as u8;
        r.shadowed = true;
    }
    if let Some(d) = text.decoration {
        skin.decoration.line = u8::from(d.underline) | (u8::from(d.line_through) << 2);
        skin.decoration.color = text.decoration_color.map_or([0.0; 4], linear);
        skin.decoration.thickness = text.decoration_thickness.map_or(0.0, |p| p.0);
    }
    skin.decoration.style = text.decoration_style.unwrap_or_default();
    skin.decoration.offset = text.underline_offset.map(|p| p.0);
    skin.caret = text.caret_color.or(props.caret_color).map(linear);
    skin.placeholder = text
        .placeholder_color
        .or(props.placeholder_color)
        .map(linear);
    skin.selection = text.selection_color.or(props.selection_color).map(linear);
    // The outline (`text-stroke-*`), its colour the text's by default.
    if let Some(w) = text.stroke_width {
        skin.text_stroke.width = w.0;
        skin.text_stroke.color = text.stroke_color.map_or(skin.ink, linear);
        r.stroked = true;
    }
    let t = &mut r.text;
    if let Some(s) = text.font_size {
        r.font_size = s.0;
        t.sized = true;
    }
    r.font_size_em = text.font_size_em;
    t.sized |= r.font_size_em.is_some();
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
            TextAlign::Auto => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
            TextAlign::Left => 3,
            TextAlign::Justify => 4,
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
    if let Some(h) = text.hyphens {
        t.hyphens = h;
        t.set |= Text::HYPHENS;
    }
    // NSH's family list, slant and variant forms go to the text's layout
    // as they are (text.rs): no copy of their meaning here.
    if let Some(f) = text.font_family {
        t.family = Some(f);
        t.set |= Text::FAMILY;
    }
    if let Some(i) = text.italic {
        t.italic = i;
        t.set |= Text::ITALIC;
    }
    if let Some(v) = text.font_variant {
        t.variant = v.0;
        t.set |= Text::VARIANT;
    }
    if let Some(w) = props.overflow_wrap {
        t.wrap = w;
        t.set |= Text::WRAP;
    }
    if let Some(v) = props.vertical {
        t.vertical = v;
        t.set |= Text::VERTICAL;
    }
    if let Some(on) = props.selectable {
        t.select = on;
        t.set |= Text::SELECT;
    }
    r.motion = motion_of(style, theme);
    if let Some(nowrap) = props.no_wrap {
        t.nowrap = nowrap;
        t.set |= Text::NOWRAP;
    }
    if let Some(n) = props.number_of_lines {
        t.clamp = n.min(u32::from(u16::MAX)) as u16;
        t.set |= Text::CLAMP;
    }
    if let Some(mode) = props.ellipsize {
        // `text-clip` cuts with no ellipsis; Parley places one at a
        // line's end only, so every other mode is the tail's.
        t.overflow = u8::from(mode != nsh_tailwind::style::EllipsizeMode::Clip);
        t.set |= Text::OVERFLOW;
    }
    r
}

/// What `transition-*`, `duration-*`, `delay-*`, `ease-*` and `animate-*`
/// ask of motion; NSH's milliseconds become seconds.
fn motion_of(style: &Style, theme: &Theme) -> motion::Spec {
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
    m.animate = style.view.animation.map(|a| {
        let (keyframes, peak) = match a.keyframes {
            Keyframes::Spin => (motion::Keyframes::Spin, 0.0),
            Keyframes::Ping => (motion::Keyframes::Ping, 0.0),
            Keyframes::Pulse => (motion::Keyframes::Pulse, 0.0),
            Keyframes::Bounce => (motion::Keyframes::Bounce, 0.0),
            Keyframes::Pop => (motion::Keyframes::Pop, 0.0),
            Keyframes::Stamp => (motion::Keyframes::Stamp, 0.0),
            Keyframes::Beat => (
                motion::Keyframes::Beat,
                theme.scale("combo-pop").unwrap_or(0.0),
            ),
            Keyframes::Float => (
                motion::Keyframes::Float,
                theme.scale("damage-pop").unwrap_or(0.0),
            ),
            Keyframes::Shimmer => (motion::Keyframes::Shimmer, 0.0),
            Keyframes::Rise => (motion::Keyframes::Rise, 0.0),
            Keyframes::Charge => (motion::Keyframes::Charge, 0.0),
            Keyframes::Nudge => (motion::Keyframes::Nudge, 0.0),
            Keyframes::Breathe => (
                motion::Keyframes::Breathe,
                theme.scale("ready-pulse").unwrap_or(0.0),
            ),
            Keyframes::Shake => (motion::Keyframes::Shake, 0.0),
            Keyframes::Flash => (
                motion::Keyframes::Flash,
                theme.opacity("flash").unwrap_or(0.0),
            ),
            Keyframes::Recoil => (motion::Keyframes::Recoil, 0.0),
        };
        motion::Loop {
            keyframes,
            duration: a.duration / 1000.0,
            delay: a.delay / 1000.0,
            ease: a.easing,
            iterations: a.iterations,
            peak,
        }
    });
    m
}
