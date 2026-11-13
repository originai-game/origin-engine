use super::*;

// ---- element values to CSS

fn px(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(n) if n.text() == "0" => Ok("0".into()),
        Value::Number(n) => Ok(format!("{}px", n.text())),
        Value::String(s) if s.starts_with("var(--") => Ok(s.clone()),
        other => Err(format!("'{}' is not a length", text_of(other))),
    }
}

fn bound(vals: &[Value]) -> Option<Result<String, String>> {
    match vals {
        [Value::Word(b), e] if b == "bind" => {
            let src = text_of(e);
            let simple = !src.is_empty()
                && src
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$'));
            Some(Ok(if simple {
                format!("bind({src})")
            } else {
                format!("bind({})", quoted(&src))
            }))
        }
        [Value::Word(b), ..] if b == "bind" => Some(Err(
            "a declaration binds without a format (a format is a text's: `text bind gold compact`)"
                .into(),
        )),
        _ => None,
    }
}

fn paint_css(vals: &[Value]) -> Result<String, String> {
    match vals {
        [Value::Word(t), Value::Word(role)] if t == "token" => {
            Ok(format!("var(--{})", kebab(role)))
        }
        [Value::String(s)] | [Value::Word(s)] if s.starts_with('#') => Ok(s.clone()),
        [Value::Word(role)] | [Value::String(role)] => Ok(format!("var(--{})", kebab(role))),
        _ => Err("a paint: token <role> or \"#rrggbb\"".into()),
    }
}

fn transition_css(vals: &[Value]) -> Result<String, String> {
    let w: Vec<String> = vals.iter().map(text_of).collect();
    let num = |i: usize| w.get(i).cloned().ok_or("a transition needs its numbers");
    match w.first().map(String::as_str) {
        Some("instant") => Ok("none".into()),
        Some("spring") => Ok(match w.get(2) {
            Some(b) => format!("-og-spring({}ms, {b})", num(1)?),
            None => format!("-og-spring({}ms)", num(1)?),
        }),
        Some("tween") => {
            let ease = match w.get(2).map(String::as_str) {
                None => "",
                Some("linear") => " linear",
                Some("out") => " ease-out",
                Some("exit") => " ease-in",
                Some("inOut") => " ease-in-out",
                Some(e) => return Err(format!("'{e}' is not an ease")),
            };
            Ok(format!("{}ms{ease}", num(1)?))
        }
        Some("token") => Ok(format!("var(--motion-{})", kebab(&num(1)?))),
        Some(t) => Ok(format!("var(--motion-{})", kebab(t))),
        None => Err("a transition".into()),
    }
}

fn track_css(v: &Value) -> Result<String, String> {
    match v {
        Value::Number(_) => px(v),
        Value::Word(s) | Value::String(s) => match s.strip_prefix("minmax:") {
            Some(r) => {
                let (a, b) = r.split_once(':').ok_or("minmax:<min>:<max>")?;
                let len = |t: &str| {
                    if t == "0" || t.starts_with("var(--") {
                        t.to_string()
                    } else {
                        format!("{t}px")
                    }
                };
                Ok(format!("minmax({}, {})", len(a), len(b)))
            }
            None => Ok(s.clone()),
        },
    }
}

