//! CSS Grid track lists, line names and the grid shorthands (CSS Grid §7,
//! §8.3), read from their text: a track list's `[names]` are no tokens of
//! the declaration tokenizer, so these declarations are lowered here.

use super::Value;
use crate::element::{AutoRepeat, Breadth, GridLine, Lines, Template, Track};

/// A grid axis's tracks and lines (`grid-template-columns`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackList {
    pub tracks: Vec<Track>,
    pub lines: Lines,
}

/// The top-level words of `s`: split at whitespace outside `()` and `[]`,
/// with each `[…]` its own word.
fn words(s: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut bracket = false;
    let mut quote: Option<char> = None;
    for c in s.chars() {
        if let Some(q) = quote {
            cur.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return Err("a ')' closes nothing".into());
                }
            }
            '[' if depth == 0 => {
                if bracket {
                    return Err("a '[' inside line names".into());
                }
                if !cur.trim().is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.clear();
                bracket = true;
                continue;
            }
            ']' if depth == 0 => {
                if !bracket {
                    return Err("a ']' closes nothing".into());
                }
                cur.push(']');
                out.push(format!("[{}", std::mem::take(&mut cur)));
                bracket = false;
                continue;
            }
            c if c.is_whitespace() && depth == 0 && !bracket => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if depth != 0 || bracket || quote.is_some() {
        return Err("close each '(', '[' and quote".into());
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

/// `[a b]` → its names.
fn names(w: &str) -> Option<Result<Vec<String>, String>> {
    let inner = w.strip_prefix('[')?.strip_suffix(']')?;
    Some(
        inner
            .split_whitespace()
            .map(|n| {
                if super::syntax::identifier(n) && !matches!(n, "span" | "auto") {
                    Ok(n.to_string())
                } else {
                    Err(format!("'{n}' is not a line name"))
                }
            })
            .collect(),
    )
}

/// The arguments of `f(…)` split at top-level commas.
fn args<'a>(w: &'a str, f: &str) -> Option<Vec<&'a str>> {
    let inner = w.strip_prefix(f)?.strip_prefix('(')?.strip_suffix(')')?;
    let mut out = Vec::new();
    let (mut depth, mut from) = (0i32, 0);
    for (i, c) in inner.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(inner[from..i].trim());
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push(inner[from..].trim());
    Some(out)
}

/// A length in CSS px: a bare number (px), `<n><unit>` of an absolute
/// unit, or a length token.
fn length(w: &str) -> Option<f32> {
    if let Ok(v) = w.parse::<f32>() {
        return Some(v);
    }
    if let Some(name) = w.strip_prefix("var(--").and_then(|r| r.strip_suffix(')')) {
        return crate::Theme::default().length(name);
    }
    let split = w
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e')))
        .filter(|&i| i > 0)?;
    let (n, unit) = w.split_at(split);
    let n: f64 = n.parse().ok()?;
    let k = if unit == "px" {
        1.0
    } else {
        super::calc::physical(unit)?
    };
    Some((n * k) as f32)
}

fn breadth(w: &str) -> Result<Breadth, String> {
    if let Some(v) = length(w) {
        return Ok(Breadth::Px(v));
    }
    if let Some(p) = w.strip_suffix('%').and_then(|p| p.parse::<f32>().ok()) {
        return Ok(Breadth::Percent(p / 100.0));
    }
    if let Some(f) = w.strip_suffix("fr").and_then(|f| f.parse::<f32>().ok()) {
        if f < 0.0 {
            return Err(format!("'{w}' is negative"));
        }
        return Ok(Breadth::Fr(f));
    }
    match w {
        "auto" => Ok(Breadth::Auto),
        "min-content" => Ok(Breadth::MinContent),
        "max-content" => Ok(Breadth::MaxContent),
        _ => Err(format!(
            "'{w}' is not a track size: <length>, <n>%, <n>fr, auto, min-content, max-content, minmax(…) or fit-content(…)"
        )),
    }
}

