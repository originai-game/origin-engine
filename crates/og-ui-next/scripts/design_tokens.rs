//! `packages/design/tokens.json` from the design language's tokens
//! (`packages/design/tokens/*.css`): the same tokens as one tree, for a
//! client that reads JSON rather than CSS (origin-agent pins the package).
//! A token is a custom property in a `:root` block, named by its path:
//! `--motion-duration-fast` is `motion.duration.fast`. A
//! `:root[data-scheme=dark]` block holds what the dark scheme changes.
//! Media-wrapped root tokens stay conditional CSS in `_media`, never
//! unconditional values. The seeds' and the palette's colours are written
//! resolved, from the token table the default pack is generated from
//! (`og_ui_next::Tokens`, pack/design.rs), so the two never disagree.
//!
//! `scripts/gen-tokens.sh` writes the file; `tests/design_tokens.rs` holds
//! it equal to this module's output.

use og_ui_next::Tokens;
use serde_json::{Map, Number, Value};
use std::path::Path;

/// The groups a name may pass through, longest first where one starts
/// another. A name's leading words that spell a group are its path; the
/// rest is the token's own name (`--density-compact-control-sm` is
/// `control-sm` in `density.compact`). A new token in a group listed here
/// needs nothing; a new group is added here.
const GROUPS: &[&str] = &[
    "color",
    "controls",
    "density",
    "density.comfortable",
    "density.compact",
    "density.pad",
    "density.touch",
    "derived",
    "derived.alias",
    "derived.shadow",
    "derived.tint",
    "derived.tint.danger-45",
    "derived.tint.line-70",
    "derived.tint.shadow-30",
    "derived.tint.shadow-35",
    "derived.tint.shadow-45",
    "derived.tint.sheen-50",
    "derived.tint.sheen-60",
    "elevation",
    "elevation.floating",
    "elevation.raised",
    "filter",
    "font",
    "font.cjk",
    "font.cjk.lineHeight",
    "font.leading",
    "font.scale",
    "font.scale.2xl",
    "font.scale.3xl",
    "font.scale.4xl",
    "font.scale.5xl",
    "font.scale.6xl",
    "font.scale.7xl",
    "font.scale.8xl",
    "font.scale.base",
    "font.scale.body",
    "font.scale.caption",
    "font.scale.display-2xl",
    "font.scale.display-lg",
    "font.scale.display-md",
    "font.scale.display-sm",
    "font.scale.display-xl",
    "font.scale.lg",
    "font.scale.meta",
    "font.scale.reading",
    "font.scale.sm",
    "font.scale.xl",
    "font.scale.xs",
    "font.step",
    "font.tracking",
    "font.weight",
    "glass",
    "glassContrast",
    "hue",
    "linear",
    "linear.color",
    "linear.over",
    "materialScope",
    "materialScope.color",
    "materialScope.shadow",
    "measure",
    "metal",
    "metal.bronze",
    "metal.gold",
    "metal.silver",
    "motion",
    "motion.duration",
    "motion.easing",
    "motion.named",
    "motion.named.emphasis",
    "motion.named.enter",
    "motion.named.exit",
    "motion.named.panel",
    "motion.named.playful",
    "motion.named.ui",
    "motion.scale",
    "motion.spring",
    "motion.spring.default",
    "motion.spring.momentum",
    "motion.spring.sheet",
    "motion.timing",
    "motion.timing.cascade",
    "motion.timing.enter",
    "motion.timing.exit",
    "motion.timing.hover",
    "motion.timing.press",
    "motion.timing.screen",
    "motion.timing.toast",
    "opacity",
    "radius",
    "role",
    "safeArea",
    "seed",
    "shadow",
    "size",
    "space",
    "state",
    "stroke",
    "vars",
];

/// Token names the tree spells in camel case.
const CAMEL: &[&str] = &[
    "lineHeight",
    "letterSpacing",
    "fontFeatures",
    "holdDanger",
    "nonText",
];

/// Tokens whose value is a list of space-separated tuples.
const LISTS: &[&str] = &["contrast", "nonText", "glassContrast.pairs"];

/// Tokens that look like numbers and are read as text (a length of zero,
/// a scale a `transform` takes as written).
const TEXT: &[&str] = &["state.focus-ring-offset-field", "vars.press-scale"];

