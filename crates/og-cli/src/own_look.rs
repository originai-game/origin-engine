//! The game's own look (docs/ui.md §12.7, the owner, 2026-11-05): the UI
//! library, the kits and the style packs are a polished starting point, and
//! a game is not finished by shipping them as they are. Their mechanics
//! (behaviour, layout rules, input, localization, pad navigation) are
//! reused; the look is the game's. This module finds what of the templates
//! a game still ships unchanged:
//!
//! - `own-style`: a look or code that draws in a built-in pack
//!   (`ui "pixel"`, `@styles/pixel/style.ogt`), a style of the game's
//!   identical to a built-in pack, or one still as `og new` generated it;
//! - `eject-unchanged`: a screen or component document copied from the
//!   library or a kit (`og ui eject`) and left as shipped;
//! - `fallback-icons`: the development fallback icons
//!   (`development-fallback-icons`, an icons.json with
//!   `developmentFallback: true`, or a copy of one of its vectors);
//! - `art-slot`: an `art` element still drawing the style's placeholder.
//!
//! `og ui check` and `og polish` report each as a warning with its path and
//! what to change; `og build --publish` refuses the game while any is left.
//!
//! "Unchanged" is decided by content, never by time: a document's canonical
//! text (no comments, blank lines or trailing white space) against the
//! engine's shipped version and against the fingerprint the copy recorded
//! when it was made (`# og:ejected from ..., fingerprint <hex>:`); a
//! style's fingerprint (`style_fingerprint`, shared with packages/create's
//! own-style.ts) against every built-in pack's and the one `og new` wrote
//! (`# og:generated from ..., fingerprint <hex>:`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One thing of the templates the game ships as it was shipped.
#[derive(Debug)]
pub struct Finding {
    pub rule: &'static str,
    /// Project-relative (`assets/styles/my-game/style.ogt`, `src/main.ts:12`).
    pub path: String,
    pub measured: String,
    /// What to change, where.
    pub change: String,
    pub fix: String,
}

/// FNV-1a 64 in lowercase hex (16 digits): og_core's stable hash.
pub fn fnv_hex(bytes: &[u8]) -> String {
    format!("{:016x}", og_core::hash::hash_bytes(bytes))
}

/// An .ogt's canonical text: every line with its trailing white space cut,
/// without blank lines and `#` comment lines, joined by "\n"; with
/// `drop_name`, without the top-level `name` line (a renamed copy of a
/// style is the same style).
pub fn canonical(text: &str, drop_name: bool) -> String {
    text.lines()
        .map(str::trim_end)
        .filter(|l| {
            let t = l.trim_start();
            !t.is_empty()
                && !t.starts_with('#')
                && !(drop_name && (l.starts_with("name ") || l.starts_with("name\t")))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The quoted strings of `text`, in order.
fn strings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut s = String::new();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    s.push(c);
                    if let Some(n) = chars.next() {
                        s.push(n);
                    }
                }
                '"' => break,
                c => s.push(c),
            }
        }
        out.push(s);
    }
    out
}

const FILE_EXTS: [&str; 11] = [
    "png", "jpg", "jpeg", "webp", "ttf", "otf", "ogt", "wav", "ogg", "mp3", "svg",
];

/// A style directory's fingerprint: FNV-1a 64 of its canonical style.ogt
/// (without its name) followed, for every quoted string in it that names a
/// file (and that file's `@2x` and `@3x`), in order of first mention, by
/// `"\n<string> <FNV-1a 64 of the file's bytes>"`. A string names a file
/// relative to the directory, or, starting `styles/`, an asset id under
/// the assets directory two levels up; `@` ids and missing files add
/// nothing. None without a style.ogt. packages/create/bin/own-style.ts
/// computes the same (`styleFingerprint`; the tests hold one vector equal).
pub fn style_fingerprint(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("style.ogt")).ok()?;
    let text = canonical(&text, true);
    let mut out = text.clone();
    let mut seen = std::collections::HashSet::new();
    for s in strings(&text) {
        let Some((stem, ext)) = s.rsplit_once('.') else {
            continue;
        };
        if s.starts_with('@') || !FILE_EXTS.iter().any(|e| ext.eq_ignore_ascii_case(e)) {
            continue;
        }
        for rel in [
            s.clone(),
            format!("{stem}@2x.{ext}"),
            format!("{stem}@3x.{ext}"),
        ] {
            if !seen.insert(rel.clone()) {
                continue;
            }
            let mut candidates = vec![dir.join(&rel)];
            if rel.starts_with("styles/") {
                candidates.push(dir.join("../..").join(&rel));
            }
            if let Some(bytes) = candidates
                .iter()
                .find(|p| p.is_file())
                .and_then(|p| std::fs::read(p).ok())
            {
                out.push_str(&format!("\n{rel} {}", fnv_hex(&bytes)));
            }
        }
    }
    Some(fnv_hex(out.as_bytes()))
}

