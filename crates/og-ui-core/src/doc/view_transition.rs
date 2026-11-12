//! CSS View Transitions for layers (docs/ui.md §2.1, CSS View Transitions
//! Levels 1 and 2). A screen document opts in with `@view-transition {
//! navigation: auto; }`: its layer joining the stack and leaving it are its
//! navigations, and the layer is the `root` snapshot. By default the layer
//! cross-fades (the UA's `-ua-view-transition-fade-in` and `-fade-out`,
//! 0.25s). `::view-transition-old(root)` animates the outgoing view (the
//! leaving layer), `::view-transition-new(root)` the incoming one (the
//! joining layer), with `animation` and the document's `@keyframes`;
//! `::view-transition-group(root)` and `::view-transition-image-pair(root)`
//! give both their inherited timing (`animation-duration`, its other
//! longhands but the name). `types` are labels that
//! `:active-view-transition-type()` matches, as in the spec.
//!
//! Elements with a `view-transition-name` are shared elements: each name is
//! a group of its own ([`crate::view_transition`] runs them), styled by
//! `::view-transition-group`, `-image-pair`, `-old` and `-new` with `(name)`,
//! `(*)` and `(.class)` arguments (`view-transition-class`), cascaded by
//! the spec's specificity: `*` under a name under a class.
//!
//! Groups nest (Level 2's nested groups): `view-transition-group` on a
//! shared element (`normal`, `contain`, `nearest` or an ancestor's name)
//! puts its group in an ancestor's `::view-transition-group-children`,
//! which moves it with that group, relative to its padding box, and clips
//! it there with `overflow`.
//!
//! The engine's full-frame covers are the only `-og-` part: the built-in
//! keyframes `-og-iris`, `-og-dissolve`, `-og-wipe` and `-og-slide` (with
//! `-from-right`, `-from-top` or `-from-bottom`). On `old` a cover closes
//! over the outgoing view, and the incoming view shows once it has closed;
//! on `new` it opens on the incoming view. `::view-transition
//! { background-color }` is the cover's colour, and `background-image:
//! url("<texture asset>")` covers with that texture stretched over the
//! frame, tinted by the colour (white without one).

use super::syntax::Error;
use super::*;
use crate::motion::css::CssAnimation;
use crate::{ScreenWipe, Side};

/// A screen's view transition (`LayerSpec::transition`).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewTransition {
    /// The outgoing view's animations (`::view-transition-old(root)`).
    pub old: Vec<CssAnimation>,
    /// The incoming view's (`::view-transition-new(root)`).
    pub new: Vec<CssAnimation>,
    /// The covers' colour (`None`: the style's backdrop).
    pub color: Option<Srgba>,
    /// A texture asset the covers show, stretched over the frame
    /// (`background-image: url("…")`), bound when the document loads.
    pub image: Option<String>,
    /// The pseudo-element rules that apply (their types among the
    /// declared ones), in cascade order: what styles the named groups.
    pub rules: Vec<Rule>,
    /// The document, which qualifies its keyframes' names.
    pub document: String,
}

/// Which pseudo-element a rule styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Overlay,
    Group,
    GroupChildren,
    ImagePair,
    Old,
    New,
}

/// A named pseudo-element's argument: `*` (no name) or a name, and the
/// classes it asks for (`*.card`, `hero.card`, `.card`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Select {
    pub name: Option<String>,
    pub classes: Vec<String>,
}

impl Select {
    /// Whether it selects the group of `name` with `classes`.
    pub fn matches(&self, name: &str, classes: &[String]) -> bool {
        self.name.as_deref().is_none_or(|n| n == name)
            && self.classes.iter().all(|c| classes.contains(c))
    }
}

/// One `::view-transition-*` rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub part: Part,
    pub select: Select,
    /// Its animation longhands, `(name, value)` in source order.
    pub longhands: Vec<(String, String)>,
}

/// A named group's animations: the group's, its image pair's, its old
/// and new images', and its children's (`::view-transition-group-children`,
/// which holds its nested groups: only the author's, as nothing inherits
/// into it); and the axes (across, down) along which the group, the image
/// pair and the children clip what they hold to their box (`overflow`
/// other than `visible`: by the UA's stylesheet none does, and an image
/// taller than its group by its aspect overflows it).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroupStyle {
    pub group: Vec<CssAnimation>,
    pub pair: Vec<CssAnimation>,
    pub old: Vec<CssAnimation>,
    pub new: Vec<CssAnimation>,
    pub children: Vec<CssAnimation>,
    pub group_clip: [bool; 2],
    pub pair_clip: [bool; 2],
    pub children_clip: [bool; 2],
}

