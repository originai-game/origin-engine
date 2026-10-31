//! `og api find <words>` and `og api show <name>`: the API reference one
//! entry at a time. The wiki pages `og api gen` writes (the knowledge
//! package's `wiki/`, shipped in @origingame/engine) give every function,
//! component, type, member, constant and value an anchor (`<a id="kind-…">`);
//! an entry is the anchor's block (to the next anchor or `## ` section) or,
//! for a table row (constants, record columns), the row. `find` ranks entries
//! by their name and first sentence; `show` prints one entry without its IDL
//! source, so an agent reads a few kilobytes instead of a 400 KB page.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The documented name, as the heading or row writes it (`og.CharacterController`, `Camera.lookAt`, `BODY_STATIC`).
    pub name: String,
    /// The anchor's kind: fn, record, type, member, const, value or column.
    pub kind: String,
    /// The wiki page, without `.md` (the module).
    pub page: String,
    /// The first sentence of its doc.
    pub summary: String,
    /// The entry's text as the page writes it.
    pub body: String,
}

/// Every anchored entry of one wiki page.
pub fn entries(page: &str, text: &str) -> Vec<Entry> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    // The page's module overview (`# The `input` module` to its contents): `og api show input`.
    if let Some(h) = lines
        .iter()
        .position(|l| l.starts_with("# The `") && l.ends_with("` module"))
    {
        let end = lines[h..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map_or(lines.len(), |n| h + n);
        let summary = lines[h + 1..end]
            .iter()
            .find(|l| !l.trim().is_empty())
            .map(|l| first_sentence(l))
            .unwrap_or_default();
        out.push(Entry {
            name: between(lines[h], '`', '`').unwrap_or_default().to_string(),
            kind: "module".to_string(),
            page: page.to_string(),
            summary,
            body: lines[h..end].join("\n").trim().to_string(),
        });
    }
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some((kind, rest)) = row_anchor(line) {
            let name = between(rest, '`', '`').unwrap_or_default().to_string();
            let summary = line
                .trim_end()
                .trim_end_matches('|')
                .rsplit('|')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            out.push(Entry {
                name,
                kind,
                page: page.to_string(),
                summary: first_sentence(&summary),
                body: line.to_string(),
            });
            i += 1;
            continue;
        }
        if let Some(kind) = block_anchor(line) {
            let mut j = i + 1;
            while j < lines.len()
                && block_anchor(lines[j]).is_none()
                && !lines[j].starts_with("## ")
            {
                j += 1;
            }
            let block = &lines[i + 1..j];
            let heading = block
                .iter()
                .find(|l| l.starts_with('#'))
                .copied()
                .unwrap_or("");
            let name = between(heading, '`', '`').unwrap_or_default().to_string();
            let summary = block
                .iter()
                .skip_while(|l| !l.starts_with('#'))
                .skip(1)
                .find(|l| !l.trim().is_empty())
                .map(|l| first_sentence(l))
                .unwrap_or_default();
            out.push(Entry {
                name,
                kind,
                page: page.to_string(),
                summary,
                body: block.join("\n").trim().to_string(),
            });
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

fn block_anchor(line: &str) -> Option<String> {
    let rest = line.strip_prefix("<a id=\"")?;
    Some(rest.split('-').next()?.to_string())
}

fn row_anchor(line: &str) -> Option<(String, &str)> {
    let rest = line.strip_prefix("| <a id=\"")?;
    let kind = rest.split('-').next()?.to_string();
    let after = rest.split_once("</a>")?.1;
    Some((kind, after))
}

fn between(s: &str, open: char, close: char) -> Option<&str> {
    let start = s.find(open)? + open.len_utf8();
    let end = s[start..].find(close)? + start;
    Some(&s[start..end])
}

fn first_sentence(s: &str) -> String {
    let s = s.trim();
    let end = s
        .match_indices(". ")
        .map(|(i, _)| i + 1)
        .find(|&i| i >= 20)
        .unwrap_or(s.len());
    let mut out = s[..end].to_string();
    if out.chars().count() > 200 {
        out = out.chars().take(199).collect::<String>() + "…";
    }
    out
}

/// The words of a name or text, lowercased, camelCase and snake_case split.
fn words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() && prev_lower && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            prev_lower = c.is_lowercase() || c.is_ascii_digit();
            cur.extend(c.to_lowercase());
        } else {
            prev_lower = false;
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn bare(name: &str) -> String {
    name.trim_start_matches("og.").to_lowercase()
}

/// The entries matching `query`, best first: an exact name, then entries
/// carrying every word (in the name, then the page or first sentence, then
/// anywhere in their text: field and parameter docs), then, when none carries
/// every word, those carrying the most of them.
pub fn find<'a>(all: &'a [Entry], query: &str, kind: Option<&str>) -> Vec<&'a Entry> {
    let every = rank(all, query, kind, true);
    if every.is_empty() {
        rank(all, query, kind, false)
    } else {
        every
    }
}

