//! The wiki (`packages/knowledge/wiki/`) is a map an agent can trust: every
//! quoted signature is the IDL's own text at the line it cites, every module
//! has a page, every declared name is in the index and every index link lands
//! on an anchor, and every binding symbol the runtime binds is on a page.
//!
//! The IDL is the repository's `crates/og-api/api.ogt`; `OG_WIKI_TEST_IDL`
//! names another file to run the same checks against.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn idl_path() -> PathBuf {
    match std::env::var("OG_WIKI_TEST_IDL") {
        Ok(p) => PathBuf::from(p),
        Err(_) => Path::new(env!("CARGO_MANIFEST_DIR")).join("api.ogt"),
    }
}

struct Wiki {
    source: Vec<String>,
    pages: BTreeMap<String, String>,
    bindings: String,
    api: og_api::Api,
}

fn wiki() -> Wiki {
    let path = idl_path();
    let text = std::fs::read_to_string(&path).unwrap();
    let api = og_api::parse(&text).unwrap_or_else(|d| panic!("{}: {d:?}", path.display()));
    let files = og_api::generate(&api);
    let prefix = format!("{}/", og_api::WIKI_DIR);
    let pages = files
        .iter()
        .filter_map(|f| {
            f.path
                .strip_prefix(&prefix)
                .map(|n| (n.to_string(), f.contents.clone()))
        })
        .collect();
    let bindings = files
        .iter()
        .find(|f| f.path.ends_with("types/bindings.json"))
        .expect("the binding table is generated")
        .contents
        .clone();
    Wiki {
        source: text.lines().map(str::to_string).collect(),
        pages,
        bindings,
        api,
    }
}

/// Every ```ogt quote on a page with the line its `<sub>` cites.
fn quotes(page: &str) -> Vec<(Vec<String>, usize)> {
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
            .and_then(|l| l.strip_prefix("<sub>`api.ogt` line "))
            .and_then(|l| l.strip_suffix("</sub>"))
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("a quote without its api.ogt line: {body:?}"));
        out.push((body, cite));
    }
    out
}

#[test]
fn every_quote_is_the_idl_text_at_the_line_it_cites() {
    let w = wiki();
    let mut checked = 0;
    for (name, page) in &w.pages {
        for (body, line) in quotes(page) {
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
fn every_module_has_a_page_and_every_index_link_lands() {
    let w = wiki();
    assert_eq!(w.pages.len(), w.api.modules.len() + 1);
    for m in &w.api.modules {
        let page = w
            .pages
            .get(&format!("{}.md", m.name))
            .unwrap_or_else(|| panic!("no page for module {}", m.name));
        assert!(page.starts_with("<!-- Generated from crates/og-api/api.ogt"));
        assert!(
            w.pages["index.md"].contains(&format!("[`{}`]({}.md)", m.name, m.name)),
            "the index does not list module {}",
            m.name
        );
    }
    let index = &w.pages["index.md"];
    let mut links = 0;
    for part in index.split("](").skip(1) {
        let target = &part[..part.find(')').unwrap()];
        let (file, anchor) = target.split_once('#').unwrap_or((target, ""));
        let page = w
            .pages
            .get(file)
            .unwrap_or_else(|| panic!("index links to a missing page {file}"));
        if !anchor.is_empty() {
            assert!(
                page.contains(&format!("<a id=\"{anchor}\"></a>")),
                "index links to {file}#{anchor}, which has no anchor"
            );
        }
        links += 1;
    }
    // Every item and member of the IDL is in the index.
    let mut names = 0;
    for m in &w.api.modules {
        for item in &m.items {
            let name = item.name();
            assert!(
                index.contains(&format!("{name}`]({}.md#", m.name))
                    || index.contains(&format!("{name}()`]({}.md#", m.name)),
                "the index lacks {}.{name}",
                m.name
            );
            names += 1;
        }
    }
    assert!(links > names, "{links} links for {names} names");
}

#[test]
fn every_binding_symbol_is_documented_on_a_page() {
    let w = wiki();
    let symbols: Vec<&str> = w
        .bindings
        .split("\"symbol\":")
        .skip(1)
        .filter_map(|s| s.trim_start().strip_prefix('"')?.split('"').next())
        .collect();
    assert!(symbols.len() > 50, "only {} symbols", symbols.len());
    // On the module pages themselves, not only in the index's table.
    let all: String = w
        .pages
        .iter()
        .filter(|(name, _)| *name != "index.md")
        .map(|(_, page)| page.as_str())
        .collect();
    for s in symbols {
        assert!(
            all.contains(&format!("Binding symbol `{s}`")),
            "{s} is on no module page"
        );
    }
}
