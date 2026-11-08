//! HTML lists: `<ul>`, `<ol>` and `<li>` with CSS list markers.
//!
//! `list-style-type` and `list-style-position` are inherited properties, so
//! they ride the inherited custom-property channel: an author's
//! declaration becomes `--og-list-style-type` (or `-position`), and each
//! `<li>`'s user-agent rule reads them into its `markerType` and
//! `markerPosition`. An ordinal comes from the element's place among its
//! `<li>` siblings, the count the structural selectors keep for
//! `:nth-child(… of li)`, so a keyed `{#each}` reorder or insert renumbers
//! the items whose number changed without a pass of its own.
use super::StyleDeclaration;
use super::matching::{self, Tree};
use super::selectors::Selector;
use super::variables;
use serde_json::Value;

/// The user-agent rules HTML's stylesheet gives these elements (and the
/// headings' type steps), cascaded before every author rule.
pub(super) const UA_CSS: &str = "
ul, ol { padding: 0 0 0 40px; }
ul { --og-list-style-type: disc; }
ol { --og-list-style-type: decimal; }
ul ul, ol ul { --og-list-style-type: circle; }
ul ul ul, ul ol ul, ol ul ul, ol ol ul { --og-list-style-type: square; }
li {
  -og-marker-type: var(--og-list-style-type, disc);
  -og-marker-position: var(--og-list-style-position, outside);
}
h1 { font: var(--text-xl3); }
h2 { font: var(--text-xl); }
h3 { font: var(--text-lg); }
h4 { font: var(--text-body); }
h5 { font: var(--text-sm); }
h6 { font: var(--text-xs); }
";

const TYPES: [&str; 9] = [
    "none",
    "disc",
    "circle",
    "square",
    "decimal",
    "lower-alpha",
    "upper-alpha",
    "lower-roman",
    "upper-roman",
];
const POSITIONS: [&str; 2] = ["outside", "inside"];

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
/// shorthand onto their inherited custom properties; other declarations
/// pass through.
pub(super) fn expand(d: StyleDeclaration) -> Result<Vec<StyleDeclaration>, String> {
    let property = |p: &str| format!("--og-list-style-{p}");
    match d.name.as_str() {
        "list-style-type" => {
            if !d.value.has_variables() {
                checked(&d.name, &d.source, &TYPES)?;
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
        "list-style" => {
            if d.value.has_variables() {
                return Err(
                    "list-style: write var() in list-style-type or list-style-position, not in the shorthand"
                        .into(),
                );
            }
            // Omitted parts take their initial values, as CSS's shorthand does.
            let (mut ty, mut position) = (None, None);
            for word in d.source.split_whitespace() {
                if POSITIONS.contains(&word) && position.is_none() {
                    position = Some(word);
                } else if TYPES.contains(&word) && ty.is_none() {
                    ty = Some(word);
                } else if word.starts_with("url(") {
                    return Err(
                        "list-style: marker images are not supported; use list-style-type".into(),
                    );
                } else {
                    return Err(format!(
                        "list-style takes a type ({}) and a position (outside, inside), not {word}",
                        TYPES.join(", ")
                    ));
                }
            }
            Ok(vec![
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
pub(crate) fn ordinal<T: Tree>(tree: &T, id: &T::Id, filter: &[Selector], values: bool) -> i64 {
    let facts = tree.facts(id);
    if let Some(own) = integer(facts.attributes.get("value")) {
        return own;
    }
    let list = tree.parent(id);
    let attribute = |name: &str| {
        list.as_ref()
            .and_then(|l| tree.facts(l).attributes.get(name).cloned())
    };
    let reversed = attribute("reversed")
        .is_some_and(|v| !matches!(v, Value::Null | Value::Bool(false)) && v != "false");
    let step = if reversed { -1 } else { 1 };
    if values {
        let mut between = 0;
        let mut previous = tree.previous(id);
        while let Some(p) = previous {
            let facts = tree.facts(&p);
            if matching::filter_matches(&facts, filter) {
                between += 1;
                if let Some(v) = integer(facts.attributes.get("value")) {
                    return v + step * between;
                }
            }
            previous = tree.previous(&p);
        }
    }
    let (before, total) = tree.filtered(id, filter);
    let start =
        integer(attribute("start").as_ref()).unwrap_or(if reversed { total as i64 } else { 1 });
    start + step * before as i64
}

fn alphabetic(mut n: i64, base: u8) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(base + (n % 26) as u8);
        n /= 26;
    }
    out.reverse();
    String::from_utf8(out).expect("ASCII letters")
}

fn roman(mut n: i64) -> String {
    const DIGITS: [(i64, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for (value, digits) in DIGITS {
        while n >= value {
            out.push_str(digits);
            n -= value;
        }
    }
    out
}

/// What an item's marker shows for `kind` (a `markerType`) and its number,
/// with CSS's suffix (a bullet's character, which is drawn as a shape);
/// `None` for `none`. Alphabetic numbers need 1 or more
/// and roman ones 1 to 3999; outside that range a marker falls back to
/// decimal, as CSS's counter styles do.
pub(crate) fn marker(kind: &str, n: i64) -> Option<String> {
    let number = match kind {
        "none" => return None,
        "disc" => return Some("\u{2022}".into()),
        "circle" => return Some("\u{25E6}".into()),
        "square" => return Some("\u{25AA}".into()),
        "lowerAlpha" if n >= 1 => alphabetic(n, b'a'),
        "upperAlpha" if n >= 1 => alphabetic(n, b'A'),
        "lowerRoman" if (1..4000).contains(&n) => roman(n),
        "upperRoman" if (1..4000).contains(&n) => roman(n).to_uppercase(),
        _ => n.to_string(),
    };
    Some(format!("{number}."))
}

/// Whether a marker of `kind` shows a number.
pub(crate) fn numbered(kind: &str) -> bool {
    !matches!(kind, "none" | "disc" | "circle" | "square")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_styles_follow_css() {
        let m = |k: &str, n| marker(k, n).unwrap();
        assert_eq!(m("decimal", 3), "3.");
        assert_eq!(m("decimal", -2), "-2.");
        assert_eq!(m("lowerAlpha", 1), "a.");
        assert_eq!(m("lowerAlpha", 26), "z.");
        assert_eq!(m("lowerAlpha", 27), "aa.");
        assert_eq!(m("upperAlpha", 703), "AAA.");
        assert_eq!(m("lowerAlpha", 0), "0.");
        assert_eq!(m("lowerRoman", 4), "iv.");
        assert_eq!(m("upperRoman", 1994), "MCMXCIV.");
        assert_eq!(m("upperRoman", 4000), "4000.");
        assert_eq!(marker("none", 1), None);
    }
}
