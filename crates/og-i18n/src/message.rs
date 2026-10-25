//! A subset of ICU MessageFormat: literal text, `{name}`, `{n, number}`,
//! `{n, plural, [offset:k] =N {..} zero|one|two|few|many|other {..}}` (with
//! `#`, the number less the offset, inside), `{key, select, word {..}
//! other {..}}`, and apostrophe quoting (`''` is a quote; `'{` starts a
//! quoted run up to the next lone apostrophe). Parsed once, formatted
//! against named arguments.

use crate::plural::plural_category;
use std::collections::BTreeMap;

/// An argument's value.
#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    Str(String),
    Num(f64),
}

impl Arg {
    fn as_num(&self) -> Option<f64> {
        match self {
            Arg::Num(n) => Some(*n),
            Arg::Str(s) => s.trim().parse().ok(),
        }
    }
}

/// Why a message did not parse, with the character offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageError {
    pub at: usize,
    pub reason: String,
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at character {}: {}", self.at, self.reason)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Part {
    Text(String),
    Arg(String),
    Number(String),
    /// The plural's number, `#`.
    Hash,
    Plural {
        name: String,
        offset: f64,
        cases: Vec<(String, Vec<Part>)>,
    },
    Select {
        name: String,
        cases: Vec<(String, Vec<Part>)>,
    },
}

/// A parsed message.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    parts: Vec<Part>,
}

struct Parser<'a> {
    chars: Vec<char>,
    at: usize,
    _src: &'a str,
}

impl Parser<'_> {
    fn err<T>(&self, reason: impl Into<String>) -> Result<T, MessageError> {
        Err(MessageError {
            at: self.at,
            reason: reason.into(),
        })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.at += 1;
        }
    }

    fn word(&mut self) -> String {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '=' | ':'))
        {
            self.at += 1;
        }
        self.chars[start..self.at].iter().collect()
    }

    /// Parts up to an unmatched `}` (inside a case) or the end.
    fn parts(&mut self, in_plural: bool, nested: bool) -> Result<Vec<Part>, MessageError> {
        let mut out = Vec::new();
        let mut text = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\'' => {
                    self.at += 1;
                    match self.peek() {
                        Some('\'') => {
                            text.push('\'');
                            self.at += 1;
                        }
                        Some('{' | '}' | '#' | '|') => {
                            // A quoted run, to the next lone apostrophe.
                            while let Some(q) = self.peek() {
                                self.at += 1;
                                if q == '\'' {
                                    if self.peek() == Some('\'') {
                                        text.push('\'');
                                        self.at += 1;
                                        continue;
                                    }
                                    break;
                                }
                                text.push(q);
                            }
                        }
                        _ => text.push('\''),
                    }
                }
                '{' => {
                    if !text.is_empty() {
                        out.push(Part::Text(std::mem::take(&mut text)));
                    }
                    self.at += 1;
                    out.push(self.argument()?);
                }
                '}' if nested => break,
                '}' => return self.err("an unmatched '}' (write '}' quoted: '}')"),
                '#' if in_plural => {
                    if !text.is_empty() {
                        out.push(Part::Text(std::mem::take(&mut text)));
                    }
                    self.at += 1;
                    out.push(Part::Hash);
                }
                c => {
                    text.push(c);
                    self.at += 1;
                }
            }
        }
        if !text.is_empty() {
            out.push(Part::Text(text));
        }
        Ok(out)
    }

    fn argument(&mut self) -> Result<Part, MessageError> {
        self.skip_ws();
        let name = self.word();
        if name.is_empty() {
            return self.err("an argument needs a name: {name}");
        }
        self.skip_ws();
        match self.peek() {
            Some('}') => {
                self.at += 1;
                Ok(Part::Arg(name))
            }
            Some(',') => {
                self.at += 1;
                self.skip_ws();
                let kind = self.word();
                self.skip_ws();
                match kind.as_str() {
                    "number" => {
                        if self.peek() == Some(',') {
                            // A style (integer, percent) is accepted and ignored.
                            self.at += 1;
                            self.skip_ws();
                            self.word();
                            self.skip_ws();
                        }
                        if self.peek() != Some('}') {
                            return self.err(format!("'{{{name}, number' needs a closing '}}'"));
                        }
                        self.at += 1;
                        Ok(Part::Number(name))
                    }
                    "plural" | "select" => {
                        if self.peek() != Some(',') {
                            return self
                                .err(format!("'{{{name}, {kind}' needs ', ' then its cases"));
                        }
                        self.at += 1;
                        let plural = kind == "plural";
                        let mut offset = 0.0;
                        let mut cases = Vec::new();
                        loop {
                            self.skip_ws();
                            match self.peek() {
                                Some('}') => {
                                    self.at += 1;
                                    break;
                                }
                                None => {
                                    return self.err(format!("'{{{name}, {kind}' is not closed"));
                                }
                                _ => {}
                            }
                            let sel = self.word();
                            if plural && let Some(o) = sel.strip_prefix("offset:") {
                                offset = o.parse().map_err(|_| MessageError {
                                    at: self.at,
                                    reason: format!("'offset:{o}' is not a number"),
                                })?;
                                continue;
                            }
                            if sel.is_empty() {
                                return self.err(format!(
                                    "a {kind} case needs a selector before its '{{'"
                                ));
                            }
                            if plural
                                && !sel.starts_with('=')
                                && !["zero", "one", "two", "few", "many", "other"]
                                    .contains(&sel.as_str())
                            {
                                return self.err(format!(
                                    "'{sel}' is not a plural case: =N, zero, one, two, few, many or other"
                                ));
                            }
                            self.skip_ws();
                            if self.peek() != Some('{') {
                                return self
                                    .err(format!("the case '{sel}' needs '{{' and its message"));
                            }
                            self.at += 1;
                            let body = self.parts(plural, true)?;
                            if self.peek() != Some('}') {
                                return self.err(format!("the case '{sel}' is not closed"));
                            }
                            self.at += 1;
                            cases.push((sel, body));
                        }
                        if !cases.iter().any(|(s, _)| s == "other") {
                            return self.err(format!("'{{{name}, {kind}' needs an 'other' case"));
                        }
                        Ok(if plural {
                            Part::Plural {
                                name,
                                offset,
                                cases,
                            }
                        } else {
                            Part::Select { name, cases }
                        })
                    }
                    other => self.err(format!(
                        "'{other}' is not an argument type: number, plural or select"
                    )),
                }
            }
            _ => self.err(format!("'{{{name}' needs '}}' or ', number|plural|select'")),
        }
    }
}