/// One element value as its CSS value (`conv` of a prop not written jointly).
fn value_css(prop: &str, c: Conv, vals: &[Value]) -> Result<String, String> {
    if let Some(b) = bound(vals) {
        return b;
    }
    let one = || match vals {
        [v] => Ok(v),
        _ => Err(format!("{prop} takes one value")),
    };
    match c {
        Conv::Len => px(one()?),
        Conv::Num => match one()? {
            Value::Number(n) => Ok(n.text().to_string()),
            v => Err(format!("'{}' is not a number", text_of(v))),
        },
        Conv::Angle => match one()? {
            Value::Number(n) if n.text() == "0" => Ok("0".into()),
            Value::Number(n) => Ok(format!("{}rad", n.text())),
            v => Err(format!("'{}' is not an angle", text_of(v))),
        },
        Conv::Paint => paint_css(vals),
        Conv::Margin => match vals {
            [_, _, _, Value::Number(a)] if a.text() != "0" => Ok("auto".into()),
            [Value::Number(p), Value::Number(e), Value::Number(c), _] => {
                let pct = super::calc::number(c.as_f32() as f64 * 100.0);
                let terms: Vec<String> = [
                    (pct, "%"),
                    (p.text().to_string(), "px"),
                    (e.text().to_string(), "em"),
                ]
                .into_iter()
                .filter(|(v, _)| v != "0")
                .map(|(v, u)| format!("{v}{u}"))
                .collect();
                Ok(match terms.as_slice() {
                    [] => "0".into(),
                    [t] => t.clone(),
                    _ => format!("calc({})", terms.join(" + ").replace("+ -", "- ")),
                })
            }
            _ => Err(format!("{prop} takes a length or auto")),
        },
        Conv::Insets => {
            let n: Vec<String> = vals.iter().map(px).collect::<Result<_, _>>()?;
            Ok(match n.as_slice() {
                [a] => a.clone(),
                [x, y] => format!("{y} {x}"),
                [l, t, r, b] => format!("{t} {r} {b} {l}"),
                _ => return Err("padding takes 1, 2 or 4 numbers".into()),
            })
        }
        Conv::Bool(t, f) => match vals {
            [] => Ok(t.into()),
            [v] => match v.as_bool() {
                Some(true) => Ok(t.into()),
                Some(false) => Ok(f.into()),
                None => Err(format!("{prop} takes true or false")),
            },
            _ => Err(format!("{prop} takes true or false")),
        },
        Conv::Kebab => Ok(kebab(&text_of(one()?))),
        Conv::Kw(table) => {
            let w = text_of(one()?);
            table
                .iter()
                .find(|(e, _)| *e == w)
                .map(|(_, c)| c.to_string())
                .ok_or_else(|| format!("'{w}' is not a {prop}"))
        }
        Conv::Var(ns) => Ok(format!("var(--{ns}-{})", kebab(&text_of(one()?)))),
        Conv::Raw => Ok(text_of(one()?)),
        Conv::Ident => {
            let w = text_of(one()?);
            Ok(if is_ident(&w) { w } else { quoted(&w) })
        }
        Conv::Strs => Ok(vals
            .iter()
            .map(|v| quoted(&text_of(v)))
            .collect::<Vec<_>>()
            .join(" ")),
        Conv::Tracks => Ok(vals
            .iter()
            .map(track_css)
            .collect::<Result<Vec<_>, _>>()?
            .join(" ")),
        Conv::Transition => transition_css(vals),
        Conv::Origin => match vals {
            [Value::Number(a), Value::Number(b)] => {
                Ok(format!("{}% {}%", shift(a.text(), 2), shift(b.text(), 2)))
            }
            _ => Err("pivot takes two fractions".into()),
        },
        Conv::Size => {
            if let [v] = vals
                && px(v).is_ok()
            {
                return px(v);
            }
            let w = vals.first().map(text_of).unwrap_or_default();
            match w.as_str() {
                "fixed" => px(vals.get(1).ok_or("fixed <px>")?),
                "fit" => Ok("fit-content".into()),
                "grow" => Ok("stretch".into()),
                "minContent" => Ok("min-content".into()),
                "maxContent" => Ok("max-content".into()),
                "percent" => Ok(format!(
                    "{}%",
                    shift(&vals.get(1).map(text_of).ok_or("percent <0–1>")?, 2)
                )),
                "calc" => {
                    let n: Vec<f64> = vals[2..]
                        .iter()
                        .map(|v| v.as_number().map_or(0.0, |n| n.as_f32() as f64))
                        .collect();
                    let sum = |k: usize| {
                        let (pct, px) = (
                            n.get(2 * k).copied().unwrap_or(0.0),
                            n.get(2 * k + 1).copied().unwrap_or(0.0),
                        );
                        super::calc::sum_text(
                            &[("%", pct * 100.0), ("px", px)]
                                .into_iter()
                                .filter(|(_, v)| *v != 0.0)
                                .collect::<Vec<_>>(),
                        )
                    };
                    let op = vals.get(1).map(text_of).unwrap_or_default();
                    let k = n.len() / 2;
                    let list = (0..k).map(sum).collect::<Vec<_>>().join(", ");
                    Ok(match op.as_str() {
                        "sum" => sum(0),
                        "clamp" => format!("clamp({}, {}, {})", sum(0), sum(1), sum(2)),
                        "roundUp" => format!("round(up, {list})"),
                        "roundDown" => format!("round(down, {list})"),
                        "roundToZero" => format!("round(to-zero, {list})"),
                        op => format!("{op}({list})"),
                    })
                }
                _ => Err("a size".into()),
            }
        }
        Conv::Joint => Err(format!("{prop} is written with its partner")),
    }
}