/// The provenance a copy recorded: `# og:<verb> from <what>, fingerprint <hex>:`.
fn recorded(text: &str, verb: &str) -> Option<(String, String)> {
    let head = format!("# og:{verb} from ");
    let line = text.lines().find(|l| l.starts_with(&head))?;
    let rest = &line[head.len()..];
    let (from, after) = rest.split_once(", fingerprint ")?;
    let fp: String = after.chars().take_while(char::is_ascii_hexdigit).collect();
    (fp.len() == 16).then(|| (from.to_string(), fp))
}

/// The provenance line `og ui eject` writes after a copied document's `---`.
pub fn ejected_line(from: &str, text: &str) -> String {
    format!(
        "# og:ejected from {from}, fingerprint {}: a starting point to change for the game. og ui check reports it and og build --publish refuses it until it changes (docs/ui.md §11).",
        fnv_hex(canonical(text, false).as_bytes())
    )
}

/// `text` with `line` after its `---` (a copied document's provenance).
pub fn with_provenance(text: &str, line: &str) -> String {
    match text.find("\n---\n") {
        Some(i) => format!("{}\n---\n{line}\n{}", &text[..i], &text[i + 5..]),
        None => text.to_string(),
    }
}

/// Every file under `dir` whose name ends in `.ext`, sorted; directories in
/// `skip` (absolute) are left out.
fn walk(dir: &Path, ext: &str, skip: &[PathBuf], out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = read.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if !skip.contains(&p) && p.file_name().is_some_and(|n| n != "node_modules") {
                walk(&p, ext, skip, out);
            }
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The first line's type of an .ogt (`type screen` -> `screen`).
fn ogt_type(text: &str) -> &str {
    text.lines()
        .next()
        .and_then(|l| l.strip_prefix("type "))
        .unwrap_or("")
        .trim()
}

/// The built-in packs by fingerprint.
fn builtin_styles() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(dir) = og_cook::cook::look::styles_dir() else {
        return out;
    };
    let Ok(read) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in read.flatten() {
        if let Some(fp) = style_fingerprint(&e.path()) {
            out.insert(fp, e.file_name().to_string_lossy().into_owned());
        }
    }
    out
}

/// The library's and the bundled kits' screen and component documents, by
/// canonical fingerprint: what `og ui eject` copies, as this engine ships it.
fn shipped_documents(root: &Path, package: Option<&Path>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(package) = package {
        let mut files = Vec::new();
        walk(&package.join("src/ui/screens"), "ogt", &[], &mut files);
        for f in files {
            if let Ok(text) = std::fs::read_to_string(&f) {
                let name = f.file_stem().unwrap_or_default().to_string_lossy();
                out.insert(
                    fnv_hex(canonical(&text, false).as_bytes()),
                    format!("the library's {name} screen"),
                );
            }
        }
    }
    if let Ok(all) = crate::ui::extensions::discover(root) {
        for e in all
            .iter()
            .filter(|e| e.manifest.kind == "kit" && !e.installed)
        {
            for doc in &e.documents {
                if let Ok(text) = std::fs::read_to_string(e.directory.join(doc)) {
                    out.insert(
                        fnv_hex(canonical(&text, false).as_bytes()),
                        format!("the {} kit's {doc}", e.manifest.name),
                    );
                }
            }
        }
    }
    out
}

