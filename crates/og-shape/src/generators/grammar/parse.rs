//! A grammar's text: `type grammar`, its params, its start rule and its
//! rules, one operation a line.
//!
//! ```text
//! type grammar
//! version 1
//! ---
//! param floor 3.2
//! param height 12
//! start Lot
//! rule Lot {
//!   extrude height {
//!     sides Facade
//!     top Roof
//!   }
//! }
//! rule Facade {
//!   split y {
//!     part 4.2 Ground
//!     repeat floor Floor
//!     part 0.4 Cornice
//!   }
//! }
//! rule Glass {
//!   recess 0.05
//!   label glass
//! }
//! ```
//!
//! A number is written or names a param (a bare word); a rule or a label
//! is a word or a string.

use std::collections::BTreeSet;

use og_core::ogt::{Block, Document, Entry, Value};

use super::{Axis, Cond, Faces, GRAMMAR_TYPE, Grammar, Num, Op, RoofKind, Rule, Size, SplitPart};

fn at(e: &Entry, msg: impl std::fmt::Display) -> String {
    format!("line {}: `{}`: {msg}", e.line, e.key)
}

/// Exactly `min` to `max` values.
fn count(e: &Entry, min: usize, max: usize) -> Result<(), String> {
    let n = e.values.len();
    if n < min || n > max {
        let want = if min == max {
            format!("{min}")
        } else {
            format!("{min} to {max}")
        };
        return Err(at(e, format!("takes {want} values, not {n}")));
    }
    Ok(())
}

fn no_block(e: &Entry) -> Result<(), String> {
    match e.block {
        Some(_) => Err(at(e, "takes no block")),
        None => Ok(()),
    }
}

fn block(e: &Entry) -> Result<&Block, String> {
    e.block.as_ref().ok_or_else(|| at(e, "takes a block"))
}

fn name(e: &Entry, i: usize) -> Result<String, String> {
    match e.values.get(i) {
        Some(Value::Word(w)) => Ok(w.clone()),
        Some(Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(at(
            e,
            format!("value {} is a name (a word or a string)", i + 1),
        )),
    }
}

fn number(e: &Entry, i: usize) -> Result<f64, String> {
    match e.values.get(i) {
        Some(Value::Number(n)) if n.as_f64().is_finite() => Ok(n.as_f64()),
        _ => Err(at(e, format!("value {} is a number", i + 1))),
    }
}

fn num(e: &Entry, i: usize) -> Result<Num, String> {
    match e.values.get(i) {
        Some(Value::Number(n)) if n.as_f64().is_finite() => Ok(Num::Lit(n.as_f64())),
        Some(Value::Word(w)) => Ok(Num::Param(w.clone())),
        _ => Err(at(
            e,
            format!("value {} is a number or a param's name", i + 1),
        )),
    }
}

/// Every entry of `b` is one of `keys`.
fn known(b: &Block, keys: &[&str], what: &str) -> Result<(), String> {
    for e in &b.entries {
        if !keys.contains(&e.key.as_str()) {
            return Err(at(e, format!("{what} takes {}", keys.join(", "))));
        }
    }
    Ok(())
}

/// Reads a grammar's text.
pub fn parse(text: &str) -> Result<Grammar, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != GRAMMAR_TYPE {
        return Err(format!(
            "a grammar is `type {GRAMMAR_TYPE}`, not `type {}`",
            doc.ty
        ));
    }
    if doc.version != 1 {
        return Err(format!("a grammar is version 1, not {}", doc.version));
    }
    let mut g = Grammar::default();
    let mut start = None;
    for e in &doc.body.entries {
        match e.key.as_str() {
            "param" => {
                count(e, 2, 2)?;
                no_block(e)?;
                let k = name(e, 0)?;
                if g.params.insert(k.clone(), number(e, 1)?).is_some() {
                    return Err(at(e, format!("param `{k}` is declared twice")));
                }
            }
            "start" => {
                count(e, 1, 1)?;
                no_block(e)?;
                start = Some(name(e, 0)?);
            }
            "rule" => {
                count(e, 1, 1)?;
                let k = name(e, 0)?;
                let ops = block(e)?
                    .entries
                    .iter()
                    .map(op)
                    .collect::<Result<Vec<_>, _>>()?;
                let rule = Rule {
                    name: k.clone(),
                    line: e.line,
                    ops,
                };
                ends(&rule)?;
                if g.rules.insert(k.clone(), rule).is_some() {
                    return Err(at(e, format!("rule `{k}` is written twice")));
                }
            }
            other => {
                return Err(at(
                    e,
                    format!("a grammar takes param, start and rule, not `{other}`"),
                ));
            }
        }
    }
    g.start = start.ok_or("a grammar names the rule a lot starts at: `start <rule>`")?;
    check(&g)?;
    Ok(g)
}