fn kebab(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_uppercase() {
            out.push('-');
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The path of the custom property `name` (without its `--`).
fn path(name: &str) -> Vec<String> {
    let group = GROUPS
        .iter()
        .filter(|g| {
            let spelled = kebab(&g.replace('.', "-"));
            name.strip_prefix(spelled.as_str())
                .is_some_and(|rest| rest.starts_with('-'))
        })
        .max_by_key(|g| g.len());
    let (mut path, rest): (Vec<String>, &str) = match group {
        Some(g) => (
            g.split('.').map(str::to_string).collect(),
            &name[kebab(&g.replace('.', "-")).len() + 1..],
        ),
        None => (Vec::new(), name),
    };
    let leaf = CAMEL
        .iter()
        .find(|c| kebab(c) == rest)
        .map_or(rest.to_string(), |c| c.to_string());
    path.push(leaf);
    path
}

fn value(path: &[String], text: &str) -> Value {
    let dotted = path.join(".");
    if LISTS.contains(&dotted.as_str()) {
        return Value::Array(
            text.split(',')
                .map(|tuple| {
                    Value::Array(
                        tuple
                            .split_whitespace()
                            .map(|word| Value::String(word.to_string()))
                            .collect(),
                    )
                })
                .collect(),
        );
    }
    if !TEXT.contains(&dotted.as_str()) {
        if let Ok(n) = text.parse::<u64>() {
            return Value::Number(n.into());
        }
        if let Some(n) = text.parse::<f64>().ok().and_then(Number::from_f64) {
            return Value::Number(n);
        }
    }
    Value::String(text.to_string())
}

fn insert(tree: &mut Map<String, Value>, path: &[String], leaf: Value) -> Result<(), String> {
    let (last, groups) = path.split_last().expect("a path has a name");
    let mut at = tree;
    for group in groups {
        at = at
            .entry(group.clone())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| format!("--{}: {group} is a token, not a group", path.join("-")))?;
    }
    if at.insert(last.clone(), leaf).is_some() {
        return Err(format!("{} is set twice", path.join(".")));
    }
    Ok(())
}

/// One file's tokens into `tree`.
fn read(file: &str, css: &str, tree: &mut Map<String, Value>) -> Result<String, String> {
    let mut text = String::new();
    let mut rest = css;
    while let Some(open) = rest.find("/*") {
        text.push_str(&rest[..open]);
        let close = rest[open..]
            .find("*/")
            .ok_or_else(|| format!("{file}: a comment is not closed"))?;
        rest = &rest[open + close + 2..];
    }
    text.push_str(rest);

    let mut normalized = String::new();
    let mut rest = text.as_str();
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim();
        let close =
            block_end(&rest[open..]).ok_or_else(|| format!("{file}: {selector} is not closed"))?;
        let body = &rest[open + 1..open + close];
        rest = &rest[open + close + 1..];
        if selector.starts_with("@media ") {
            // Validate the same token grammar without merging conditional
            // values into the unconditional tree. Keep the CSS in source
            // order for a client's media-aware root cascade.
            let body = read(file, body, &mut Map::new())?;
            let block = format!("{selector} {{{body}}}");
            normalized.push_str(&block);
            tree.entry("_media")
                .or_insert_with(|| Value::Array(Vec::new()))
                .as_array_mut()
                .unwrap()
                .push(Value::String(block));
            continue;
        }
        // High contrast's tokens are the default pack's `contrast`
        // (pack/design.rs), not part of this tree.
        if selector.contains("[data-contrast=") {
            continue;
        }
        let dark = match selector {
            ":root" => false,
            ":root[data-scheme=dark]" => true,
            _ => {
                return Err(format!(
                    "{file}: {selector}: tokens are set on :root or :root[data-scheme=dark]"
                ));
            }
        };
        // A conditional common token overrides both schemes' own; a dark
        // override keeps its higher specificity, as in the input.
        let selector = if dark {
            selector
        } else {
            ":root, :root[data-scheme=light]"
        };
        normalized.push_str(&format!("{selector} {{{body}}}\n"));
        for declaration in body.split(';').map(str::trim).filter(|d| !d.is_empty()) {
            let (name, text) = declaration
                .split_once(':')
                .ok_or_else(|| format!("{file}: {declaration}: not a declaration"))?;
            let name = name
                .trim()
                .strip_prefix("--")
                .ok_or_else(|| format!("{file}: {name}: a token is a custom property"))?;
            let mut path = path(name);
            let leaf = value(&path, text.trim());
            if dark {
                // The dark scheme's values sit under `dark`, the linear
                // fills' under `linear.dark`.
                if path[0] == "linear" {
                    path.insert(1, "dark".to_string());
                } else {
                    path.insert(0, "dark".to_string());
                }
            }
            insert(tree, &path, leaf).map_err(|e| format!("{file}: {e}"))?;
        }
    }
    if !rest.trim().is_empty() {
        return Err(format!("{file}: text after the last block"));
    }
    Ok(normalized)
}

