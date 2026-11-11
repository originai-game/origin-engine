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
//! lines are not part of the data and are not written back; [`format`]
//! rewrites a text in the canonical layout and keeps them.

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

    /// Parses only the header (`type`, `version`, up to `---`), with an
    /// empty body: what a file is, for formats whose body has lines of
    /// their own (og-ui's documents and styles).
    pub fn parse_header(text: &str) -> Result<Document> {
        let mut head = String::new();
        for l in text.split('\n') {
            head.push_str(l);
            head.push('\n');
            if l.trim() == "---" {
                break;
            }
        }
        parse(&head)
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
    scan(line, n).map(|(tokens, _)| tokens)
}

/// A line's tokens and its comment (from its `#`), if it has one.
fn scan(line: &str, n: usize) -> Result<(Vec<Token>, Option<&str>)> {
    let mut tokens = Vec::new();
    let mut comment = None;
    let mut chars = line.char_indices().peekable();
    while let Some(&(start, c)) = chars.peek() {
        if c == ' ' || c == '\t' {
            chars.next();
            continue;
        }
        if c == '#' {
            comment = Some(&line[start..]);
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
    Ok((tokens, comment))
}

/// Parses `.ogt` text into a [`Document`].
/// One body line of an `.ogt` text, as [`parse`] reads it.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    /// Blank, or only a comment.
    Blank,
    /// `}`: the open block ends.
    Close,
    /// `key value*`; `opens` when it ends with `{` (its `block` is then
    /// `Some`, empty).
    Entry { entry: Entry, opens: bool },
}

/// Reads one body line (`n` is its 1-based number, kept on the entry and
/// in errors) by the grammar above. Formats that keep `.ogt`'s lines but
/// add forms of their own (og-ui's documents and styles) read their other
/// lines with it.
pub fn parse_line(line: &str, n: usize) -> Result<Line> {
    let mut tokens = tokenize(line, n)?.into_iter();
    let key = match tokens.next() {
        None => return Ok(Line::Blank),
        Some(Token::Close) => {
            if tokens.next().is_some() {
                return Err(parse_error(n, "'}' must be alone on its line"));
            }
            return Ok(Line::Close);
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
    }
    Ok(Line::Entry { entry, opens })
}

/// One value as the canonical writer writes it; `None` for a word that is
/// not an identifier (it would not read back).
pub fn value_text(value: &Value) -> Option<String> {
    let mut out = String::new();
    match value {
        Value::Number(n) => out.push_str(n.text()),
        Value::String(s) => write_string(&mut out, s),
        Value::Word(w) if is_word(w) => out.push_str(w),
        Value::Word(_) => return None,
    }
    Some(out)
}

/// `text` in the canonical layout, with its comments: what
/// [`Document::to_text`] writes for the same data (two spaces per level,
/// single spaces between tokens, canonical strings, `\n` line ends, a final
/// newline), each comment kept on its line (a line's own two spaces after
/// its tokens) and one blank line kept wherever the text had one or more
/// between two lines of a block. The order of entries is data and stays.
/// `parse(format(t)) == parse(t)`, and `format(format(t)) == format(t)`.
pub fn format(text: &str) -> Result<String> {
    let data = parse(text)?;
    let mut out = String::new();
    let (mut depth, mut body, mut blank) = (0usize, false, false);
    // Whether a blank line may be written here: not first in the file or
    // in a block, and not last.
    let mut inside = false;
    for (i, line) in text.split('\n').enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if !body && line.trim_matches([' ', '\t']) == "---" {
            out.push_str("---\n");
            (body, blank, inside) = (true, false, false);
            continue;
        }
        let (tokens, comment) = scan(line, i + 1)?;
        let comment = comment.map(str::trim_end);
        if tokens.is_empty() && comment.is_none() {
            blank = inside;
            continue;
        }
        let closes = matches!(tokens.first(), Some(Token::Close));
        if closes {
            depth = depth.saturating_sub(1);
        } else if blank {
            out.push('\n');
        }
        blank = false;
        out.extend(std::iter::repeat_n("  ", depth));
        for (j, token) in tokens.iter().enumerate() {
            if j > 0 {
                out.push(' ');
            }
            match token {
                Token::Word(w) => out.push_str(w),
                Token::Number(n) => out.push_str(n.text()),
                Token::Str(s) => write_string(&mut out, s),
                Token::Open => out.push('{'),
                Token::Close => out.push('}'),
            }
        }
        if let Some(c) = comment {
            if !tokens.is_empty() {
                out.push_str("  ");
            }
            out.push_str(c);
        }
        out.push('\n');
        let opens = matches!(tokens.last(), Some(Token::Open));
        if opens {
            depth += 1;
        }
        inside = !opens;
    }
    if parse(&out).as_ref() != Ok(&data) {
        return Err(Error::new(
            ErrorKind::Invalid,
            "the formatted text does not read back as the same data",
        ));
    }
    Ok(out)
}

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
        let (entry, opens) = match parse_line(line, n)? {
            Line::Blank => continue,
            Line::Close => {
                let (mut entry, _) = stack.pop().ok_or_else(|| parse_error(n, "unmatched '}'"))?;
                entry.block.get_or_insert_with(Block::new);
                match stack.last_mut() {
                    Some((parent, _)) => parent.block.get_or_insert_with(Block::new).push(entry),
                    None => root.push(entry),
                }
                continue;
            }
            Line::Entry { entry, opens } => (entry, opens),
        };
        if opens {
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

#[cfg(test)]
mod ogt_contracts {
    //! `.ogt`: the reader and the canonical writer round-trip.

    use crate::ErrorKind;
    use crate::ogt::{Block, Document, Entry, Number, Value, format, parse};
    use std::path::{Path, PathBuf};

    const CANONICAL: &str = r#"type scene
version 3
---
name "Main \"hall\" \\ east"
gravity 0 -9.81 0
seed 4294967295
tiny 1e-7
huge -6.02e23
flags true false linear og.Transform ease-in
note "line one\nline two\ttabbed, # not a comment, { not a block }"
unicode "天气 é \u{1}"
entity "player" {
  Transform {
    position 0 1.5 -2
    rotation 0 0 0 1
  }
  Health {
    hp 100
  }
  children {
    entity "sword" {
      Transform {
        position 0.25 0 0
      }
    }
  }
  empty {
  }
}
entity "camera" {
  Camera {
    fov 60
  }
}
"#;

    fn values(entry: &Entry) -> Vec<String> {
        entry
            .values
            .iter()
            .map(|v| match v {
                Value::Number(n) => format!("n:{}", n.text()),
                Value::String(s) => format!("s:{s}"),
                Value::Word(w) => format!("w:{w}"),
            })
            .collect()
    }

    #[test]
    fn canonical_text_round_trips_exactly() {
        let doc = parse(CANONICAL).unwrap();
        assert_eq!(doc.to_text().unwrap(), CANONICAL);
        assert_eq!(parse(&doc.to_text().unwrap()).unwrap(), doc);
    }

    #[test]
    fn the_tree_holds_what_the_text_says() {
        let doc = parse(CANONICAL).unwrap();
        assert_eq!((doc.ty.as_str(), doc.version), ("scene", 3));
        let body = &doc.body;
        assert_eq!(
            body.get("name").unwrap().string(0).unwrap(),
            "Main \"hall\" \\ east"
        );
        assert_eq!(
            values(body.get("gravity").unwrap()),
            ["n:0", "n:-9.81", "n:0"]
        );
        assert_eq!(body.get("gravity").unwrap().f32(1).unwrap(), -9.81f32);
        assert_eq!(
            body.get("seed")
                .unwrap()
                .number(0)
                .unwrap()
                .as_u64()
                .unwrap(),
            4294967295
        );
        assert_eq!(body.get("tiny").unwrap().number(0).unwrap().as_f64(), 1e-7);
        let flags = body.get("flags").unwrap();
        assert_eq!(flags.values[0].as_bool(), Some(true));
        assert_eq!(flags.values[1].as_bool(), Some(false));
        assert_eq!(flags.word(3).unwrap(), "og.Transform");
        assert_eq!(
            body.get("note").unwrap().string(0).unwrap(),
            "line one\nline two\ttabbed, # not a comment, { not a block }"
        );
        assert_eq!(
            body.get("unicode").unwrap().string(0).unwrap(),
            "天气 é \u{1}"
        );

        let entities: Vec<_> = body.get_all("entity").collect();
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].string(0).unwrap(), "player");
        assert_eq!(entities[0].line, 12);
        let player = entities[0].block.as_ref().unwrap();
        let position = player
            .get("Transform")
            .unwrap()
            .block
            .as_ref()
            .unwrap()
            .get("position")
            .unwrap();
        assert_eq!(values(position), ["n:0", "n:1.5", "n:-2"]);
        let sword = player
            .get("children")
            .unwrap()
            .block
            .as_ref()
            .unwrap()
            .get("entity")
            .unwrap();
        assert_eq!(sword.string(0).unwrap(), "sword");
        assert_eq!(player.get("empty").unwrap().block, Some(Block::new()));
        assert_eq!(
            entities[0].number(0).unwrap_err().kind(),
            ErrorKind::NotFound
        );
    }

    #[test]
    fn loose_text_reads_as_its_canonical_form() {
        let loose = "# a header comment\r\ntype   scene\r\nversion 3\n---\n\n\t name \"Main \\\"hall\\\" \\\\ east\"   # trailing\n gravity\t0 -9.81 0\nseed 4294967295\ntiny 1e-7\nhuge -6.02e23\nflags true false linear og.Transform ease-in\nnote \"line one\\nline two\\ttabbed, # not a comment, { not a block }\"\nunicode \"天气 \\u{e9} \\u{1}\"\n  entity \"player\" {\n      Transform {\n position 0 1.5 -2\nrotation 0 0 0 1\n}\nHealth {\nhp 100\n}\nchildren {\nentity \"sword\" {\nTransform {\nposition 0.25 0 0\n}\n}\n}\nempty {\n}\n}\nentity \"camera\" {\n# inside\nCamera {\nfov 60\n}\n}";
        let doc = parse(loose).unwrap();
        assert_eq!(doc, parse(CANONICAL).unwrap());
        assert_eq!(doc.to_text().unwrap(), CANONICAL);
    }

    #[test]
    fn formatting_keeps_comments_and_blank_lines_and_changes_no_data() {
        let loose = "# what this is\r\n\r\ntype   scene # the kind\r\nversion 3\n---\n\n\n# the hero\n\t entity \"player\"   {   # trailing\n\n      Health {\n# inside, \"quoted\" { and } kept as written\nhp   100\n\n\n\nmax 1e2 #tight\n\n}\n\n\n}\nnote \"a # in a string\"\n\n\n";
        let formatted = "# what this is\n\ntype scene  # the kind\nversion 3\n---\n# the hero\nentity \"player\" {  # trailing\n  Health {\n    # inside, \"quoted\" { and } kept as written\n    hp 100\n\n    max 1e2  #tight\n  }\n}\nnote \"a # in a string\"\n";
        assert_eq!(format(loose).unwrap(), formatted);
        assert_eq!(format(formatted).unwrap(), formatted);
        assert_eq!(parse(formatted).unwrap(), parse(loose).unwrap());
        // Without comments or blank lines it is the canonical writer's text.
        assert_eq!(format(CANONICAL).unwrap(), CANONICAL);
        // Text the reader refuses is refused the same way, never rewritten.
        assert_eq!(
            format("type scene\nversion 1\n---\na {\n").unwrap_err(),
            parse("type scene\nversion 1\n---\na {\n").unwrap_err()
        );
    }

    #[test]
    fn documents_built_in_memory_write_and_read_back() {
        let mut doc = Document::new("material", 1);
        doc.body.push(
            Entry::new("albedo")
                .with(Number::from_f32(0.1).unwrap())
                .with(Number::from_f32(0.7).unwrap())
                .with(Number::from_f32(1e-9).unwrap()),
        );
        doc.body
            .push(Entry::new("roughness").with(Number::from_f64(0.30000000000000004).unwrap()));
        doc.body.push(
            Entry::new("count")
                .with(Number::from(-42i64))
                .with(Number::from(u64::MAX)),
        );
        doc.body
            .push(Entry::new("path").with(Value::String("assets/a b.gltf".into())));
        doc.body.push(
            Entry::new("pass")
                .with(Value::Word("forward".into()))
                .with_block(
                    Block::new().with(Entry::new("depth").with(Value::Word("true".into()))),
                ),
        );
        let text = doc.to_text().unwrap();
        let back = parse(&text).unwrap();
        assert_eq!(back, doc);
        assert_eq!(back.body.get("albedo").unwrap().f32(0).unwrap(), 0.1f32);
        assert_eq!(back.body.get("albedo").unwrap().f32(2).unwrap(), 1e-9f32);
        assert_eq!(
            back.body
                .get("roughness")
                .unwrap()
                .number(0)
                .unwrap()
                .as_f64(),
            0.30000000000000004
        );
        assert_eq!(
            back.body
                .get("count")
                .unwrap()
                .number(1)
                .unwrap()
                .as_u64()
                .unwrap(),
            u64::MAX
        );
        assert!(text.contains("albedo 0.1 0.7 1e-9\n"), "{text}");

        assert!(Number::from_f32(f32::NAN).is_err());
        let mut bad = Document::new("scene", 1);
        bad.body.push(Entry::new("two words"));
        assert_eq!(bad.to_text().unwrap_err().kind(), ErrorKind::Invalid);
        let mut bad = Document::new("scene", 1);
        bad.body
            .push(Entry::new("k").with(Value::Word("not a word".into())));
        assert!(bad.to_text().is_err());
    }

    #[test]
    fn malformed_text_is_refused_with_its_line() {
        let h = "type scene\nversion 1\n---\n";
        let cases: &[(String, &str)] = &[
            (String::new(), "missing header"),
            (
                "version 1\ntype scene\n---\n".into(),
                "line 1: expected 'type",
            ),
            (
                "type scene\nversion x\n---\n".into(),
                "line 2: expected 'version",
            ),
            (
                "type scene\nversion 1\nname 2\n".into(),
                "line 3: expected '---'",
            ),
            (format!("{h}a \"open\n"), "line 4: unterminated string"),
            (
                format!("{h}a \"x\"y\n"),
                "line 4: a string must be followed",
            ),
            (format!("{h}a \"\\q\"\n"), "line 4: unknown escape"),
            (
                format!("{h}a {{\nb 1\n"),
                "line 4: block 'a' opened here is not closed",
            ),
            (format!("{h}a 1\n}}\n"), "line 5: unmatched '}'"),
            (format!("{h}\"k\" 1\n"), "line 4: a line starts with a key"),
            (format!("{h}true 1\n"), "line 4: a line starts with a key"),
            (
                format!("{h}a List<u32>\n"),
                "line 4: 'List<u32>' is not a number",
            ),
            (format!("{h}a 1.\n"), "line 4: '1.' is not a number"),
            (
                format!("{h}a 1e999\n"),
                "line 4: number 1e999 is out of range",
            ),
            (format!("{h}a {{ b\n}}\n"), "line 4: '{' must end its line"),
            (format!("{h}a {{\n}} x\n"), "line 5: '}' must be alone"),
        ];
        for (text, needle) in cases {
            let err = parse(text).expect_err(text);
            assert_eq!(err.kind(), ErrorKind::Parse);
            assert!(
                err.reason().contains(needle),
                "{text:?}: got '{}', expected '{needle}'",
                err.reason()
            );
        }
    }

    fn ogt_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.join("parts.txt").is_file() {
                // A document split into parts (the API IDL): one file, the
                // concatenation `parts.txt` lists.
                out.push(path);
            } else if path.is_dir() {
                if !matches!(name.as_ref(), "target" | ".git" | "node_modules")
                    && !name.starts_with('.')
                {
                    ogt_files(&path, out);
                }
            } else if name.ends_with(".ogt") {
                out.push(path);
            }
        }
    }

    /// Every `.ogt` file in the repository (the API IDL, templates, fixtures)
    /// but UI documents and styles either round-trips through the canonical
    /// writer, or is a fixture that declares `# expect: <line>: ...` and is
    /// refused at that line.
    #[test]
    fn every_ogt_file_in_the_repository_round_trips() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut files = Vec::new();
        ogt_files(&root, &mut files);
        files.sort();
        let mut round_tripped = 0;
        for path in &files {
            // The walk includes gitignored copies another gate step rewrites
            // while this test runs (`packages/create`'s packed `template/`,
            // a project's `cooked/`); one that vanished between the listing
            // and the read is not a file of the repository.
            let text = if path.is_dir() {
                let parts = std::fs::read_to_string(path.join("parts.txt"))
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                parts
                    .lines()
                    .map(|part| {
                        std::fs::read_to_string(path.join(part))
                            .unwrap_or_else(|e| panic!("{}/{part}: {e}", path.display()))
                    })
                    .collect()
            } else {
                match std::fs::read_to_string(path) {
                    Ok(text) => text,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => panic!("{}: {e}", path.display()),
                }
            };
            // UI documents and styles keep the header but write their own
            // lines (CSS declarations, read with `parse_line` by og-ui,
            // which has their round-trip tests).
            if Document::parse_header(&text)
                .is_ok_and(|h| matches!(h.ty.as_str(), "screen" | "component" | "style"))
            {
                continue;
            }
            let expect_line = text
                .lines()
                .find_map(|l| l.strip_prefix("# expect: "))
                .and_then(|e| e.split_once(':'))
                .and_then(|(line, _)| line.trim().parse::<usize>().ok());
            match parse(&text) {
                Ok(doc) => {
                    let canonical = doc
                        .to_text()
                        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    let again =
                        parse(&canonical).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    assert_eq!(again, doc, "{}", path.display());
                    assert_eq!(again.to_text().unwrap(), canonical, "{}", path.display());
                    // The formatter keeps the file's data, settles in one
                    // pass, and adds nothing to text without comments.
                    let formatted =
                        format(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    assert_eq!(parse(&formatted).unwrap(), doc, "{}", path.display());
                    assert_eq!(format(&formatted).unwrap(), formatted, "{}", path.display());
                    assert_eq!(format(&canonical).unwrap(), canonical, "{}", path.display());
                    round_tripped += 1;
                }
                Err(err) => {
                    let line = expect_line.unwrap_or_else(|| panic!("{}: {err}", path.display()));
                    assert!(
                        err.reason().starts_with(&format!("line {line}:")),
                        "{}: refused as '{}', the fixture expects line {line}",
                        path.display(),
                        err.reason()
                    );
                }
            }
        }
        assert!(
            round_tripped >= 1,
            "no .ogt file found under {}",
            root.display()
        );
    }
}
