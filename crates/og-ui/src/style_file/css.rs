//! The CSS dialect of styles (`version 3`, docs/ui.md §3): a part
//! dressing is a selector block, its part a pseudo-element, its variants
//! classes and its states pseudo-classes (`Button::base.primary:hover {`,
//! `Slot::frame[rarity=epic] {`); the colour and length tokens are custom
//! properties (`palette { --panel-raised: #211a15 }`, `space { --space-xs:
//! 4px }`); what a dressing sets is declarations, CSS where CSS has the
//! property (`box-shadow`, `mix-blend-mode`) and `-og-` extensions for the
//! rest (`-og-slice`, `-og-fill`, `-og-stroke`, `-og-effect`,
//! `-og-material`); the element properties a dressing overrides are the
//! documents' own declarations (`opacity: 0.4`, `height: 7px`). The art
//! geometry, fonts, icons, typography, elevation, shapes, motion and sounds
//! keep their `.ogt` lines.
//!
//! [`read_style_doc`] reads any version into the `.ogt` entries the parser
//! reads (each on its source line, so errors and a dressing's source lines
//! name the version 3 file), [`write_style_doc`] writes entries as version
//! 3 (what the tools that edit a style write), and [`migrate_style`]
//! writes a version 2 text as version 3 line for line, comments kept,
//! checking the two parse to the same style. One table, [`DRESS`], drives
//! all three. The subset is strict: a property, selector or value it does
//! not know is an error that names the form to write.

use crate::doc::css::{
    Tok, camel, comment_of, describe, is_decl, kebab, quoted, split_decls, tokens, with_comment,
};
use crate::dress::{APPEARANCE_PROPS, Selector};
use og_core::ogt::{self, Block, Document, Entry, Line, Number, Value};

/// A dressing key, its declaration and the value positions written in px.
pub const DRESS: &[(&str, &str, &[usize])] = &[
    ("slice", "-og-slice", &[]),
    ("tone", "-og-tone", &[]),
    ("recolour", "-og-recolour", &[]),
    ("shadow", "box-shadow", &[0, 1, 2, 3]),
    ("glow", "-og-glow", &[]),
    ("glass", "-og-glass", &[]),
    ("corners", "-og-corners", &[]),
    ("crest", "-og-crest", &[]),
    ("edges", "-og-edges", &[]),
    ("tile", "-og-tile", &[]),
    ("shape", "-og-shape", &[]),
    ("press", "-og-press", &[]),
    ("hover", "-og-hover", &[]),
    ("fill", "-og-fill", &[]),
    ("stroke", "-og-stroke", &[0]),
    ("effect", "-og-effect", &[]),
    ("material", "-og-material", &[]),
    ("ornament", "-og-ornament", &[]),
    ("ornamentTint", "-og-ornament-tint", &[]),
    ("blend", "mix-blend-mode", &[]),
    ("mask", "-og-mask", &[]),
    ("sound", "-og-sound", &[]),
    ("segments", "-og-segments", &[]),
    ("ink", "-og-ink", &[]),
    ("step", "-og-step", &[]),
];

/// Dressing keys that take no value: written `-og-corners: true`.
const BARE: &[&str] = &["corners", "crest", "edges", "tile"];

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

/// Selector flags written as pseudo-classes, and their pseudo-class.
const PSEUDO: &[(&str, &str)] = &[
    ("hover", "hover"),
    ("pressed", "active"),
    ("focused", "focus-visible"),
    ("disabled", "disabled"),
    ("selected", "selected"),
    ("on", "checked"),
    ("expanded", "expanded"),
];

