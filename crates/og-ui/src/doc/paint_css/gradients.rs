use super::*;

// ---- parsing

/// A colour token: any CSS colour ([`crate::css_color`]); `var(--role)`
/// stays a role and a colour over roles an expression, both resolved when
/// drawn.
pub(crate) fn color(t: &Tok) -> Result<ColorRef, String> {
    if let Tok::Func(f, a) = t
        && f == "var"
        && let [v] = a.as_slice()
        && let [Tok::Ident(n)] = v.as_slice()
        && let Some(name) = n.strip_prefix("--")
    {
        // `--panel-raised` names the role `panelRaised`, as in a colour
        // expression ([`crate::css_color`]).
        return Ok(ColorRef::Token {
            name: crate::doc::css::camel(name),
            alpha: 1.0,
        });
    }
    let c = crate::css_color::parse(t)?;
    Ok(match c.literal() {
        Some(s) => ColorRef::Literal(s),
        None => ColorRef::Css(std::sync::Arc::new(c)),
    })
}

pub(super) fn is_color(t: &Tok) -> bool {
    color(t).is_ok()
}

pub(super) fn number(t: &Tok) -> Result<(f32, &str), String> {
    match t {
        Tok::Num(n, u) => n
            .parse::<f32>()
            .map(|v| (v, u.as_str()))
            .map_err(|_| format!("'{n}' is not a number")),
        t => Err(format!("'{}' is not a number", describe(t))),
    }
}

pub(super) fn pos(t: &Tok) -> Result<Pos, String> {
    match number(t)? {
        (v, "%") => Ok(Pos::Frac(v / 100.0)),
        (v, "px") => Ok(Pos::Px(v)),
        (0.0, "") => Ok(Pos::Px(0.0)),
        _ => Err(format!("'{}' is not a length: <n>px or <n>%", describe(t))),
    }
}

pub(super) fn angle(t: &Tok) -> Result<f32, String> {
    match number(t)? {
        (v, "deg") => Ok(v.to_radians()),
        (v, "rad") => Ok(v),
        (v, "turn") => Ok(v * 2.0 * PI),
        (v, "grad") => Ok(v * PI / 200.0),
        (0.0, "") => Ok(0.0),
        _ => Err(format!("'{}' is not an angle", describe(t))),
    }
}

/// `at <x> <y>` (keywords, lengths or percentages); centred by default.
pub(super) fn position(t: &[Tok]) -> Result<[Pos; 2], String> {
    let word = |w: &str| match w {
        "left" | "top" => Some(Pos::Frac(0.0)),
        "center" => Some(Pos::Frac(0.5)),
        "right" | "bottom" => Some(Pos::Frac(1.0)),
        _ => None,
    };
    let one = |t: &Tok| match t {
        Tok::Ident(w) => word(w).ok_or_else(|| format!("'{w}' is not a position")),
        t => pos(t),
    };
    match t {
        [] => Ok([Pos::Frac(0.5); 2]),
        [Tok::Ident(w)] if w == "top" || w == "bottom" => Ok([Pos::Frac(0.5), one(&t[0])?]),
        [a] => Ok([one(a)?, Pos::Frac(0.5)]),
        // `top left` names y first.
        [Tok::Ident(a), Tok::Ident(b)] if matches!(a.as_str(), "top" | "bottom") => {
            Ok([one(&Tok::Ident(b.clone()))?, one(&Tok::Ident(a.clone()))?])
        }
        [a, b] => Ok([one(a)?, one(b)?]),
        _ => {
            Err("a position is one or two of: left, center, right, top, bottom, <n>px, <n>%".into())
        }
    }
}

/// A `*-gradient(…)` function, or `None` when `t` is not one.
pub(super) fn gradient(t: &Tok) -> Option<Result<CssGradient, String>> {
    let Tok::Func(f, args) = t else { return None };
    let repeating = f.starts_with("repeating-");
    let kind = f.strip_prefix("repeating-").unwrap_or(f);
    if !matches!(
        kind,
        "linear-gradient" | "radial-gradient" | "conic-gradient"
    ) {
        return None;
    }
    Some(gradient_of(kind, repeating, args).map_err(|e| format!("{f}: {e}")))
}

