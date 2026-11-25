//! `og api find <words>`, `og api show <name>` and `og api list`: the API
//! reference one entry at a time. The wiki pages `og api gen` writes (the
//! knowledge package's `wiki/`, shipped in @origingame/engine) give every
//! function, component, type, member, constant, value and guide section an
//! anchor (`og_idl::search::entries`). `find` ranks entries with the index
//! `og api gen` ships beside the pages (`search.tsv`, `og_idl::search`: BM25F
//! and static embeddings, fused); `show` loads only the matched page(s) and
//! prints one entry without its IDL source, so an agent reads a few kilobytes
//! instead of a whole wiki.

use og_idl::search::{Doc, Entry, Hit, SearchIndex, bare, entries, entry_title, section_title};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Guides larger than this print their overview and a numbered section table
/// unless `--all` is passed, so `og api show components --guide` stays within
/// a model's context budget.
const GUIDE_INLINE_MAX_BYTES: usize = 28_000;

/// Cached wiki directory and search index, kept in process by `og mcp`.
pub struct WikiCache {
    dir: PathBuf,
    index: SearchIndex,
}

impl WikiCache {
    pub fn open() -> Result<Self, String> {
        let dir = wiki_dir().ok_or_else(|| {
            format!(
                "no API reference found: {}; or set OG_WIKI_DIR to the knowledge package's wiki/",
                crate::project::NO_ENGINE
            )
        })?;
        let index = index(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
        Ok(Self { dir, index })
    }

    pub fn find_json(
        &self,
        query: &str,
        kind: Option<&str>,
        limit: usize,
    ) -> (serde_json::Value, bool) {
        let hits = find(&self.index, query, kind);
        let rows: Vec<serde_json::Value> = hits
            .iter()
            .take(limit)
            .map(|(d, score)| {
                serde_json::json!({
                    "name": d.name,
                    "kind": d.kind,
                    "page": format!("{}.md", d.page),
                    "summary": d.summary,
                    "score": (score.score * 10000.0).round() / 10000.0,
                    "bm25": score.bm25.map(|b| (b * 1000.0).round() / 1000.0),
                    "cosine": (f64::from(score.cosine) * 1000.0).round() / 1000.0,
                })
            })
            .collect();
        (
            serde_json::json!({ "query": query, "total": hits.len(), "entries": rows }),
            !hits.is_empty(),
        )
    }

    pub fn show_output(
        &self,
        query: &str,
        guide: bool,
        every: bool,
        json: bool,
    ) -> Result<String, String> {
        show_formatted(&self.dir, &self.index, query, None, guide, every, json)
    }

    pub fn list_json(&self, kind: Option<&str>) -> serde_json::Value {
        list_value(&self.index, kind)
    }
}

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

/// A module's members by kind (subpages for fn/record/type, and row entries on
/// the module page for value/const/column), one line each: what `og api show
/// <module>` lists after the overview. With a core tier, only the core ones
/// unless `all`.
fn members(ix: &SearchIndex, module: &str, overview: &str, all: bool) -> String {
    let core = core(overview).filter(|_| !all);
    let mut out = String::new();
    let mut hidden = 0;
    for kind in ["fn", "record", "type", "value", "const", "column"] {
        let on_module_page = matches!(kind, "value" | "const" | "column");
        let mut names: Vec<&str> = ix
            .docs
            .iter()
            .filter(|d| {
                d.kind == kind
                    && if on_module_page {
                        d.page == module
                    } else {
                        d.page.split_once('/').is_some_and(|(m, _)| m == module)
                    }
            })
            .map(|d| d.name.as_str())
            .collect();
        names.dedup();
        if let Some(core) = &core {
            let before = names.len();
            names.retain(|n| core.contains(n));
            hidden += before - names.len();
        }
        if !names.is_empty() {
            let _ = writeln!(out, "{kind}: {}", names.join(", "));
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
        let _ = writeln!(
            out,
            "… {hidden} advanced; `og api show {module} --all` lists them."
        );
    }
    out
}

/// The module an entry's page belongs to (`physics` for `physics/fn-raycast`).
fn module_of(page: &str) -> &str {
    page.split('/').next().unwrap_or(page)
}

/// Whether `d` is a guide overview (as opposed to one of its `§N` sections).
fn is_guide_overview(d: &Doc) -> bool {
    d.kind == "guide" && !d.name.contains(" §")
}

/// The entries named `name` (with or without `og.`, any case); a module's
/// or guide's own name as written (`input`, not `Input` or `og.Input`) is its
/// overview. A guide named like a module (`physics`) is shown with `guide`.
#[allow(dead_code)]
pub fn show<'a>(all: &'a [Entry], name: &str, guide: bool) -> Vec<&'a Entry> {
    let trimmed = name.trim();
    let whole = |kind: &str| -> Vec<&'a Entry> {
        all.iter()
            .filter(|e| e.kind == kind && e.name == trimmed)
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
    let items: Vec<&Entry> = all
        .iter()
        .filter(|e| e.kind != "module" && !(e.kind == "guide" && !e.name.contains(" §")))
        .collect();
    let exact: Vec<&Entry> = items
        .iter()
        .copied()
        .filter(|e| bare(&e.name) == want)
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    // `CharacterController` names `og.CharacterController`; `castRay` names `og.physics.castRay`.
    let leaf: Vec<&Entry> = items
        .iter()
        .copied()
        .filter(|e| bare(&e.name).rsplit('.').next() == Some(want.as_str()))
        .collect();
    if !leaf.is_empty() {
        return leaf;
    }
    // A guide section's heading title (`Button`, `StatBar`, `mount`, `createSignal`).
    items
        .into_iter()
        .filter(|e| e.kind == "guide" && entry_title(e).is_some_and(|t| bare(t) == want))
        .collect()
}