/// `overflow`, `overflow-x` or `overflow-y` (and the logical
/// `overflow-inline` and `overflow-block`) on a group or an image pair, as
/// its `overflow-x` and `overflow-y` longhands.
fn overflow(name: &str, value: &str) -> Result<Vec<(String, String)>, String> {
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.is_empty()
        || words.len() > 2
        || !words
            .iter()
            .all(|w| matches!(*w, "visible" | "hidden" | "clip" | "scroll" | "auto"))
    {
        return Err(format!(
            "{name} takes visible, hidden, clip, scroll or auto, not '{value}'"
        ));
    }
    let x = |v: &str| ("overflow-x".to_string(), v.to_string());
    let y = |v: &str| ("overflow-y".to_string(), v.to_string());
    Ok(match (name, words.as_slice()) {
        ("overflow", [a]) => vec![x(a), y(a)],
        ("overflow", [a, b]) => vec![x(a), y(b)],
        ("overflow-x" | "overflow-inline", [a]) => vec![x(a)],
        ("overflow-y" | "overflow-block", [a]) => vec![y(a)],
        _ => return Err(format!("{name} takes one value, not '{value}'")),
    })
}

/// The UA's cross-fade keyframes.
pub const FADE_IN: &str = "-ua-view-transition-fade-in";
pub const FADE_OUT: &str = "-ua-view-transition-fade-out";
/// The UA's group animation: from the old element's box to the new one's
/// (the spec's `-ua-view-transition-group-anim-<name>`).
pub const GROUP_ANIM: &str = "-ua-view-transition-group-anim";

/// The cover a built-in keyframes name plays, and where it comes from.
pub fn cover(name: &str) -> Option<(ScreenWipe, Side)> {
    let name = name.strip_prefix("-og-")?;
    let (kind, side) = match name.split_once("-from-") {
        Some((k, s)) => (k, s),
        None => (name, "left"),
    };
    let kind = match kind {
        "iris" => ScreenWipe::Iris,
        "dissolve" => ScreenWipe::Dissolve,
        "wipe" => ScreenWipe::Wipe,
        "slide" => ScreenWipe::Slide,
        _ => return None,
    };
    let side = match side {
        "left" => Side::Left,
        "right" => Side::Right,
        "top" => Side::Top,
        "bottom" => Side::Bottom,
        _ => return None,
    };
    Some((kind, side))
}

/// The built-in keyframes names (the UA's and the covers).
fn builtin(name: &str) -> bool {
    name == FADE_IN || name == FADE_OUT || name == GROUP_ANIM || cover(name).is_some()
}

/// The keyframes of the UA's cross-fade (`None` for any other name).
pub fn builtin_keyframes(name: &str) -> Option<crate::motion::css::Keyframes> {
    use crate::motion::css::{Keyframe, KeyframeTrack, Keyframes};
    let offset = match name {
        FADE_IN => 0.0,
        FADE_OUT => 1.0,
        _ => return None,
    };
    Some(Keyframes {
        tracks: vec![KeyframeTrack {
            prop: crate::element::Prop::Opacity,
            keys: vec![Keyframe {
                offset,
                value: 0.0,
                ease: None,
                composite: None,
            }],
        }],
        styles: Vec::new(),
        events: Vec::new(),
    })
}

/// Whether a rule's selector or prelude is a view-transition one.
pub(super) fn selects(selector: &str) -> bool {
    selector.starts_with("@view-transition") || selector.contains("::view-transition")
}

