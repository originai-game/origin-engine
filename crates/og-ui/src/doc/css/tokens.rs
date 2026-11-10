use super::*;

// ---- reading CSS values

pub(crate) fn want_len(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Func(f, _) if f == "var" => {
            let name = kebab(&want_var(t, "")?);
            Err(format!("unresolved CSS variable --{name}"))
        }
        Tok::Num(n, u)
            if u == "px"
                || (u.is_empty()
                    && n.trim_start_matches('-')
                        .trim_matches(['0', '.'])
                        .is_empty()) =>
        {
            number(n)
        }
        Tok::Num(n, u) if u.is_empty() => Err(format!("'{n}' needs its unit: {n}px")),
        Tok::Func(f, a) if f == "bind" => Err(format!(
            "bind({}) binds a whole declaration",
            a.iter()
                .flatten()
                .map(describe)
                .collect::<Vec<_>>()
                .join(" ")
        )),
        t => Err(format!("'{}' is not a length (px)", describe(t))),
    }
}

/// A margin edge as its px, em, fraction of the parent's content width and
/// auto flag: `12px`, `0.67em`, `5%`, `0`, `auto` or a reduced
/// `calc(50% - 12px)`.
pub(super) fn margin_vals(t: &Tok) -> Result<Vec<Value>, String> {
    let n = |v: f64| number(&super::calc::number(v));
    if matches!(t, Tok::Ident(w) if w == "auto") {
        return [0.0, 0.0, 0.0, 1.0].into_iter().map(n).collect();
    }
    if let Tok::Func(f, _) = t
        && f == "var"
    {
        let v = want_len(t)?;
        return Ok(vec![v, n(0.0)?, n(0.0)?, n(0.0)?]);
    }
    let [pct, px, em] = want_lin(t)?;
    [px, em, pct, 0.0].into_iter().map(n).collect()
}

/// A size's reduced math function as its ogt form, `calc <op> <pct> <px>
/// …`: `calc(100% - 12px)` → `calc sum 1 -12`, `min(100%, 600px)` → `calc
/// min 1 0 0 600`.
pub(super) fn size_calc(name: &str, f: &str, args: &[Vec<Tok>]) -> Result<Vec<Value>, String> {
    // `round(<strategy>, …)` names its strategy first.
    let (op, args) = match (f, args.first().map(Vec::as_slice)) {
        ("round", Some([Tok::Ident(s)])) => (
            match s.as_str() {
                "nearest" => "round",
                "up" => "roundUp",
                "down" => "roundDown",
                "to-zero" => "roundToZero",
                s => return Err(format!("{name}: '{s}' is not a rounding strategy")),
            },
            &args[1..],
        ),
        ("calc", _) => ("sum", args),
        _ => (f, args),
    };
    let lengths: Vec<[f64; 3]> = if f == "calc" {
        vec![want_lin(&Tok::Func(f.into(), args.to_vec()))?]
    } else {
        args.iter()
            .map(|arg| match arg.as_slice() {
                [t] => want_lin(t),
                _ => Err(format!("{f}() takes one length an argument")),
            })
            .collect::<Result<_, _>>()?
    };
    let (count, how) = match f {
        "calc" | "abs" => (1..=1, "one length"),
        "clamp" => (3..=3, "three lengths"),
        "round" => (1..=2, "a length and its step"),
        "mod" | "rem" => (2..=2, "two lengths"),
        _ => (1..=3, "one to three lengths"),
    };
    if !count.contains(&lengths.len()) {
        return Err(format!("{name}: {f}() takes {how}"));
    }
    if lengths.iter().any(|l| l[2] != 0.0) {
        return Err(format!("{name} takes px and %, not em"));
    }
    let mut out = vec![Value::Word("calc".into()), Value::Word(op.into())];
    for [pct, px, _] in lengths {
        out.push(number(&super::calc::number(pct))?);
        out.push(number(&super::calc::number(px))?);
    }
    Ok(out)
}

/// A length as its fraction, px and em terms: `12px`, `50%`, `1.5em`, `0`,
/// or a `calc()` doc/calc.rs reduced to a sum (`calc(50% - 12px + 1em)`).
pub(crate) fn want_lin(t: &Tok) -> Result<[f64; 3], String> {
    fn term(t: &Tok, sign: f64, out: &mut [f64; 3]) -> Result<(), String> {
        let Tok::Num(n, u) = t else {
            return Err(format!("'{}' is not a length (px, %, em)", describe(t)));
        };
        let v: f64 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
        let k = match u.as_str() {
            "%" => 0,
            "px" => 1,
            "em" => 2,
            "" if v == 0.0 => 1,
            _ => return Err(format!("'{n}{u}' is not a length (px, %, em)")),
        };
        out[k] += sign * if k == 0 { v / 100.0 } else { v };
        Ok(())
    }
    let mut out = [0.0; 3];
    match t {
        Tok::Func(f, args) if f == "calc" => {
            let [toks] = args.as_slice() else {
                return Err("calc() takes one expression".into());
            };
            let mut sign = 1.0;
            for (k, t) in toks.iter().enumerate() {
                match t {
                    Tok::Ident(op) if k % 2 == 1 && (op == "+" || op == "-") => {
                        sign = if op == "-" { -1.0 } else { 1.0 };
                    }
                    t if k % 2 == 0 => term(t, sign, &mut out)?,
                    t => return Err(format!("'{}' is not part of a reduced calc()", describe(t))),
                }
            }
        }
        Tok::Func(f, _) if matches!(f.as_str(), "min" | "max" | "clamp") => {
            return Err(format!(
                "{f}() over lengths in more than one unit takes a width or a height; a margin takes calc()"
            ));
        }
        t => term(t, 1.0, &mut out)?,
    }
    Ok(out)
}