/// One `<track-size>` (or a house `minmax:<min>:<max>`).
fn track(w: &str) -> Result<Track, String> {
    if let Some(r) = w.strip_prefix("minmax:") {
        let (a, b) = r.split_once(':').ok_or("minmax:<min>:<max>")?;
        let n = |v: &str| length(v).ok_or_else(|| "minmax:<min>:<max>".to_string());
        return Ok(Track::MinMax {
            min: n(a)?,
            max: n(b)?,
        });
    }
    if let Some(a) = args(w, "minmax") {
        let [min, max] = a.as_slice() else {
            return Err("minmax(<min>, <max>)".into());
        };
        let (min, max) = (breadth(min)?, breadth(max)?);
        if matches!(min, Breadth::Fr(_)) {
            return Err("a minmax() minimum is not an fr".into());
        }
        return Ok(match (min, max) {
            (Breadth::Px(a), Breadth::Px(b)) => Track::MinMax { min: a, max: b },
            (min, max) => Track::Range { min, max },
        });
    }
    if let Some(a) = args(w, "fit-content") {
        return match a.as_slice() {
            [l] => match breadth(l)? {
                b @ (Breadth::Px(_) | Breadth::Percent(_)) => Ok(Track::FitContent(b)),
                _ => Err("fit-content(<length> | <n>%)".into()),
            },
            _ => Err("fit-content(<length> | <n>%)".into()),
        };
    }
    Ok(match breadth(w)? {
        Breadth::Px(v) => Track::Px(v),
        Breadth::Percent(p) => Track::Percent(p),
        Breadth::Fr(f) => Track::Fr(f),
        Breadth::Auto => Track::Auto,
        Breadth::MinContent => Track::MinContent,
        Breadth::MaxContent => Track::MaxContent,
    })
}

/// Names and tracks of a list, the names of line `k` at `names[k]`.
fn list(ws: &[String]) -> Result<(Vec<Track>, Vec<Vec<String>>), String> {
    let mut tracks = Vec::new();
    let mut lines: Vec<Vec<String>> = vec![Vec::new()];
    for w in ws {
        if let Some(n) = names(w) {
            lines.last_mut().unwrap().extend(n?);
        } else {
            tracks.push(track(w)?);
            lines.push(Vec::new());
        }
    }
    Ok((tracks, lines))
}

/// `<track-list>`, `<auto-track-list>`, `none`, `subgrid [<line-names>]*`
/// or `masonry` (CSS Grid §7.2; the house's bare numbers are px).
pub fn parse(text: &str) -> Result<TrackList, String> {
    let ws = words(text.trim())?;
    let mut out = TrackList::default();
    match ws.first().map(String::as_str) {
        None | Some("none") if ws.len() <= 1 => return Ok(out),
        Some("masonry") if ws.len() == 1 => {
            out.lines.kind = Template::Masonry;
            return Ok(out);
        }
        Some("subgrid") => {
            out.lines.kind = Template::Subgrid;
            for w in &ws[1..] {
                if let Some(a) = args(w, "repeat") {
                    let [n, body] = a.as_slice() else {
                        return Err("repeat(<n>, [names]…)".into());
                    };
                    let n: usize = n.parse().map_err(|_| "a subgrid repeats <n> times")?;
                    let group: Vec<Vec<String>> = words(body)?
                        .iter()
                        .map(|w| names(w).ok_or("a subgrid repeats line names")?)
                        .collect::<Result<_, _>>()?;
                    for _ in 0..n {
                        out.lines.names.extend(group.iter().cloned());
                    }
                } else {
                    out.lines
                        .names
                        .push(names(w).ok_or("a subgrid takes line names: subgrid [a] [b]")??);
                }
            }
            return Ok(out);
        }
        _ => {}
    }
    let mut lines: Vec<Vec<String>> = vec![Vec::new()];
    for w in &ws {
        if let Some(n) = names(w) {
            lines.last_mut().unwrap().extend(n?);
            continue;
        }
        if let Some(a) = args(w, "repeat") {
            let [n, body] = a.as_slice() else {
                return Err("repeat(<n> | auto-fill | auto-fit, <tracks>)".into());
            };
            let (tracks, names) = list(&words(body)?)?;
            if tracks.is_empty() {
                return Err("repeat() repeats at least one track".into());
            }
            match *n {
                "auto-fill" | "auto-fit" => {
                    if out.lines.repeat.is_some() {
                        return Err("a track list takes one auto-fill or auto-fit repeat()".into());
                    }
                    if tracks.iter().any(|t| {
                        let (min, max, _) = t.breadths();
                        !matches!(min, Breadth::Px(_) | Breadth::Percent(_))
                            && !matches!(max, Breadth::Px(_) | Breadth::Percent(_))
                    }) {
                        return Err(format!(
                            "repeat({n}, …) repeats fixed sizes: a length, a percentage or minmax() with one"
                        ));
                    }
                    let at = out.tracks.len();
                    let before = std::mem::take(lines.last_mut().unwrap());
                    let mut names = names;
                    names[0] = [before, std::mem::take(&mut names[0])].concat();
                    out.lines.repeat = Some(AutoRepeat {
                        at,
                        fit: *n == "auto-fit",
                        tracks,
                        names,
                        after: Vec::new(),
                    });
                    // The names after it join its last line when it expands.
                    lines.last_mut().unwrap().clear();
                }
                n => {
                    let n: usize = n
                        .parse()
                        .ok()
                        .filter(|&n| n > 0)
                        .ok_or_else(|| format!("'{n}' is not a repeat count"))?;
                    for _ in 0..n {
                        lines.last_mut().unwrap().extend(names[0].iter().cloned());
                        for (t, nm) in tracks.iter().zip(&names[1..]) {
                            out.tracks.push(*t);
                            lines.push(nm.clone());
                        }
                    }
                }
            }
            continue;
        }
        out.tracks.push(track(w)?);
        lines.push(Vec::new());
    }
    if let Some(r) = &mut out.lines.repeat {
        // Line `at` (just before the repeat) keeps no names of its own:
        // the repeat's first line holds them; the names written after it
        // are on line `at` of `lines` until the next track.
        r.after = std::mem::take(&mut lines[r.at]);
    }
    if lines.iter().any(|l| !l.is_empty()) || out.lines.repeat.is_some() {
        out.lines.names = lines;
    }
    Ok(out)
}