/// `view-transition-name`, `view-transition-class` and
/// `view-transition-group` on an element, as the props
/// `viewTransitionName` (empty: none), `viewTransitionClass` and
/// `viewTransitionGroup` (empty: normal).
pub(super) fn lower(
    name: &str,
    value: &str,
) -> Option<Result<Vec<super::tracks::Lowered>, String>> {
    if !matches!(
        name,
        "view-transition-name" | "view-transition-class" | "view-transition-group"
    ) || value.contains("bind(")
    {
        return None;
    }
    let words = match super::css::tokens(value) {
        Ok(t) => t,
        Err(e) => return Some(Err(e)),
    };
    let mut idents = Vec::new();
    for t in &words {
        match t {
            super::css::Tok::Ident(w) => idents.push(w.clone()),
            t => {
                return Some(Err(format!(
                    "'{}' is not a name: {name} takes {}",
                    super::css::describe(t),
                    form(name).unwrap_or_default()
                )));
            }
        }
    }
    let text = |s: String| vec![Value::String(s)];
    Some(match (name, idents.as_slice()) {
        ("view-transition-name", [w]) if w == "none" => {
            Ok(vec![("viewTransitionName", text(String::new()))])
        }
        ("view-transition-name", [w]) if w == "root" => Err(
            "view-transition-name: root is the layer's own name; name the element otherwise".into(),
        ),
        ("view-transition-name", [w]) => Ok(vec![("viewTransitionName", text(w.clone()))]),
        ("view-transition-class", [w]) if w == "none" => {
            Ok(vec![("viewTransitionClass", text(String::new()))])
        }
        ("view-transition-class", ws) if !ws.is_empty() && !ws.iter().any(|w| w == "none") => {
            Ok(vec![("viewTransitionClass", text(ws.join(" ")))])
        }
        ("view-transition-group", [w]) if w == "normal" => {
            Ok(vec![("viewTransitionGroup", text(String::new()))])
        }
        ("view-transition-group", [w]) => Ok(vec![("viewTransitionGroup", text(w.clone()))]),
        _ => Err(format!("{name} takes {}", form(name).unwrap_or_default())),
    })
}

/// How the element properties are written (the catalogue's column).
pub(super) fn form(prop: &str) -> Option<&'static str> {
    Some(match prop {
        "viewTransitionName" | "view-transition-name" => "none | auto | match-element | <name>",
        "viewTransitionClass" | "view-transition-class" => "none | <name>+",
        "viewTransitionGroup" | "view-transition-group" => "normal | contain | nearest | <name>",
        _ => return None,
    })
}