pub(crate) fn want_num(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u.is_empty() => number(n),
        t => Err(format!("'{}' is not a number", describe(t))),
    }
}

pub(super) fn want_angle(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "rad" => number(n),
        Tok::Num(n, u)
            if u.is_empty()
                && n.trim_start_matches('-')
                    .trim_matches(['0', '.'])
                    .is_empty() =>
        {
            number(n)
        }
        Tok::Num(n, u) if u == "deg" || u == "turn" || u == "grad" => {
            let v: f64 = n.parse().map_err(|_| format!("'{n}' is not a number"))?;
            let r = match u.as_str() {
                "deg" => v.to_radians(),
                "grad" => (v * 0.9).to_radians(),
                _ => v * std::f64::consts::TAU,
            };
            Ok(Value::Number(
                Number::from_f32(r as f32).map_err(|e| e.reason().to_string())?,
            ))
        }
        t => Err(format!(
            "'{}' is not an angle (rad, deg or turn)",
            describe(t)
        )),
    }
}

pub(crate) fn want_ms(t: &Tok) -> Result<Value, String> {
    match t {
        Tok::Num(n, u) if u == "ms" => number(n),
        Tok::Num(n, u) if u == "s" => number(&shift(n, 3)),
        t => Err(format!("'{}' is not a duration (ms or s)", describe(t))),
    }
}

pub(super) fn want_ident<'a>(t: &'a Tok, what: &str) -> Result<&'a str, String> {
    match t {
        Tok::Ident(s) => Ok(s),
        t => Err(format!("'{}' is not {what}", describe(t))),
    }
}

/// `var(--<ns>-<name>)`'s name (camel case).
pub(super) fn want_var(t: &Tok, ns: &str) -> Result<String, String> {
    match t {
        Tok::Func(f, a) if f == "var" => match a.as_slice() {
            [v] => match v.as_slice() {
                [Tok::Ident(n)] => {
                    let n = n.strip_prefix("--").ok_or("var(--name)")?;
                    match ns {
                        "" => Ok(camel(n)),
                        _ => n
                            .strip_prefix(ns)
                            .and_then(|r| r.strip_prefix('-'))
                            .map(camel)
                            .ok_or_else(|| format!("var(--{n}) is not a --{ns}- token")),
                    }
                }
                _ => Err("var(--name)".into()),
            },
            _ => Err("var(--name) takes no fallback".into()),
        },
        t => Err(format!(
            "'{}' is not a token: var(--{ns}{}…)",
            describe(t),
            if ns.is_empty() { "" } else { "-" }
        )),
    }
}

/// Preserve CSS colour text until typed parsing; converting to hex here
/// quantizes fractional alpha before the painter receives it.
pub(crate) fn paint_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
    match t {
        [Tok::Func(f, _)] if f == "var" => Err("unresolved CSS colour variable".into()),
        [t] => {
            crate::css_color::parse(t)?;
            Ok(vec![Value::String(tok_text(t))])
        }
        _ => Err(
            "a colour: a named colour, #hex, rgb(), hsl(), oklch(), color-mix(), var(--role) …"
                .into(),
        ),
    }
}

