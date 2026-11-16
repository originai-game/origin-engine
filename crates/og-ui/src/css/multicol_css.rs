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
        // CSS Multicol 2 §4.2: `auto` or the columns' height.
        "column-height" => match v {
            "auto" => Ok(vec![("columnHeight", text("auto"))]),
            v if px(v).is_some() => Ok(vec![("columnHeight", text(v))]),
            v => Err(format!("'{v}' is no column-height: auto or a px length")),
        },
        // CSS Multicol 2 §4.4.
        "column-wrap" => match v {
            "auto" | "nowrap" | "wrap" => Ok(vec![("columnWrap", text(v))]),
            v => Err(format!("'{v}' is no column-wrap: auto, nowrap or wrap")),
        },
        // `columns: [<column-width> || <column-count>] [/ <column-height>]?`,
        // either `auto`; it resets `column-wrap` (CSS Multicol 2 §4.5).
        "columns" => {
            let (mut width, mut cnt) = ("auto", "auto");
            let (v, height) = match v.split_once('/') {
                Some((a, h)) => match h.trim() {
                    "auto" => (a.trim(), "auto"),
                    h if px(h).is_some() => (a.trim(), h),
                    h => {
                        return Some(Err(format!(
                            "'{h}' is no column height: columns: 12em / 20em"
                        )));
                    }
                },
                None => (v, "auto"),
            };
            let words: Vec<&str> = v.split_whitespace().collect();
            if words.is_empty() || words.len() > 2 {
                return Some(Err(
                    "columns: <column-width> || <column-count> [/ <column-height>]".into(),
                ));
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
                ("columnHeight", text(height)),
                ("columnWrap", text("auto")),
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
        // CSS Fragmentation 3 §5.4.
        "box-decoration-break" | "-webkit-box-decoration-break" => match v {
            "slice" | "clone" => Ok(vec![("boxDecorationBreak", text(v))]),
            v => Err(format!("'{v}' is no box-decoration-break: slice or clone")),
        },
        // CSS Fragmentation 3 §4.4: a positive integer.
        "orphans" | "widows" => match v.parse::<u16>() {
            Ok(n) if n > 0 => Ok(vec![(
                if name == "orphans" {
                    "orphans"
                } else {
                    "widows"
                },
                text(v),
            )]),
            _ => Err(format!("'{v}' is no {name}: a positive integer of lines")),
        },
        "column-span" => match v {
            "none" | "all" => Ok(vec![("columnSpan", text(v))]),
            v => Err(format!("'{v}' is no column-span: none or all")),
        },
        // CSS Box 4 §3.3: `none | block | [block-start || block-end]`.
        "margin-trim" => {
            let ws: Vec<&str> = v.split_whitespace().collect();
            let ok = match ws.as_slice() {
                ["none"] | ["block"] => true,
                [a] => matches!(*a, "block-start" | "block-end"),
                [a, b] => {
                    a != b
                        && [a, b]
                            .iter()
                            .all(|w| matches!(**w, "block-start" | "block-end"))
                }
                _ => false,
            };
            if ok {
                Ok(vec![("marginTrim", text(v))])
            } else {
                Err(format!(
                    "'{v}' is no margin-trim: none, block, block-start or block-end"
                ))
            }
        }
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
        // CSS Gap Decorations 1 §3.2: where a rule breaks.
        "column-rule-break" | "row-rule-break" | "rule-break" => match v {
            "normal" | "none" | "intersection" => Ok(match name {
                "column-rule-break" => vec![("columnRuleBreak", text(v))],
                "row-rule-break" => vec![("rowRuleBreak", text(v))],
                _ => vec![("columnRuleBreak", text(v)), ("rowRuleBreak", text(v))],
            }),
            v => Err(format!("'{v}' is no {name}: normal, none or intersection")),
        },
        // The row gaps' rules (CSS Gap Decorations 1 §4), read as the
        // column rules are; `rule-*` sets both.
        n if n.starts_with("row-rule") || n.starts_with("rule") => {
            let column = format!("column-{}", n.strip_prefix("row-").unwrap_or(n));
            let both = !n.starts_with("row-");
            let parts = lower(&column, value)?;
            parts.map(|parts| {
                parts
                    .into_iter()
                    .flat_map(|(k, v)| {
                        let row = match k {
                            "columnRuleWidth" => "rowRuleWidth",
                            "columnRuleStyle" => "rowRuleStyle",
                            _ => "rowRuleColor",
                        };
                        let mut out = vec![(row, v.clone())];
                        if both {
                            out.push((k, v));
                        }
                        out
                    })
                    .collect()
            })
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
