//! `.ogt`, the engine's one text format (scenes, materials, tuning, import
//! settings, `project.ogt`, the API IDL).
//!
//! ```text
//! type scene
//! version 1
//! ---
//! name "Main"
//! gravity 0 -9.81 0
//! entity "player" {
//!   Transform {
//!     position 0 1 0
//!   }
//!   tags hero controllable
//! }
//! ```
//!
//! Grammar, one construct per line:
//!
//! - a header: `type <word>`, then `version <integer>`, then `---`;
//! - `key value*` — an entry; `key value* {` opens a nested block that a line
//!   holding only `}` closes;
//! - values are numbers (bare: `-1`, `0.25`, `6.02e23`), strings (quoted, with
//!   the escapes `\\ \" \n \r \t \u{hex}`) and words (bare identifiers such as
//!   `true`, `linear`, `og.Transform`);
//! - keys and words are `[A-Za-z_][A-Za-z0-9_.-]*` (a key is never `true` or
//!   `false`); tokens are separated by
//!   spaces or tabs; indentation is not significant;
//! - blank lines and `#` comments (from a `#` that starts a token to the end
//!   of the line) are ignored.
//!
//! Round-trip invariants: [`Document::to_text`] writes the canonical form (two
//! spaces per level, single spaces between tokens, `\n` line ends, a final
//! newline); `parse(to_text(d)) == d` for every document, and
//! `to_text(parse(t)) == t` for canonical text. Numbers keep their exact
//! source text, so nothing is lost to float formatting. Comments and blank
//! lines are not part of the data and are not written back.

use crate::error::{Error, ErrorKind, Result};
use std::fmt::Write as _;

/// A number exactly as written (validated). Converting to `f32`/`f64` parses
/// the text directly, so `0.1` read as `f32` is the nearest `f32` to 0.1.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Number(String);

fn is_number_text(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    if b.first() == Some(&b'-') {
        i = 1;
    }
    let digits = |i: &mut usize| {
        let start = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        *i > start
    };
    if !digits(&mut i) {
        return false;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        if !digits(&mut i) {
            return false;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        if !digits(&mut i) {
            return false;
        }
    }
    i == b.len()
}

impl Number {
    /// A number from its text (`-1`, `0.25`, `1e-7`). The text must be finite
    /// as an `f64`.
    pub fn parse(text: &str) -> Result<Number> {
        if !is_number_text(text) {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("'{text}' is not a number"),
            ));
        }
        if !text.parse::<f64>().is_ok_and(f64::is_finite) {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("number {text} is out of range"),
            ));
        }
        Ok(Number(text.to_string()))
    }

    /// The shortest text that reads back as exactly `value`.
    pub fn from_f64(value: f64) -> Result<Number> {
        if !value.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("{value} has no .ogt text"),
            ));
        }
        let a = value.abs();
        let text = if a != 0.0 && !(1e-5..1e16).contains(&a) {
            format!("{value:e}")
        } else {
            format!("{value}")
        };
        Ok(Number(text))
    }

    /// The shortest text that reads back as exactly `value` when read as `f32`.
    pub fn from_f32(value: f32) -> Result<Number> {
        if !value.is_finite() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("{value} has no .ogt text"),
            ));
        }
        let a = value.abs();
        let text = if a != 0.0 && !(1e-5..1e16).contains(&a) {
            format!("{value:e}")
        } else {
            format!("{value}")
        };
        Ok(Number(text))
    }

    pub fn text(&self) -> &str {
        &self.0
    }

    pub fn as_f64(&self) -> f64 {
        self.0.parse().expect("validated number text")
    }

    pub fn as_f32(&self) -> f32 {
        self.0.parse().expect("validated number text")
    }

    /// The value as an integer; refused for text with a fraction or exponent.
    pub fn as_i64(&self) -> Result<i64> {
        self.0
            .parse()
            .map_err(|_| Error::new(ErrorKind::Invalid, format!("{} is not an integer", self.0)))
    }

    pub fn as_u64(&self) -> Result<u64> {
        self.0.parse().map_err(|_| {
            Error::new(
                ErrorKind::Invalid,
                format!("{} is not an unsigned integer", self.0),
            )
        })
    }
}

