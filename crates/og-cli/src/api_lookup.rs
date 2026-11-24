//! `og api find <words>` and `og api show <name>`: the API reference one
//! entry at a time. The wiki pages `og api gen` writes (the knowledge
//! package's `wiki/`, shipped in @origingame/engine) give every function,
//! component, type, member, constant and value an anchor (`<a id="kind-…">`);
//! an entry is the anchor's block (to the next anchor or `## ` section) or,
//! for a table row (constants, record columns), the row. `find` ranks entries
//! by their name and first sentence; `show` prints one entry without its IDL
//! source, so an agent reads a few kilobytes instead of a 400 KB page.

//! `og api find <words>` and `og api show <name>`: the API reference one
//! entry at a time. The wiki pages `og api gen` writes (the knowledge
//! package's `wiki/`, shipped in @origingame/engine) give every function,
//! component, type, member, constant, value and guide section an anchor;
//! an entry is the anchor's block or table row (`og_idl::search::entries`).
//! `find` ranks entries with the index `og api gen` ships beside the pages
//! (`search.tsv`, `og_idl::search`: BM25F and static embeddings, fused); `show` prints one entry without
//! its IDL source, so an agent reads a few kilobytes instead of a page.

use og_idl::search::{Doc, Entry, Hit, SearchIndex, bare, entries};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The ranked entries for `query`, best first, with their scores.
pub fn find<'a>(ix: &'a SearchIndex, query: &str, kind: Option<&str>) -> Vec<(&'a Doc, Hit)> {
    ix.search(query, kind)
        .into_iter()
        .map(|h| (&ix.docs[h.doc], h))
        .collect()
}

/// The core items a module overview names (`og_idl`'s core line), or none
/// when the module declares no tiers.
fn core(overview: &str) -> Option<Vec<&str>> {
    let line = overview
        .lines()
        .find_map(|l| l.strip_prefix(og_idl::CORE_PREFIX))?;
    let names = line.split_once("; ").map_or(line, |(n, _)| n);
    Some(names.split(", ").map(|n| n.trim_matches('`')).collect())
}

/// A module's members that have a page of their own, by kind, one line each
/// of names: what `og api show <module>` lists after the overview. With a
/// core tier, only the core ones unless `all`.
fn members(ix: &SearchIndex, module: &str, overview: &str, all: bool) -> String {
    let core = core(overview).filter(|_| !all);
    let mut out = String::new();
    let mut hidden = 0;
    for kind in ["fn", "record", "interface", "type"] {
        let mut names: Vec<&str> = ix
            .docs
            .iter()
            .filter(|d| d.kind == kind && d.page.split_once('/').is_some_and(|(m, _)| m == module))
            .map(|d| d.name.as_str())
            .collect();
        names.dedup();
        if let Some(core) = &core {
            let before = names.len();
            names.retain(|n| core.contains(n));
            hidden += before - names.len();
        }
        if !names.is_empty() {
            out.push_str(&format!("{kind}: {}\n", names.join(", ")));
        }
    }
    if !out.is_empty() {
        out.insert_str(
            0,
            if core.is_some() {
                "\nCore members (`og api show <name>` for one):\n"
            } else {
                "\nMembers (`og api show <name>` for one):\n"
            },
        );
    }
    if hidden > 0 {
        out.push_str(&format!(
            "… {hidden} advanced; `og api show {module} --all` lists them.\n"
        ));
    }
    out
}

/// The module an entry's page belongs to (`physics` for `physics/fn-raycast`).
fn module_of(page: &str) -> &str {
    page.split('/').next().unwrap_or(page)
}

/// The entries named `name` (with or without `og.`, any case); a module's
/// or guide's own name as written (`input`, not `Input` or `og.Input`) is its
/// overview. A guide named like a module (`physics`) is shown with `guide`.
pub fn show<'a>(all: &'a [Entry], name: &str, guide: bool) -> Vec<&'a Entry> {
    let whole = |kind: &str| -> Vec<&'a Entry> {
        all.iter()
            .filter(|e| e.kind == kind && e.name == name.trim())
            .collect()
    };
    for kind in if guide {
        ["guide", "module"]
    } else {
        ["module", "guide"]
    } {
        let found = whole(kind);
        if !found.is_empty() {
            return found;
        }
    }
    let want = bare(name);
    let all: Vec<&Entry> = all
        .iter()
        .filter(|e| e.kind != "module" && !(e.kind == "guide" && !e.name.contains(" §")))
        .collect();
    let exact: Vec<&Entry> = all
        .iter()
        .copied()
        .filter(|e| bare(&e.name) == want)
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    // `CharacterController` names `og.CharacterController`; `castRay` names `og.physics.castRay`.
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