/// The track list values of a `Ty::Tracks` prop (one CSS text, or the
/// house's words and numbers).
pub fn of_values(vals: &[Value]) -> Result<TrackList, String> {
    let text: Vec<String> = vals
        .iter()
        .map(|v| match v {
            Value::Number(n) => n.text().to_string(),
            Value::Word(w) | Value::String(w) => w.clone(),
        })
        .collect();
    parse(&text.join(" "))
}

/// A prop and its values.
pub(crate) type Lowered = (&'static str, Vec<Value>);

/// The lowered entries of a grid declaration read from its text, or
/// `None` for one the token path lowers.
pub(crate) fn lower(name: &str, value: &str) -> Option<Result<Vec<Lowered>, String>> {
    let v = value.trim();
    let text = |s: &str| vec![Value::String(s.trim().to_string())];
    let checked = |s: &str| parse(s).map(|_| text(s));
    Some(match name {
        "grid-template-columns" => checked(v).map(|v| vec![("columns", v)]),
        "grid-template-rows" => checked(v).map(|v| vec![("rows", v)]),
        "grid-auto-columns" | "grid-auto-rows" => (|| {
            let l = parse(v)?;
            if l.lines.kind != Template::Tracks || l.lines.repeat.is_some() || l.tracks.is_empty() {
                return Err(format!("{name} takes track sizes"));
            }
            Ok(vec![(
                if name == "grid-auto-columns" {
                    "autoColumns"
                } else {
                    "autoRows"
                },
                text(v),
            )])
        })(),
        "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
            GridLine::parse(v).map(|_| {
                vec![(
                    match name {
                        "grid-column-start" => "columnStart",
                        "grid-column-end" => "columnEnd",
                        "grid-row-start" => "rowStart",
                        _ => "rowEnd",
                    },
                    text(v),
                )]
            })
        }
        // The house's forms (`<n>`, `<n> / span <n>`, `span <n>`) keep
        // their numbers; any other line is its two longhands.
        "grid-column" | "grid-row" if !numeric_lines(v) => {
            let (s, e) = name_pair(name);
            line_pair(v).map(|(a, b)| vec![(s, text(&a)), (e, text(&b))])
        }
        "grid-area" if v.contains('/') => (|| {
            let parts: Vec<&str> = v.split('/').map(str::trim).collect();
            if parts.len() > 4 || parts.iter().any(|p| p.is_empty()) {
                return Err("grid-area: <line> [/ <line>]{0,3}".to_string());
            }
            for p in &parts {
                GridLine::parse(p)?;
            }
            // A missing line is its opposite's name, else auto (§8.4).
            let copy = |from: &str| {
                if super::syntax::identifier(from)
                    && GridLine::parse(from).is_ok_and(|l| matches!(l, GridLine::Line(0, _)))
                {
                    from.to_string()
                } else {
                    "auto".to_string()
                }
            };
            let rs = parts[0].to_string();
            let cs = parts.get(1).map_or_else(|| copy(&rs), |s| s.to_string());
            let re = parts.get(2).map_or_else(|| copy(&rs), |s| s.to_string());
            let ce = parts.get(3).map_or_else(|| copy(&cs), |s| s.to_string());
            Ok(vec![
                ("rowStart", text(&rs)),
                ("columnStart", text(&cs)),
                ("rowEnd", text(&re)),
                ("columnEnd", text(&ce)),
            ])
        })(),
        "grid-template" => template(v, false),
        "grid" => template(v, true),
        _ => return None,
    })
}