macro_rules! number_from_int {
    ($($t:ty),*) => {$(
        impl From<$t> for Number {
            fn from(value: $t) -> Number {
                Number(value.to_string())
            }
        }
    )*};
}
number_from_int!(i32, u32, i64, u64);

/// One value on an entry line.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Number(Number),
    String(String),
    /// A bare identifier: `true`, `false`, an enum case, a type name.
    Word(String),
}

impl Value {
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_word(&self) -> Option<&str> {
        match self {
            Value::Word(w) => Some(w),
            _ => None,
        }
    }

    /// The words `true` and `false`.
    pub fn as_bool(&self) -> Option<bool> {
        match self.as_word()? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
}

impl From<Number> for Value {
    fn from(n: Number) -> Value {
        Value::Number(n)
    }
}

/// `key value*`, optionally with a nested block.
///
/// `line` is the 1-based source line the entry was read from (0 for entries
/// built in memory); it is for error messages and is ignored by `==`.
#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub key: String,
    pub values: Vec<Value>,
    pub block: Option<Block>,
    pub line: usize,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.key == other.key && self.values == other.values && self.block == other.block
    }
}

impl Eq for Entry {}

impl std::hash::Hash for Entry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.values.hash(state);
        self.block.hash(state);
    }
}

impl Entry {
    pub fn new(key: impl Into<String>) -> Entry {
        Entry {
            key: key.into(),
            values: Vec::new(),
            block: None,
            line: 0,
        }
    }

    /// Appends a value (builder style).
    pub fn with(mut self, value: impl Into<Value>) -> Entry {
        self.values.push(value.into());
        self
    }

    /// Attaches a nested block (builder style).
    pub fn with_block(mut self, block: Block) -> Entry {
        self.block = Some(block);
        self
    }

    fn missing(&self, i: usize, what: &str) -> Error {
        Error::new(
            ErrorKind::NotFound,
            format!("'{}' has no {what} at position {i}", self.key),
        )
    }

    pub fn number(&self, i: usize) -> Result<&Number> {
        self.values
            .get(i)
            .and_then(Value::as_number)
            .ok_or_else(|| self.missing(i, "number"))
    }

    pub fn f32(&self, i: usize) -> Result<f32> {
        Ok(self.number(i)?.as_f32())
    }

    pub fn string(&self, i: usize) -> Result<&str> {
        self.values
            .get(i)
            .and_then(Value::as_str)
            .ok_or_else(|| self.missing(i, "string"))
    }

    pub fn word(&self, i: usize) -> Result<&str> {
        self.values
            .get(i)
            .and_then(Value::as_word)
            .ok_or_else(|| self.missing(i, "word"))
    }
}

/// An ordered list of entries; keys may repeat (a scene lists many `entity`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Block {
    pub entries: Vec<Entry>,
}

impl Block {
    pub fn new() -> Block {
        Block::default()
    }

    /// Appends an entry (builder style).
    pub fn with(mut self, entry: Entry) -> Block {
        self.entries.push(entry);
        self
    }

    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// The first entry with `key`.
    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.key == key)
    }

    /// Every entry with `key`, in order.
    pub fn get_all<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a Entry> + 'a {
        self.entries.iter().filter(move |e| e.key == key)
    }
}

/// A whole `.ogt` file: the header and the root block.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Document {
    /// What the file describes: `scene`, `material`, `project`, ...
    pub ty: String,
    pub version: u32,
    pub body: Block,
}

impl Document {
    pub fn new(ty: impl Into<String>, version: u32) -> Document {
        Document {
            ty: ty.into(),
            version,
            body: Block::new(),
        }
    }

    /// Parses `.ogt` text. Errors name the line.
    pub fn parse(text: &str) -> Result<Document> {
        parse(text)
    }

