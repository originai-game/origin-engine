//! A minimal `.ogt` reader, private to `og-api`.
//!
//! It implements exactly the grammar in `crates/og-api/README.md` and nothing
//! else. `og-core` owns the engine's `.ogt` reader and writer; when that lands
//! this module is deleted and the IDL loader reads `og-core`'s tree instead.
//! Until then the two must accept the same text, so the grammar is written
//! down rather than inferred from this code.

use std::fmt;

/// A parsed `.ogt` document: the two header lines and the body tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub kind: String,
    pub version: u32,
    pub nodes: Vec<Node>,
}

/// One `key value...` line, with its nested block when the line ends in `{`.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub key: String,
    pub values: Vec<Value>,
    pub block: bool,
    pub children: Vec<Node>,
    pub line: usize,
}

/// One value on a line. Numbers keep their source text so integers stay exact.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Num(String),
    Bool(bool),
    Word(String),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_word(&self) -> Option<&str> {
        match self {
            Value::Word(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(s) => s.parse().ok(),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Num(s) | Value::Word(s) => f.write_str(s),
            Value::Bool(b) => write!(f, "{b}"),
        }
    }
}

/// A grammar error with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub line: usize,
    pub message: String,
}

fn err(line: usize, message: impl Into<String>) -> SyntaxError {
    SyntaxError {
        line,
        message: message.into(),
    }
}

/// Parse a whole `.ogt` document.
pub fn parse(text: &str) -> Result<Document, SyntaxError> {
    let lines: Vec<&str> = text.split('\n').collect();
    let line_at = |i: usize| lines.get(i).map(|l| l.strip_suffix('\r').unwrap_or(l));

    let kind = match line_at(0).and_then(|l| l.strip_prefix("type ")) {
        Some(k) if is_word(k) => k.to_string(),
        _ => return Err(err(1, "the first line must be `type <name>`")),
    };
    let version = match line_at(1).and_then(|l| l.strip_prefix("version ")) {
        Some(v) if !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()) => v
            .parse::<u32>()
            .map_err(|_| err(2, "the version does not fit in 32 bits"))?,
        _ => {
            return Err(err(
                2,
                "the second line must be `version <unsigned integer>`",
            ))
        }
    };
    if line_at(2) != Some("---") {
        return Err(err(3, "the third line must be the separator `---`"));
    }

    // A stack of open blocks; the bottom entry is the document body.
    let mut stack: Vec<Node> = vec![Node {
        key: String::new(),
        values: Vec::new(),
        block: true,
        children: Vec::new(),
        line: 3,
    }];
    for (index, raw) in lines.iter().enumerate().skip(3) {
        let line_no = index + 1;
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let tokens = tokenize(raw, line_no)?;
        if tokens.is_empty() {
            continue;
        }
        if tokens.len() == 1 && tokens[0] == Token::Close {
            if stack.len() == 1 {
                return Err(err(line_no, "`}` closes no open block"));
            }
            let done = stack.pop().expect("stack holds the body");
            stack
                .last_mut()
                .expect("stack holds the body")
                .children
                .push(done);
            continue;
        }
        let mut tokens = tokens;
        let block = matches!(tokens.last(), Some(Token::Open));
        if block {
            tokens.pop();
        }
        let mut iter = tokens.into_iter();
        let key = match iter.next() {
            Some(Token::Bare(k)) if is_word(&k) && k != "true" && k != "false" => k,
            Some(Token::Bare(k)) => {
                return Err(err(
                    line_no,
                    format!("`{k}` is not a key: a key is a word, a letter or `_` then letters, digits, `_`, `.` or `-`"),
                ))
            }
            _ => return Err(err(line_no, "a line must start with a key")),
        };
        let mut values = Vec::new();
        for token in iter {
            values.push(match token {
                Token::Str(s) => Value::Str(s),
                Token::Bare(b) => classify(&b, line_no)?,
                Token::Open => return Err(err(line_no, "`{` may only end a line")),
                Token::Close => return Err(err(line_no, "`}` must stand on its own line")),
            });
        }
        let node = Node {
            key,
            values,
            block,
            children: Vec::new(),
            line: line_no,
        };
        if block {
            stack.push(node);
        } else {
            stack
                .last_mut()
                .expect("stack holds the body")
                .children
                .push(node);
        }
    }
    if stack.len() > 1 {
        let open = stack.last().expect("non-empty");
        return Err(err(
            open.line,
            format!("the block opened by `{}` is never closed", open.key),
        ));
    }
    let body = stack.pop().expect("stack holds the body");
    Ok(Document {
        kind,
        version,
        nodes: body.children,
    })
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Bare(String),
    Open,
    Close,
}