// Balance blocks without treating braces or quotes in comments and
// quoted token values as delimiters. `docs` keeps comments for notes.
fn block_end(text: &str) -> Option<usize> {
    let mut depth = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut chars = text.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if quote.is_none() && ch == '/' && chars.peek().is_some_and(|(_, c)| *c == '*') {
            chars.next();
            while let Some((_, ch)) = chars.next() {
                if ch == '*' && chars.peek().is_some_and(|(_, c)| *c == '/') {
                    chars.next();
                    break;
                }
            }
            continue;
        }
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() {
            match ch {
                '\'' | '"' => quote = Some(ch),
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// The tree as `tokens.json`'s text: groups nested, names as the tree
/// spells them, each group's note (`docs`) as its `_doc`.
fn write(mut tree: Map<String, Value>, docs: &[(Vec<String>, String)]) -> Result<String, String> {
    for (path, doc) in docs {
        let mut at = Some(&mut tree);
        for group in path {
            at = at
                .and_then(|t| t.get_mut(group))
                .and_then(Value::as_object_mut);
        }
        if let Some(group) = at {
            group
                .entry("_doc".to_string())
                .or_insert_with(|| Value::String(doc.clone()));
        }
    }
    tree.insert(
        "_doc".to_string(),
        Value::String(
            "Generated from packages/design/tokens/*.css by crates/og-ui-next/scripts/gen-tokens.sh; never edit it by hand."
                .to_string(),
        ),
    );
    let mut text = serde_json::to_string_pretty(&tree).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text)
}

/// The groups' notes in every `.css` file in `dir`: a comment in a block
/// is the note of the deepest group shared by the declarations after it
/// that are in the first one's top-level group (up to the next comment);
/// the first such comment for a group wins. A comment on a token of a
/// top-level group with no subgroups notes nothing.
fn docs(dir: &Path) -> Result<Vec<(Vec<String>, String)>, String> {
    let mut out: Vec<(Vec<String>, String)> = Vec::new();
    for file in css_files(dir)? {
        let css = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let mut rest = css.as_str();
        while let Some(open) = rest.find('{') {
            let selector = strip_comments(&rest[..open]);
            let Some(close) = block_end(&rest[open..]) else {
                break;
            };
            let body = &rest[open + 1..open + close];
            rest = &rest[open + close + 1..];
            if selector.trim().starts_with("@media ") {
                continue;
            }
            let dark = selector.trim() == ":root[data-scheme=dark]";
            let mut note: Option<String> = None;
            let mut paths: Vec<Vec<String>> = Vec::new();
            let mut flush = |note: &mut Option<String>, paths: &mut Vec<Vec<String>>| {
                if let Some(text) = note.take() {
                    let first = paths.first().map(|p| p[0].clone());
                    let shared = paths
                        .iter()
                        .take_while(|p| Some(&p[0]) == first.as_ref())
                        .map(|p| &p[..p.len() - 1])
                        .reduce(|a, b| &a[..a.iter().zip(b).take_while(|(x, y)| x == y).count()])
                        .unwrap_or(&[]);
                    if !shared.is_empty() && !out.iter().any(|(p, _)| p == shared) {
                        out.push((shared.to_vec(), text));
                    }
                }
                paths.clear();
            };
            let mut body = body;
            loop {
                body = body.trim_start();
                if body.is_empty() {
                    break;
                }
                if let Some(after) = body.strip_prefix("/*") {
                    let end = after.find("*/").unwrap_or(after.len());
                    flush(&mut note, &mut paths);
                    note = Some(
                        after[..end]
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" "),
                    );
                    body = after.get(end + 2..).unwrap_or("");
                    continue;
                }
                let end = body.find(';').unwrap_or(body.len());
                if let Some(name) = body[..end]
                    .split_once(':')
                    .and_then(|(n, _)| n.trim().strip_prefix("--"))
                {
                    let mut path = path(name);
                    if dark {
                        if path[0] == "linear" {
                            path.insert(1, "dark".to_string());
                        } else {
                            path.insert(0, "dark".to_string());
                        }
                    }
                    paths.push(path);
                }
                body = body.get(end + 1..).unwrap_or("");
            }
            flush(&mut note, &mut paths);
        }
    }
    Ok(out)
}

fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find("/*") {
        out.push_str(&rest[..open]);
        rest = rest[open..]
            .find("*/")
            .map_or("", |close| &rest[open + close + 2..]);
    }
    out.push_str(rest);
    out
}