    /// The canonical text. Refused if a key, word or the type is not a valid
    /// identifier (it would not read back).
    pub fn to_text(&self) -> Result<String> {
        let mut out = String::new();
        if !is_word(&self.ty) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("document type '{}' is not a word", self.ty),
            ));
        }
        let _ = write!(out, "type {}\nversion {}\n---\n", self.ty, self.version);
        write_block(&mut out, &self.body, 0)?;
        Ok(out)
    }
}

fn is_word(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

fn write_block(out: &mut String, block: &Block, depth: usize) -> Result<()> {
    for entry in &block.entries {
        if !is_word(&entry.key) || entry.key == "true" || entry.key == "false" {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("key '{}' is not a word", entry.key),
            ));
        }
        out.extend(std::iter::repeat_n("  ", depth));
        out.push_str(&entry.key);
        for value in &entry.values {
            out.push(' ');
            match value {
                Value::Number(n) => out.push_str(n.text()),
                Value::String(s) => write_string(out, s),
                Value::Word(w) if is_word(w) => out.push_str(w),
                Value::Word(w) => {
                    return Err(Error::new(
                        ErrorKind::Invalid,
                        format!("'{}' value '{w}' is not a word; use a string", entry.key),
                    ));
                }
            }
        }
        match &entry.block {
            Some(inner) => {
                out.push_str(" {\n");
                write_block(out, inner, depth + 1)?;
                out.extend(std::iter::repeat_n("  ", depth));
                out.push_str("}\n");
            }
            None => out.push('\n'),
        }
    }
    Ok(())
}

fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

enum Token {
    Word(String),
    Number(Number),
    Str(String),
    Open,
    Close,
}

fn parse_error(line: usize, reason: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Parse, format!("line {line}: {reason}"))
}

fn tokenize(line: &str, n: usize) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some(&(start, c)) = chars.peek() {
        if c == ' ' || c == '\t' {
            chars.next();
            continue;
        }
        if c == '#' {
            break;
        }
        if c == '"' {
            chars.next();
            let mut s = String::new();
            loop {
                match chars.next() {
                    None => return Err(parse_error(n, "unterminated string")),
                    Some((_, '"')) => break,
                    Some((_, '\\')) => match chars.next() {
                        Some((_, '"')) => s.push('"'),
                        Some((_, '\\')) => s.push('\\'),
                        Some((_, 'n')) => s.push('\n'),
                        Some((_, 'r')) => s.push('\r'),
                        Some((_, 't')) => s.push('\t'),
                        Some((_, 'u')) => {
                            if chars.next().map(|(_, c)| c) != Some('{') {
                                return Err(parse_error(n, "expected '{' after \\u"));
                            }
                            let mut hex = String::new();
                            loop {
                                match chars.next() {
                                    Some((_, '}')) => break,
                                    Some((_, h)) if h.is_ascii_hexdigit() && hex.len() < 6 => {
                                        hex.push(h)
                                    }
                                    _ => return Err(parse_error(n, "malformed \\u{...} escape")),
                                }
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .ok()
                                .and_then(char::from_u32)
                                .ok_or_else(|| {
                                    parse_error(n, format!("\\u{{{hex}}} is not a character"))
                                })?;
                            s.push(code);
                        }
                        Some((_, other)) => {
                            return Err(parse_error(n, format!("unknown escape \\{other}")));
                        }
                        None => return Err(parse_error(n, "unterminated string")),
                    },
                    Some((_, c)) => s.push(c),
                }
            }
            if let Some(&(_, next)) = chars.peek()
                && next != ' '
                && next != '\t'
            {
                return Err(parse_error(n, "a string must be followed by a space"));
            }
            tokens.push(Token::Str(s));
            continue;
        }
        let mut end = line.len();
        while let Some(&(i, c)) = chars.peek() {
            if c == ' ' || c == '\t' {
                end = i;
                break;
            }
            chars.next();
        }
        let text = &line[start..end];
        tokens.push(match text {
            "{" => Token::Open,
            "}" => Token::Close,
            t if t.starts_with(|c: char| c.is_ascii_digit() || c == '-') => {
                Token::Number(Number::parse(t).map_err(|e| parse_error(n, e.reason()))?)
            }
            t if is_word(t) => Token::Word(t.to_string()),
            t => {
                return Err(parse_error(
                    n,
                    format!("'{t}' is not a number, string or word"),
                ));
            }
        });
    }
    Ok(tokens)
}