fn name_pair(name: &str) -> (&'static str, &'static str) {
    if name == "grid-column" {
        ("columnStart", "columnEnd")
    } else {
        ("rowStart", "rowEnd")
    }
}

/// `<n>`, `<n> / span <n>` or `span <n>`.
fn numeric_lines(v: &str) -> bool {
    let int = |s: &str| s.trim().parse::<u32>().is_ok_and(|n| n > 0);
    let span = |s: &str| {
        s.trim()
            .strip_prefix("span")
            .is_some_and(|n| n.starts_with(char::is_whitespace) && int(n))
    };
    match v.split_once('/') {
        Some((a, b)) => int(a) && span(b),
        None => int(v) || span(v),
    }
}

/// `<line> [/ <line>]`: a lone name ends at the same name (§8.4).
fn line_pair(v: &str) -> Result<(String, String), String> {
    let (a, b) = match v.split_once('/') {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (v.trim(), None),
    };
    GridLine::parse(a)?;
    let b = match b {
        Some(b) => {
            GridLine::parse(b)?;
            b.to_string()
        }
        None if matches!(GridLine::parse(a)?, GridLine::Line(0, _)) => a.to_string(),
        None => "auto".into(),
    };
    Ok((a.to_string(), b))
}

/// `grid-template` (or with `grid`, also `grid`'s auto-flow forms): every
/// sub-property is set, the ones not written to their initial values.
fn template(v: &str, grid: bool) -> Result<Vec<Lowered>, String> {
    let text = |s: &str| vec![Value::String(s.trim().to_string())];
    let word = |s: &str| vec![Value::Word(s.into())];
    let mut out: Vec<Lowered> = Vec::new();
    let reset_auto = |out: &mut Vec<Lowered>| {
        if grid {
            out.extend([
                ("autoColumns", text("auto")),
                ("autoRows", text("auto")),
                ("flow", word("row")),
                ("dense", word("false")),
            ]);
        }
    };
    if v == "none" {
        out.extend([("columns", text("none")), ("rows", text("none"))]);
        out.push(("areas", Vec::new()));
        reset_auto(&mut out);
        return Ok(out);
    }
    let (left, right) = match split_slash(v) {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (v, None),
    };
    // `grid`'s auto-flow forms.
    let auto_flow = |side: &str| -> Option<(bool, String)> {
        let ws: Vec<&str> = side.split_whitespace().collect();
        let mut dense = false;
        let mut flow = false;
        let mut rest = Vec::new();
        for w in ws {
            match w {
                "auto-flow" if !flow => flow = true,
                "dense" if !dense => dense = true,
                w => rest.push(w),
            }
        }
        flow.then(|| (dense, rest.join(" ")))
    };
    if grid {
        if let Some(r) = right
            && let Some((dense, cols)) = auto_flow(r)
        {
            parse(left)?;
            out.extend([
                ("rows", text(left)),
                ("columns", text("none")),
                ("areas", Vec::new()),
                ("autoRows", text("auto")),
                (
                    "autoColumns",
                    text(if cols.is_empty() { "auto" } else { &cols }),
                ),
                ("flow", word("column")),
                ("dense", word(if dense { "true" } else { "false" })),
            ]);
            parse(if cols.is_empty() { "auto" } else { &cols })?;
            return Ok(out);
        }
        if let Some((dense, rows)) = auto_flow(left) {
            let cols = right.ok_or("grid: auto-flow [<rows>] / <columns>")?;
            parse(cols)?;
            parse(if rows.is_empty() { "auto" } else { &rows })?;
            out.extend([
                ("columns", text(cols)),
                ("rows", text("none")),
                ("areas", Vec::new()),
                ("autoColumns", text("auto")),
                (
                    "autoRows",
                    text(if rows.is_empty() { "auto" } else { &rows }),
                ),
                ("flow", word("row")),
                ("dense", word(if dense { "true" } else { "false" })),
            ]);
            return Ok(out);
        }
    }
    if left.contains('"') || left.contains('\'') {
        // [<line-names>? <string> <track-size>? <line-names>?]+ [/ <columns>]?
        let ws = words(left)?;
        let mut areas = Vec::new();
        let mut rows = String::new();
        let mut i = 0;
        while i < ws.len() {
            let w = &ws[i];
            if let Some(s) = w
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| w.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            {
                areas.push(s.split_whitespace().collect::<Vec<_>>().join(" "));
                let size = ws
                    .get(i + 1)
                    .filter(|n| !n.starts_with('[') && !n.starts_with('"') && !n.starts_with('\''));
                match size {
                    Some(sz) => {
                        track(sz)?;
                        rows.push_str(sz);
                        i += 1;
                    }
                    None => rows.push_str("auto"),
                }
                rows.push(' ');
            } else if names(w).is_some() {
                rows.push_str(w);
                rows.push(' ');
            } else {
                return Err(format!(
                    "grid-template: '{w}' is not an area row: [names] \"areas\" [size] [names]"
                ));
            }
            i += 1;
        }
        let cols = right.unwrap_or("none");
        let l = parse(cols)?;
        if l.lines.repeat.is_some() {
            return Err("grid-template's columns after areas take no auto repeat".into());
        }
        out.extend([("rows", text(&rows)), ("columns", text(cols))]);
        out.push(("areas", areas.into_iter().map(Value::String).collect()));
        reset_auto(&mut out);
        return Ok(out);
    }
    let cols = right.ok_or_else(|| {
        format!(
            "{}: <rows> / <columns>{}",
            if grid { "grid" } else { "grid-template" },
            if grid {
                ", <rows> / auto-flow [dense] [<columns>] or auto-flow [dense] [<rows>] / <columns>"
            } else {
                ""
            }
        )
    })?;
    parse(left)?;
    parse(cols)?;
    out.extend([
        ("rows", text(left)),
        ("columns", text(cols)),
        ("areas", Vec::new()),
    ]);
    reset_auto(&mut out);
    Ok(out)
}