fn group(int: &str, sep: char) -> String {
    let mut out = String::new();
    for (k, c) in int.chars().enumerate() {
        if k > 0 && (int.len() - k).is_multiple_of(3) {
            out.push(sep);
        }
        out.push(c);
    }
    out
}

/// A number as `language` writes it: grouped thousands, up to three
/// fraction digits.
pub fn format_number(n: f64, language: &str) -> String {
    let (thousands, decimal) = match language {
        "de" | "es" | "it" | "pt" | "nl" | "id" | "tr" | "da" | "el" => ('.', ','),
        "fr" | "ru" | "pl" | "uk" | "cs" | "sk" | "sv" | "fi" | "nb" | "no" | "bg" => {
            ('\u{202f}', ',')
        }
        _ => (',', '.'),
    };
    if !n.is_finite() {
        return n.to_string();
    }
    let neg = n < 0.0;
    let text = format!("{:.3}", n.abs());
    let (int, frac) = text.split_once('.').unwrap_or((&text, ""));
    let frac = frac.trim_end_matches('0');
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&group(int, thousands));
    if !frac.is_empty() {
        out.push(decimal);
        out.push_str(frac);
    }
    out
}

impl Message {
    pub fn parse(text: &str) -> Result<Message, MessageError> {
        let mut p = Parser {
            chars: text.chars().collect(),
            at: 0,
            _src: text,
        };
        let parts = p.parts(false, false)?;
        Ok(Message { parts })
    }

    /// The argument names it uses, sorted.
    pub fn arguments(&self) -> Vec<String> {
        fn walk(parts: &[Part], out: &mut Vec<String>) {
            for p in parts {
                match p {
                    Part::Arg(n) | Part::Number(n) => out.push(n.clone()),
                    Part::Plural { name, cases, .. } | Part::Select { name, cases } => {
                        out.push(name.clone());
                        for (_, c) in cases {
                            walk(c, out);
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.parts, &mut out);
        og_core::sort::sort(&mut out);
        out.dedup();
        out
    }

    /// The text for `args` in `language` (its plural rules and numbers). A
    /// missing argument shows as `{name}`.
    pub fn format(&self, language: &str, args: &BTreeMap<String, Arg>) -> String {
        let mut out = String::new();
        self.format_into(&self.parts, language, args, None, &mut out);
        out
    }

    fn format_into(
        &self,
        parts: &[Part],
        lang: &str,
        args: &BTreeMap<String, Arg>,
        hash: Option<f64>,
        out: &mut String,
    ) {
        for p in parts {
            match p {
                Part::Text(t) => out.push_str(t),
                Part::Arg(n) => match args.get(n) {
                    Some(Arg::Str(s)) => out.push_str(s),
                    Some(Arg::Num(v)) => out.push_str(&format_number(*v, lang)),
                    None => {
                        out.push('{');
                        out.push_str(n);
                        out.push('}');
                    }
                },
                Part::Number(n) => match args.get(n).and_then(Arg::as_num) {
                    Some(v) => out.push_str(&format_number(v, lang)),
                    None => {
                        out.push('{');
                        out.push_str(n);
                        out.push('}');
                    }
                },
                Part::Hash => {
                    if let Some(v) = hash {
                        out.push_str(&format_number(v, lang));
                    } else {
                        out.push('#');
                    }
                }
                Part::Plural {
                    name,
                    offset,
                    cases,
                } => {
                    let Some(v) = args.get(name).and_then(Arg::as_num) else {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                        continue;
                    };
                    let exact = format!("={}", format_number(v, "en").replace(',', ""));
                    let case = cases
                        .iter()
                        .find(|(s, _)| *s == exact)
                        .or_else(|| {
                            let k = plural_category(lang, v - offset).keyword();
                            cases.iter().find(|(s, _)| s == k)
                        })
                        .or_else(|| cases.iter().find(|(s, _)| s == "other"));
                    if let Some((_, body)) = case {
                        self.format_into(body, lang, args, Some(v - offset), out);
                    }
                }
                Part::Select { name, cases } => {
                    let key = match args.get(name) {
                        Some(Arg::Str(s)) => s.clone(),
                        Some(Arg::Num(v)) => format_number(*v, "en"),
                        None => String::new(),
                    };
                    let case = cases
                        .iter()
                        .find(|(s, _)| *s == key)
                        .or_else(|| cases.iter().find(|(s, _)| s == "other"));
                    if let Some((_, body)) = case {
                        self.format_into(body, lang, args, hash, out);
                    }
                }
            }
        }
    }
}