/// Every entry of every page under `dir`, its page named by its path
/// without `.md` (`physics`, `physics/record-CharacterController`).
pub fn load(dir: &Path) -> std::io::Result<Vec<Entry>> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for e in std::fs::read_dir(dir)? {
            let p = e?.path();
            if p.is_dir() {
                walk(&p, out)?;
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(dir, &mut files)?;
    // In page-name order, the order `og api gen` built the index in.
    let mut pages: Vec<(String, PathBuf)> = files
        .into_iter()
        .map(|p| {
            let rel = p.strip_prefix(dir).unwrap_or(&p).with_extension("");
            (rel.to_string_lossy().replace('\\', "/"), p)
        })
        .collect();
    pages.sort();
    let mut all = Vec::new();
    for (page, p) in pages {
        all.extend(entries(&page, &std::fs::read_to_string(&p)?));
    }
    Ok(all)
}

/// The shipped index, or one built from the pages when a wiki has none.
pub fn index(dir: &Path) -> std::io::Result<SearchIndex> {
    match std::fs::read_to_string(dir.join(og_idl::search::INDEX_FILE)) {
        Ok(text) => SearchIndex::from_text(&text).ok_or_else(|| {
            std::io::Error::other(format!(
                "{} is not an og api find index; run `og api gen`",
                og_idl::search::INDEX_FILE
            ))
        }),
        Err(_) => Ok(SearchIndex::build(&load(dir)?)),
    }
}

const KINDS: [&str; 9] = [
    "module", "fn", "record", "type", "member", "const", "value", "column", "guide",
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
    let mut kind = None;
    let mut limit = 12usize;
    let mut json = false;
    let mut every = false;
    let mut guide = false;
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
            "--json" => json = true,
            "--all" => every = true,
            "--guide" => guide = true,
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
    let unreadable = |e: std::io::Error| {
        eprintln!("og api {cmd}: cannot read {}: {e}", dir.display());
        ExitCode::FAILURE
    };
    let ix = match index(&dir) {
        Ok(ix) => ix,
        Err(e) => return unreadable(e),
    };
    if cmd == "show" {
        let all = match load(&dir) {
            Ok(all) => all,
            Err(e) => return unreadable(e),
        };
        let hits = show(&all, &query, guide);
        if hits.is_empty() {
            let near = find(&ix, &query, kind);
            eprintln!("og api show: no entry named `{query}`.");
            if !near.is_empty() {
                eprintln!("Close matches:");
                for (d, _) in near.iter().take(8) {
                    eprintln!("  {}  ({}, {})", d.name, d.kind, module_of(&d.page));
                }
            }
            return ExitCode::FAILURE;
        }
        for (i, e) in hits.iter().enumerate() {
            if i > 0 {
                println!("\n---\n");
            }
            print!("{}", render(e));
            if e.kind == "module" {
                print!("{}", members(&ix, &e.page, &e.body, every));
                if all.iter().any(|g| g.kind == "guide" && g.name == e.name) {
                    println!(
                        "The `{0}` guide (how to use it): `og api show {0} --guide`.",
                        e.name
                    );
                }
            }
        }
        return ExitCode::SUCCESS;
    }
    let hits = find(&ix, &query, kind);
    if json {
        let rows: Vec<serde_json::Value> = hits
            .iter()
            .take(limit)
            .map(|(d, score)| {
                serde_json::json!({
                    "name": d.name, "kind": d.kind, "page": format!("{}.md", d.page),
                    "summary": d.summary,
                    "score": (score.score * 10000.0).round() / 10000.0,
                    "bm25": score.bm25.map(|b| (b * 1000.0).round() / 1000.0),
                    "cosine": (f64::from(score.cosine) * 1000.0).round() / 1000.0,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({ "query": query, "total": hits.len(), "entries": rows })
        );
        return if hits.is_empty() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }
    if hits.is_empty() {
        println!("og api find: nothing matches `{query}`; try fewer or other words.");
        return ExitCode::FAILURE;
    }
    for (d, _) in hits.iter().take(limit) {
        println!(
            "{}  ({}, {}) — {}",
            d.name,
            d.kind,
            module_of(&d.page),
            d.summary
        );
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

    fn dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(og_idl::WIKI_DIR)
    }

    fn wiki() -> Vec<Entry> {
        load(&dir()).unwrap()
    }

    /// The shipped index is the one the pages build: `og api gen` wrote it
    /// from these pages, and reading it back loses nothing.
    #[test]
    fn the_shipped_index_matches_the_pages() {
        let shipped = index(&dir()).unwrap();
        let built = SearchIndex::build(&wiki());
        let docs = shipped
            .docs
            .iter()
            .zip(&built.docs)
            .position(|(a, b)| a != b);
        assert_eq!(shipped.docs.len(), built.docs.len());
        assert!(
            docs.is_none(),
            "{:?}",
            docs.map(|i| (&shipped.docs[i], &built.docs[i]))
        );
        assert!(
            shipped.terms == built.terms,
            "the terms differ; run `og api gen`"
        );
    }
}