/// Match `name` against the already-loaded `SearchIndex` and load only the
/// matched page file(s) from disk.
fn match_docs<'a>(ix: &'a SearchIndex, name: &str, guide: bool) -> Vec<&'a Doc> {
    let trimmed = name.trim();
    let whole = |kind: &str| -> Vec<&'a Doc> {
        ix.docs
            .iter()
            .filter(|d| d.kind == kind && d.name == trimmed)
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
    let items: Vec<&Doc> = ix
        .docs
        .iter()
        .filter(|d| d.kind != "module" && !is_guide_overview(d))
        .collect();
    let exact: Vec<&Doc> = items
        .iter()
        .copied()
        .filter(|d| bare(&d.name) == want)
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    let leaf: Vec<&Doc> = items
        .iter()
        .copied()
        .filter(|d| bare(&d.name).rsplit('.').next() == Some(want.as_str()))
        .collect();
    if !leaf.is_empty() {
        return leaf;
    }
    items
        .into_iter()
        .filter(|d| section_title(d).is_some_and(|t| bare(t) == want))
        .collect()
}

fn load_page(dir: &Path, page: &str) -> std::io::Result<(String, Vec<Entry>)> {
    let path = dir.join(format!("{page}.md"));
    let text = std::fs::read_to_string(&path)?;
    let parsed = entries(page, &text);
    Ok((text, parsed))
}