/// `a / b` split at its top-level slash outside strings and brackets.
fn split_slash(v: &str) -> Option<(&str, &str)> {
    let (mut depth, mut quote) = (0i32, None);
    for (i, c) in v.char_indices() {
        match c {
            '"' | '\'' if quote == Some(c) => quote = None,
            '"' | '\'' if quote.is_none() => quote = Some(c),
            '(' | '[' if quote.is_none() => depth += 1,
            ')' | ']' if quote.is_none() => depth -= 1,
            '/' if quote.is_none() && depth == 0 => return Some((&v[..i], &v[i + 1..])),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_lists() {
        let l = parse("[a] 100px [b c] repeat(2, 1fr [d]) minmax(min-content, 2fr)").unwrap();
        assert_eq!(
            l.tracks,
            vec![
                Track::Px(100.0),
                Track::Fr(1.0),
                Track::Fr(1.0),
                Track::Range {
                    min: Breadth::MinContent,
                    max: Breadth::Fr(2.0)
                }
            ]
        );
        assert_eq!(
            l.lines.names,
            vec![
                vec!["a".to_string()],
                vec!["b".into(), "c".into()],
                vec!["d".into()],
                vec!["d".into()],
                vec![]
            ]
        );
        let r = parse("[x] 10px repeat(auto-fill, [s] minmax(120px, 1fr) [e]) [y] 20px").unwrap();
        let rep = r.lines.repeat.unwrap();
        assert_eq!((rep.at, rep.fit), (1, false));
        assert_eq!(rep.names, vec![vec!["s".to_string()], vec!["e".into()]]);
        assert_eq!(rep.after, vec!["y".to_string()]);
        assert!(parse("repeat(auto-fill, 1fr)").is_err());
        assert_eq!(
            parse("subgrid [a] [b]").unwrap().lines.kind,
            Template::Subgrid
        );
        // The house's words.
        assert_eq!(
            parse("120 minmax:80:200 25%").unwrap().tracks,
            vec![
                Track::Px(120.0),
                Track::MinMax {
                    min: 80.0,
                    max: 200.0
                },
                Track::Percent(0.25)
            ]
        );
    }

    #[test]
    fn shorthands() {
        let t = template("[top] \"a a\" 40px [mid] \"b c\" / 1fr 2fr", false).unwrap();
        let get = |k: &str| t.iter().find(|e| e.0 == k).map(|e| e.1.clone());
        assert_eq!(
            get("rows"),
            Some(vec![Value::String("[top] 40px [mid] auto".into())])
        );
        assert_eq!(
            get("areas"),
            Some(vec![
                Value::String("a a".into()),
                Value::String("b c".into())
            ])
        );
        let g = template("auto-flow dense 50px / 1fr 1fr", true).unwrap();
        assert!(g.contains(&("dense", vec![Value::Word("true".into())])));
        assert!(g.contains(&("autoRows", vec![Value::String("50px".into())])));
        assert_eq!(
            line_pair("header").unwrap(),
            ("header".to_string(), "header".to_string())
        );
        assert!(numeric_lines("2 / span 3") && !numeric_lines("2 / 4"));
    }
}