/// The declarations of `props` (element props, or a state's values when
/// `state`), in order, as one line: `width: stretch; max-width: 640px`.
/// Props written jointly (`x` and `y` as `translate`) share one
/// declaration; one missing its partner takes `partner(prop)`'s value
/// (the element's own, else its neutral value).
pub fn decl_line(
    props: &[(String, Vec<Value>)],
    state: bool,
    partner: &dyn Fn(&str) -> Option<Vec<Value>>,
) -> Result<String, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut done: Vec<&str> = Vec::new();
    let get = |k: &str| props.iter().find(|(p, _)| p == k).map(|(_, v)| v.clone());
    let fill = |k: &str| -> Result<String, String> {
        let v = get(k)
            .or_else(|| partner(k))
            .or_else(|| neutral(k).map(|n| vec![Value::Number(Number::parse(n).unwrap())]))
            .ok_or_else(|| format!("{k} has no value"))?;
        if let Some(b) = bound(&v) {
            return b;
        }
        match conv_joint(k) {
            Conv::Angle => value_css(k, Conv::Angle, &v),
            Conv::Len => value_css(k, Conv::Len, &v),
            _ => value_css(k, Conv::Num, &v),
        }
    };
    for (prop, vals) in props {
        let prop = prop.as_str();
        if done.contains(&prop) {
            continue;
        }
        let css = if state {
            STATE_VALUES
                .iter()
                .find(|(p, _)| *p == prop)
                .map(|(_, c)| *c)
        } else {
            css_name(prop)
        }
        .ok_or_else(|| format!("'{prop}' has no CSS form"))?;
        match prop {
            "x" | "y" => {
                done.extend(["x", "y"]);
                out.push((css.into(), format!("{} {}", fill("x")?, fill("y")?)));
            }
            "scaleX" | "scaleY" => {
                done.extend(["scaleX", "scaleY"]);
                out.push((
                    css.into(),
                    format!("{} {}", fill("scaleX")?, fill("scaleY")?),
                ));
            }
            "flow" | "dense" => {
                done.extend(["flow", "dense"]);
                let mut v = Vec::new();
                if let Some(f) = get("flow") {
                    v.push(value_css("flow", Conv::Kebab, &f)?);
                }
                if let Some(d) = get("dense")
                    && d.first().and_then(Value::as_bool).unwrap_or(true)
                {
                    v.push("dense".into());
                }
                if v.is_empty() {
                    v.push("row".into());
                }
                out.push((css.into(), v.join(" ")));
            }
            "cell" | "span" => {
                done.extend(["cell", "span"]);
                let pair = |k: &str| -> Result<Option<[String; 2]>, String> {
                    match get(k).as_deref() {
                        None => Ok(None),
                        Some([a, b]) => Ok(Some([text_of(a), text_of(b)])),
                        Some(_) => Err(format!("{k} takes two numbers")),
                    }
                };
                let (cell, span) = (pair("cell")?, pair("span")?);
                for (axis, name) in [(0, "grid-column"), (1, "grid-row")] {
                    let start = cell.as_ref().map(|c| c[axis].clone()).filter(|s| s != "0");
                    let n = span.as_ref().map(|c| c[axis].clone()).filter(|s| s != "0");
                    let v = match (start, n) {
                        (Some(s), Some(n)) => format!("{s} / span {n}"),
                        (Some(s), None) => s,
                        (None, Some(n)) => format!("span {n}"),
                        (None, None) => continue,
                    };
                    out.push((name.into(), v));
                }
            }
            "width" | "height" if !state => {
                done.push(prop);
                if bound(vals).is_some() {
                    out.push((css.into(), value_css(prop, Conv::Len, vals)?));
                    continue;
                }
                let w = vals.first().map(text_of).unwrap_or_default();
                match w.as_str() {
                    "fit" | "grow" => {
                        out.push((
                            css.into(),
                            if w == "fit" { "fit-content" } else { "stretch" }.into(),
                        ));
                        if let Some(min) = vals.get(1)
                            && text_of(min) != "0"
                        {
                            out.push((format!("min-{css}"), px(min)?));
                        }
                        if let Some(max) = vals.get(2) {
                            out.push((format!("max-{css}"), px(max)?));
                        }
                    }
                    _ => out.push((css.into(), value_css(prop, Conv::Size, vals)?)),
                }
            }
            _ => {
                done.push(prop);
                let c = if prop == "ink" {
                    Conv::Paint
                } else {
                    conv(prop)
                };
                out.push((css.into(), value_css(prop, c, vals)?));
            }
        }
    }
    Ok(out
        .into_iter()
        .map(|(n, v)| format!("{n}: {v}"))
        .collect::<Vec<_>>()
        .join("; "))
}

fn conv_joint(k: &str) -> Conv {
    match k {
        "x" | "y" => Conv::Len,
        _ => Conv::Num,
    }
}
