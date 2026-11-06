//! The wiki (`packages/knowledge/wiki/`) is a map an agent can trust: every
//! quoted signature is the IDL's own text at the line it cites, every module
//! has a page whose contents link every item, every link on every page lands
//! on a page and an anchor, and every binding symbol the runtime binds is on a page.
//!
//! The IDL is the repository's `crates/og-api/api/`; `OG_WIKI_TEST_IDL`
//! names another parts directory to run the same checks against.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn idl_path() -> PathBuf {
    match std::env::var("OG_WIKI_TEST_IDL") {
        Ok(p) => PathBuf::from(p),
        Err(_) => Path::new(env!("CARGO_MANIFEST_DIR")).join("../og-api/api"),
    }
}

struct Wiki {
    source: Vec<String>,
    pages: BTreeMap<String, String>,
    api: og_idl::Api,
}

fn wiki() -> Wiki {
    let path = idl_path();
    let api = og_idl::read(&path).unwrap_or_else(|e| panic!("{e}"));
    let files = og_idl::generate(&api);
    let prefix = format!("{}/", og_idl::WIKI_DIR);
    let pages = files
        .iter()
        .filter_map(|f| {
            f.path
                .strip_prefix(&prefix)
                .map(|n| (n.to_string(), f.contents.clone()))
        })
        .collect();
    Wiki {
        source: api.source.clone(),
        pages,
        api,
    }
}

/// Every ```ogt quote on a page with the line its `<sub>` cites.
fn quotes(page: &str, api: &og_idl::Api) -> Vec<(Vec<String>, usize)> {
    let mut out = Vec::new();
    let mut lines = page.lines().peekable();
    while let Some(l) = lines.next() {
        if l != "```ogt" {
            continue;
        }
        let mut body = Vec::new();
        for l in lines.by_ref() {
            if l == "```" {
                break;
            }
            body.push(l.to_string());
        }
        let cite = lines
            .by_ref()
            .find(|l| !l.trim().is_empty())
            .and_then(|l| l.strip_prefix("<sub>Module `"))
            .and_then(|l| l.strip_suffix("</sub>"))
            .and_then(|l| l.split_once("` line "))
            .and_then(|(name, n)| {
                let module = api.modules.iter().find(|m| m.name == name)?;
                Some(module.line + n.parse::<usize>().ok()? - 1)
            })
            .unwrap_or_else(|| panic!("a quote without its module-local line: {body:?}"));
        out.push((body, cite));
    }
    out
}