fn gradient_of(kind: &str, repeating: bool, args: &[Vec<Tok>]) -> Result<CssGradient, String> {
    let first = args.first().ok_or("no colour stops")?;
    let prelude = !first.is_empty() && !is_color(&first[0]);
    let (shape, stops) = match kind {
        "linear-gradient" => {
            let dir = match first.as_slice() {
                _ if !prelude => Dir::Angle(PI),
                [a] => Dir::Angle(angle(a)?),
                [Tok::Ident(to), sides @ ..] if to == "to" => {
                    let (mut x, mut y) = (0.0, 0.0);
                    for s in sides {
                        match s {
                            Tok::Ident(w) if w == "left" => x = -1.0,
                            Tok::Ident(w) if w == "right" => x = 1.0,
                            Tok::Ident(w) if w == "top" => y = -1.0,
                            Tok::Ident(w) if w == "bottom" => y = 1.0,
                            s => return Err(format!("'{}' is not a side", describe(s))),
                        }
                    }
                    match (x, y) {
                        (0.0, 0.0) => return Err("`to` names a side or a corner".into()),
                        (0.0, y) => Dir::Angle(if y < 0.0 { 0.0 } else { PI }),
                        (x, 0.0) => Dir::Angle(if x > 0.0 { PI / 2.0 } else { 1.5 * PI }),
                        (x, y) => Dir::Corner(x, y),
                    }
                }
                _ => return Err("<angle> or to <side> [<side>]".into()),
            };
            (Shape::Linear(dir), &args[prelude as usize..])
        }
        "radial-gradient" => {
            let (mut circle, mut size, mut at) =
                (false, Extent::FarthestCorner, [Pos::Frac(0.5); 2]);
            if prelude {
                let split = first
                    .iter()
                    .position(|t| matches!(t, Tok::Ident(w) if w == "at"))
                    .unwrap_or(first.len());
                if split < first.len() {
                    at = position(&first[split + 1..])?;
                }
                let mut lengths = Vec::new();
                for t in &first[..split] {
                    match t {
                        Tok::Ident(w) if w == "circle" => circle = true,
                        Tok::Ident(w) if w == "ellipse" => circle = false,
                        Tok::Ident(w) => {
                            size = match w.as_str() {
                                "closest-side" => Extent::ClosestSide,
                                "farthest-side" => Extent::FarthestSide,
                                "closest-corner" => Extent::ClosestCorner,
                                "farthest-corner" => Extent::FarthestCorner,
                                _ => return Err(format!("'{w}' is not a radial shape or size")),
                            }
                        }
                        t => lengths.push(pos(t)?),
                    }
                }
                match lengths.as_slice() {
                    [] => {}
                    [r] => {
                        circle = true;
                        size = Extent::Given(*r, *r);
                    }
                    [x, y] => size = Extent::Given(*x, *y),
                    _ => return Err("a radial size is one or two lengths".into()),
                }
            }
            (
                Shape::Radial { circle, size, at },
                &args[prelude as usize..],
            )
        }
        _ => {
            let (mut from, mut at) = (0.0, [Pos::Frac(0.5); 2]);
            if prelude {
                let mut i = 0;
                while i < first.len() {
                    match &first[i] {
                        Tok::Ident(w) if w == "from" => {
                            from = angle(first.get(i + 1).ok_or("from <angle>")?)?;
                            i += 2;
                        }
                        Tok::Ident(w) if w == "at" => {
                            at = position(&first[i + 1..])?;
                            i = first.len();
                        }
                        t => {
                            return Err(format!(
                                "'{}' is not part of a conic prelude",
                                describe(t)
                            ));
                        }
                    }
                }
            }
            (Shape::Conic { from, at }, &args[prelude as usize..])
        }
    };
    let conic = matches!(shape, Shape::Conic { .. });
    let mut out = Vec::new();
    for s in stops {
        let (c, rest) = s.split_first().ok_or("an empty colour stop")?;
        let c = color(c)?;
        let at = |t: &Tok| -> Result<crate::Length, String> {
            if conic {
                Ok(crate::Length {
                    pct: match number(t)? {
                        (v, "%") => v / 100.0,
                        _ => angle(t)? / (2.0 * PI),
                    },
                    px: 0.0,
                })
            } else {
                let [pct, px, em] = super::super::css::want_lin(t)?;
                if em != 0.0 {
                    return Err("a gradient stop needs resolved px and % lengths".into());
                }
                Ok(crate::Length {
                    pct: pct as f32,
                    px: px as f32,
                })
            }
        };
        match rest {
            [] => out.push((c, None)),
            [a] => out.push((c, Some(at(a)?))),
            // `red 10% 30%`: two stops of one colour.
            [a, b] => {
                out.push((c.clone(), Some(at(a)?)));
                out.push((c, Some(at(b)?)));
            }
            _ => return Err("a stop is <colour> [<position> [<position>]]".into()),
        }
    }
    if out.len() < 2 {
        return Err("a gradient takes at least two colour stops".into());
    }
    if out.len() > og_render::UI_GRADIENT_STOPS {
        return Err(format!(
            "a gradient takes at most {} stops",
            og_render::UI_GRADIENT_STOPS
        ));
    }
    Ok(CssGradient {
        shape,
        repeating,
        stops: out,
    })
}
