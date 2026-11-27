//! Checks on native rules, independent of the author representation.
use super::{Cond, Grammar, Num, Op};

/// Validate terminal ordering and every named rule and parameter, including
/// unreachable rules. Shape-dependent bounds and shrinking recursion are
/// checked while building a lot.
pub fn validate(g: &Grammar) -> Result<(), String> {
    if !g.rules.contains_key(&g.start) {
        return Err(format!("start rule `{}` is not declared", g.start));
    }
    for (name, value) in &g.params {
        if name.is_empty() || !value.is_finite() {
            return Err(format!("param `{name}` needs a name and a finite default"));
        }
    }
    for (name, rule) in &g.rules {
        let at = |reason: String| format!("rule `{name}` (line {}): {reason}", rule.line);
        if name.is_empty() || name != &rule.name {
            return Err(at("rule key must equal its nonempty name".into()));
        }
        if rule.ops.iter().filter(|o| o.ends()).count() != 1
            || !rule.ops.last().is_some_and(Op::ends)
        {
            return Err(at("ends in exactly one of label, extrude, split, inset, roof, insert, pick, case or drop; recess and detail come before it".into()));
        }
        for op in &rule.ops {
            let (rules, nums, params) = names(op);
            for r in rules {
                if !g.rules.contains_key(r) {
                    return Err(at(format!("rule `{r}` is not declared")));
                }
            }
            for n in nums {
                match n {
                    Num::Lit(n) if !n.is_finite() => {
                        return Err(at("numbers must be finite".into()));
                    }
                    Num::Param(p) if !g.params.contains_key(p) => {
                        return Err(at(format!("param `{p}` is not declared")));
                    }
                    _ => {}
                }
            }
            for p in params {
                if !g.params.contains_key(p) {
                    return Err(at(format!("param `{p}` is not declared")));
                }
            }
            match op {
                Op::Detail(n) if *n > 16 => return Err(at("detail is 0..16".into())),
                Op::Split { parts, .. } if parts.is_empty() => {
                    return Err(at("split needs at least one part".into()));
                }
                Op::Pick(options)
                    if options.is_empty()
                        || options.iter().any(|(w, _)| !w.is_finite() || *w <= 0.0) =>
                {
                    return Err(at("pick needs options with finite positive weights".into()));
                }
                Op::Label(s) | Op::Insert(s) if s.is_empty() => {
                    return Err(at("label/model ID must not be empty".into()));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// The rules, numbers and parameter predicates an operation names.
fn names(op: &Op) -> (Vec<&str>, Vec<&Num>, Vec<&str>) {
    let (mut rules, mut nums, mut params) = (Vec::new(), Vec::new(), Vec::new());
    match op {
        Op::Recess { depth, jamb } => {
            nums.push(depth);
            rules.extend(jamb.as_deref());
        }
        Op::Extrude { depth, faces } => {
            nums.push(depth);
            for f in [
                &faces.top,
                &faces.bottom,
                &faces.front,
                &faces.back,
                &faces.side,
                &faces.sides,
            ] {
                rules.extend(f.as_deref());
            }
        }
        Op::Split { parts, .. } => {
            for p in parts {
                nums.push(&p.length);
                rules.push(&p.rule);
            }
        }
        Op::Inset {
            width,
            border,
            inner,
        } => {
            nums.push(width);
            rules.extend([border.as_str(), inner.as_str()]);
        }
        Op::Roof {
            angle,
            overhang,
            slope,
            gable,
            eave,
            ..
        } => {
            nums.extend([angle, overhang]);
            rules.push(slope);
            rules.extend(gable.as_deref());
            rules.extend(eave.as_deref());
        }
        Op::Pick(options) => rules.extend(options.iter().map(|o| o.1.as_str())),
        Op::Case { arms, otherwise } => {
            for (c, r) in arms {
                match c {
                    Cond::Wider(n) | Cond::Taller(n) => nums.push(n),
                    Cond::Param(p, n) => {
                        params.push(p.as_str());
                        nums.push(n);
                    }
                }
                rules.push(r);
            }
            rules.extend(otherwise.as_deref());
        }
        Op::Detail(_) | Op::Label(_) | Op::Insert(_) | Op::Drop => {}
    }
    (rules, nums, params)
}
