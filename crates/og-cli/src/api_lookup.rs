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

/// A module's members that have a page of their own, by kind, one line each
/// of names: what `og api show <module>` lists after the overview.
fn members(ix: &SearchIndex, module: &str) -> String {
    let mut out = String::new();
    for kind in ["fn", "record", "interface", "type"] {
        let mut names: Vec<&str> = ix
            .docs
            .iter()
            .filter(|d| d.kind == kind && d.page.split_once('/').is_some_and(|(m, _)| m == module))
            .map(|d| d.name.as_str())
            .collect();
        names.dedup();
        if !names.is_empty() {
            out.push_str(&format!("{kind}: {}\n", names.join(", ")));
        }
    }
    if !out.is_empty() {
        out.insert_str(0, "\nMembers (`og api show <name>` for one):\n");
    }
    out
}

/// The module an entry's page belongs to (`physics` for `physics/fn-raycast`).
fn module_of(page: &str) -> &str {
    page.split('/').next().unwrap_or(page)
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
        let hits = show(&all, &query);
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
                print!("{}", members(&ix, &e.page));
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

    fn names<'a>(hits: &[(&'a Doc, Hit)]) -> Vec<&'a str> {
        hits.iter().map(|(d, _)| d.name.as_str()).collect()
    }

    /// Every anchor the generated wiki carries, on every page and item page,
    /// is one entry with a name: a change to the wiki's layout fails here,
    /// not silently in a lookup.
    #[test]
    fn every_anchor_of_the_generated_wiki_is_an_entry() {
        fn anchors(dir: &Path) -> usize {
            std::fs::read_dir(dir)
                .unwrap()
                .map(|e| e.unwrap().path())
                .map(|p| {
                    if p.is_dir() {
                        anchors(&p)
                    } else if p.extension().is_some_and(|x| x == "md") {
                        std::fs::read_to_string(&p)
                            .unwrap()
                            .lines()
                            .filter(|l| l.starts_with("<a id=\"") || l.starts_with("| <a id=\""))
                            .count()
                    } else {
                        0
                    }
                })
                .sum()
        }
        let all = wiki();
        let overviews = all.iter().filter(|e| e.body.starts_with("# The `")).count();
        assert_eq!(all.len() - overviews, anchors(&dir()));
        let unnamed: Vec<_> = all
            .iter()
            .filter(|e| e.name.is_empty() || e.summary.is_empty())
            .map(|e| format!("{}:{}", e.page, e.kind))
            .collect();
        assert!(unnamed.is_empty(), "{unnamed:?}");
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

    #[test]
    fn find_ranks_the_named_entry_first_and_show_prints_one_without_its_idl() {
        let ix = index(&dir()).unwrap();
        let all = wiki();
        assert_eq!(
            names(&find(&ix, "character controller", None))[0],
            "og.CharacterController"
        );
        assert_eq!(
            names(&find(&ix, "CharacterController", None))[0],
            "og.CharacterController"
        );
        assert!(names(&find(&ix, "raycast", Some("fn"))).contains(&"og.physics.castRay"));
        // Scores fall from first to last.
        let hits = find(&ix, "ray cast against colliders", None);
        assert!(hits.windows(2).all(|w| w[0].1.score >= w[1].1.score));
        let one = show(&all, "CharacterController");
        assert_eq!(one.len(), 1);
        let text = render(one[0]);
        assert!(text.contains("| `jumpSpeed` |"), "{text}");
        assert!(text.len() < 12_000, "{}", text.len());
        let ray = render(show(&all, "og.physics.castRay")[0]);
        assert!(ray.contains("og.physics.castRay(world: World"), "{ray}");
        assert!(
            !ray.contains("```ogt") && !ray.contains("Binding symbol"),
            "{ray}"
        );
        // A table row (a constant) is an entry too.
        let c = show(&all, "SHAPE_BOX");
        assert_eq!((c.len(), c[0].kind.as_str()), (1, "const"));
        assert!(show(&all, "NoSuchThing").is_empty());
        // Words only a field's doc carries find the record; a word nothing
        // carries leaves the rest ranked; a typo still finds it.
        assert!(
            names(&find(&ix, "jump speed coyote", Some("record")))
                .contains(&"og.CharacterController")
        );
        assert_eq!(
            names(&find(&ix, "character controller zzzqx", None))[0],
            "og.CharacterController"
        );
        assert!(
            names(&find(&ix, "charcter controler", None))[..5].contains(&"og.CharacterController")
        );
    }

    /// recall@1 and recall@5 of `find` over a `query\texpected` set
    /// (` | ` separates equally right answers).
    fn recall(ix: &SearchIndex, set: &str) -> (usize, usize, usize) {
        let (mut n, mut at1, mut at5) = (0, 0, 0);
        for line in set.lines().skip(1).filter(|l| !l.trim().is_empty()) {
            let (query, expected) = line.split_once('\t').unwrap();
            let want: Vec<&str> = expected.split(" | ").map(str::trim).collect();
            let got = names(&find(ix, query, None));
            n += 1;
            at1 += usize::from(got.first().is_some_and(|g| want.contains(g)));
            at5 += usize::from(got.iter().take(5).any(|g| want.contains(g)));
        }
        (n, at1, at5)
    }

    /// The evaluation set (`tests/fixtures/api-find-queries.tsv`): queries
    /// written by purpose, partial name or with a typo, each with the entry
    /// it should find; its misses were fixed at the source (IDL `aliases`,
    /// thin docs). The held-out set (`api-find-heldout.tsv`) was written
    /// before any alias and is never tuned against: it measures whether a
    /// change generalises. The floors are the measured recall less a query
    /// or two, so a ranking change that loses queries fails.
    #[test]
    fn find_recalls_the_evaluation_sets() {
        let ix = index(&dir()).unwrap();
        let (n, at1, at5) = recall(&ix, include_str!("../tests/fixtures/api-find-queries.tsv"));
        assert!(n >= 160, "{n}");
        assert!(
            at1 >= 60 && at5 >= 129,
            "recall@1 {at1}/{n}, recall@5 {at5}/{n}"
        );
        let (n, at1, at5) = recall(&ix, include_str!("../tests/fixtures/api-find-heldout.tsv"));
        assert!(n >= 40, "{n}");
        assert!(
            at1 >= 8 && at5 >= 19,
            "held out: recall@1 {at1}/{n}, recall@5 {at5}/{n}"
        );
    }

    /// A module's name shows its overview; the resource of the same name
    /// stays one `og.` away.
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
        // Its members follow, each named once.
        let ix = index(&dir()).unwrap();
        let listed = members(&ix, "anim");
        assert!(
            listed.contains("record: ") && listed.contains("og.anim.clip"),
            "{listed}"
        );
        assert_eq!(listed.matches("og.anim.clip,").count(), 1, "{listed}");
        let resource = show(&all, "og.Input");
        assert!(resource.iter().all(|e| e.kind == "record") && !resource.is_empty());
        assert!(all.iter().filter(|e| e.kind == "module").count() > 20);
    }
}