/// An entry's body without the IDL source block and the binding line, which
/// only the engine's own tools read.
fn clean_body(body: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    for line in body.lines() {
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
        if line.starts_with("Binding symbol ") || line.starts_with("<a id=\"") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// An entry as `show` prints it.
pub fn render(e: &Entry) -> String {
    let text = clean_body(&e.body);
    format!("{text}\n\n({} · {}; page {}.md)\n", e.name, e.kind, e.page)
}

/// Render a guide overview: inline all sections when the page fits in
/// [`GUIDE_INLINE_MAX_BYTES`] or `every` is set; otherwise print the overview
/// and a compact section table so a huge catalogue guide stays readable.
fn render_guide_overview(
    overview: &Entry,
    page_text: &str,
    page_entries: &[Entry],
    every: bool,
) -> (String, Option<Vec<serde_json::Value>>) {
    let sections: Vec<&Entry> = page_entries
        .iter()
        .filter(|e| e.kind == "guide" && e.name.contains(" §"))
        .collect();
    if sections.is_empty() || every || page_text.len() <= GUIDE_INLINE_MAX_BYTES {
        let mut body = clean_body(&overview.body);
        for s in &sections {
            body.push_str("\n\n");
            body.push_str(&clean_body(&s.body));
        }
        let text = format!(
            "{body}\n\n({} · {}; page {}.md)\n",
            overview.name, overview.kind, overview.page
        );
        return (text, None);
    }
    let mut body = clean_body(&overview.body);
    let _ = writeln!(
        body,
        "\n\nSections ({} total, {} KB full page; `og api show \"{} §N\"`, `og api show <Title>`, or `--all`):\n",
        sections.len(),
        page_text.len().div_ceil(1024),
        overview.name
    );
    let mut toc = Vec::with_capacity(sections.len());
    for s in &sections {
        let title = entry_title(s).unwrap_or("");
        if title.is_empty() {
            let _ = writeln!(body, "- `{}`", s.name);
        } else {
            let _ = writeln!(body, "- `{}` — {title}", s.name);
        }
        toc.push(serde_json::json!({
            "name": s.name,
            "title": title,
            "summary": s.summary,
        }));
    }
    let text = format!(
        "{}\n\n({} · {}; page {}.md)\n",
        body.trim_end(),
        overview.name,
        overview.kind,
        overview.page
    );
    (text, Some(toc))
}

fn show_formatted(
    dir: &Path,
    ix: &SearchIndex,
    query: &str,
    kind: Option<&str>,
    guide: bool,
    every: bool,
    json: bool,
) -> Result<String, String> {
    let matched = match_docs(ix, query, guide);
    if matched.is_empty() {
        let near = find(ix, query, kind);
        let mut msg = format!("og api show: no entry named `{query}`.");
        if !near.is_empty() {
            msg.push_str("\nClose matches:");
            for (d, _) in near.iter().take(8) {
                let _ = write!(msg, "\n  {}  ({}, {})", d.name, d.kind, module_of(&d.page));
            }
        }
        return Err(msg);
    }
    // Load each distinct referenced page once.
    let pages: BTreeSet<&str> = matched.iter().map(|d| d.page.as_str()).collect();
    let mut loaded: Vec<(&str, String, Vec<Entry>)> = Vec::with_capacity(pages.len());
    for p in pages {
        let (raw, parsed) =
            load_page(dir, p).map_err(|e| format!("cannot read {}/{p}.md: {e}", dir.display()))?;
        loaded.push((p, raw, parsed));
    }
    let mut text_out = String::new();
    let mut json_entries = Vec::new();
    for (i, doc) in matched.iter().enumerate() {
        let Some((_, page_text, page_entries)) = loaded.iter().find(|(p, _, _)| *p == doc.page)
        else {
            continue;
        };
        let Some(entry) = page_entries
            .iter()
            .find(|e| e.kind == doc.kind && e.name == doc.name)
        else {
            continue;
        };
        let (rendered, sections_toc) = if is_guide_overview(doc) {
            render_guide_overview(entry, page_text, page_entries, every)
        } else {
            (render(entry), None)
        };
        let module_members = if entry.kind == "module" {
            Some(members(ix, &entry.page, &entry.body, every))
        } else {
            None
        };
        let has_guide = entry.kind == "module"
            && ix
                .docs
                .iter()
                .any(|g| is_guide_overview(g) && g.name == entry.name);
        if json {
            let mut row = serde_json::json!({
                "name": entry.name,
                "kind": entry.kind,
                "page": format!("{}.md", entry.page),
                "summary": entry.summary,
                "text": rendered,
            });
            if let Some(m) = &module_members
                && !m.is_empty()
            {
                row["members"] = serde_json::json!(m);
            }
            if let Some(toc) = sections_toc {
                row["sections"] = serde_json::json!(toc);
            }
            if has_guide {
                row["guide"] = serde_json::json!(entry.name);
            }
            json_entries.push(row);
        } else {
            if i > 0 {
                text_out.push_str("\n---\n\n");
            }
            text_out.push_str(&rendered);
            if let Some(m) = &module_members {
                text_out.push_str(m);
            }
            if has_guide {
                let _ = writeln!(
                    text_out,
                    "The `{0}` guide (how to use it): `og api show {0} --guide`.",
                    entry.name
                );
            }
        }
    }
    if json {
        Ok(format!(
            "{}\n",
            serde_json::json!({ "query": query, "entries": json_entries })
        ))
    } else {
        Ok(text_out)
    }
}

fn list_value(ix: &SearchIndex, kind: Option<&str>) -> serde_json::Value {
    let want_modules = kind.is_none() || kind == Some("module");
    let want_guides = kind.is_none() || kind == Some("guide");
    let modules: Vec<serde_json::Value> = if want_modules {
        ix.docs
            .iter()
            .filter(|d| d.kind == "module")
            .map(|d| {
                serde_json::json!({
                    "name": d.name,
                    "page": format!("{}.md", d.page),
                    "summary": d.summary,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    let guides: Vec<serde_json::Value> = if want_guides {
        ix.docs
            .iter()
            .filter(|d| is_guide_overview(d))
            .map(|d| {
                let sections = ix
                    .docs
                    .iter()
                    .filter(|s| s.kind == "guide" && s.page == d.page && s.name.contains(" §"))
                    .count();
                serde_json::json!({
                    "name": d.name,
                    "page": format!("{}.md", d.page),
                    "sections": sections,
                    "summary": d.summary,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    serde_json::json!({ "modules": modules, "guides": guides })
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

/// `og api find <words> [--kind K] [--limit N] [--json]`,
/// `og api show <name> [--all] [--guide] [--json]`, or
/// `og api list [--kind module|guide] [--json]`.
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
                Some(k)
                    if (cmd == "list" && matches!(*k, "module" | "guide"))
                        || (cmd != "list" && KINDS.contains(k)) =>
                {
                    kind = Some(*k);
                }
                _ => {
                    if cmd == "list" {
                        eprintln!("og api list: --kind is module or guide");
                    } else {
                        eprintln!("og api {cmd}: --kind is one of {}", KINDS.join(", "));
                    }
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
    let unreadable = |e: std::io::Error| {
        eprintln!("og api {cmd}: cannot read {}: {e}", dir.display());
        ExitCode::FAILURE
    };
    let ix = match index(&dir) {
        Ok(ix) => ix,
        Err(e) => return unreadable(e),
    };
    if cmd == "list" {
        if !rest.is_empty() {
            eprintln!("og api list: unexpected arguments `{}`", rest.join(" "));
            return ExitCode::from(2);
        }
        let val = list_value(&ix, kind);
        if json {
            println!("{val}");
            return ExitCode::SUCCESS;
        }
        if let Some(mods) = val["modules"].as_array()
            && !mods.is_empty()
        {
            println!("Modules ({}):", mods.len());
            for m in mods {
                println!(
                    "  {} — {}",
                    m["name"].as_str().unwrap_or(""),
                    m["summary"].as_str().unwrap_or("")
                );
            }
        }
        if let Some(guides) = val["guides"].as_array()
            && !guides.is_empty()
        {
            if kind.is_none() {
                println!();
            }
            println!("Guides ({}):", guides.len());
            for g in guides {
                println!(
                    "  {} ({} §) — {}",
                    g["name"].as_str().unwrap_or(""),
                    g["sections"].as_u64().unwrap_or(0),
                    g["summary"].as_str().unwrap_or("")
                );
            }
        }
        return ExitCode::SUCCESS;
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
        return match show_formatted(&dir, &ix, &query, kind, guide, every, json) {
            Ok(out) => {
                print!("{out}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        };
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
