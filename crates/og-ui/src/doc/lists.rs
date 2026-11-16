//! HTML lists: `<ul>`, `<ol>` and `<li>` with CSS list markers.
//!
//! `list-style-type` and `list-style-position` are inherited properties, so
//! they use the standard-property cascade, lowered into `markerType` and
//! `markerPosition`. An ordinal comes from the element's place among its
//! `<li>` siblings, the count the structural selectors keep for
//! `:nth-child(… of li)`, so a keyed `{#each}` reorder or insert renumbers
//! the items whose number changed without a pass of its own.
use super::StyleDeclaration;
use super::matching::{self, Tree};
use super::selectors::Selector;
use super::variables;
pub use crate::css::image_css::list_style_image as image;
use serde_json::Value;

/// The user-agent rules HTML's stylesheet gives these elements (and the
/// headings' type steps), cascaded before every author rule: list
/// indents and markers, and the block margins (`margin: 0` resets them).
/// A `<button>` centres its label, as HTML's sheet has it, so it does not
/// inherit its parent's `text-align`, and a figure on it (a cost) takes the
/// plate's ink: `--figure-ink` is the button's colour inside it, under any
/// style's own rule for a button. A `<menu>` is a menu (its buttons
/// are menu items): each item spans the menu, so its items are as wide as
/// each other, with their labels at the start, as a desktop's and a game's
/// menus are. Emphasis marks take their side from the content language
/// (CSS Text Decoration 4 Appendix B).
pub(super) const UA_CSS: &str = "
ul, ol { padding: 0 0 0 40px; }
li { display: list-item; }
ul { list-style-type: disc; }
ol { list-style-type: decimal; }
ul ul, ol ul { list-style-type: circle; }
ul ul ul, ul ol ul, ol ul ul, ol ol ul { list-style-type: square; }
p, ul, ol { margin-block: 1em; }
ul ul, ul ol, ol ul, ol ol { margin-block: 0; }
h1 { font: var(--text-xl3); letter-spacing: var(--type-xl3-tracking); margin-block: 0.67em; }
h2 { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); margin-block: 0.83em; }
h3 { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); margin-block: 1em; }
h4 { font: var(--text-body); margin-block: 1.33em; }
h5 { font: var(--text-sm); margin-block: 1.67em; }
h6 { font: var(--text-xs); margin-block: 2.33em; }
small { font: var(--text-sm); }
button { text-align: center; --figure-ink: currentColor; }
menu > button { width: stretch; text-align: start; }
table { display: table; }
caption { display: table-caption; text-align: center; }
thead { display: table-header-group; }
tbody { display: table-row-group; }
tfoot { display: table-footer-group; }
tr { display: table-row; }
td, th { display: table-cell; padding: 1px; }
th { font-weight: bold; text-align: center; }
:root:lang(zh), [lang|=zh] { text-emphasis-position: under right; }
[lang|=ja], [lang|=ko] { text-emphasis-position: over right; }
";

const POSITIONS: [&str; 2] = ["outside", "inside"];

pub(super) fn initial(property: &str) -> Option<&'static str> {
    match property {
        "list-style-type" => Some("disc"),
        "list-style-position" => Some("outside"),
        "list-style-image" => Some("none"),
        _ => None,
    }
}

/// The selector an item's ordinal counts: its `<li>` siblings.
pub(crate) fn items() -> Vec<Selector> {
    Selector::parse_list("li").expect("li is a selector")
}

fn declaration(name: &str, value: &str, important: bool) -> Result<StyleDeclaration, String> {
    Ok(StyleDeclaration {
        name: name.into(),
        value: variables::Value::parse(value)?,
        source: value.into(),
        important,
        pending_shorthand: None,
    })
}

fn checked<'a>(name: &str, value: &'a str, allowed: &[&str]) -> Result<&'a str, String> {
    let value = value.trim();
    if allowed.contains(&value) {
        Ok(value)
    } else {
        Err(format!("{name} takes {}, not {value}", allowed.join(", ")))
    }
}