fn rank<'a>(all: &'a [Entry], query: &str, kind: Option<&str>, every: bool) -> Vec<&'a Entry> {
    let q = query.trim();
    let qwords = words(q);
    if qwords.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(i64, &Entry)> = all
        .iter()
        .filter(|e| kind.is_none_or(|k| e.kind == k))
        .filter_map(|e| {
            let name = bare(&e.name);
            let nwords = words(&e.name);
            let swords = words(&e.summary);
            let body = e.body.to_lowercase();
            let mut score = 0i64;
            let mut matched = 0;
            if name == bare(q) || name.rsplit('.').next() == Some(&bare(q)) {
                score += 1000;
            }
            for w in &qwords {
                let hit = if nwords.iter().any(|n| n == w) {
                    40
                } else if nwords.iter().any(|n| n.starts_with(w.as_str()))
                    || name.contains(w.as_str())
                {
                    25
                } else if e.page == *w {
                    10
                } else if swords.iter().any(|s| s == w || s.starts_with(w.as_str())) {
                    6
                } else if w.len() > 2 && body.contains(w.as_str()) {
                    2
                } else if every {
                    return None;
                } else {
                    0
                };
                matched += usize::from(hit > 0);
                score += hit;
            }
            if matched == 0 {
                return None;
            }
            score += 100 * matched as i64;
            // Components and functions before the members and columns that repeat their names.
            score += match e.kind.as_str() {
                "record" | "fn" => 3,
                "type" | "value" | "const" => 2,
                _ => 0,
            };
            Some((score - nwords.len() as i64, e))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));
    scored.into_iter().map(|(_, e)| e).collect()
}

/// The entries named `name` (with or without `og.`, any case); a module's
/// own name as written (`input`, not `Input` or `og.Input`) is its overview.
pub fn show<'a>(all: &'a [Entry], name: &str) -> Vec<&'a Entry> {
    let module: Vec<&Entry> = all
        .iter()
        .filter(|e| e.kind == "module" && e.name == name.trim())
        .collect();
    if !module.is_empty() {
        return module;
    }
    let want = bare(name);
    let all: Vec<&Entry> = all.iter().filter(|e| e.kind != "module").collect();
    let exact: Vec<&Entry> = all
        .iter()
        .copied()
        .filter(|e| bare(&e.name) == want)
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    // `CharacterController` names `og.CharacterController`; `raycast` names `og.physics.raycast`.
    all.into_iter()
        .filter(|e| bare(&e.name).rsplit('.').next() == Some(want.as_str()))
        .collect()
}

