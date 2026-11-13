//! Reads the design language's tokens (`packages/design/tokens/*.css`)
//! into the tree the generators walk. A token is a custom property in a
//! `:root` block, named by its path: `--motion-duration-fast` is
//! `motion.duration.fast`. A `:root[data-scheme=dark]` block holds what
//! the dark scheme changes.

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
fn read(file: &str, css: &str, tree: &mut Map<String, Value>) -> Result<(), String> {
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

    let mut rest = text.as_str();
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim();
        let close = rest[open..]
            .find('}')
            .ok_or_else(|| format!("{file}: {selector} is not closed"))?;
        let body = &rest[open + 1..open + close];
        rest = &rest[open + close + 1..];
        let dark = match selector {
            ":root" => false,
            ":root[data-scheme=dark]" => true,
            _ => {
                return Err(format!(
                    "{file}: {selector}: tokens are set on :root or :root[data-scheme=dark]"
                ));
            }
        };
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
    Ok(())
}

/// `tokens.json`, the same tokens as one tree for a client that reads
/// JSON (origin-agent): groups nested, names as the tree spells them, each
/// group's note (`docs`) as its `_doc`.
pub fn json(tokens: &Value, docs: &[(Vec<String>, String)]) -> Result<String, String> {
    let mut tree = tokens.as_object().cloned().unwrap_or_default();
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
            "Generated from packages/design/tokens/*.css by crates/og-ui/scripts/gen-theme.sh; never edit it by hand."
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
pub fn docs(dir: &Path) -> Result<Vec<(Vec<String>, String)>, String> {
    let mut out: Vec<(Vec<String>, String)> = Vec::new();
    for file in css_files(dir)? {
        let css = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let mut rest = css.as_str();
        while let Some(open) = rest.find('{') {
            let selector = strip_comments(&rest[..open]);
            let Some(close) = rest[open..].find('}') else {
                break;
            };
            let body = &rest[open + 1..open + close];
            rest = &rest[open + close + 1..];
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

/// The tokens of every `.css` file in `dir`, in file-name order.
pub fn load(dir: &Path) -> Result<Value, String> {
    let files = css_files(dir)?;
    if files.is_empty() {
        return Err(format!("{}: no token files", dir.display()));
    }
    let mut tree = Map::new();
    for path in files {
        let css = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let file = path.file_name().unwrap_or_default().to_string_lossy();
        read(&file, &css, &mut tree)?;
    }
    Ok(Value::Object(tree))
}
