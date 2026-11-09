//! Legacy style metadata reader and writer. Appearance belongs exclusively
//! to CSS stylesheets; this format retains asset and token metadata until
//! the CSS pack loader replaces it.

use crate::doc::css::{
    Tok, camel, comment_of, describe, is_decl, kebab, quoted, split_decls, tokens, with_comment,
};
use og_core::ogt::{self, Block, Document, Entry, Line, Number, Value};

/// The token blocks: their custom properties' prefix, and whether their
/// values are lengths.
const TOKENS: &[(&str, &str, bool)] = &[
    ("palette", "", false),
    ("roles", "", false),
    ("colors", "", false),
    ("space", "space-", true),
    ("size", "size-", true),
    ("radius", "radius-", true),
    ("stroke", "stroke-", true),
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Ctx {
    Top,
    /// A token block, by its prefix and whether its values are lengths.
    Token(&'static str, bool),
    /// `.ogt` lines.
    Ogt,
}

fn child_ctx(parent: Ctx, key: &str) -> Ctx {
    match parent {
        Ctx::Top => TOKENS
            .iter()
            .find(|(k, _, _)| *k == key)
            .map_or(Ctx::Ogt, |(_, p, l)| Ctx::Token(p, *l)),
        _ => Ctx::Ogt,
    }
}

// ---- writing

fn is_hex(s: &str) -> bool {
    s.strip_prefix('#').is_some_and(|h| {
        matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit())
    })
}

/// One value: a colour bare (`#ffcc00`), a role `var(--line-strong)`, a
/// keyword kebab case, a length in px at the positions that are lengths.
fn value_css(v: &Value, px: bool, role: &dyn Fn(&str) -> bool) -> String {
    match v {
        Value::Number(n) if px && n.text() != "0" => format!("{}px", n.text()),
        Value::Number(n) => n.text().to_string(),
        Value::String(s) if is_hex(s) || s.starts_with("var(--") => s.clone(),
        Value::String(s) => quoted(s),
        Value::Word(w) if role(w) => format!("var(--{})", kebab(w)),
        Value::Word(w) => kebab(w),
    }
}

fn values_css(vals: &[Value], px: &dyn Fn(usize) -> bool, role: &dyn Fn(&str) -> bool) -> String {
    vals.iter()
        .enumerate()
        .map(|(i, v)| value_css(v, px(i), role))
        .collect::<Vec<_>>()
        .join(" ")
}

/// An `.ogt` line as the canonical writer writes it.
fn ogt_line(e: &Entry) -> Result<String, String> {
    let mut s = e.key.clone();
    for v in &e.values {
        s.push(' ');
        s.push_str(
            &ogt::value_text(v)
                .ok_or_else(|| format!("line {}: '{}' is not a word", e.line, e.key))?,
        );
    }
    Ok(s)
}

fn entry_css(ctx: Ctx, e: &Entry, role: &dyn Fn(&str) -> bool) -> Result<String, String> {
    match ctx {
        Ctx::Top if e.key == "dress" => Err(format!(
            "line {}: dress blocks were removed; use a CSS stylesheet",
            e.line
        )),
        Ctx::Top | Ctx::Ogt => ogt_line(e),
        Ctx::Token(prefix, len) => {
            if e.block.is_some() {
                return Err(format!("line {}: '{}' takes no block here", e.line, e.key));
            }
            Ok(format!(
                "--{prefix}{}: {}",
                kebab(&e.key),
                values_css(&e.values, &|_| len, role)
            ))
        }
    }
}

/// The palette roles and the style's own (`roles { }`): words written `var(--…)`.
fn role_names(body: &Block) -> Vec<String> {
    let mut names: Vec<String> = crate::style::Palette::NAMES
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(b) = body.get("roles").and_then(|e| e.block.as_ref()) {
        names.extend(b.entries.iter().map(|e| e.key.clone()));
    }
    names
}

fn write_block(
    out: &mut String,
    b: &Block,
    ctx: Ctx,
    depth: usize,
    role: &dyn Fn(&str) -> bool,
) -> Result<(), String> {
    for e in &b.entries {
        let line = entry_css(ctx, e, role)?;
        let child = child_ctx(ctx, &e.key);
        let pad = "  ".repeat(depth);
        match &e.block {
            Some(inner) => {
                out.push_str(&format!("{pad}{line} {{\n"));
                write_block(out, inner, child, depth + 1, role)?;
                out.push_str(&format!("{pad}}}\n"));
            }
            None => out.push_str(&format!("{pad}{line}\n")),
        }
    }
    Ok(())
}

/// A style's entries (any version's, read by [`read_style_doc`]) as a
/// version 3 text, without comments.
pub fn write_style_doc(doc: &Document) -> Result<String, String> {
    let roles = role_names(&doc.body);
    let role = |w: &str| roles.iter().any(|r| r == w);
    let mut out = format!("type {}\nversion 3\n---\n", doc.ty);
    write_block(&mut out, &doc.body, Ctx::Top, 0, &role)?;
    Ok(out)
}

/// A version 2 style text as version 3, line for line with its comments
/// (a version 1 text is `og style upgrade`'s), checked to parse to the same style.
pub fn migrate_style(text: &str) -> Result<String, String> {
    let version = header(text)?.version;
    let text = match version {
        3 => return Ok(text.to_string()),
        1 => return Err("a version 1 style: `og style upgrade` writes it as version 3".into()),
        2 => text,
        v => return Err(format!("style version {v} (this engine reads 1 to 3)")),
    };
    let old = super::parse_style(text)?;
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    let roles = role_names(&doc.body);
    let role = |w: &str| roles.iter().any(|r| r == w);
    let mut out = Vec::new();
    let mut lines = text.split('\n').enumerate();
    for (_, l) in lines.by_ref() {
        let t = l.trim();
        out.push(if t == "version 2" {
            l.replace("version 2", "version 3")
        } else {
            l.to_string()
        });
        if t == "---" {
            break;
        }
    }
    let mut stack = vec![Ctx::Top];
    for (i, raw) in lines {
        let n = i + 1;
        let (code, comment) = comment_of(raw);
        let indent = &raw[..raw.len() - raw.trim_start().len()];
        let ctx = *stack.last().unwrap();
        let line = match ogt::parse_line(code, n).map_err(|e| e.to_string())? {
            Line::Blank => String::new(),
            Line::Close => {
                stack.pop();
                "}".into()
            }
            Line::Entry { entry, opens } => {
                let s = entry_css(ctx, &entry, &role)?;
                if opens {
                    let c = child_ctx(ctx, &entry.key);
                    stack.push(c);
                    format!("{s} {{")
                } else {
                    s
                }
            }
        };
        let l = if line.is_empty() {
            String::new()
        } else {
            format!("{indent}{line}")
        };
        out.push(match comment {
            Some(c) if l.is_empty() => format!("{indent}{c}"),
            Some(c) => with_comment(l, c, raw),
            None => l,
        });
    }
    let new = out.join("\n");
    let mut parsed =
        super::parse_style(&new).map_err(|e| format!("the migrated style does not parse: {e}"))?;
    parsed.version = old.version;
    if parsed != old {
        return Err(
            "the migrated style reads differently from the version 2 one (a bug in og ui migrate)"
                .into(),
        );
    }
    Ok(new)
}

// ---- reading

fn header(text: &str) -> Result<Document, String> {
    let mut head = String::new();
    for l in text.split('\n') {
        head.push_str(l);
        head.push('\n');
        if l.trim() == "---" {
            break;
        }
    }
    Document::parse(&head).map_err(|e| e.to_string())
}

/// One declaration's value as `.ogt` values: `var(--x)` and keywords as
/// words (camel case), colours as strings, numbers with `px` or no unit.
fn ogt_values(value: &str) -> Result<Vec<Value>, String> {
    let mut in_str = false;
    for c in value.chars() {
        match c {
            '"' => in_str = !in_str,
            ',' if !in_str => return Err("values are separated by spaces, not commas".into()),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for t in tokens(value)? {
        out.push(match &t {
            Tok::Ident(s) if s.starts_with("--") => {
                return Err(format!("'{s}' is a token: write var({s})"));
            }
            Tok::Ident(s) => Value::Word(camel(s)),
            Tok::Func(f, a) if f == "var" => match a.as_slice() {
                [v] => match v.as_slice() {
                    [Tok::Ident(n)] if n.starts_with("--") => Value::String(format!("var({n})")),
                    _ => return Err("var(--name)".into()),
                },
                _ => return Err("var(--name) takes no fallback".into()),
            },
            Tok::Hash(h) if is_hex(h) => Value::String(h.clone()),
            Tok::Str(s) => Value::String(s.clone()),
            Tok::Num(n, u) if u.is_empty() || u == "px" => {
                Value::Number(Number::parse(n).map_err(|_| format!("'{n}' is not a number"))?)
            }
            t => return Err(format!("'{}' is not a style value", describe(t))),
        });
    }
    Ok(out)
}

/// Resolve a style's token aliases before parsing its paint geometry. The
/// source remains symbolic in the cooked blob; selecting a style reparses it.
/// As in CSS Variables §2.3, a cycle is invalid, including a self-reference.
pub(super) fn resolve_tokens(doc: &mut Document) -> Result<(), String> {
    use std::collections::BTreeMap;
    let mut definitions = BTreeMap::new();
    for (block, prefix, _) in TOKENS {
        if let Some(b) = doc.body.get(block).and_then(|e| e.block.as_ref()) {
            for e in &b.entries {
                if let Some(v) = e.values.first() {
                    definitions
                        .entry(format!("{prefix}{}", kebab(&e.key)))
                        .or_insert_with(|| v.clone());
                }
            }
        }
    }
    fn resolve(
        v: &Value,
        definitions: &BTreeMap<String, Value>,
        stack: &mut Vec<String>,
    ) -> Result<Value, String> {
        let Value::String(s) = v else {
            return Ok(v.clone());
        };
        let Some(name) = s.strip_prefix("var(--").and_then(|s| s.strip_suffix(')')) else {
            return Ok(v.clone());
        };
        if stack.iter().any(|n| n == name) {
            return Err(format!("cyclic token --{name}"));
        }
        let value = definitions
            .get(name)
            .ok_or_else(|| format!("unresolved token --{name}"))?;
        stack.push(name.to_owned());
        let result = resolve(value, definitions, stack);
        stack.pop();
        result
    }
    fn walk(b: &mut Block, definitions: &BTreeMap<String, Value>) -> Result<(), String> {
        for e in &mut b.entries {
            for (index, v) in e.values.iter_mut().enumerate() {
                let resolved = resolve(v, definitions, &mut Vec::new())
                    .map_err(|m| format!("line {}: {m}", e.line))?;
                let paint = e.key == "recolour" && index == 1;
                *v = if paint && matches!(&resolved, Value::String(s) if is_hex(s)) {
                    match v {
                        Value::String(s) if s.starts_with("var(--") => {
                            Value::Word(camel(&s[6..s.len() - 1]))
                        }
                        _ => resolved,
                    }
                } else {
                    resolved
                };
            }
            if let Some(b) = &mut e.block {
                walk(b, definitions)?
            }
        }
        Ok(())
    }
    walk(&mut doc.body, &definitions)
}

struct Frame {
    ctx: Ctx,
    entry: Entry,
}

impl Frame {
    fn new(ctx: Ctx, entry: Entry) -> Frame {
        Frame { ctx, entry }
    }
}

/// A version 3 line's code and its `#` comment: a `#` that starts a token
/// outside a string and no colour (`#0d0a08` is a value).
fn css_comment(line: &str) -> (&str, Option<&str>) {
    let b = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'#' if !in_str
                && (i == 0 || b[i - 1] == b' ' || b[i - 1] == b'\t')
                && !b.get(i + 1).is_some_and(u8::is_ascii_hexdigit) =>
            {
                return (line[..i].trim_end(), Some(&line[i..]));
            }
            _ => {}
        }
        i += 1;
    }
    (line.trim_end(), None)
}

fn lower(text: &str) -> Result<Document, String> {
    let head = header(text)?;
    let mut root = Entry::new("");
    root.block = Some(Block::new());
    let mut stack = vec![Frame::new(Ctx::Top, root)];
    for (i, raw) in text
        .lines()
        .enumerate()
        .skip_while(|(_, l)| l.trim() != "---")
        .skip(1)
    {
        let n = i + 1;
        let (code, _) = css_comment(raw.trim());
        if code.is_empty() {
            continue;
        }
        let error = |m: String| format!("line {n}: {m}");
        if code == "}" {
            if stack.len() == 1 {
                return Err(error("'}' closes nothing".into()));
            }
            let frame = stack.pop().unwrap();
            stack
                .last_mut()
                .unwrap()
                .entry
                .block
                .get_or_insert_with(Block::new)
                .push(frame.entry);
            continue;
        }
        let ctx = stack.last().unwrap().ctx;
        if let Ctx::Token(prefix, _) = ctx {
            for (name, value) in split_decls(code).map_err(error)? {
                let key = name
                    .strip_prefix("--")
                    .and_then(|r| r.strip_prefix(prefix))
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        error(format!(
                            "`{name}` is not a token of this block: write `--{prefix}<name>: …`"
                        ))
                    })?;
                let mut entry = Entry::new(camel(key));
                entry.line = n;
                entry.values = ogt_values(&value).map_err(error)?;
                stack
                    .last_mut()
                    .unwrap()
                    .entry
                    .block
                    .get_or_insert_with(Block::new)
                    .push(entry);
            }
        } else {
            if code.contains("::") || code.starts_with("dress ") {
                return Err(error(
                    "part rules belong in a CSS stylesheet, not style metadata".into(),
                ));
            }
            if is_decl(code) {
                return Err(error("a declaration belongs in a token block".into()));
            }
            if let Line::Entry { entry, opens } =
                ogt::parse_line(code, n).map_err(|e| e.to_string())?
            {
                if opens {
                    stack.push(Frame::new(child_ctx(ctx, &entry.key), entry));
                } else {
                    stack
                        .last_mut()
                        .unwrap()
                        .entry
                        .block
                        .get_or_insert_with(Block::new)
                        .push(entry);
                }
            }
        }
    }
    if stack.len() != 1 {
        return Err("a block is not closed".into());
    }
    Ok(Document {
        ty: head.ty,
        version: head.version,
        body: stack.pop().unwrap().entry.block.unwrap_or_default(),
    })
}

/// A style text of any version as its `.ogt` entries (version 3's lowered,
/// each on its source line); its `version` is the file's.
pub fn read_style_doc(text: &str) -> Result<Document, String> {
    if header(text)?.version == 3 {
        lower(text)
    } else {
        Document::parse(text).map_err(|e| e.to_string())
    }
}
