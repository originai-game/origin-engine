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

/// Sentinel sRGB colour used to detect inherited `currentColor` usages
/// resolved by `nsh-tailwind` when an element specifies no `text-*` of its own.
/// Its RGB triplet `(1, 2, 3)` is checked via `c.0 >> 8 == CURRENT_RGB`, while
/// the low alpha byte preserves any `/alpha` modifier (e.g. `border-current/50`).
pub(super) const CURRENT_RGB: u32 = 0x01_02_03;
pub(super) const CURRENT_SENTINEL: Color = Color((CURRENT_RGB << 8) | 0xff);

/// What an element's classes are resolved against.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Env {
    /// The UI's box, CSS px (breakpoints).
    pub window: [f32; 2],
    pub dark: bool,
    pub rtl: bool,
    /// The player touches the screen (`pointer-coarse:`).
    pub touch: bool,
    /// The player prefers reduced motion (`motion-reduce:`).
    pub reduce_motion: bool,
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
                reduce_motion: env.reduce_motion,
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
        list.apply_data_inherited(
            &cx,
            &has,
            &self.theme,
            Some(CURRENT_SENTINEL),
            &mut out,
            &mut props,
            |out| {
                for (name, value) in declarations(style) {
                    let value = self.theme.rem_lengths(value);
                    let value = match value.parse::<f32>() {
                        Ok(n) => StyleValue::Number(n),
                        Err(_) => StyleValue::Str(&value),
                    };
                    let _ = out.set(&camel(name), value);
                }
            },
        );
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

mod record;
use record::record_of;