fn tokenize(line: &str, line_no: usize) -> Result<Vec<Token>, SyntaxError> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some(&(_, c)) = chars.peek() {
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
                    None => return Err(err(line_no, "unterminated string")),
                    Some((_, '"')) => break,
                    Some((_, '\\')) => match chars.next() {
                        Some((_, '"')) => s.push('"'),
                        Some((_, '\\')) => s.push('\\'),
                        Some((_, 'n')) => s.push('\n'),
                        Some((_, 'r')) => s.push('\r'),
                        Some((_, 't')) => s.push('\t'),
                        Some((_, 'u')) => s.push(unicode_escape(&mut chars, line_no)?),
                        _ => {
                            return Err(err(
                                line_no,
                                "unknown escape: use \\\" \\\\ \\n \\r \\t or \\u{XXXX}",
                            ))
                        }
                    },
                    Some((_, ch)) => s.push(ch),
                }
            }
            if let Some(&(_, next)) = chars.peek() {
                if next != ' ' && next != '\t' {
                    return Err(err(
                        line_no,
                        "a string must be followed by a space or the line end",
                    ));
                }
            }
            tokens.push(Token::Str(s));
            continue;
        }
        let mut bare = String::new();
        while let Some(&(_, ch)) = chars.peek() {
            if ch == ' ' || ch == '\t' {
                break;
            }
            bare.push(ch);
            chars.next();
        }
        tokens.push(match bare.as_str() {
            "{" => Token::Open,
            "}" => Token::Close,
            _ => Token::Bare(bare),
        });
    }
    Ok(tokens)
}

fn unicode_escape(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    line_no: usize,
) -> Result<char, SyntaxError> {
    let bad = || err(line_no, "a \\u escape is written \\u{XXXX}");
    if chars.next().map(|c| c.1) != Some('{') {
        return Err(bad());
    }
    let mut hex = String::new();
    loop {
        match chars.next() {
            Some((_, '}')) => break,
            Some((_, c)) if c.is_ascii_hexdigit() && hex.len() < 6 => hex.push(c),
            _ => return Err(bad()),
        }
    }
    u32::from_str_radix(&hex, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(bad)
}

fn classify(token: &str, line_no: usize) -> Result<Value, SyntaxError> {
    match token {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    if token.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        if is_number(token) {
            return Ok(Value::Num(token.to_string()));
        }
        return Err(err(line_no, format!("`{token}` is not a number")));
    }
    if is_word(token) {
        return Ok(Value::Word(token.to_string()));
    }
    Err(err(
        line_no,
        format!("`{token}` is not a number, string or word: quote strings, write numbers bare"),
    ))
}

/// `-?digits(.digits)?([eE][+-]?digits)?`.
fn is_number(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && b[i] == b'-' {
        i += 1;
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
        if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
            i += 1;
        }
        if !digits(&mut i) {
            return false;
        }
    }
    i == b.len()
}

/// A word: `[A-Za-z_][A-Za-z0-9_.-]*`, as og-core's reader defines it.
fn is_word(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_alphabetic() || ch == '_')
        && c.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_blocks_values_and_comments() {
        let doc = parse(
            "type api\nversion 1\n---\n# comment\nmodule world {\n  doc \"a \\\"b\\\" \\u{e9}\" # trailing\n  field x f32 {\n    default -1.5e2\n    tunable true\n  }\n}\n",
        )
        .unwrap();
        assert_eq!(doc.kind, "api");
        assert_eq!(doc.version, 1);
        let module = &doc.nodes[0];
        assert_eq!(module.key, "module");
        assert_eq!(module.values, vec![Value::Word("world".into())]);
        assert_eq!(
            module.children[0].values,
            vec![Value::Str("a \"b\" é".into())]
        );
        let field = &module.children[1];
        assert_eq!(field.line, 7);
        assert_eq!(field.children[0].values, vec![Value::Num("-1.5e2".into())]);
        assert_eq!(field.children[1].values, vec![Value::Bool(true)]);
    }

    #[test]
    fn refuses_malformed_text_with_the_line() {
        let cases = [
            ("version 1\n", 1, "type"),
            ("type api\nversion x\n", 2, "version"),
            ("type api\nversion 1\n--\n", 3, "separator"),
            ("type api\nversion 1\n---\na \"open\n", 4, "unterminated"),
            ("type api\nversion 1\n---\na {\n", 4, "never closed"),
            ("type api\nversion 1\n---\n}\n", 4, "closes no"),
            ("type api\nversion 1\n---\n\"k\" 1\n", 4, "start with a key"),
            (
                "type api\nversion 1\n---\na List<T>\n",
                4,
                "not a number, string or word",
            ),
            ("type api\nversion 1\n---\na 1.\n", 4, "not a number"),
            ("type api\nversion 1\n---\na { b\n", 4, "only end a line"),
        ];
        for (text, line, needle) in cases {
            let e = parse(text).expect_err(text);
            assert_eq!(e.line, line, "{text:?}: {e:?}");
            assert!(e.message.contains(needle), "{text:?}: {}", e.message);
        }
    }
}