/// Lowers `list-style-type`, `list-style-position` and the `list-style`
/// shorthand onto their inherited standard properties; other declarations
/// pass through.
pub(super) fn expand(d: StyleDeclaration) -> Result<Vec<StyleDeclaration>, String> {
    let property = |p: &str| format!("list-style-{p}");
    match d.name.as_str() {
        "list-style-type" => {
            if !d.value.has_variables() {
                crate::css::generated::list_style_type(&d.source)?;
            }
            Ok(vec![declaration(
                &property("type"),
                &d.source,
                d.important,
            )?])
        }
        "list-style-position" => {
            if !d.value.has_variables() {
                checked(&d.name, &d.source, &POSITIONS)?;
            }
            Ok(vec![declaration(
                &property("position"),
                &d.source,
                d.important,
            )?])
        }
        "list-style-image" => {
            if !d.value.has_variables() {
                image(&d.source)?;
            }
            Ok(vec![declaration(
                &property("image"),
                &d.source,
                d.important,
            )?])
        }
        "list-style" => {
            if d.value.has_variables() {
                return Err(
                    "list-style: write var() in list-style-type or list-style-position, not in the shorthand"
                        .into(),
                );
            }
            // Omitted parts take their initial values, as CSS's shorthand does.
            // A `none` sets the type, then the image (CSS Lists 3 §3.5).
            let (mut ty, mut position, mut picture) = (None, None, None);
            for word in components(&d.source) {
                if POSITIONS.contains(&word) && position.is_none() {
                    position = Some(word);
                } else if (word.starts_with("url(") || word.starts_with("image("))
                    && picture.is_none()
                {
                    image(word)?;
                    picture = Some(word);
                } else if word == "none" && ty.is_some() && picture.is_none() {
                    picture = Some(word);
                } else if ty.is_none() && crate::css::generated::list_style_type(word).is_ok() {
                    ty = Some(word);
                } else {
                    return Err(format!(
                        "list-style takes a type (a counter style, symbols(…), a \"string\" or none), a position (outside, inside) and an image, not {word}"
                    ));
                }
            }
            Ok(vec![
                declaration(&property("image"), picture.unwrap_or("none"), d.important)?,
                declaration(&property("type"), ty.unwrap_or("disc"), d.important)?,
                declaration(
                    &property("position"),
                    position.unwrap_or("outside"),
                    d.important,
                )?,
            ])
        }
        _ => Ok(vec![d]),
    }
}

/// A value's top-level component values as written: a string or a
/// function is one.
fn components(value: &str) -> Vec<&str> {
    let mut p = cssparser::Parser::new(value);
    let mut out = Vec::new();
    loop {
        p.skip_whitespace();
        let start = p.position();
        let Ok(token) = p.next() else {
            return out;
        };
        if matches!(token, cssparser::Token::Function(_)) {
            let _ = p.parse_nested_block(|p| {
                while p.next().is_ok() {}
                Ok::<_, cssparser::ParseError<()>>(())
            });
        }
        out.push(p.slice_from(start));
    }
}

fn integer(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(n) => n.as_f64().map(|n| n as i64),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// An `<li>`'s number: its own `value`, else counted from the nearest
/// earlier item with one (when `values`: some item of the document has a
/// `value`), else from its list's `start` (1, or the item count when
/// `reversed`), one step per item before it.
pub fn ordinal<T: Tree>(tree: &T, id: &T::Id, filter: &[Selector], values: bool) -> i64 {
    let facts = tree.facts(id);
    if let Some(own) = integer(facts.attributes.get("value")) {
        return own;
    }
    let list = tree.parent(id);
    let attribute = |name: &str| list.and_then(|l| tree.facts(l).attributes.get(name).cloned());
    let reversed = attribute("reversed")
        .is_some_and(|v| !matches!(v, Value::Null | Value::Bool(false)) && v != "false");
    let step = if reversed { -1 } else { 1 };
    if values {
        let mut between = 0;
        let mut previous = tree.previous(id);
        while let Some(p) = previous {
            let facts = tree.facts(p);
            if matching::filter_matches(&facts, filter) {
                between += 1;
                if let Some(v) = integer(facts.attributes.get("value")) {
                    return v + step * between;
                }
            }
            previous = tree.previous(p);
        }
    }
    let (before, total) = tree.filtered(id, filter);
    let start =
        integer(attribute("start").as_ref()).unwrap_or(if reversed { total as i64 } else { 1 });
    start + step * before as i64
}

/// Whether a marker of `list-style-type` `kind` is text: every type but
/// `none` and the bullets drawn as shapes (which no `@counter-style` may
/// redefine).
pub fn numbered(kind: &str) -> bool {
    !matches!(kind, "none" | "disc" | "circle" | "square")
}