/// The development fallback icons' vectors as shipped, by canonical fingerprint.
fn fallback_vectors(package: Option<&Path>) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    if let Some(package) = package {
        let mut files = Vec::new();
        walk(
            &package.join("extensions").join(FALLBACK_KIT),
            "ogt",
            &[],
            &mut files,
        );
        for f in files {
            if let Ok(text) = std::fs::read_to_string(&f)
                && ogt_type(&text) == "vector"
            {
                out.insert(fnv_hex(canonical(&text, false).as_bytes()));
            }
        }
    }
    out
}

const FALLBACK_KIT: &str = "development-fallback-icons";

const RESTYLE: &str = "palette { } first (the primary and its states, the surfaces, the ink, from the look's art), then the frames and buttons (art/, repacked with og style art, or the part dressings), the fonts, the icons, the motion and the sounds";

/// Every template the game at `root` still ships unchanged. `package` is
/// the engine package (`project::engine_package`): the library's screens
/// and the fallback icons are read from it.
pub fn findings(root: &Path, package: Option<&Path>) -> Vec<Finding> {
    let mut out = Vec::new();
    let assets = root.join("assets");
    let builtins = builtin_styles();

    // Looks that draw in a built-in pack (their own `ui` line or their preset's).
    if let Ok(bundle) = og_cook::runtime::Bundle::open(&root.join("cooked")) {
        for o in bundle
            .manifest()
            .outputs()
            .filter(|o| o.ty == "og.Look" && !o.id.starts_with('@'))
        {
            let Ok(look) = bundle
                .read(&o.id)
                .map_err(|e| e.to_string())
                .and_then(|b| bundle.look(&o.id, &b).map_err(|e| e.to_string()))
            else {
                continue;
            };
            let Some(slug) = look
                .ui
                .as_deref()
                .and_then(|u| u.strip_prefix("@styles/"))
                .and_then(|u| u.strip_suffix("/style.ogt"))
            else {
                continue;
            };
            out.push(Finding {
                rule: "own-style",
                path: format!("assets/{}", o.id),
                measured: format!(
                    "the look draws its UI in the built-in {slug} pack (its `ui` line, or the preset's it extends)"
                ),
                change: format!(
                    "assets/{}: `ui \"styles/<name>/style.ogt\"`, a style of the game's own",
                    o.id
                ),
                fix: format!(
                    "copy the pack into the game (`og style eject {slug} <name>`), point the look's `ui` line at it and restyle it: {RESTYLE}"
                ),
            });
        }
    }

    // Code that loads a built-in pack.
    let mut code = Vec::new();
    for ext in ["ts", "tsx", "js", "mjs"] {
        walk(&root.join("src"), ext, &[], &mut code);
    }
    code.sort();
    for f in &code {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            let Some(at) = line
                .find("og.load.style(")
                .and_then(|i| line[i..].find("@styles/").map(|j| i + j + 8))
            else {
                continue;
            };
            let slug: String = line[at..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            let pack = if slug.is_empty() || !line[at + slug.len()..].starts_with('/') {
                "a built-in pack".to_string()
            } else {
                format!("the built-in {slug} pack")
            };
            out.push(Finding {
                rule: "own-style",
                path: format!("{}:{}", rel(root, f), n + 1),
                measured: format!("the code loads {pack} (`@styles/...`)"),
                change: format!(
                    "{}:{}: og.load.style(world, \"styles/<name>/style.ogt\"), a style of the game's own",
                    rel(root, f),
                    n + 1
                ),
                fix: format!(
                    "copy the pack into the game (`og style eject <pack> <name>`), load the copy and restyle it: {RESTYLE}"
                ),
            });
        }
    }

    // The game's styles: identical to a built-in pack, or still as generated.
    let styles = assets.join("styles");
    if let Ok(read) = std::fs::read_dir(&styles) {
        let mut dirs: Vec<PathBuf> = read.flatten().map(|e| e.path()).collect();
        dirs.sort();
        for dir in dirs {
            let Some(fp) = style_fingerprint(&dir) else {
                continue;
            };
            let file = format!("{}/style.ogt", rel(root, &dir));
            let text = std::fs::read_to_string(dir.join("style.ogt")).unwrap_or_default();
            let measured = if let Some(slug) = builtins.get(&fp) {
                format!(
                    "identical to the built-in {slug} pack: its style.ogt (without its name and comments) and every file it names are the pack's"
                )
            } else if let Some((from, was)) = recorded(&text, "generated")
                && was == fp
            {
                format!(
                    "unchanged since og new generated it from {from} (fingerprint {fp}): a recoloured copy, not yet the game's look"
                )
            } else {
                continue;
            };
            out.push(Finding {
                rule: "own-style",
                path: file.clone(),
                measured,
                change: format!("{file}: {RESTYLE}"),
                fix: "restyle it for the game's look: comments, blank lines and the name do not count; a changed palette, part, font, icon or painted frame does (docs/ui.md §12.7)".into(),
            });
        }
    }

    // Ejected documents left as shipped.
    let shipped = shipped_documents(root, package);
    let kits = assets.join("ui-kits");
    let mut docs = Vec::new();
    walk(&assets, "ogt", &[kits.clone(), styles.clone()], &mut docs);
    for f in &docs {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        if !matches!(ogt_type(&text), "screen" | "component") {
            continue;
        }
        let fp = fnv_hex(canonical(&text, false).as_bytes());
        let from = recorded(&text, "ejected")
            .filter(|(_, was)| *was == fp)
            .map(|(from, _)| from)
            .or_else(|| shipped.get(&fp).cloned());
        if let Some(from) = from {
            let path = rel(root, f);
            out.push(Finding {
                rule: "eject-unchanged",
                path: path.clone(),
                measured: format!(
                    "{from} as shipped (fingerprint {fp}): only comments, blank lines or trailing spaces differ"
                ),
                change: format!(
                    "{path}: its layout, hierarchy, art slots and text, for this game"
                ),
                fix: "change the screen for the game (the components keep their behaviour, input and localization), or delete it and build the screen in code".into(),
            });
        }
    }

    // The development fallback icons.
    let kit = kits.join(FALLBACK_KIT);
    let fallback_fix = format!(
        "draw the game's own icons (its style's `icons` atlas, or vectors of its own made with the art brief's icon kind), drop fallbackIcons from og.ui.configure and delete assets/ui-kits/{FALLBACK_KIT}"
    );
    if kit.is_dir() {
        out.push(Finding {
            rule: "fallback-icons",
            path: rel(root, &kit),
            measured: "the development fallback icons are installed (development-only vectors, not the game's)".into(),
            change: "the game's icon set: its style's `icons`, og.ui.configure without fallbackIcons".into(),
            fix: fallback_fix.clone(),
        });
    }
    let mut jsons = Vec::new();
    walk(&assets, "json", std::slice::from_ref(&kit), &mut jsons);
    for f in &jsons {
        let fallback = f.file_name().is_some_and(|n| n == "icons.json")
            && std::fs::read(f)
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                .is_some_and(|v| v["developmentFallback"] == serde_json::Value::Bool(true));
        if fallback {
            out.push(Finding {
                rule: "fallback-icons",
                path: rel(root, f),
                measured: "an icon set marked developmentFallback: true (the development fallback icons, moved)".into(),
                change: "the game's icon set: its style's `icons`, og.ui.configure without fallbackIcons".into(),
                fix: fallback_fix.clone(),
            });
        }
    }
    let vectors = fallback_vectors(package);
    if !vectors.is_empty() {
        let mut ogts = Vec::new();
        walk(&assets, "ogt", std::slice::from_ref(&kit), &mut ogts);
        let copied: Vec<String> = ogts
            .iter()
            .filter(|f| {
                std::fs::read_to_string(f).is_ok_and(|t| {
                    ogt_type(&t) == "vector"
                        && vectors.contains(&fnv_hex(canonical(&t, false).as_bytes()))
                })
            })
            .map(|f| rel(root, f))
            .collect();
        if let Some(first) = copied.first() {
            out.push(Finding {
                rule: "fallback-icons",
                path: first.clone(),
                measured: format!(
                    "{} vector icon(s) copied from the development fallback icons: {}{}",
                    copied.len(),
                    copied
                        .iter()
                        .take(4)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", "),
                    if copied.len() > 4 { ", …" } else { "" }
                ),
                change: "those icons: the game's own art".into(),
                fix: fallback_fix,
            });
        }
    }

    // Art slots still on the style's placeholder.
    let mut ogts = Vec::new();
    walk(&assets, "ogt", std::slice::from_ref(&styles), &mut ogts);
    for f in &ogts {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        if !matches!(ogt_type(&text), "screen" | "component") {
            continue;
        }
        let id = rel(&assets, f);
        let Ok(doc) = og_ui::UiDocument::parse(&id, &text) else {
            continue;
        };
        for s in og_ui::slot::document_slots(&doc) {
            if s.textured {
                continue;
            }
            out.push(Finding {
                rule: "art-slot",
                path: format!("assets/{id}:{}", s.line),
                measured: format!(
                    "the art slot {} ({}) still draws the style's placeholder",
                    s.path,
                    s.spec.kind.name()
                ),
                change: format!("{}: its `texture`", s.path),
                fix: "make its art in the game's look (og ui slots --json gives the slot's kind, aspect and focal point to the art brief) and set the element's texture".into(),
            });
        }
    }
    out
}

