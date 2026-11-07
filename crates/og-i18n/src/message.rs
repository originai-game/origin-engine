//! A subset of ICU MessageFormat: literal text, `{name}`, `{n, number}`
//! (styles `compact` and `currency/EUR`), `{d, date[, full|long|medium|
//! short]}` and `{t, time[, medium|short]}` (seconds since 1970-01-01 UTC),
//! `{n, plural, [offset:k] =N {..} zero|one|two|few|many|other {..}}` (with
//! `#`, the number less the offset, inside), `{key, select, word {..}
//! other {..}}`, and apostrophe quoting (`''` is a quote; `'{` starts a
//! quoted run up to the next lone apostrophe). Parsed once, formatted
//! against named arguments.

use crate::format::{
    DateStyle, TimeStyle, format_compact, format_currency, format_date, format_time,
};
use crate::locale::language;
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

/// How a `{n, number, style}` writes its number.
#[derive(Clone, Debug, PartialEq)]
enum NumberStyle {
    Plain,
    Compact,
    /// An ISO 4217 code.
    Currency(String),
}

#[derive(Clone, Debug, PartialEq)]
enum Part {
    Text(String),
    Arg(String),
    Number(String, NumberStyle),
    Date(String, DateStyle),
    Time(String, TimeStyle),
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
                        let mut style = NumberStyle::Plain;
                        if self.peek() == Some(',') {
                            // compact, currency/CODE; integer and percent
                            // are accepted and written plain.
                            self.at += 1;
                            self.skip_ws();
                            let word = self.word();
                            if word == "compact" {
                                style = NumberStyle::Compact;
                            } else if word == "currency" {
                                if self.peek() != Some('/') {
                                    return self.err(format!(
                                        "'{{{name}, number, currency' needs its code: currency/EUR"
                                    ));
                                }
                                self.at += 1;
                                let code = self.word();
                                if code.len() != 3 || !code.chars().all(|c| c.is_ascii_alphabetic())
                                {
                                    return self.err(format!(
                                        "'{code}' is not a currency code (ISO 4217, such as EUR)"
                                    ));
                                }
                                style = NumberStyle::Currency(code.to_ascii_uppercase());
                            }
                            self.skip_ws();
                        }
                        if self.peek() != Some('}') {
                            return self.err(format!("'{{{name}, number' needs a closing '}}'"));
                        }
                        self.at += 1;
                        Ok(Part::Number(name, style))
                    }
                    "date" | "time" => {
                        let mut word = "medium".to_string();
                        if self.peek() == Some(',') {
                            self.at += 1;
                            self.skip_ws();
                            word = self.word();
                            self.skip_ws();
                        }
                        let part = if kind == "date" {
                            DateStyle::named(&word).map(|s| Part::Date(name.clone(), s))
                        } else {
                            TimeStyle::named(&word).map(|s| Part::Time(name.clone(), s))
                        };
                        let Some(part) = part else {
                            let styles = if kind == "date" {
                                "full, long, medium or short"
                            } else {
                                "medium or short"
                            };
                            return self.err(format!("'{word}' is not a {kind} style: {styles}"));
                        };
                        if self.peek() != Some('}') {
                            return self.err(format!("'{{{name}, {kind}' needs a closing '}}'"));
                        }
                        self.at += 1;
                        Ok(part)
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
                        "'{other}' is not an argument type: number, date, time, plural or select"
                    )),
                }
            }
            _ => self.err(format!(
                "'{{{name}' needs '}}' or ', number|date|time|plural|select'"
            )),
        }
    }
}

/// A number as `language` writes it: grouped thousands, up to three
/// fraction digits.
pub fn format_number(n: f64, language: &str) -> String {
    crate::format::format_decimal(n, None, true, language)
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
                    Part::Arg(n) | Part::Number(n, _) | Part::Date(n, _) | Part::Time(n, _) => {
                        out.push(n.clone())
                    }
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

    /// The text for `args` in `locale` (a BCP 47 tag or a bare language: its
    /// plural rules, numbers, currencies and dates). A missing argument shows
    /// as `{name}`.
    pub fn format(&self, locale: &str, args: &BTreeMap<String, Arg>) -> String {
        let mut out = String::new();
        let lang = language(locale);
        self.format_into(&self.parts, locale, &lang, args, None, &mut out);
        out
    }

    fn format_into(
        &self,
        parts: &[Part],
        locale: &str,
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
                    Some(Arg::Num(v)) => out.push_str(&format_number(*v, locale)),
                    None => {
                        out.push('{');
                        out.push_str(n);
                        out.push('}');
                    }
                },
                Part::Number(n, style) => match args.get(n).and_then(Arg::as_num) {
                    Some(v) => out.push_str(&match style {
                        NumberStyle::Plain => format_number(v, locale),
                        NumberStyle::Compact => format_compact(v, locale),
                        NumberStyle::Currency(code) => format_currency(v, code, locale),
                    }),
                    None => {
                        out.push('{');
                        out.push_str(n);
                        out.push('}');
                    }
                },
                Part::Date(n, _) | Part::Time(n, _) => match args.get(n).and_then(Arg::as_num) {
                    Some(v) => out.push_str(&match p {
                        Part::Date(_, s) => format_date(v, 0, *s, locale),
                        Part::Time(_, s) => format_time(v, 0, *s, locale),
                        _ => unreachable!(),
                    }),
                    None => {
                        out.push('{');
                        out.push_str(n);
                        out.push('}');
                    }
                },
                Part::Hash => {
                    if let Some(v) = hash {
                        out.push_str(&format_number(v, locale));
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
                        self.format_into(body, locale, lang, args, Some(v - offset), out);
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
                        self.format_into(body, locale, lang, args, hash, out);
                    }
                }
            }
        }
    }
}