/// `#rrggbb`, or `#rrggbbaa` when not opaque.
pub(crate) fn hex(c: crate::Srgba) -> String {
    let [r, g, b] = c.rgb;
    let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

pub(super) fn transition_vals(t: &[Tok]) -> Result<Vec<Value>, String> {
    let t = match t.first() {
        Some(Tok::Ident(a)) if a == "all" => &t[1..],
        _ => t,
    };
    match t {
        [Tok::Ident(n)] if n == "none" => Ok(vec![Value::Word("instant".into())]),
        [v @ Tok::Func(f, _)] if f == "var" => Ok(vec![
            Value::Word("token".into()),
            Value::Word(want_var(v, "motion")?),
        ]),
        [Tok::Func(f, a)] if f == "-og-spring" => {
            let mut out = vec![Value::Word("spring".into())];
            match a.as_slice() {
                [d] => out.push(want_ms(d.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?),
                [d, b] => {
                    out.push(want_ms(d.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?);
                    out.push(want_num(b.first().ok_or("-og-spring(<ms>ms, <bounce>)")?)?);
                }
                _ => return Err("-og-spring(<ms>ms, <bounce>)".into()),
            }
            Ok(out)
        }
        [d, rest @ ..] if matches!(d, Tok::Num(..)) => {
            let mut out = vec![Value::Word("tween".into()), want_ms(d)?];
            match rest {
                [] => {}
                [Tok::Ident(e)] => out.push(Value::Word(
                    match e.as_str() {
                        "linear" => "linear",
                        "ease-out" => "out",
                        "ease-in" => "exit",
                        "ease-in-out" => "inOut",
                        _ => {
                            return Err(format!(
                                "'{e}' is not an ease: linear, ease-out, ease-in, ease-in-out"
                            ));
                        }
                    }
                    .into(),
                )),
                _ => return Err("<ms>ms [ease]".into()),
            }
            Ok(out)
        }
        _ => Err(
            "a transition: none, <ms>ms [ease], -og-spring(<ms>ms, <bounce>) or var(--motion-<name>)"
                .into(),
        ),
    }
}

pub(super) fn one(t: &[Tok], name: &str) -> Result<Tok, String> {
    match t {
        [v] => Ok(v.clone()),
        _ => Err(format!("{name} takes one value")),
    }
}

/// A grid line: `<start>`, `<start> / span <n>` or `span <n>` (0: auto).
pub(super) fn grid_line(t: &[Tok]) -> Result<(Option<Value>, Option<Value>), String> {
    let span = |t: &[Tok]| match t {
        [Tok::Ident(s), n] if s == "span" => want_num(n).map(Some),
        _ => Err("span <n>".to_string()),
    };
    match t {
        [n @ Tok::Num(..)] => Ok((Some(want_num(n)?), None)),
        [n @ Tok::Num(..), Tok::Slash, rest @ ..] => Ok((Some(want_num(n)?), span(rest)?)),
        [Tok::Ident(s), _] if s == "span" => Ok((None, span(t)?)),
        _ => Err("a grid line: <n>, <n> / span <n> or span <n>".into()),
    }
}

/// A `display` value as its prop word (CSS Display 3 §2): one keyword or
/// the multi-keyword form `[block | inline | run-in] [flow | flow-root |
/// flex | grid | table] [list-item]`.
pub(super) fn display_word(v: &str) -> Result<&'static str, String> {
    let one = match v {
        "block" | "run-in" => Some("block"),
        "inline" => Some("inline"),
        "inline-block" => Some("inlineBlock"),
        "flow-root" => Some("flowRoot"),
        "list-item" => Some("listItem"),
        "flex" => Some("flex"),
        "inline-flex" => Some("inlineFlex"),
        "grid" => Some("grid"),
        "inline-grid" => Some("inlineGrid"),
        "table" => Some("table"),
        "inline-table" => Some("inlineTable"),
        "contents" => Some("contents"),
        "table-row-group" => Some("tableRowGroup"),
        "table-header-group" => Some("tableHeaderGroup"),
        "table-footer-group" => Some("tableFooterGroup"),
        "table-row" => Some("tableRow"),
        "table-cell" => Some("tableCell"),
        "table-caption" => Some("tableCaption"),
        "table-column" => Some("tableColumn"),
        "table-column-group" => Some("tableColumnGroup"),
        _ => None,
    };
    if let Some(w) = one {
        return Ok(w);
    }
    let how = || {
        format!(
            "'{v}' is not a display: block, inline, inline-block, flow-root, flex, grid, table, contents, none, list-item, a table part, or [block | inline] [flow | flow-root | flex | grid | table] [list-item]"
        )
    };
    let (mut outer, mut inner, mut item) = (None, None, false);
    for w in v.split_whitespace() {
        match w {
            "block" | "inline" | "run-in" if outer.is_none() => {
                outer = Some(if w == "inline" { "inline" } else { "block" })
            }
            "flow" | "flow-root" | "flex" | "grid" | "table" if inner.is_none() => inner = Some(w),
            "list-item" if !item => item = true,
            _ => return Err(how()),
        }
    }
    let inline = outer == Some("inline");
    Ok(match (inline, inner.unwrap_or("flow"), item) {
        (false, "flow", true) | (false, "flow-root", true) => "listItem",
        (_, "flex" | "grid" | "table", true) => return Err(how()),
        (true, _, true) => return Err(how()),
        (false, "flow", false) => "block",
        (true, "flow", false) => "inline",
        (false, "flow-root", false) => "flowRoot",
        (true, "flow-root", false) => "inlineBlock",
        (false, "flex", false) => "flex",
        (true, "flex", false) => "inlineFlex",
        (false, "grid", false) => "grid",
        (true, "grid", false) => "inlineGrid",
        (false, "table", false) => "table",
        (true, "table", false) => "inlineTable",
        _ => return Err(how()),
    })
}