/// The findings as og ui check's (ui-check.ts `UiFinding`): warnings of
/// the project, not of a frame.
pub fn to_json(findings: &[Finding]) -> serde_json::Value {
    serde_json::Value::Array(
        findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "rule": f.rule,
                    "severity": "warning",
                    "paths": [f.path],
                    "measured": f.measured,
                    "limit": "the game's own look: og build --publish refuses the templates as shipped (docs/ui.md §12.7)",
                    "prop": f.change,
                    "fix": f.fix,
                    "cells": ["project files"],
                })
            })
            .collect(),
    )
}

/// `og build --publish`'s refusal of a game that still ships templates as
/// they were shipped, each named with its path and what to change.
pub fn refuse(root: &Path, package: Option<&Path>) -> Result<(), String> {
    let found = findings(root, package);
    if found.is_empty() {
        return Ok(());
    }
    let mut msg = format!(
        "the game still ships {} template part(s) as the engine shipped them; the kit and the packs are a starting point, and a published game's look is its own (docs/ui.md §12.7):",
        found.len()
    );
    for f in &found {
        msg.push_str(&format!(
            "\n  {}: {} ({})\n    change {}",
            f.path, f.measured, f.rule, f.change
        ));
    }
    msg.push_str("\nog ui check lists the same during development, with the fix for each.");
    Err(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("og-own-look-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("project.ogt"),
            "type project\nversion 1\n---\nname \"G\"\n",
        )
        .unwrap();
        dir
    }

    /// The vector packages/create's own-style test holds too: the two
    /// fingerprints are one function.
    #[test]
    fn fingerprint_matches_the_create_package() {
        assert_eq!(fnv_hex(b"a"), "af63dc4c8601ec8c");
        let dir = scratch("vector").join("assets/styles/x");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("style.ogt"),
            "type style\nversion 2\n---\n# a comment\nname \"x\"\natlas \"atlas.png\" 4 4  \n\npalette {\n  ink \"#ffffff\"\n}\nicons \"@styles/pixel/icons.png\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("atlas.png"), [1u8, 2, 3]).unwrap();
        std::fs::write(dir.join("atlas@2x.png"), [4u8]).unwrap();
        assert_eq!(style_fingerprint(&dir).unwrap(), "3cee0326d15ccf5c");
    }

    #[test]
    fn a_renamed_copy_of_a_pack_is_identical_and_any_restyle_is_not() {
        let root = scratch("identical");
        let builtin = og_cook::cook::look::styles_dir().unwrap().join("pixel");
        let mine = root.join("assets/styles/my-game");
        fn copy(from: &Path, to: &Path) {
            std::fs::create_dir_all(to).unwrap();
            for e in std::fs::read_dir(from).unwrap().flatten() {
                if e.path().is_dir() {
                    copy(&e.path(), &to.join(e.file_name()));
                } else {
                    std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
                }
            }
        }
        copy(&builtin, &mine);
        let text = std::fs::read_to_string(mine.join("style.ogt")).unwrap();
        let renamed = text.replacen("name \"pixel\"", "name \"my-game\"", 1);
        std::fs::write(
            mine.join("style.ogt"),
            format!("# my own notes\n{renamed}\n\n"),
        )
        .unwrap();
        let f = findings(&root, None);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, "own-style");
        assert!(
            f[0].measured
                .contains("identical to the built-in pixel pack"),
            "{f:?}"
        );
        // One painted frame changed: the look changed.
        let atlas = mine.join("atlas.png");
        let mut bytes = std::fs::read(&atlas).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        std::fs::write(&atlas, &bytes).unwrap();
        assert!(findings(&root, None).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_generated_style_is_unchanged_until_its_content_changes() {
        let root = scratch("generated");
        let mine = root.join("assets/styles/g");
        std::fs::create_dir_all(&mine).unwrap();
        let body = "type style\nversion 2\n---\nname \"g\"\nscheme dark\npalette {\n  primary \"#123456\"\n}\n";
        std::fs::write(mine.join("style.ogt"), body).unwrap();
        let fp = style_fingerprint(&mine).unwrap();
        let generated = body.replacen(
            "---\n",
            &format!("---\n# og:generated from the built-in pixel pack, fingerprint {fp}: notes\n"),
            1,
        );
        std::fs::write(mine.join("style.ogt"), &generated).unwrap();
        let f = findings(&root, None);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].measured
                .contains("unchanged since og new generated it from the built-in pixel pack"),
            "{f:?}"
        );
        // A touched file with the same content is still unchanged: content, not time.
        std::fs::write(
            mine.join("style.ogt"),
            format!("{generated}\n# tuned later\n"),
        )
        .unwrap();
        assert_eq!(findings(&root, None).len(), 1);
        std::fs::write(
            mine.join("style.ogt"),
            generated.replace("#123456", "#123457"),
        )
        .unwrap();
        assert!(findings(&root, None).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_ejected_document_is_unchanged_by_its_recorded_fingerprint() {
        let root = scratch("ejected");
        let ui = root.join("assets/ui");
        std::fs::create_dir_all(&ui).unwrap();
        let shipped = "type screen\nversion 1\n---\nname \"pause\"\ncolumn \"Root\" {\n}\n";
        let copy = with_provenance(
            shipped,
            &ejected_line("the library's pause screen", shipped),
        );
        assert!(copy.contains("\n---\n# og:ejected from the library's pause screen, fingerprint "));
        std::fs::write(ui.join("pause.ogt"), format!("{copy}\n# a note\n")).unwrap();
        let f = findings(&root, None);
        assert_eq!(
            f.iter().filter(|f| f.rule == "eject-unchanged").count(),
            1,
            "{f:?}"
        );
        assert!(
            f[0].measured
                .starts_with("the library's pause screen as shipped"),
            "{f:?}"
        );
        std::fs::write(ui.join("pause.ogt"), copy.replace("\"Root\"", "\"Pause\"")).unwrap();
        assert!(
            findings(&root, None)
                .iter()
                .all(|f| f.rule != "eject-unchanged")
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn fallback_icons_are_found_installed_marked_or_copied() {
        let root = scratch("icons");
        let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine");
        let kit = root.join("assets/ui-kits").join(FALLBACK_KIT);
        std::fs::create_dir_all(&kit).unwrap();
        let rules = |f: &[Finding]| f.iter().filter(|f| f.rule == "fallback-icons").count();
        assert_eq!(rules(&findings(&root, Some(&package))), 1);
        std::fs::remove_dir_all(&kit).unwrap();
        // A vector of the set, moved and renamed.
        let src = package
            .join("extensions")
            .join(FALLBACK_KIT)
            .join("vectors/items/key.ogt");
        let to = root.join("assets/icons/door.ogt");
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&src, &to).unwrap();
        let f = findings(&root, Some(&package));
        assert_eq!(rules(&f), 1, "{f:?}");
        assert!(f.iter().any(|f| f.path == "assets/icons/door.ogt"), "{f:?}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