#[test]
fn every_quote_is_the_idl_text_at_the_line_it_cites() {
    let w = wiki();
    let mut checked = 0;
    for (name, page) in &w.pages {
        for (body, line) in quotes(page, &w.api) {
            let marker = body
                .last()
                .is_some_and(|l| l.trim_start().starts_with("# ") && l.contains("follow"));
            let quoted = if marker {
                &body[..body.len() - 1]
            } else {
                &body[..]
            };
            let src = &w.source[line - 1..line - 1 + quoted.len()];
            let indent = src[0].len() - src[0].trim_start().len();
            for (q, s) in quoted.iter().zip(src) {
                let s = s.get(indent..).unwrap_or(s.trim_start()).trim_end();
                assert_eq!(q, s, "{name}: the quote cited at api.ogt:{line} differs");
            }
            // A whole block is balanced; a header (with the marker) is open.
            let opens = quoted
                .iter()
                .filter(|l| l.trim_end().ends_with('{'))
                .count();
            let closes = quoted.iter().filter(|l| l.trim() == "}").count();
            if marker {
                assert_eq!(closes, 0, "{name}: header at {line} closes");
            } else {
                assert_eq!(opens, closes, "{name}: block at {line} is cut");
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} quotes");
}

#[test]
fn adding_lines_to_one_module_leaves_other_pages_and_search_unchanged() {
    let source = include_str!("fixtures/valid.ogt");
    let before = og_idl::parse(source).unwrap();
    let after = og_idl::parse(&source.replace("module reflect {", "module reflect {\n\n")).unwrap();
    let before = og_idl::generate_wiki(&before);
    let after = og_idl::generate_wiki(&after);
    for file in &before {
        if file.path == "packages/knowledge/wiki/reflect.md" || file.path.contains("/reflect/") {
            continue;
        }
        assert_eq!(file, after.iter().find(|f| f.path == file.path).unwrap());
    }
}

#[test]
fn search_rows_are_local_and_both_index_versions_read() {
    use og_idl::search::{Entry, SearchIndex};
    let entry = |name: &str| Entry {
        name: name.into(),
        kind: "fn".into(),
        page: name.into(),
        summary: format!("{name} moves particles."),
        body: "Particles move rapidly.".into(),
    };
    let old = SearchIndex::build(&[entry("zebra")]);
    let new = SearchIndex::build(&[entry("aardvark"), entry("zebra")]);
    let text = new.to_text();
    assert_eq!(SearchIndex::from_text(&text).unwrap(), new);
    assert!(text.ends_with(old.to_text().split_once('\n').unwrap().1));
    let legacy = "# og api find index 3: legacy\ndoc\tzebra\tfn\tzebra\t1,0,0,0,0\t1:AA==\tZebra\nterm\tzebra\t0:1,0,0,0,0\n";
    let parsed = SearchIndex::from_text(legacy).unwrap();
    assert_eq!(parsed.bm25("zebra", None)[0].0, 0);
    assert_eq!(SearchIndex::from_text(&parsed.to_text()).unwrap(), parsed);
}

/// A link's target page, resolved against the page that holds it.
fn resolve(from: &str, target: &str) -> String {
    let mut parts: Vec<&str> = from.split('/').collect();
    parts.pop();
    for seg in target.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    parts.join("/")
}

#[test]
fn every_module_has_a_page_and_every_link_lands() {
    let w = wiki();
    let index = &w.pages["index.md"];
    assert!(w.pages.contains_key(og_idl::search::INDEX_FILE));
    // Every guide beside the IDL is a page the index links, its sections anchored.
    assert!(!w.api.guides.is_empty());
    for g in &w.api.guides {
        let page = &w.pages[&format!("guide-{}.md", g.name)];
        assert!(page.starts_with("<!-- Generated from crates/og-api/api.ogt"));
        assert!(page.contains(&format!("# The `{}` guide: ", g.name)));
        assert!(page.contains(&format!(
            "<a id=\"guide-{}-1\"></a>\n### `{} §1` ",
            g.name, g.name
        )));
        assert!(index.contains(&format!("[`{}`](guide-{}.md)", g.name, g.name)));
    }
    // Every module has a page the index lists, and its contents link every item.
    for m in &w.api.modules {
        let page = w
            .pages
            .get(&format!("{}.md", m.name))
            .unwrap_or_else(|| panic!("no page for module {}", m.name));
        assert!(page.starts_with("<!-- Generated from crates/og-api/api.ogt"));
        assert!(
            index.contains(&format!("[`{}`]({}.md)", m.name, m.name)),
            "the index does not list module {}",
            m.name
        );
        for item in &m.items {
            let name = item.name();
            assert!(
                page.contains(&format!("{name}`](")) || page.contains(&format!("{name}()`](")),
                "{}.md's contents lack {name}",
                m.name
            );
        }
    }
    // Every link on every page lands on a page, and on an anchor when it names one.
    let mut links = 0;
    for (from, text) in w.pages.iter().filter(|(n, _)| n.ends_with(".md")) {
        for part in text.split("](").skip(1) {
            let Some(end) = part.find(')') else { continue };
            let target = &part[..end];
            if target.contains("://") || target.contains(' ') {
                continue;
            }
            let (file, anchor) = target.split_once('#').unwrap_or((target, ""));
            let file = if file.is_empty() {
                from.clone()
            } else {
                resolve(from, file)
            };
            let Some(page) = w.pages.get(&file) else {
                panic!("{from} links to a missing page {file}");
            };
            if !anchor.is_empty() {
                assert!(
                    page.contains(&format!("<a id=\"{anchor}\"></a>")),
                    "{from} links to {file}#{anchor}, which has no anchor"
                );
            }
            links += 1;
        }
    }
    assert!(links > 2000, "{links} links");
    // An item page is small: the reason the wiki is split.
    let big: Vec<_> = w
        .pages
        .iter()
        .filter(|(n, t)| n.contains('/') && t.len() > 64_000)
        .map(|(n, t)| (n.clone(), t.len()))
        .collect();
    assert!(big.is_empty(), "{big:?}");
}