/// A rule ends in exactly one operation that ends it, its last.
fn ends(rule: &Rule) -> Result<(), String> {
    let n = rule.ops.iter().filter(|o| o.ends()).count();
    if n != 1 || !rule.ops.last().is_some_and(Op::ends) {
        return Err(format!(
            "rule `{}` (line {}) ends in one of label, extrude, split, inset, roof, insert, pick, case or drop, its last line; recess and detail come before it",
            rule.name, rule.line
        ));
    }
    Ok(())
}

fn op(e: &Entry) -> Result<Op, String> {
    let key = e.key.as_str();
    if !matches!(key, "extrude" | "split" | "roof" | "pick" | "case") {
        no_block(e)?;
    }
    Ok(match key {
        "recess" => {
            count(e, 1, 2)?;
            Op::Recess {
                depth: num(e, 0)?,
                jamb: if e.values.len() > 1 {
                    Some(name(e, 1)?)
                } else {
                    None
                },
            }
        }
        "detail" => {
            count(e, 1, 1)?;
            let d = number(e, 0)?;
            if d < 0.0 || d.fract() != 0.0 || d > 16.0 {
                return Err(at(e, "a detail level is a whole number, 0 to 16"));
            }
            Op::Detail(d as u32)
        }
        "label" => {
            count(e, 1, 1)?;
            Op::Label(name(e, 0)?)
        }
        "insert" => {
            count(e, 1, 1)?;
            match &e.values[0] {
                Value::String(s) if !s.is_empty() => Op::Insert(s.clone()),
                _ => return Err(at(e, "names a model by its asset id, in quotes")),
            }
        }
        "drop" => {
            count(e, 0, 0)?;
            Op::Drop
        }
        "extrude" => {
            count(e, 1, 1)?;
            let b = block(e)?;
            known(
                b,
                &["top", "bottom", "front", "back", "side", "sides"],
                "extrude",
            )?;
            let mut faces = Faces::default();
            for f in &b.entries {
                count(f, 1, 1)?;
                no_block(f)?;
                let slot = match f.key.as_str() {
                    "top" => &mut faces.top,
                    "bottom" => &mut faces.bottom,
                    "front" => &mut faces.front,
                    "back" => &mut faces.back,
                    "side" => &mut faces.side,
                    _ => &mut faces.sides,
                };
                if slot.replace(name(f, 0)?).is_some() {
                    return Err(at(f, "is named twice"));
                }
            }
            Op::Extrude {
                depth: num(e, 0)?,
                faces,
            }
        }
        "split" => {
            count(e, 1, 1)?;
            let axis = match e.values[0].as_word() {
                Some("x") => Axis::X,
                Some("y") => Axis::Y,
                _ => return Err(at(e, "splits along x (across the face) or y (up it)")),
            };
            let b = block(e)?;
            known(b, &["part", "fill", "repeat"], "a split")?;
            let parts = b
                .entries
                .iter()
                .map(|p| {
                    count(p, 2, 2)?;
                    no_block(p)?;
                    let size = match p.key.as_str() {
                        "part" => Size::Abs,
                        "fill" => Size::Fill,
                        _ => Size::Repeat,
                    };
                    Ok(SplitPart {
                        size,
                        length: num(p, 0)?,
                        rule: name(p, 1)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            if parts.is_empty() {
                return Err(at(e, "has at least one part"));
            }
            Op::Split { axis, parts }
        }
        "inset" => {
            count(e, 3, 3)?;
            Op::Inset {
                width: num(e, 0)?,
                border: name(e, 1)?,
                inner: name(e, 2)?,
            }
        }
        "roof" => {
            count(e, 1, 2)?;
            let kind = match e.values[0].as_word() {
                Some("flat") => RoofKind::Flat,
                Some("gable") => RoofKind::Gable,
                Some("hip") => RoofKind::Hip,
                _ => return Err(at(e, "is flat, gable or hip")),
            };
            let angle = match (kind, e.values.len()) {
                (RoofKind::Flat, 1) => Num::Lit(0.0),
                (RoofKind::Flat, _) => return Err(at(e, "a flat roof takes no angle")),
                (_, 2) => num(e, 1)?,
                _ => return Err(at(e, "a gable or hip roof takes its angle in degrees")),
            };
            let b = block(e)?;
            known(b, &["overhang", "slope", "gable", "eave"], "a roof")?;
            let (mut overhang, mut slope, mut gable, mut eave) = (Num::Lit(0.0), None, None, None);
            for f in &b.entries {
                count(f, 1, 1)?;
                no_block(f)?;
                match f.key.as_str() {
                    "overhang" => overhang = num(f, 0)?,
                    "slope" => slope = Some(name(f, 0)?),
                    "gable" => gable = Some(name(f, 0)?),
                    _ => eave = Some(name(f, 0)?),
                }
            }
            let slope = slope.ok_or_else(|| at(e, "names its slopes' rule: `slope <rule>`"))?;
            Op::Roof {
                kind,
                angle,
                overhang,
                slope,
                gable,
                eave,
            }
        }
        "pick" => {
            count(e, 0, 0)?;
            let b = block(e)?;
            known(b, &["option"], "pick")?;
            let options = b
                .entries
                .iter()
                .map(|o| {
                    count(o, 2, 2)?;
                    no_block(o)?;
                    let w = number(o, 0)?;
                    if w <= 0.0 {
                        return Err(at(o, "an option's weight is above 0"));
                    }
                    Ok((w, name(o, 1)?))
                })
                .collect::<Result<Vec<_>, String>>()?;
            if options.is_empty() {
                return Err(at(e, "has at least one option"));
            }
            Op::Pick(options)
        }
        "case" => {
            count(e, 0, 0)?;
            let b = block(e)?;
            known(b, &["wider", "taller", "param", "otherwise"], "case")?;
            let (mut arms, mut otherwise) = (Vec::new(), None);
            for c in &b.entries {
                no_block(c)?;
                if otherwise.is_some() {
                    return Err(at(c, "comes after `otherwise`, which is last"));
                }
                match c.key.as_str() {
                    "wider" | "taller" => {
                        count(c, 2, 2)?;
                        let n = num(c, 0)?;
                        let cond = if c.key == "wider" {
                            Cond::Wider(n)
                        } else {
                            Cond::Taller(n)
                        };
                        arms.push((cond, name(c, 1)?));
                    }
                    "param" => {
                        count(c, 3, 3)?;
                        arms.push((Cond::Param(name(c, 0)?, num(c, 1)?), name(c, 2)?));
                    }
                    _ => {
                        count(c, 1, 1)?;
                        otherwise = Some(name(c, 0)?);
                    }
                }
            }
            Op::Case { arms, otherwise }
        }
        other => {
            return Err(at(
                e,
                format!(
                    "`{other}` is not an operation: recess, detail, label, extrude, split, inset, roof, insert, pick, case or drop"
                ),
            ));
        }
    })
}

/// The rules and params an operation names.
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

/// Every rule and param named is there.
fn check(g: &Grammar) -> Result<(), String> {
    let all = |set: &BTreeSet<&str>| set.iter().copied().collect::<Vec<_>>().join(", ");
    let rules: BTreeSet<&str> = g.rules.keys().map(String::as_str).collect();
    let params: BTreeSet<&str> = g.params.keys().map(String::as_str).collect();
    if !rules.contains(g.start.as_str()) {
        return Err(format!(
            "the start rule `{}` is not written; the rules: {}",
            g.start,
            all(&rules)
        ));
    }
    for rule in g.rules.values() {
        for op in &rule.ops {
            let (r, n, p) = names(op);
            for name in r {
                if !rules.contains(name) {
                    return Err(format!(
                        "rule `{}` (line {}) names rule `{name}`, which is not written; the rules: {}",
                        rule.name,
                        rule.line,
                        all(&rules)
                    ));
                }
            }
            let named = n.into_iter().filter_map(|n| match n {
                Num::Param(p) => Some(p.as_str()),
                Num::Lit(_) => None,
            });
            for name in named.chain(p) {
                if !params.contains(name) {
                    return Err(format!(
                        "rule `{}` (line {}) names param `{name}`, which is not declared; the params: {}",
                        rule.name,
                        rule.line,
                        if params.is_empty() {
                            "none".into()
                        } else {
                            all(&params)
                        }
                    ));
                }
            }
        }
    }
    Ok(())
}
