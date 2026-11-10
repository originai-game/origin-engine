//! The CSS value lexer: one declaration's value as tokens.

use og_core::ogt::{self, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    /// A number with its unit (`px`, `%`, `ms`, empty).
    Num(String, String),
    Str(String),
    Hash(String),
    Func(String, Vec<Vec<Tok>>),
    Slash,
}

pub fn describe(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => quoted(s),
        Tok::Func(f, _) => format!("{f}(…)"),
        Tok::Slash => "/".into(),
    }
}

/// The value of one declaration as tokens.
pub fn tokens(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let (toks, end) = tokens_in(&c, &mut i, false)?;
    if end {
        return Err("a ')' closes nothing".into());
    }
    Ok(toks.into_iter().flatten().collect())
}

/// Tokens up to the end (or, in a function, its `)`), split at commas.
fn tokens_in(c: &[char], i: &mut usize, func: bool) -> Result<(Vec<Vec<Tok>>, bool), String> {
    let mut args: Vec<Vec<Tok>> = vec![Vec::new()];
    while *i < c.len() {
        let ch = c[*i];
        if ch.is_whitespace() {
            *i += 1;
        } else if ch == ')' {
            *i += 1;
            let _ = func;
            return Ok((args, true));
        } else if ch == ',' {
            *i += 1;
            args.push(Vec::new());
        } else if ch == '/' {
            *i += 1;
            args.last_mut().unwrap().push(Tok::Slash);
        } else if ch == '*' {
            // A product in a relative colour's channel (`calc(l * 0.8)`).
            *i += 1;
            args.last_mut().unwrap().push(Tok::Ident("*".into()));
        } else if matches!(ch, '+' | '-') && c.get(*i + 1).is_none_or(|d| d.is_whitespace()) {
            // A math operator (CSS needs spaces around + and -).
            *i += 1;
            args.last_mut().unwrap().push(Tok::Ident(ch.to_string()));
        } else if matches!(ch, '"' | '\'') {
            let quote = ch;
            let mut value = String::new();
            *i += 1;
            while *i < c.len() && c[*i] != quote {
                if c[*i] == '\\' {
                    *i += 1;
                    let start = *i;
                    while *i < c.len() && *i - start < 6 && c[*i].is_ascii_hexdigit() {
                        *i += 1;
                    }
                    if *i > start {
                        let hex: String = c[start..*i].iter().collect();
                        let scalar = u32::from_str_radix(&hex, 16).unwrap();
                        value.push(
                            char::from_u32(scalar)
                                .filter(|c| *c != '\0')
                                .unwrap_or('\u{fffd}'),
                        );
                        if c.get(*i).is_some_and(|c| c.is_whitespace()) {
                            *i += 1;
                        }
                        continue;
                    }
                    if c.get(*i).is_some_and(|c| matches!(c, '\n' | '\r')) {
                        *i += 1;
                        continue;
                    }
                } else if matches!(c[*i], '\n' | '\r') {
                    return Err("a CSS string cannot contain an unescaped newline".into());
                }
                let Some(ch) = c.get(*i) else { break };
                value.push(*ch);
                *i += 1;
            }
            if *i >= c.len() {
                return Err("a string is not closed".into());
            }
            *i += 1;
            args.last_mut().unwrap().push(Tok::Str(value));
        } else if ch == '#' {
            let start = *i;
            *i += 1;
            while *i < c.len() && c[*i].is_ascii_alphanumeric() {
                *i += 1;
            }
            args.last_mut()
                .unwrap()
                .push(Tok::Hash(c[start..*i].iter().collect()));
        } else if ch.is_ascii_digit()
            || (ch == '-' || ch == '.')
                && c.get(*i + 1)
                    .is_some_and(|d| d.is_ascii_digit() || *d == '.')
        {
            let start = *i;
            *i += 1;
            while *i < c.len()
                && (c[*i].is_ascii_digit()
                    || c[*i] == '.'
                    || (matches!(c[*i], 'e' | 'E')
                        && c.get(*i + 1)
                            .is_some_and(|d| d.is_ascii_digit() || *d == '-')))
            {
                if matches!(c[*i], 'e' | 'E') {
                    *i += 1;
                }
                *i += 1;
            }
            let n: String = c[start..*i].iter().collect();
            let us = *i;
            while *i < c.len() && (c[*i].is_ascii_alphabetic() || c[*i] == '%') {
                *i += 1;
            }
            let unit: String = c[us..*i].iter().collect();
            // A physical length is px at 96px to the inch (CSS Values 4 §6.2).
            let (n, unit) = match super::calc::physical(&unit) {
                Some(k) => (
                    super::calc::number(n.parse::<f64>().unwrap_or(0.0) * k),
                    "px".to_string(),
                ),
                None => (n, unit),
            };
            args.last_mut().unwrap().push(Tok::Num(n, unit));
        } else if ch.is_ascii_alphabetic() || matches!(ch, '-' | '_' | '$') {
            let start = *i;
            while *i < c.len()
                && (c[*i].is_ascii_alphanumeric() || matches!(c[*i], '-' | '_' | '.' | '$'))
            {
                *i += 1;
            }
            let id: String = c[start..*i].iter().collect();
            if c.get(*i) == Some(&'(') {
                *i += 1;
                let (inner, closed) = tokens_in(c, i, true)?;
                if !closed {
                    return Err(format!("{id}( is not closed"));
                }
                args.last_mut().unwrap().push(Tok::Func(id, inner));
            } else {
                args.last_mut().unwrap().push(Tok::Ident(id));
            }
        } else {
            return Err(format!("'{ch}' is not part of a value"));
        }
    }
    Ok((args, false))
}

/// A token written back as CSS.
pub fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Hash(s) => s.clone(),
        Tok::Num(n, u) => format!("{n}{u}"),
        Tok::Str(s) => quoted(s),
        Tok::Slash => "/".into(),
        Tok::Func(f, args) => format!(
            "{f}({})",
            args.iter()
                .map(|a| a.iter().map(tok_text).collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

pub fn quoted(s: &str) -> String {
    ogt::value_text(&Value::String(s.to_string())).unwrap_or_default()
}

pub fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in s.chars() {
        if c == '-' {
            up = true;
        } else if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}
