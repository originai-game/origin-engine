//! The component catalogue (docs/ui.md §6): every element kind a screen
//! or component document can use, with its props as a document writes them
//! (a CSS property or an attribute) and the states it takes, as one guide
//! (`crates/og-api/guides/components.md`) that `og api gen` writes from
//! [`super::schema`] and `og api check` holds equal, so `og api find
//! dropdown` answers with the kind's own section. A document uses only
//! what the catalogue has: anything else is an error naming the fix.

use std::fmt::Write;

use super::css::{css_name, value_form};
use super::schema::{Kind, On, PROPS, PropSpec, Ty};

/// Where the catalogue lives, from the repository root.
pub const PATH: &str = "crates/og-api/guides/components.md";

fn value(p: &PropSpec) -> String {
    let words: &[&str] = match p.ty {
        Ty::Word(ws) => ws,
        _ => &[],
    };
    if p.name == "motion" {
        return "`[<property>] <duration> [<timing-function>] [<delay>], …`".into();
    }
    let text = value_form(p.name, words).unwrap_or_else(|| match p.ty {
        Ty::Word(ws) => ws.join(" | "),
        t => t.name().to_string(),
    });
    format!("`{}`", text.replace('|', "\\|"))
}

/// How a document writes `p`: its CSS declaration or HTML attribute.
fn written(p: &PropSpec) -> String {
    if let Some((tag, name, _)) = super::schema::ELEMENT_ATTRIBUTES
        .iter()
        .find(|(_, _, prop)| *prop == p.name)
    {
        return format!("`<{tag} {name}>`");
    }
    match css_name(p.name) {
        Some("grid-column") => "`grid-column:`, `grid-row:`".into(),
        Some(css) => format!("`{css}:`"),
        None => format!(
            "`{}`",
            match p.name {
                "suggestionLabels" => "<datalist><option label>".into(),
                "suggestionKeys" => "<datalist><option i18n>".into(),
                "stepBy" => "step".into(),
                "input" => "value".into(),
                "maxChars" => "maxlength".into(),
                "minChars" => "minlength".into(),
                "lines" => "rows".into(),
                "exclusive" => "name".into(),
                "multiline" => "<textarea>".into(),
                "key" => "i18n".into(),
                "name" => "aria-label".into(),
                "texture" => "src".into(),
                "tabIndex" => "tabindex".into(),
                other => super::schema::props::svg::svg_attr(other)
                    .map_or_else(|| super::css::kebab(other), str::to_string),
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
                    Some("translate" | "scale" | "grid-column" | "grid-auto-flow")
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
        "\nStyle interaction with `#Buy:hover { scale: 1.04 }`. The screens guide lists state selectors and their scoped cascade. Bind game state with `class:active={$selected}` and style `.active`. Custom properties inherit computed values from the mounted parent; `var(--name, fallback)` resolves after specificity and `!important`. CSS motion uses shared timing values; markup contains no state blocks or timelines. `transition` and its longhands (`transition-property`, `-duration`, `-timing-function`, `-delay`) and `animation` and its longhands (`animation-name`, `-duration`, `-timing-function`, `-delay`, `-iteration-count`, `-direction`, `-fill-mode`, `-play-state`) follow CSS Transitions and Animations Level 1 over the animatable values: opacity, translate, scale, rotate, border-radius, background, color and z-index (keyframes animate the scalar ones). Other animatable properties, including transform with skew functions, interpolate their computed values.\n",
    );
    out.push_str("\n### Exposed widget parts\n\nThese declarations also construct widget part identities and are exported by documentSchema and dump_ui. Classes (such as primary), ARIA attributes (such as aria-selected), and author-declared custom parts are separate from state pseudo-classes. Native API styling hosts and their markup counterparts are listed separately where their names differ.\n\n");
    out.push_str(&super::schema::parts::markdown());
    out.push_str("\n### CSS value functions\n\n| Function | Grammar | Reason and behavior |\n|---|---|---|\n");
    for (name, grammar, reason) in super::schema::CSS_FUNCTIONS {
        let _ = writeln!(out, "| `{name}` | `{grammar}` | {reason} |");
    }
    out.push_str(
        "\n### Media extensions\n\n| Feature | Grammar | Reason and behavior |\n|---|---|---|\n",
    );
    for (name, grammar, reason) in super::schema::MEDIA_FEATURES {
        let _ = writeln!(out, "| `{name}` | `{grammar}` | {reason} |");
    }
    for (n, k) in Kind::ALL
        .iter()
        .copied()
        .filter(|k| *k != Kind::Slot)
        .enumerate()
    {
        let own: Vec<&PropSpec> = PROPS
            .iter()
            .filter(|p| matches!(p.on, On::Kinds(ks) if ks.contains(&k)))
            .collect();
        let written = super::schema::tags::of(k);
        let _ = writeln!(
            out,
            "\n## {}. {} (`{}`): {}\n\n{} {} {}.",
            n + 2,
            written
                .iter()
                .map(|(tag, _)| format!("`<{tag}>`"))
                .collect::<Vec<_>>()
                .join(", "),
            k.name(),
            k.doc().trim_end_matches('.'),
            k.doc(),
            if k.container() {
                "A container with nested children:"
            } else {
                "A leaf; text-bearing controls accept text content:"
            },
            written
                .iter()
                .map(|(_, example)| format!("`{example}`"))
                .collect::<Vec<_>>()
                .join(" or ")
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
        assert!(md.contains("`<column>`") && md.contains("`<og-rive>`"));
        let sections: Vec<&str> = md.split("\n## ").skip(1).collect();
        assert_eq!(
            sections.len(),
            Kind::ALL.iter().filter(|k| **k != Kind::Slot).count() + 1,
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
        for (k, s) in Kind::ALL
            .iter()
            .copied()
            .filter(|k| *k != Kind::Slot)
            .zip(&sections[1..])
        {
            assert!(s.contains(&format!("`{}`", k.name())), "{s}");
            for p in PROPS.iter().filter(|p| p.on != On::All) {
                assert_eq!(
                    s.contains(&written(p)),
                    PROPS.iter().any(|other| other.on != On::All
                        && other.applies(k)
                        && written(other) == written(p)),
                    "{} under {}",
                    p.name,
                    k.name()
                );
            }
        }
        assert!(md.contains("\n| `<og-rive src>` |") && !md.contains("\n| `-og-frame:` |"));
    }
}