/// Reads a screen's view-transition rules, `(selector, body, at,
/// conditional)` in source order, into its layer spec.
pub(super) fn apply(
    doc: &mut UiDocument,
    rules: &[(&str, &str, usize, bool)],
) -> Result<(), Error> {
    let mut opted = false;
    let mut types: Vec<String> = Vec::new();
    for &(selector, body, at, conditional) in rules {
        if conditional {
            return Err((
                at,
                format!(
                    "{selector}: write view-transition rules at the top level, not inside @media or @container"
                ),
            ));
        }
        if selector != "@view-transition" {
            continue;
        }
        for d in variables::declarations(body).map_err(|e| (at, e))? {
            let value = d.source.trim();
            match d.name.as_str() {
                "navigation" => match value {
                    "auto" => opted = true,
                    "none" => opted = false,
                    _ => return Err((at, "navigation is auto or none".into())),
                },
                "types" => types = value.split_whitespace().map(str::to_string).collect(),
                other => {
                    return Err((
                        at,
                        format!("@view-transition takes navigation and types, not {other}"),
                    ));
                }
            }
        }
    }
    // (specificity, rule): a type-qualified rule outranks a plain one (its
    // pseudo-class), as a class does a name and a name `*`; then source
    // order.
    let mut parts: Vec<((usize, bool), Rule)> = Vec::new();
    let known: Vec<String> = doc
        .sheet
        .keyframe_rules
        .iter()
        .map(|(n, ..)| n.clone())
        .collect();
    let keyframes = |name: &str| builtin(name) || known.iter().any(|k| k == name);
    let rules: Vec<(&str, &str, usize, bool)> = rules
        .iter()
        .flat_map(|&(list, body, at, c)| list.split(',').map(move |s| (s.trim(), body, at, c)))
        .collect();
    for &(selector, body, at, _) in &rules {
        if selector == "@view-transition" {
            continue;
        }
        let Some((part, select, typed, applies)) = pseudo(selector, &types).map_err(|e| (at, e))?
        else {
            return Err((
                at,
                format!("{selector}: write @view-transition or a ::view-transition pseudo-element"),
            ));
        };
        if !applies {
            continue;
        }
        let mut longhands = Vec::new();
        for d in variables::declarations(body).map_err(|e| (at, e))? {
            let clips = matches!(part, Part::Group | Part::GroupChildren | Part::ImagePair)
                && matches!(
                    d.name.as_str(),
                    "overflow" | "overflow-x" | "overflow-y" | "overflow-inline" | "overflow-block"
                );
            let allowed = match part {
                Part::Overlay => matches!(
                    d.name.as_str(),
                    "background" | "background-color" | "background-image"
                ),
                _ => clips || d.name.starts_with("animation"),
            };
            if !allowed {
                return Err((
                    at,
                    format!(
                        "{selector} takes {}, not {}",
                        match part {
                            Part::Overlay => {
                                "background-color (the cover's colour) and background-image (its texture)"
                            }
                            Part::Group | Part::ImagePair => {
                                "animation and overflow (a clip to its box)"
                            }
                            Part::GroupChildren => {
                                "animation and overflow (a clip of its nested groups to its group's padding box)"
                            }
                            _ => "animation",
                        },
                        d.name
                    ),
                ));
            }
            if part == Part::Overlay {
                // `background` is a colour or an image, as written.
                let value = d.source.trim().to_string();
                let image = d.name == "background-image"
                    || (d.name == "background" && value.starts_with("url("));
                let name = if image {
                    "background-image"
                } else {
                    "background-color"
                };
                longhands.push((name.into(), value));
                continue;
            }
            if clips {
                longhands.extend(overflow(&d.name, d.source.trim()).map_err(|e| (at, e))?);
                continue;
            }
            for d in super::motion::expand(d, &keyframes).map_err(|e| (at, e))? {
                longhands.push((d.name, d.source));
            }
        }
        let rank = (
            usize::from(typed) + select.classes.len(),
            select.name.is_some(),
        );
        parts.push((
            rank,
            Rule {
                part,
                select,
                longhands,
            },
        ));
    }
    if !opted {
        doc.layer.transition = None;
        return Ok(());
    }
    if doc.kind != DocKind::Screen {
        return Err((
            rules[0].2,
            "a view transition is a screen's: its layer is the root that joins and leaves the stack"
                .into(),
        ));
    }
    parts.sort_by_key(|(rank, _)| *rank);
    let last = |name: &str| {
        parts
            .iter()
            .filter(|(_, r)| r.part == Part::Overlay)
            .flat_map(|(_, r)| r.longhands.iter())
            .rfind(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    };
    let (color, image) = (last("background-color"), last("background-image"));
    let color = match color {
        Some(value) => Some(
            crate::style_file::parse_color(&value)
                .ok_or((rules[0].2, format!("'{value}' is not a hex colour")))?,
        ),
        None => None,
    };
    let image = match image {
        Some(value) if value == "none" => None,
        Some(value) => Some(cover_image(&value).map_err(|e| (rules[0].2, e))?),
        None => None,
    };
    let mut t = ViewTransition {
        old: Vec::new(),
        new: Vec::new(),
        color,
        image,
        rules: parts
            .into_iter()
            .map(|(_, r)| r)
            .filter(|r| r.part != Part::Overlay)
            .collect(),
        document: doc.name.clone(),
    };
    // The root: the layer's own animations. Its group and image pair give
    // only their timing (the layer is its group and its images at once).
    let mut diagnostics = Vec::new();
    let root = t.cascade("root", &[], &mut diagnostics);
    if let Some(error) = diagnostics.first() {
        return Err((rules[0].2, error.clone()));
    }
    (t.old, t.new) = (root.old, root.new);
    doc.layer.transition = Some(t);
    Ok(())
}

/// `url("<texture asset>")`: the cover's texture by its path under `assets/`.
fn cover_image(value: &str) -> Result<String, String> {
    let path = value
        .strip_prefix("url(")
        .and_then(|v| v.strip_suffix(')'))
        .map(|v| v.trim().trim_matches(['"', '\'']))
        .filter(|p| !p.is_empty())
        .ok_or_else(|| {
            format!(
                "::view-transition background-image: '{value}' is url(\"<texture asset>\") or none"
            )
        })?;
    Ok(path.trim_start_matches('/').to_string())
}

impl ViewTransition {
    /// The animations of the group `name` with `classes`, cascaded from
    /// the UA's stylesheet and the rules: the group plays the UA's box
    /// animation for 0.25s (fill both), the image pair and the images
    /// inherit its timing (but its name), the old image fades out and the
    /// new one in.
    pub fn group(&self, name: &str, classes: &[String]) -> GroupStyle {
        self.cascade(name, classes, &mut Vec::new())
    }

    fn cascade(&self, name: &str, classes: &[String], diagnostics: &mut Vec<String>) -> GroupStyle {
        let declared = |part: Part| {
            self.rules
                .iter()
                .filter(move |r| r.part == part && r.select.matches(name, classes))
                .flat_map(|r| r.longhands.iter().cloned())
        };
        let of = |part: Part| -> Vec<(String, String)> {
            declared(part)
                .filter(|(n, _)| n.starts_with("animation"))
                .collect()
        };
        // The last `overflow-x` and `overflow-y` the cascade gives a part
        // (an inherited property it is not: the pair's is its own).
        let clip = |part: Part| {
            let mut axes = [false; 2];
            for (n, v) in declared(part) {
                let axis = match n.as_str() {
                    "overflow-x" => 0,
                    "overflow-y" => 1,
                    _ => continue,
                };
                axes[axis] = v != "visible";
            }
            axes
        };
        let timing = |w: &[(String, String)]| -> Vec<(String, String)> {
            w.iter()
                .filter(|(n, _)| n != "animation-name")
                .cloned()
                .collect()
        };
        let pair_of = |n: &str, v: &str| (n.to_string(), v.to_string());
        let mut group = vec![
            pair_of("animation-name", GROUP_ANIM),
            pair_of("animation-duration", "0.25s"),
            pair_of("animation-fill-mode", "both"),
        ];
        group.extend(of(Part::Group));
        let mut pair = timing(&group);
        pair.extend(of(Part::ImagePair));
        let image = |own: Part, ua: &str| {
            let mut w = timing(&pair);
            w.push(pair_of("animation-name", ua));
            w.extend(of(own));
            w
        };
        let (old, new) = (image(Part::Old, FADE_OUT), image(Part::New, FADE_IN));
        let children = of(Part::GroupChildren);
        let mut resolve = |winners: &[(String, String)]| {
            let (_, mut animations) = super::motion::resolve(winners, &self.document, diagnostics);
            for a in &mut animations {
                if let Some(bare) = a.name.split_once('/').map(|(_, n)| n.to_string())
                    && builtin(&bare)
                {
                    a.name = bare;
                }
            }
            animations
        };
        GroupStyle {
            group: resolve(&group),
            pair: resolve(&pair),
            old: resolve(&old),
            new: resolve(&new),
            children: resolve(&children),
            group_clip: clip(Part::Group),
            pair_clip: clip(Part::ImagePair),
            children_clip: clip(Part::GroupChildren),
        }
    }
}

/// A pseudo-element selector: its part and argument, whether it names a
/// transition type, and whether it applies (its types are among the
/// declared ones). `None` when it is no view-transition selector.
fn pseudo(selector: &str, types: &[String]) -> Result<Option<(Part, Select, bool, bool)>, String> {
    let at = selector.find("::view-transition").ok_or("")?;
    let (prefix, element) = selector.split_at(at);
    let prefix = prefix
        .trim()
        .trim_start_matches("html")
        .trim_start_matches(":root");
    let (typed, applies) = match prefix {
        "" => (false, true),
        p => {
            let list = p
                .strip_prefix(":active-view-transition-type(")
                .and_then(|r| r.strip_suffix(')'))
                .ok_or_else(|| {
                    format!("{selector}: a view-transition rule is qualified only by :active-view-transition-type(…)")
                })?;
            let names: Vec<&str> = list.split(',').map(str::trim).collect();
            (true, names.iter().any(|n| types.iter().any(|t| t == n)))
        }
    };
    let element = element.trim();
    if element == "::view-transition" {
        return Ok(Some((Part::Overlay, Select::default(), typed, applies)));
    }
    let (name, arg) = element
        .strip_suffix(')')
        .and_then(|e| e.split_once('('))
        .ok_or_else(|| format!("{element}: name its group, as (root), (*), (card) or (.card)"))?;
    let part = match name {
        "::view-transition-group" => Part::Group,
        "::view-transition-group-children" => Part::GroupChildren,
        "::view-transition-image-pair" => Part::ImagePair,
        "::view-transition-old" => Part::Old,
        "::view-transition-new" => Part::New,
        _ => return Ok(None),
    };
    let mut words = arg.trim().split('.');
    let head = words.next().unwrap_or_default();
    let classes: Vec<String> = words.map(str::to_string).collect();
    let ident = |w: &str| {
        let mut c = w.chars();
        c.next()
            .is_some_and(|f| f.is_alphabetic() || f == '_' || f == '-')
            && w.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    };
    let name = match head {
        "*" => None,
        "" if !classes.is_empty() => None,
        n if ident(n) => Some(n.to_string()),
        _ => {
            return Err(format!(
                "{element}: its argument is *, a name or classes (card, *.card, .card)"
            ));
        }
    };
    if classes.iter().any(|c| !ident(c)) {
        return Err(format!(
            "{element}: '.{}' is not a class",
            classes.join(".")
        ));
    }
    if name.as_deref() == Some("root") && !classes.is_empty() {
        return Err(format!("{element}: the root has no view-transition-class"));
    }
    Ok(Some((part, Select { name, classes }, typed, applies)))
}