/// An entry as `show` prints it: the page text without the IDL source block
/// and the binding line, which only the engine's own tools read.
pub fn render(e: &Entry) -> String {
    let mut out = String::new();
    let mut skip = false;
    for line in e.body.lines() {
        if line.starts_with("```ogt") {
            skip = true;
            continue;
        }
        if skip {
            if line.starts_with("```") {
                skip = false;
            }
            continue;
        }
        if line.starts_with("Binding symbol ") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    let text = out.trim_end().to_string();
    format!("{text}\n\n({} · {}; page {}.md)\n", e.name, e.kind, e.page)
}

/// The wiki directory: `OG_WIKI_DIR`, the engine package's `knowledge/wiki`,
/// or `packages/knowledge/wiki` of the engine checkout.
fn wiki_dir() -> Option<PathBuf> {
    if let Ok(d) = std::env::var("OG_WIKI_DIR") {
        return Some(PathBuf::from(d));
    }
    let cwd = std::env::current_dir().ok();
    if let Some(pkg) = crate::project::engine_package(cwd.as_deref()) {
        for d in [pkg.join("knowledge/wiki"), pkg.join("../knowledge/wiki")] {
            if d.join("index.md").is_file() {
                return Some(d);
            }
        }
    }
    let root = cwd
        .as_deref()
        .and_then(crate::find_root)
        .or_else(|| crate::find_root(&std::env::current_exe().ok()?))?;
    let d = root.join(og_idl::WIKI_DIR);
    d.join("index.md").is_file().then_some(d)
}

pub fn load(dir: &Path) -> std::io::Result<Vec<Entry>> {
    let mut pages: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    pages.sort();
    let mut all = Vec::new();
    for p in pages {
        let page = p
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        all.extend(entries(&page, &std::fs::read_to_string(&p)?));
    }
    Ok(all)
}

const KINDS: [&str; 8] = [
    "module", "fn", "record", "type", "member", "const", "value", "column",
];

/// `og api find <words> [--kind K] [--limit N]` or `og api show <name>`.
pub fn run(cmd: &str, args: &[&str]) -> ExitCode {
    let Some(dir) = wiki_dir() else {
        eprintln!(
            "og api {cmd}: no API reference found: {}; or set OG_WIKI_DIR to the knowledge package's wiki/",
            crate::project::NO_ENGINE
        );
        return ExitCode::FAILURE;
    };
    let all = match load(&dir) {
        Ok(all) => all,
        Err(e) => {
            eprintln!("og api {cmd}: cannot read {}: {e}", dir.display());
            return ExitCode::FAILURE;
        }
    };
    let mut kind = None;
    let mut limit = 12usize;
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match *a {
            "--kind" => match it.next() {
                Some(k) if KINDS.contains(k) => kind = Some(*k),
                _ => {
                    eprintln!("og api {cmd}: --kind is one of {}", KINDS.join(", "));
                    return ExitCode::from(2);
                }
            },
            "--limit" => match it.next().and_then(|n| n.parse().ok()) {
                Some(n) => limit = n,
                None => {
                    eprintln!("og api {cmd}: --limit takes a number");
                    return ExitCode::from(2);
                }
            },
            w => rest.push(w),
        }
    }
    let query = rest.join(" ");
    if query.trim().is_empty() {
        eprintln!(
            "og api {cmd}: expected {}",
            if cmd == "find" {
                "words to look for, e.g. `og api find character controller`"
            } else {
                "a name, e.g. `og api show og.CharacterController`, or a module, e.g. `og api show input`"
            }
        );
        return ExitCode::from(2);
    }
    if cmd == "show" {
        let hits = show(&all, &query);
        if hits.is_empty() {
            let near = find(&all, &query, kind);
            eprintln!("og api show: no entry named `{query}`.");
            if !near.is_empty() {
                eprintln!("Close matches:");
                for e in near.iter().take(8) {
                    eprintln!("  {}  ({}, {})", e.name, e.kind, e.page);
                }
            }
            return ExitCode::FAILURE;
        }
        for (i, e) in hits.iter().enumerate() {
            if i > 0 {
                println!("\n---\n");
            }
            print!("{}", render(e));
        }
        return ExitCode::SUCCESS;
    }
    let hits = find(&all, &query, kind);
    if hits.is_empty() {
        println!("og api find: nothing matches every word of `{query}`; try fewer or other words.");
        return ExitCode::FAILURE;
    }
    for e in hits.iter().take(limit) {
        println!("{}  ({}, {}) — {}", e.name, e.kind, e.page, e.summary);
    }
    if hits.len() > limit {
        println!(
            "… {} more; narrow the words or pass --kind or --limit.",
            hits.len() - limit
        );
    }
    println!("`og api show <name>` prints one in full.");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wiki() -> Vec<Entry> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        load(&root.join(og_idl::WIKI_DIR)).unwrap()
    }

    /// Every anchor the generated wiki carries is one entry with a name: a
    /// change to the wiki's layout fails here, not silently in a lookup.
    #[test]
    fn every_anchor_of_the_generated_wiki_is_an_entry() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = root.join(og_idl::WIKI_DIR);
        let anchors: usize = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap())
            .map(|t| {
                t.lines()
                    .filter(|l| l.starts_with("<a id=\"") || l.starts_with("| <a id=\""))
                    .count()
            })
            .sum();
        let all = wiki();
        let modules = all.iter().filter(|e| e.kind == "module").count();
        assert_eq!(all.len() - modules, anchors);
        let unnamed: Vec<_> = all
            .iter()
            .filter(|e| e.name.is_empty() || e.summary.is_empty())
            .map(|e| format!("{}:{}", e.page, e.kind))
            .collect();
        assert!(unnamed.is_empty(), "{unnamed:?}");
    }

    #[test]
    fn find_ranks_the_named_entry_first_and_show_prints_one_without_its_idl() {
        let all = wiki();
        let hits = find(&all, "character controller", None);
        assert_eq!(hits[0].name, "og.CharacterController");
        assert_eq!(
            find(&all, "CharacterController", None)[0].name,
            "og.CharacterController"
        );
        // A word only the summary carries still finds it, behind name matches.
        assert!(
            find(&all, "raycast", Some("fn"))
                .iter()
                .any(|e| e.name == "og.physics.raycast")
        );
        let one = show(&all, "CharacterController");
        assert_eq!(one.len(), 1);
        let text = render(one[0]);
        assert!(text.contains("| `jumpSpeed` |"), "{text}");
        assert!(text.len() < 12_000, "{}", text.len());
        let ray = render(show(&all, "og.physics.raycast")[0]);
        assert!(ray.contains("og.physics.raycast(world: World"), "{ray}");
        assert!(
            !ray.contains("```ogt") && !ray.contains("Binding symbol"),
            "{ray}"
        );
        // A table row (a constant) is an entry too.
        let c = show(&all, "BODY_STATIC");
        assert_eq!((c.len(), c[0].kind.as_str()), (1, "const"));
        assert!(show(&all, "NoSuchThing").is_empty());
        // Words only a field's doc carries find the record; a word nothing carries leaves the rest ranked.
        let jump: Vec<&str> = find(&all, "jump speed coyote", Some("record"))
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert!(jump.contains(&"og.CharacterController"), "{jump:?}");
        assert_eq!(
            find(&all, "character controller zzzqx", None)[0].name,
            "og.CharacterController"
        );
    }

    /// A module's name shows its overview (the prose a skill used to point at
    /// on the wiki page); the resource of the same name stays one `og.` away.
    #[test]
    fn show_of_a_module_name_is_its_overview_without_idl() {
        let all = wiki();
        let module = show(&all, "input");
        assert_eq!(
            module.len(),
            1,
            "{:?}",
            module.iter().map(|e| &e.name).collect::<Vec<_>>()
        );
        let text = render(module[0]);
        assert!(
            text.contains("# The `input` module") && text.contains("og.PointerLock.wanted"),
            "{text}"
        );
        assert!(
            !text.contains("```ogt") && !text.contains("## Contents"),
            "{text}"
        );
        let resource = show(&all, "og.Input");
        assert!(resource.iter().all(|e| e.kind == "record") && !resource.is_empty());
        assert!(all.iter().filter(|e| e.kind == "module").count() > 20);
    }

    #[test]
    fn words_split_camel_case_and_dots() {
        assert_eq!(
            words("og.physics.shapeCast"),
            ["og", "physics", "shape", "cast"]
        );
        assert_eq!(words("BODY_STATIC"), ["body", "static"]);
    }
}
