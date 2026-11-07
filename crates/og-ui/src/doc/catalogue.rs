//! The component catalogue (docs/ui.md §6): every element kind a screen
//! or component document can use, with its props as a document writes them
//! (a CSS property or an attribute) and the states it takes, as one guide
//! (`crates/og-api/guides/components.md`) that `og api gen` writes from
//! [`super::schema`] and `og api check` holds equal, so `og api find
//! dropdown` answers with the kind's own section. A document uses only
//! what the catalogue has: anything else is an error naming the fix.

use std::fmt::Write;

use super::css::{css_name, value_form};
use super::schema::{KINDS, On, PROPS, PropSpec, Ty};

/// Where the catalogue lives, from the repository root.
pub const PATH: &str = "crates/og-api/guides/components.md";

fn value(p: &PropSpec) -> String {
    let words: &[&str] = match p.ty {
        Ty::Word(ws) => ws,
        _ => &[],
    };
    let text = value_form(p.name, words).unwrap_or_else(|| match p.ty {
        Ty::Word(ws) => ws.join(" | "),
        t => t.name().to_string(),
    });
    format!("`{}`", text.replace('|', "\\|"))
}

/// How a document writes `p`: its CSS declaration or HTML attribute.
fn written(p: &PropSpec) -> String {
    match css_name(p.name) {
        Some("grid-column") => "`grid-column:`, `grid-row:`".into(),
        Some(css) => format!("`{css}:`"),
        None => format!(
            "`{}`",
            match p.name {
                "stepBy" => "step".into(),
                "input" => "value".into(),
                "key" => "i18n".into(),
                "name" => "aria-label".into(),
                "texture" => "src".into(),
                "tabIndex" => "tabindex".into(),
                other => super::css::kebab(other),
            }
        ),
    }
}

/// A row per way of writing: props one declaration sets together
/// (`translate` for x and y) share it.
fn table(out: &mut String, props: &[&PropSpec]) {
    out.push_str("| Write | Value | What |\n|---|---|---|\n");
    let mut rows: Vec<(String, String, Vec<String>)> = Vec::new();
    for p in props {
        let w = written(p);
        let two_way = if p.two_way {
            " Two-way only with an explicit bind: attribute."
        } else {
            ""
        };
        let what = format!("{}{two_way}", p.doc.replace('|', "\\|"));
        match rows.iter_mut().find(|r| r.0 == w) {
            Some(r) => r.2.push(format!("`{}`: {what}", p.name)),
            None => {
                let joint = matches!(
                    css_name(p.name),
                    Some("translate" | "scale" | "-og-skew" | "grid-column" | "grid-auto-flow")
                );
                let what = if joint {
                    format!("`{}`: {what}", p.name)
                } else {
                    what
                };
                rows.push((w, value(p), vec![what]));
            }
        }
    }
    for (w, v, what) in rows {
        let _ = writeln!(out, "| {w} | {v} | {} |", what.join(" "));
    }
}

/// The catalogue's Markdown.
pub fn markdown() -> String {
    let mut out = String::from(
        "<!-- Generated from og-ui's document schema (crates/og-ui/src/doc/schema.rs) by `og api gen`. Do not edit: `og api check` fails on any difference. -->\n\
         # The UI component catalogue\n\n\
         Core elements and their reflected properties for scriptless Svelte documents. Unknown elements, properties, and values fail cooking. The `screens` guide describes contracts, bindings, styles, and content composition.\n\n\
         Write `<button id=\"Buy\" onclick={buy}>Buy</button>`. Put CSS declarations such as `gap: 8px` in a scoped `<style>` block. Bind attributes with `value={$amount}` and writable controls with `bind:value={$amount}`. Use `<text>` for the reflected label kind and `<img>` for image. Named content uses snippets and `{@render name()}`, not legacy slot elements.\n\n",
    );
    let every: Vec<&PropSpec> = PROPS.iter().filter(|p| p.on == On::All).collect();
    out.push_str("## 1. Every element\n\nWhat every kind takes: its box, layout, paint, motion and input.\n\n");
    table(&mut out, &every);
    out.push_str(
        "\nStyle interaction with `#Buy:hover { scale: 1.04 }`, `:active`, `:focus-visible`, and `:disabled`. Bind game state with `class:active={$selected}` and style `.active`. CSS motion uses shared timing values; markup contains no state blocks or timelines.\n",
    );
    for (n, k) in KINDS.iter().filter(|k| k.name != "slot").enumerate() {
        let own: Vec<&PropSpec> = PROPS
            .iter()
            .filter(|p| matches!(p.on, On::Kinds(ks) if ks.contains(&k.name)))
            .collect();
        let _ = writeln!(
            out,
            "\n## {}. `{}`: {}\n\n{} {} `<{} id=\"Name\">…</{}>`.",
            n + 2,
            k.name,
            k.doc.trim_end_matches('.'),
            k.doc,
            if k.container {
                "A container with nested children:"
            } else {
                "A leaf; text-bearing controls accept text content:"
            },
            match k.name {
                "label" => "text",
                "image" => "img",
                other => other,
            },
            match k.name {
                "label" => "text",
                "image" => "img",
                other => other,
            }
        );
        if own.is_empty() {
            out.push_str("\nIt takes every element's props and none of its own.\n");
        } else {
            out.push_str("\nIts own props, beside every element's:\n\n");
            table(&mut out, &own);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ways it could fail: a kind left out, a prop of several kinds listed
    /// under only one of them (a filter on the first kind), or a declared
    /// prop shown by its attribute name, which a document would refuse.
    #[test]
    fn every_kind_has_a_section_listing_the_props_it_takes() {
        let md = markdown();
        let sections: Vec<&str> = md.split("\n## ").skip(1).collect();
        assert_eq!(
            sections.len(),
            KINDS.iter().filter(|k| k.name != "slot").count() + 1,
            "every element, then each kind"
        );
        let written = |p: &PropSpec| format!("\n| {} |", written(p));
        for p in PROPS.iter().filter(|p| p.on == On::All) {
            assert!(
                sections[0].contains(&written(p)),
                "{} in every element's",
                p.name
            );
        }
        for (k, s) in KINDS
            .iter()
            .filter(|k| k.name != "slot")
            .zip(&sections[1..])
        {
            assert!(s.contains(&format!("`{}`", k.name)), "{s}");
            for p in PROPS.iter().filter(|p| p.on != On::All) {
                assert_eq!(
                    s.contains(&written(p)),
                    PROPS.iter().any(|other| other.on != On::All
                        && other.applies(k.name)
                        && written(other) == written(p)),
                    "{} under {}",
                    p.name,
                    k.name
                );
            }
        }
        assert!(md.contains("| `-og-frame:` |") && !md.contains("| `frame` |"));
    }
}