fn css_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "css"))
        .collect();
    files.sort();
    Ok(files)
}

/// A resolved colour as `tokens.json` writes it: `#rrggbb`, or
/// `rgb(r g b / a)` when it is translucent.
fn colour(table: &Tokens, name: &str) -> Result<String, String> {
    let text = table
        .get(name)
        .ok_or_else(|| format!("{name} does not resolve (a var() no token sets, or a cycle)"))?;
    let c = og_css::color::parse_str(text)
        .map_err(|e| format!("{name}: `{text}`: {e}"))?
        .literal()
        .ok_or_else(|| format!("{name}: `{text}` is not a colour of its own"))?;
    let [r, g, b] = c.rgb;
    Ok(if c.alpha >= 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("rgb({r} {g} {b} / {})", (c.alpha * 1000.0).round() / 1000.0)
    })
}

/// `tokens.json` from every `.css` file in `dir`, in file-name order. The
/// colours of the seeds and the palette (`seed`, `color`, `linear.color`)
/// are the default seeds' resolved sRGB in each scheme, the dark one under
/// `dark` where the dark block sets it or it resolves apart there (a
/// derivation that reads a token the dark scheme changes); the CSS files
/// keep the derivations.
pub fn generate(dir: &Path) -> Result<String, String> {
    let files = css_files(dir)?;
    if files.is_empty() {
        return Err(format!("{}: no token files", dir.display()));
    }
    let mut tree = Map::new();
    let mut all = String::new();
    for path in files {
        let css = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let file = path.file_name().unwrap_or_default().to_string_lossy();
        read(&file, &css, &mut tree)?;
        all.push_str(&css);
        all.push('\n');
    }
    let tokens = Tokens::parse(&all);
    let (light, dark) = (tokens.scheme("light"), tokens.scheme("dark"));
    for (prefix, base, over) in [
        ("seed", &["seed"][..], &["dark", "seed"][..]),
        ("color", &["color"], &["dark", "color"]),
        (
            "linear-color",
            &["linear", "color"],
            &["linear", "dark", "color"],
        ),
    ] {
        let Some(Value::Object(lights)) = group(&mut tree, base).map(std::mem::take) else {
            return Err(format!("the tokens have no {}", base.join(".")));
        };
        let own = match group(&mut tree, over).map(std::mem::take) {
            Some(Value::Object(own)) => own,
            _ => Map::new(),
        };
        let mut darks = Map::new();
        let mut resolved = Map::new();
        for (key, value) in lights {
            if !value.is_string() {
                resolved.insert(key, value);
                continue;
            }
            let name = format!("--{prefix}-{}", kebab(&key));
            let (l, d) = (colour(&light, &name)?, colour(&dark, &name)?);
            if d != l || own.contains_key(&key) {
                darks.insert(key.clone(), Value::String(d));
            }
            resolved.insert(key, Value::String(l));
        }
        *group(&mut tree, base).expect("read above") = Value::Object(resolved);
        set(&mut tree, over, Value::Object(darks));
    }
    write(tree, &docs(dir)?)
}

/// The value at `path`, when every group on the way is there.
fn group<'a>(tree: &'a mut Map<String, Value>, path: &[&str]) -> Option<&'a mut Value> {
    let (last, groups) = path.split_last()?;
    let mut at = tree;
    for g in groups {
        at = at.get_mut(*g)?.as_object_mut()?;
    }
    at.get_mut(*last)
}

/// Sets the value at `path`, making the groups on the way.
fn set(tree: &mut Map<String, Value>, path: &[&str], value: Value) {
    let (last, groups) = path.split_last().expect("a path");
    let mut at = tree;
    for g in groups {
        at = at
            .entry(g.to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("a group");
    }
    at.insert(last.to_string(), value);
}
