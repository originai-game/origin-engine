//! CSS Multi-column Layout 1's and CSS Fragmentation 3's properties,
//! checked and kept as text ([`crate::layout::multicol`] lays the columns
//! out).

use super::Value;
use super::tracks::Lowered;

const STYLES: &[&str] = &[
    "none", "hidden", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
];

fn px(v: &str) -> Option<f32> {
    match v {
        "0" => Some(0.0),
        "thin" => Some(1.0),
        "medium" => Some(3.0),
        "thick" => Some(5.0),
        v => v
            .strip_suffix("px")?
            .parse()
            .ok()
            .filter(|n: &f32| *n >= 0.0),
    }
}

fn count(v: &str) -> Option<u16> {
    v.parse().ok().filter(|&n: &u16| n > 0)
}

pub(crate) fn lower(name: &str, value: &str) -> Option<Result<Vec<Lowered>, String>> {
    let v = value.trim();
    if v.contains("bind(") || v.contains("var(") {
        return None;
    }
    let text = |s: &str| vec![Value::String(s.to_string())];
    Some(match name {
        "column-count" => match v {
            "auto" => Ok(vec![("columnCount", text("auto"))]),
            v if count(v).is_some() => Ok(vec![("columnCount", text(v))]),
            v => Err(format!(
                "'{v}' is no column-count: auto or a positive integer"
            )),
        },
        "column-width" => match v {
            "auto" => Ok(vec![("columnWidth", text("auto"))]),
            v if px(v).is_some_and(|w| w > 0.0) => Ok(vec![("columnWidth", text(v))]),
            v => Err(format!("'{v}' is no column-width: auto or a px length")),
        },
        // `columns: <column-width> || <column-count>`, either `auto`.
        "columns" => {
            let (mut width, mut cnt) = ("auto", "auto");
            let words: Vec<&str> = v.split_whitespace().collect();
            if words.is_empty() || words.len() > 2 {
                return Some(Err("columns: <column-width> || <column-count>".into()));
            }
            for w in &words {
                if count(w).is_some() {
                    cnt = w;
                } else if px(w).is_some_and(|x| x > 0.0) {
                    width = w;
                } else if *w != "auto" {
                    return Some(Err(format!(
                        "'{w}' is no column width or count: columns: 12em, columns: 3, columns: 200px 3"
                    )));
                }
            }
            Ok(vec![
                ("columnWidth", text(width)),
                ("columnCount", text(cnt)),
            ])
        }
        "break-before" | "break-after" | "page-break-before" | "page-break-after" => {
            let key = if name.ends_with("before") {
                "breakBefore"
            } else {
                "breakAfter"
            };
            match v {
                "auto" | "avoid" | "avoid-page" | "avoid-column" | "page" | "column" | "always"
                | "all" | "left" | "right" | "recto" | "verso" | "region" | "avoid-region" => {
                    Ok(vec![(key, text(v))])
                }
                v => Err(format!(
                    "'{v}' is no {name}: auto, avoid, column, page, always…"
                )),
            }
        }
        "break-inside" | "page-break-inside" => match v {
            "auto" | "avoid" | "avoid-page" | "avoid-column" | "avoid-region" => {
                Ok(vec![("breakInside", text(v))])
            }
            v => Err(format!(
                "'{v}' is no break-inside: auto, avoid, avoid-column or avoid-page"
            )),
        },
        "column-span" => match v {
            "none" | "all" => Ok(vec![("columnSpan", text(v))]),
            v => Err(format!("'{v}' is no column-span: none or all")),
        },
        "column-fill" => match v {
            "auto" | "balance" | "balance-all" => Ok(vec![("columnFill", text(v))]),
            v => Err(format!(
                "'{v}' is no column-fill: auto, balance or balance-all"
            )),
        },
        "column-rule-width" => match px(v) {
            Some(_) => Ok(vec![("columnRuleWidth", text(v))]),
            None => Err(format!(
                "'{v}' is no column-rule-width: a px length, thin, medium or thick"
            )),
        },
        "column-rule-style" if STYLES.contains(&v) => Ok(vec![("columnRuleStyle", text(v))]),
        "column-rule-style" => Err(format!("'{v}' is no line style: {}", STYLES.join(", "))),
        "column-rule-color" => color(v).map(|c| vec![("columnRuleColor", text(c))]),
        // `column-rule: <width> || <style> || <color>`; what is left out is
        // its initial value (medium, none, currentcolor).
        "column-rule" => {
            let (mut width, mut style, mut col) = ("medium", "none", "currentcolor");
            let mut rest = Vec::new();
            for w in super::position_css::words(v) {
                if px(w).is_some() {
                    width = w;
                } else if STYLES.contains(&w) {
                    style = w;
                } else {
                    rest.push(w);
                }
            }
            let joined = rest.join(" ");
            if !rest.is_empty() {
                match color(&joined) {
                    Ok(_) => col = &joined,
                    Err(e) => return Some(Err(e)),
                }
            }
            Ok(vec![
                ("columnRuleWidth", text(width)),
                ("columnRuleStyle", text(style)),
                ("columnRuleColor", text(col)),
            ])
        }
        _ => return None,
    })
}

fn color(v: &str) -> Result<&str, String> {
    if v == "currentcolor" || crate::css_color::parse_str(v).is_ok() {
        Ok(v)
    } else {
        Err(format!("'{v}' is no colour"))
    }
}