/// CSS a dressing does not take, and what to write.
fn dress_hint(name: &str) -> &'static str {
    match name {
        "background" | "background-color" | "background-image" => {
            "a part's paint is `-og-fill: <paint>` (fill layers stack in order)"
        }
        "border" | "border-width" | "border-color" | "outline" => {
            "a part's outline is `-og-stroke: <width>px <paint> inside|outside`"
        }
        "border-radius" => "a part's corner is `-og-shape: round 4px` (or a radius name)",
        "border-image" => "a part's nine-slice is `-og-slice: <x> <y> <w> <h> <insets>`",
        "filter" | "backdrop-filter" => "effects are `-og-effect: drop-shadow …`, `outer-glow …`",
        "transform" => "a part's press and hover are `-og-press` and `-og-hover`",
        _ => "the dressing properties: docs/ui.md §3.2",
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Ctx {
    Top,
    /// A token block, by its prefix and whether its values are lengths.
    Token(&'static str, bool),
    Dress,
    /// A dressing's `appearance` block (version 2), written as declarations.
    Appearance,
    /// `.ogt` lines.
    Ogt,
}

fn child_ctx(parent: Ctx, key: &str) -> Ctx {
    match parent {
        Ctx::Top if key == "dress" => Ctx::Dress,
        Ctx::Top => TOKENS
            .iter()
            .find(|(k, _, _)| *k == key)
            .map_or(Ctx::Ogt, |(_, p, l)| Ctx::Token(p, *l)),
        Ctx::Dress if key == "appearance" => Ctx::Appearance,
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

/// `Button.base[primary][hover]` as `Button::base.primary:hover`.
pub fn selector_css(sel: &Selector) -> String {
    let mut out = format!("{}::{}", sel.component, sel.part);
    for f in &sel.flags {
        match PSEUDO.iter().find(|(s, _)| s == f) {
            Some((_, p)) => out.push_str(&format!(":{p}")),
            None => out.push_str(&format!(".{f}")),
        }
    }
    for (k, v) in &sel.attrs {
        out.push_str(&format!("[{k}={v}]"));
    }
    out
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

/// Entry `e` in `ctx` as its version 3 line, without `{` (`""` for a line
/// version 3 does not write: an `appearance` block's braces).
fn entry_css(ctx: Ctx, e: &Entry, role: &dyn Fn(&str) -> bool) -> Result<String, String> {
    let at = |m: String| format!("line {}: {m}", e.line);
    let no_block = || {
        if e.block.is_some() {
            Err(at(format!("'{}' takes no block here", e.key)))
        } else {
            Ok(())
        }
    };
    match ctx {
        Ctx::Top if e.key == "dress" => {
            let s = e
                .values
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| at("a dress names its selector".into()))?;
            Ok(selector_css(&Selector::parse(s).map_err(at)?))
        }
        Ctx::Top | Ctx::Ogt => ogt_line(e),
        Ctx::Token(prefix, len) => {
            no_block()?;
            Ok(format!(
                "--{prefix}{}: {}",
                kebab(&e.key),
                values_css(&e.values, &|_| len, role)
            ))
        }
        Ctx::Dress if e.key == "appearance" => Ok(String::new()),
        Ctx::Dress => {
            no_block()?;
            let (_, css, px) = DRESS
                .iter()
                .find(|(k, _, _)| *k == e.key)
                .ok_or_else(|| at(format!("'{}' is not a dressing setting", e.key)))?;
            if BARE.contains(&e.key.as_str()) && e.values.is_empty() {
                return Ok(format!("{css}: true"));
            }
            // An effect's, material's or ornament's first word is its kind.
            let kind = matches!(
                e.key.as_str(),
                "effect" | "material" | "ornament" | "mask" | "blend" | "segments"
            );
            let vals = e
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| value_css(v, px.contains(&i), &|w| !(kind && i == 0) && role(w)));
            Ok(format!("{css}: {}", vals.collect::<Vec<_>>().join(" ")))
        }
        Ctx::Appearance => {
            no_block()?;
            crate::doc::css::decl_line(&[(e.key.clone(), e.values.clone())], false, &|_| None)
                .map_err(at)
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
            Some(inner) if child == Ctx::Appearance => write_block(out, inner, child, depth, role)?,
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
                if ctx == Ctx::Appearance {
                    String::new()
                } else {
                    "}".into()
                }
            }
            Line::Entry { entry, opens } => {
                let s = entry_css(ctx, &entry, &role)?;
                if opens {
                    let c = child_ctx(ctx, &entry.key);
                    stack.push(c);
                    if c == Ctx::Appearance {
                        s
                    } else {
                        format!("{s} {{")
                    }
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
            // Appearance uses the same symbolic literals as documents.
            if e.key == "appearance" {
                continue;
            }
            for (index, v) in e.values.iter_mut().enumerate() {
                let resolved = resolve(v, definitions, &mut Vec::new())
                    .map_err(|m| format!("line {}: {m}", e.line))?;
                let paint = matches!(
                    e.key.as_str(),
                    "fill" | "stroke" | "effect" | "mask" | "ink" | "ornamentTint" | "glow"
                ) || (e.key == "recolour" && index == 1);
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

/// `Button::base.primary:hover[rarity=epic]` as `Button.base[primary][hover][rarity=epic]`.
fn selector_ogt(text: &str) -> Result<String, String> {
    let (component, rest) = text.split_once("::").ok_or_else(|| {
        format!(
            "'{text}' is not a part selector: Component::part, such as Button::base.primary:hover"
        )
    })?;
    let end = rest.find(['.', ':', '[']).unwrap_or(rest.len());
    let mut out = format!("{component}.{}", &rest[..end]);
    let mut r = &rest[end..];
    let word = |s: &str| s.find(['.', ':', '[']).unwrap_or(s.len());
    while !r.is_empty() {
        if let Some(s) = r.strip_prefix('.') {
            let w = &s[..word(s)];
            if let Some((_, p)) = PSEUDO.iter().find(|(f, _)| *f == w) {
                return Err(format!("`.{w}` is a state: write `:{p}`"));
            }
            out.push_str(&format!("[{w}]"));
            r = &s[w.len()..];
        } else if let Some(s) = r.strip_prefix(':') {
            let w = &s[..word(s)];
            let flag = match PSEUDO.iter().find(|(_, p)| *p == w) {
                Some((f, _)) => f,
                None if w == "focus" => {
                    return Err("`:focus` is `:focus-visible` (the focus the player moves)".into());
                }
                None => {
                    return Err(format!(
                        "`:{w}` is not a state of a part ({}); a variant is a class (`.primary`)",
                        PSEUDO
                            .iter()
                            .map(|(_, p)| format!(":{p}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            };
            out.push_str(&format!("[{flag}]"));
            r = &s[w.len()..];
        } else if let Some(s) = r.strip_prefix('[') {
            let e = s.find(']').ok_or("an unclosed [")?;
            out.push_str(&format!("[{}]", &s[..e]));
            r = &s[e + 1..];
        } else {
            return Err(format!("'{r}' is not part of a selector"));
        }
    }
    Selector::parse(&out)?;
    Ok(out)
}

/// The CSS names of the element properties a dressing overrides.
fn appearance_css(name: &str) -> bool {
    let base = name
        .strip_prefix("min-")
        .or(name.strip_prefix("max-"))
        .unwrap_or(name);
    APPEARANCE_PROPS
        .iter()
        .any(|p| crate::doc::css::css_name(p) == Some(base))
}

struct Frame {
    ctx: Ctx,
    entry: Entry,
    /// A dressing's element-property declarations: their line numbers and
    /// text, and where its `appearance` entry goes.
    appearance: Vec<(usize, String)>,
    appearance_at: Option<usize>,
}

impl Frame {
    fn new(ctx: Ctx, entry: Entry) -> Frame {
        Frame {
            ctx,
            entry,
            appearance: Vec::new(),
            appearance_at: None,
        }
    }

    /// The finished entry: its `appearance` declarations read as an
    /// element's (the documents' dialect).
    fn finish(mut self) -> Result<Entry, String> {
        if let Some(at) = self.appearance_at {
            let first = 6;
            let mut text = String::from("type screen\nversion 2\n---\nname \"a\"\ncolumn A {\n");
            for (_, l) in &self.appearance {
                text.push_str(l);
                text.push('\n');
            }
            text.push_str("}\n");
            let doc = crate::doc::css::lower(&text).map_err(|e| e.join("; "))?;
            let mut block = doc
                .body
                .get("column")
                .and_then(|e| e.block.clone())
                .unwrap_or_default();
            for e in &mut block.entries {
                e.line = self
                    .appearance
                    .get(e.line.saturating_sub(first))
                    .map_or(0, |(n, _)| *n);
            }
            let mut e = Entry::new("appearance");
            e.line = self.appearance.first().map_or(0, |(n, _)| *n);
            e.block = Some(block);
            self.entry
                .block
                .get_or_insert_with(Block::new)
                .entries
                .insert(at, e);
        }
        Ok(self.entry)
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

/// A version 3 style's lines as `.ogt` entries.
fn lower(text: &str) -> Result<Document, String> {
    let head = header(text)?;
    let mut errors: Vec<String> = Vec::new();
    let mut root = Entry::new("");
    root.block = Some(Block::new());
    let mut stack = vec![Frame::new(Ctx::Top, root)];
    let body = text
        .split('\n')
        .enumerate()
        .skip_while(|(_, l)| l.trim() != "---")
        .skip(1);
    for (i, raw) in body {
        let n = i + 1;
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let t = raw.trim();
        let mut err = |m: String| errors.push(format!("line {n}: {m}"));
        let ctx = stack.last().unwrap().ctx;
        let push = |stack: &mut Vec<Frame>, e: Entry| {
            stack
                .last_mut()
                .unwrap()
                .entry
                .block
                .get_or_insert_with(Block::new)
                .entries
                .push(e);
        };
        let (code, _) = css_comment(t);
        if code.is_empty() {
            continue;
        }
        if code == "}" {
            if stack.len() == 1 {
                err("'}' closes nothing".into());
                continue;
            }
            let f = stack.pop().unwrap();
            match f.finish() {
                Ok(e) => push(&mut stack, e),
                Err(m) => err(m),
            }
            continue;
        }
        match ctx {
            Ctx::Top
                if code.contains("::") || code.starts_with(|c: char| c.is_ascii_uppercase()) =>
            {
                let Some(sel) = code.strip_suffix('{').map(str::trim) else {
                    err("a selector opens its block: `Button::base:hover {`".into());
                    continue;
                };
                match selector_ogt(sel) {
                    Ok(s) => {
                        let mut e = Entry::new("dress").with(Value::String(s));
                        e.line = n;
                        e.block = Some(Block::new());
                        stack.push(Frame::new(Ctx::Dress, e));
                    }
                    Err(m) => {
                        err(m);
                        stack.push(Frame::new(Ctx::Ogt, Entry::new("_")));
                    }
                }
            }
            Ctx::Top | Ctx::Ogt if is_decl(code) => {
                err(format!(
                    "`{code}` is a declaration: it belongs in a selector or token block"
                ));
            }
            Ctx::Top | Ctx::Ogt => match ogt::parse_line(code, n) {
                Ok(Line::Entry { entry, opens }) => {
                    if ctx == Ctx::Top && entry.key == "dress" {
                        let hint = entry
                            .values
                            .first()
                            .and_then(Value::as_str)
                            .and_then(|s| Selector::parse(s).ok())
                            .map(|s| format!("`{} {{`", selector_css(&s)))
                            .unwrap_or_else(|| "`Component::part.variant:state {`".into());
                        err(format!(
                            "a dressing is a selector block in version 3: write {hint}"
                        ));
                    }
                    if opens {
                        let c = child_ctx(ctx, &entry.key);
                        stack.push(Frame::new(
                            if c == Ctx::Dress { Ctx::Ogt } else { c },
                            entry,
                        ));
                    } else {
                        push(&mut stack, entry);
                    }
                }
                Ok(_) => {}
                Err(e) => err(e.to_string()),
            },
            Ctx::Token(..) | Ctx::Dress if code.ends_with('{') => {
                err(format!(
                    "a {} block holds declarations, not blocks",
                    if ctx == Ctx::Dress {
                        "dressing"
                    } else {
                        "token"
                    }
                ));
                stack.push(Frame::new(Ctx::Ogt, Entry::new("_")));
            }
            Ctx::Token(..) | Ctx::Dress | Ctx::Appearance => {
                if !is_decl(code) {
                    let w = code.split_whitespace().next().unwrap_or(code);
                    let fix = match ctx {
                        Ctx::Dress => DRESS
                            .iter()
                            .find(|(k, _, _)| *k == w)
                            .map(|(_, c, _)| format!("write `{c}: …`"))
                            .unwrap_or_else(|| {
                                "a dressing holds `name: value` declarations".into()
                            }),
                        _ => format!("a token is a custom property: write `--{}: …`", kebab(w)),
                    };
                    err(format!("`{code}` is not a declaration: {fix}"));
                    continue;
                }
                let decls = match split_decls(code) {
                    Ok(d) => d,
                    Err(m) => {
                        err(m);
                        continue;
                    }
                };
                for (name, value) in decls {
                    let key = match ctx {
                        Ctx::Token(prefix, _) => {
                            match name.strip_prefix("--").and_then(|r| r.strip_prefix(prefix)) {
                                Some(k) if !k.is_empty() => camel(k),
                                _ => {
                                    err(format!(
                                        "`{name}` is not a token of this block: write `--{prefix}<name>: …`"
                                    ));
                                    continue;
                                }
                            }
                        }
                        _ => match DRESS.iter().find(|(_, c, _)| *c == name) {
                            Some((k, _, _)) => k.to_string(),
                            None if appearance_css(&name) => {
                                let f = stack.last_mut().unwrap();
                                if f.appearance_at.is_none() {
                                    f.appearance_at =
                                        Some(f.entry.block.as_ref().map_or(0, |b| b.entries.len()));
                                }
                                f.appearance.push((n, format!("{name}: {value}")));
                                continue;
                            }
                            None => {
                                err(format!(
                                    "`{name}` is not a dressing property ({})",
                                    dress_hint(&name)
                                ));
                                continue;
                            }
                        },
                    };
                    let mut e = Entry::new(key.clone());
                    e.line = n;
                    match ogt_values(&value) {
                        Ok(v)
                            if BARE.contains(&key.as_str())
                                && v == [Value::Word("true".into())] => {}
                        Ok(v) => e.values = v,
                        Err(m) => {
                            err(format!("{name}: {m}"));
                            continue;
                        }
                    }
                    push(&mut stack, e);
                }
            }
        }
    }
    if stack.len() > 1 {
        errors.push("a block is not closed".into());
    }
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let root = stack.pop().unwrap().entry;
    Ok(Document {
        ty: head.ty,
        version: head.version,
        body: root.block.unwrap_or_default(),
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