/// Parses `.ogt` text into a [`Document`].
pub fn parse(text: &str) -> Result<Document> {
    let mut lines = text
        .split('\n')
        .enumerate()
        .map(|(i, l)| (i + 1, l.strip_suffix('\r').unwrap_or(l)));
    let mut header = Vec::new();
    for (n, line) in lines.by_ref() {
        let trimmed = line.trim_matches([' ', '\t']);
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "---" {
            if header.len() != 2 {
                return Err(parse_error(
                    n,
                    "'---' before the 'type' and 'version' header lines",
                ));
            }
            break;
        }
        if header.len() == 2 {
            return Err(parse_error(n, "expected '---' after the header"));
        }
        header.push((n, tokenize(line, n)?));
    }
    let (ty, version) = match header.as_slice() {
        [(n1, t1), (n2, t2)] => {
            let ty = match t1.as_slice() {
                [Token::Word(k), Token::Word(v)] if k == "type" => v.clone(),
                _ => return Err(parse_error(*n1, "expected 'type <word>'")),
            };
            let version = match t2.as_slice() {
                [Token::Word(k), Token::Number(v)] if k == "version" => v
                    .text()
                    .parse::<u32>()
                    .map_err(|_| parse_error(*n2, "version must be a non-negative integer"))?,
                _ => return Err(parse_error(*n2, "expected 'version <integer>'")),
            };
            (ty, version)
        }
        _ => {
            return Err(Error::new(
                ErrorKind::Parse,
                "missing header: 'type <word>', 'version <integer>', '---'",
            ));
        }
    };

    // Stack of open blocks: (entry being built, line it opened on).
    let mut root = Block::new();
    let mut stack: Vec<(Entry, usize)> = Vec::new();
    for (n, line) in lines {
        let tokens = tokenize(line, n)?;
        let mut tokens = tokens.into_iter();
        let key = match tokens.next() {
            None => continue,
            Some(Token::Close) => {
                if tokens.next().is_some() {
                    return Err(parse_error(n, "'}' must be alone on its line"));
                }
                let (mut entry, _) = stack.pop().ok_or_else(|| parse_error(n, "unmatched '}'"))?;
                entry.block.get_or_insert_with(Block::new);
                match stack.last_mut() {
                    Some((parent, _)) => parent.block.get_or_insert_with(Block::new).push(entry),
                    None => root.push(entry),
                }
                continue;
            }
            Some(Token::Word(k)) if k != "true" && k != "false" => k,
            Some(_) => {
                return Err(parse_error(
                    n,
                    "a line starts with a key (a word other than true or false)",
                ));
            }
        };
        let mut entry = Entry::new(key);
        entry.line = n;
        let mut opens = false;
        for token in tokens {
            if opens {
                return Err(parse_error(n, "'{' must end its line"));
            }
            match token {
                Token::Word(w) => entry.values.push(Value::Word(w)),
                Token::Number(v) => entry.values.push(Value::Number(v)),
                Token::Str(s) => entry.values.push(Value::String(s)),
                Token::Open => opens = true,
                Token::Close => return Err(parse_error(n, "'}' must be alone on its line")),
            }
        }
        if opens {
            entry.block = Some(Block::new());
            stack.push((entry, n));
        } else {
            match stack.last_mut() {
                Some((parent, _)) => parent.block.get_or_insert_with(Block::new).push(entry),
                None => root.push(entry),
            }
        }
    }
    if let Some((entry, n)) = stack.last() {
        return Err(parse_error(
            *n,
            format!("block '{}' opened here is not closed", entry.key),
        ));
    }
    Ok(Document {
        ty,
        version,
        body: root,
    })
}
