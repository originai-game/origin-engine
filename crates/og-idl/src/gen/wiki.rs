//! The wiki generator (design §3.1): `packages/knowledge/wiki/`, a short
//! `index.md` of the modules and guides, one overview page per module (its
//! contents with signatures and one-line summaries, its values, column kinds
//! and constants) and one small page per function, record, interface and
//! type under `<module>/`, so an agent reads kilobytes, not a module. Every declaration is quoted from `api.ogt` exactly as written,
//! next to its TypeScript form, its docs and, for callables, its binding
//! symbol, so the wiki cannot drift from the IDL. Nothing here names a
//! module: pages follow whatever `api.ogt` declares, in its order.

use super::{Binding, GENERATED_BY, bindings, default_json, schema_name, ts};
use crate::index::Index;
use crate::model::*;
use std::fmt::Write;

/// `(file name, contents)` for every page, `index.md` first.
pub fn pages(ix: &Index<'_>) -> Vec<(String, String)> {
    let binds = bindings(ix);
    let mut out = vec![("index.md".to_string(), index(ix))];
    for m in &ix.api.modules {
        let (page, items) = module_pages(ix, m, &binds);
        out.push((format!("{}.md", m.name), page));
        out.extend(items);
    }
    for g in &ix.api.guides {
        out.push((format!("guide-{}.md", g.name), guide_page(g)));
    }
    out
}

// ── guides ──────────────────────────────────────────────────────────────────

/// A guide's summary: the first paragraph after its `# ` title.
fn guide_summary(text: &str) -> String {
    text.lines()
        .skip_while(|l| !l.starts_with("# "))
        .skip(1)
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A guide as a wiki page: its title as `# The \`name\` guide`, then each
/// `## N. Title` section under an anchor, headed `### \`name §N\` Title`, so
/// `og api` reads the guide as one entry and each section as another.
fn guide_page(g: &Guide) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "<!-- {GENERATED_BY} -->\n");
    let mut lines = g.text.lines().peekable();
    let title = lines
        .by_ref()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or(&g.name)
        .to_string();
    let _ = writeln!(out, "# The `{}` guide: {title}", g.name);
    let mut fence = false;
    for line in lines {
        if line.starts_with("```") {
            fence = !fence;
        }
        let section = (!fence)
            .then(|| line.strip_prefix("## "))
            .flatten()
            .and_then(|h| h.split_once(". "))
            .filter(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        match section {
            Some((n, heading)) => {
                let _ = writeln!(
                    out,
                    "<a id=\"guide-{}-{n}\"></a>\n### `{} §{n}` {heading}",
                    g.name, g.name
                );
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out
}

// ── quoting api.ogt ─────────────────────────────────────────────────────────

/// The line without a trailing `#` comment (a `#` that starts a token
/// outside a string).
fn code_part(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    let mut prev_space = true;
    for (i, c) in line.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == '#' && prev_space {
            return &line[..i];
        }
        prev_space = c == ' ' || c == '\t';
    }
    line
}

fn opens(line: &str) -> bool {
    code_part(line).trim_end().ends_with('{')
}

fn closes(line: &str) -> bool {
    code_part(line).trim() == "}"
}

/// The declaration starting at 1-based `line` with its whole block.
fn block(src: &[String], line: usize) -> Vec<&str> {
    let Some(first) = line.checked_sub(1).and_then(|i| src.get(i)) else {
        return Vec::new();
    };
    let mut out = vec![first.as_str()];
    if !opens(first) {
        return out;
    }
    let mut depth = 1;
    for l in &src[line..] {
        out.push(l);
        if opens(l) {
            depth += 1;
        } else if closes(l) {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
    }
    out
}

/// The declaration line and the `doc`/`place`/`publish`/`extends` lines
/// that follow it: a container's header without its members.
fn header(src: &[String], line: usize) -> Vec<&str> {
    let all = block(src, line);
    let mut out = Vec::new();
    for (i, l) in all.iter().enumerate() {
        let key = code_part(l).split_whitespace().next().unwrap_or("");
        if i == 0 || (matches!(key, "doc" | "place" | "publish" | "extends") && !opens(l)) {
            out.push(*l);
        } else {
            break;
        }
    }
    out
}

/// A fenced quote of IDL lines, dedented to their common indent.
fn quote(out: &mut String, ix: &Index<'_>, lines: &[&str], line: usize, more: Option<&str>) {
    if lines.is_empty() {
        return;
    }
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    out.push_str("```ogt\n");
    for l in lines {
        out.push_str(l.get(indent..).unwrap_or(l.trim_start()).trim_end());
        out.push('\n');
    }
    if let Some(m) = more {
        let _ = writeln!(out, "  # {m}");
    }
    out.push_str("```\n");
    let module = ix
        .api
        .modules
        .iter()
        .rev()
        .find(|m| m.line <= line)
        .unwrap();
    let local = line - module.line + 1;
    let _ = writeln!(out, "\n<sub>Module `{}` line {local}</sub>\n", module.name);
}

// ── text helpers ────────────────────────────────────────────────────────────

/// A table cell: one line, pipes escaped.
fn cell(s: &str) -> String {
    let s = s.replace('\n', " ").replace('|', "\\|");
    if s.is_empty() { "—".into() } else { s }
}

fn code(s: &str) -> String {
    if s.is_empty() {
        "—".into()
    } else {
        format!("`{}`", s.replace('|', "\\|"))
    }
}

/// Each doc line is a paragraph, except that consecutive lines starting
/// with `|` are the rows of one table.
/// The words an entry is also searched by (its IDL `aliases`), one line.
fn also_known_as(out: &mut String, aliases: &[String]) {
    if !aliases.is_empty() {
        out.push_str(ALSO_KNOWN_AS);
        out.push_str(&aliases.join(", "));
        out.push_str("\n\n");
    }
}

/// The prefix of an item page's aliases line, which `og api find` indexes as its own field.
pub const ALSO_KNOWN_AS: &str = "Also known as: ";

fn paragraphs(out: &mut String, doc: &[String]) {
    for (i, d) in doc.iter().enumerate() {
        out.push_str(d);
        let row = |l: Option<&String>| l.is_some_and(|l| l.starts_with('|'));
        out.push_str(if row(Some(d)) && row(doc.get(i + 1)) {
            "\n"
        } else {
            "\n\n"
        });
    }
}

fn first(doc: &[String]) -> &str {
    doc.first().map_or("", String::as_str)
}

fn number(n: f64) -> String {
    format!("{n}")
}

fn has_fns(m: &Module) -> bool {
    m.items.iter().any(|i| matches!(i, Item::Fn(_)))
}

fn place_text(m: &Module) -> String {
    if !has_fns(m) {
        return "none at module level (its members live on the types below)".to_string();
    }
    match m.place {
        Place::Top => {
            "named exports of the package: `import { … } from \"@origingame/engine\"`".to_string()
        }
        Place::Root => "members of the root object `og` (`og.<fn>`)".to_string(),
        Place::Namespace => format!("the namespace `og.{}`", m.name),
    }
}

fn publish_text(publish: bool) -> &'static str {
    if publish { "kept" } else { "dropped" }
}

// Anchors are explicit, so links never depend on a renderer's heading slugs.
fn anchor_fn(name: &str) -> String {
    format!("fn-{name}")
}
fn anchor_type(name: &str) -> String {
    format!("type-{name}")
}
fn anchor_member(owner: &str, name: &str) -> String {
    format!("member-{owner}-{name}")
}
fn anchor_value(name: &str) -> String {
    format!("value-{name}")
}
fn anchor_const(name: &str) -> String {
    format!("const-{name}")
}
fn anchor_column(name: &str) -> String {
    format!("column-{name}")
}
fn anchor_record(name: &str) -> String {
    format!("record-{name}")
}

fn find<'b, 'a>(
    binds: &'b [Binding<'a>],
    module: &str,
    owner: Option<&str>,
    name: &str,
) -> Option<&'b Binding<'a>> {
    binds
        .iter()
        .find(|b| b.module.name == module && b.owner == owner && b.name == name)
}

// ── tables ──────────────────────────────────────────────────────────────────

fn field_notes(f: &Field) -> String {
    let mut n = Vec::new();
    if f.tunable {
        n.push("tunable".to_string());
    }
    if f.readonly {
        n.push("read only".to_string());
    }
    if f.optional {
        n.push("optional".to_string());
    }
    if let Some(l) = &f.label {
        n.push(format!("label “{l}”"));
    }
    if let Some(w) = &f.worldspace {
        n.push(format!("worldspace `{w}`"));
    }
    n.join(", ")
}

fn range(f: &Field) -> String {
    let mut r = match (f.min, f.max) {
        (Some(a), Some(b)) => format!("{} … {}", number(a), number(b)),
        (Some(a), None) => format!("≥ {}", number(a)),
        (None, Some(b)) => format!("≤ {}", number(b)),
        (None, None) => String::new(),
    };
    if let Some(s) = f.step {
        if !r.is_empty() {
            r.push_str(", ");
        }
        let _ = write!(r, "step {}", number(s));
    }
    r
}

/// The prefix of a module overview's line naming its core items, which
/// `og api show <module>` reads to list them.
pub const CORE_PREFIX: &str = "- **Core** (what most games use): ";

/// The fields as one table, or, when the type declares `group`s, one
/// table under a `#### Heading` per group (fields before the first group first).
fn fields_table(out: &mut String, ix: &Index<'_>, fields: &[Field], groups: &[FieldGroup]) {
    let mut start = 0;
    while start < fields.len() {
        let group = fields[start].group.as_deref();
        let end = fields[start..]
            .iter()
            .position(|f| f.group.as_deref() != group)
            .map_or(fields.len(), |n| start + n);
        if let Some(g) = group.and_then(|n| groups.iter().find(|g| g.name == n)) {
            let _ = writeln!(out, "#### {}\n", g.name);
            paragraphs(out, &g.doc);
        }
        field_rows(out, ix, &fields[start..end]);
        start = end;
    }
}

fn field_rows(out: &mut String, ix: &Index<'_>, fields: &[Field]) {
    out.push_str("| Field | Type | Default | Range | Unit | Notes | Doc |\n");
    out.push_str("|---|---|---|---|---|---|---|\n");
    for f in fields {
        let default = default_json(ix, &f.ty, f.default.as_deref())
            .map(|j| ts::compact_json(&j))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} | {} | {} |",
            f.name,
            code(&ts::type_text(ix, &f.ty, &[])),
            code(&default),
            cell(&range(f)),
            f.unit.as_deref().map_or_else(|| "—".to_string(), code),
            cell(&match ix.capacity(&f.ty) {
                None if ix.list(&f.ty).is_some() => {
                    let (element, n) = ix.list(&f.ty).expect("a list");
                    let notes = field_notes(f);
                    let text = format!(
                        "a list of at most {n} `{element}` (`List.{element}.{n}`): reads a frozen copy, writes a whole array"
                    );
                    if notes.is_empty() {
                        text
                    } else {
                        format!("{text}, {notes}")
                    }
                }
                Some(n) => {
                    let notes = field_notes(f);
                    let text = format!("`{}({n})`: at most {n} bytes of UTF-8", f.ty.name());
                    if notes.is_empty() {
                        text
                    } else {
                        format!("{text}, {notes}")
                    }
                }
                None => field_notes(f),
            }),
            cell(&f.doc.join(" ")),
        );
    }
    out.push('\n');
}

fn params_table(out: &mut String, ix: &Index<'_>, params: &[Param], generics: &[Generic]) {
    if params.is_empty() {
        return;
    }
    let omit = omittable(params, ix);
    out.push_str("| Parameter | Type | Optional | Doc |\n|---|---|---|---|\n");
    for (p, o) in params.iter().zip(omit) {
        let mut doc = p.doc.join(" ");
        if let Some(d) = &p.default
            && let Some(j) = default_json(ix, &p.ty, Some(d))
        {
            let _ = write!(doc, " Default `{}`.", ts::compact_json(&j));
        }
        if p.flatten {
            doc.push_str(" Its fields travel as the call's own parameters on the wire.");
        }
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} |",
            p.name,
            code(&ts::type_text(ix, &p.ty, generics)),
            if o { "yes" } else { "no" },
            cell(&doc),
        );
    }
    out.push('\n');
}

// ── sections ────────────────────────────────────────────────────────────────

fn callable(
    out: &mut String,
    ix: &Index<'_>,
    f: &Function,
    path: &str,
    bind: Option<&Binding<'_>>,
    anchor: &str,
    level: &str,
) {
    let _ = writeln!(out, "<a id=\"{anchor}\"></a>\n\n{level} `{path}`\n");
    paragraphs(out, &f.doc);
    also_known_as(out, &f.aliases);
    let _ = writeln!(out, "```ts\n{path}{}\n```\n", ts::signature(ix, f));
    params_table(out, ix, &f.params, &f.generics);
    let ret = ts::type_text(ix, &f.returns, &f.generics);
    if f.returns_doc.is_empty() {
        let _ = writeln!(out, "**Returns** `{ret}`.\n");
    } else {
        let _ = writeln!(out, "**Returns** `{ret}`: {}\n", f.returns_doc.join(" "));
    }
    if let Some(b) = bind {
        let _ = writeln!(
            out,
            "Binding symbol `{}` · publish builds: {}\n",
            b.symbol,
            publish_text(b.publish)
        );
    } else if !f.host {
        out.push_str("Implemented by the TypeScript runtime over the members of this module: no binding symbol.\n\n");
    }
    quote(out, ix, &block(&ix.api.source, f.line), f.line, None);
}

fn interface(out: &mut String, ix: &Index<'_>, m: &Module, i: &Interface, binds: &[Binding<'_>]) {
    let _ = writeln!(
        out,
        "<a id=\"{}\"></a>\n\n### `{}` (interface)\n",
        anchor_type(&i.name),
        i.name
    );
    paragraphs(out, &i.doc);
    also_known_as(out, &i.aliases);
    if let Some(base) = &i.extends {
        let _ = writeln!(
            out,
            "Extends [`{base}`]({}): every member of it is a member here.\n",
            type_link(ix, base)
        );
    }
    quote(
        out,
        ix,
        &header(&ix.api.source, i.line),
        i.line,
        Some("members follow, each quoted below"),
    );
    for mem in &i.members {
        let bind = find(binds, &m.name, Some(&i.name), mem.name());
        match mem {
            Member::Fn(f) => callable(
                out,
                ix,
                f,
                &format!("{}.{}", i.name, f.name),
                bind,
                &anchor_member(&i.name, &f.name),
                "####",
            ),
            Member::Prop(p) => {
                let _ = writeln!(
                    out,
                    "<a id=\"{}\"></a>\n\n#### `{}.{}` (property)\n",
                    anchor_member(&i.name, &p.name),
                    i.name,
                    p.name
                );
                paragraphs(out, &p.doc);
                let _ = writeln!(
                    out,
                    "```ts\nreadonly {}: {}\n```\n",
                    p.name,
                    ts::type_text(ix, &p.ty, &[])
                );
                if let Some(b) = bind {
                    let _ = writeln!(
                        out,
                        "Binding symbol `{}` · publish builds: {}\n",
                        b.symbol,
                        publish_text(b.publish)
                    );
                }
                quote(out, ix, &block(&ix.api.source, p.line), p.line, None);
            }
        }
    }
}

fn record(out: &mut String, ix: &Index<'_>, m: &Module, r: &Record, binds: &[Binding<'_>]) {
    let _ = writeln!(
        out,
        "<a id=\"{}\"></a>\n\n### `og.{}` ({})\n",
        anchor_record(&r.name),
        r.name,
        r.kind.as_str()
    );
    paragraphs(out, &r.doc);
    also_known_as(out, &r.aliases);
    let _ = writeln!(
        out,
        "Token `og.{}`, registered under the schema name `{}`; its value type is `{}`.\n",
        r.name,
        schema_name(&r.name),
        r.name
    );
    fields_table(out, ix, &r.fields, &r.groups);
    quote(out, ix, &block(&ix.api.source, r.line), r.line, None);
    for f in &r.methods {
        callable(
            out,
            ix,
            f,
            &format!("{}.{}", r.name, f.name),
            find(binds, &m.name, Some(&r.name), &f.name),
            &anchor_member(&r.name, &f.name),
            "####",
        );
    }
}

fn type_item(out: &mut String, ix: &Index<'_>, item: &Item) {
    let src = &ix.api.source;
    match item {
        Item::Struct(s) => {
            let kind = if s.desc { "desc struct" } else { "struct" };
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` ({kind})\n",
                anchor_type(&s.name),
                s.name
            );
            paragraphs(out, &s.doc);
            also_known_as(out, &s.aliases);
            if s.desc {
                out.push_str(
                    "A desc: every field may be omitted and takes its default; `{}` is valid.\n\n",
                );
            }
            fields_table(out, ix, &s.fields, &s.groups);
            quote(out, ix, &block(src, s.line), s.line, None);
        }
        Item::Enum(e) => {
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` (enum)\n",
                anchor_type(&e.name),
                e.name
            );
            paragraphs(out, &e.doc);
            also_known_as(out, &e.aliases);
            let _ = writeln!(
                out,
                "A string: one of the variants below. The zero value is `\"{}\"`.\n",
                e.variants.first().map_or("", |v| v.name.as_str())
            );
            out.push_str("| Variant | Doc |\n|---|---|\n");
            for v in &e.variants {
                let _ = writeln!(out, "| `\"{}\"` | {} |", v.name, cell(&v.doc.join(" ")));
            }
            for p in &e.patterns {
                let _ = writeln!(out, "| `{}` | {} |", p.name, cell(&p.doc.join(" ")));
            }
            out.push('\n');
            quote(out, ix, &block(src, e.line), e.line, None);
        }
        Item::Union(u) => {
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` (union)\n",
                anchor_type(&u.name),
                u.name
            );
            paragraphs(out, &u.doc);
            also_known_as(out, &u.aliases);
            let _ = writeln!(out, "Tagged by `{}`.\n", u.tag);
            out.push_str("| Case | Payload | Doc |\n|---|---|---|\n");
            for c in &u.cases {
                let _ = writeln!(
                    out,
                    "| `{}: \"{}\"` | {} | {} |",
                    u.tag,
                    c.name,
                    c.payload.as_deref().map_or_else(
                        || "—".to_string(),
                        |p| format!("[`{p}`]({})", type_link(ix, p))
                    ),
                    cell(&c.doc.join(" "))
                );
            }
            out.push('\n');
            quote(out, ix, &block(src, u.line), u.line, None);
        }
        Item::Handle(h) => {
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` (handle)\n",
                anchor_type(&h.name),
                h.name
            );
            paragraphs(out, &h.doc);
            out.push_str("A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.\n\n");
            quote(out, ix, &block(src, h.line), h.line, None);
        }
        Item::Callback(c) => {
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` (callback)\n",
                anchor_type(&c.name),
                c.name
            );
            paragraphs(out, &c.doc);
            let _ = writeln!(
                out,
                "Game code supplies this function.\n\n```ts\ntype {} = {}\n```\n",
                c.name,
                ts::callback_signature(ix, c)
            );
            params_table(out, ix, &c.params, &[]);
            quote(out, ix, &block(src, c.line), c.line, None);
        }
        _ => {}
    }
}

/// The page an item has of its own, `<module>/<anchor>.md`, or `None` for
/// the values, column kinds and constants its module's page holds.
fn item_file(m: &Module, item: &Item) -> Option<String> {
    match item {
        Item::Value(_) | Item::Column(_) | Item::Const(_) => None,
        _ => Some(format!("{}/{}.md", m.name, item_label(m, item).2)),
    }
}

/// A link from an item page (under `<module>/`) to type `name`'s anchor.
fn type_link(ix: &Index<'_>, name: &str) -> String {
    ix.module_of(name).map_or_else(
        || format!("#{}", anchor_type(name)),
        |bm| {
            format!(
                "../{}/{}.md#{}",
                bm.name,
                anchor_type(name),
                anchor_type(name)
            )
        },
    )
}

/// An item page's head: the generated header and the way back to its module.
fn item_head(m: &Module) -> String {
    format!(
        "<!-- {GENERATED_BY} -->\n\nModule [`{}`](../{}.md) · [index](../index.md)\n\n",
        m.name, m.name
    )
}

/// A module's overview page and its items' own pages.
fn module_pages(
    ix: &Index<'_>,
    m: &Module,
    binds: &[Binding<'_>],
) -> (String, Vec<(String, String)>) {
    let mut out = String::new();
    let _ = writeln!(out, "<!-- {GENERATED_BY} -->\n");
    let _ = writeln!(out, "# The `{}` module\n", m.name);
    paragraphs(&mut out, &m.doc);
    let _ = writeln!(
        out,
        "- **TypeScript**: functions are {}; components, resources and events are tokens on `og`.",
        place_text(m)
    );
    let _ = writeln!(
        out,
        "- **Publish builds**: {} (members may override).",
        publish_text(m.publish)
    );
    if !m.core.is_empty() {
        let names: Vec<String> = m
            .items
            .iter()
            .filter(|i| m.core.iter().any(|c| c == i.name()))
            .map(|i| format!("`{}`", item_label(m, i).0))
            .collect();
        let _ = writeln!(
            out,
            "{CORE_PREFIX}{}; {} more are advanced (`og api show {} --all`).",
            names.join(", "),
            m.items.len() - names.len(),
            m.name
        );
    }
    out.push_str("- **Index**: [every module and guide](index.md).\n\n");
    quote(
        &mut out,
        ix,
        &header(&ix.api.source, m.line),
        m.line,
        Some("declarations follow, each on its own page"),
    );

    // Contents, in declaration order: each item's page, its signature and its first sentence.
    out.push_str("## Contents\n\n| Name | Kind | Signature | Summary |\n|---|---|---|---|\n");
    for item in &m.items {
        let (label, kind, anchor) = item_label(m, item);
        let link = item_file(m, item).unwrap_or_else(|| format!("#{anchor}"));
        let sig = match item {
            Item::Fn(f) => code(&ts::signature(ix, f)),
            _ => String::new(),
        };
        let _ = writeln!(
            out,
            "| [`{label}`]({link}) | {kind} | {sig} | {} |",
            cell(&crate::search::first_sentence(first(item_doc(item))))
        );
    }
    out.push('\n');

    // Every function, record, interface and type on a page of its own; a
    // function's overloads share one (`fn-clip`, `fn-clip-2`).
    let mut items: Vec<(String, String)> = Vec::new();
    let fns: Vec<&Function> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f),
            _ => None,
        })
        .collect();
    for item in &m.items {
        let Some(file) = item_file(m, item) else {
            continue;
        };
        if items.iter().any(|(f, _)| *f == file) {
            continue;
        }
        let mut page = item_head(m);
        match item {
            Item::Fn(f) => {
                for (i, g) in fns.iter().enumerate().filter(|(_, g)| g.name == f.name) {
                    let bind = find(binds, &m.name, None, &g.name);
                    let path = bind.map_or_else(|| value_path(m, &g.name), |b| b.path.clone());
                    let nth = fns[..i].iter().filter(|h| h.name == g.name).count();
                    let anchor = match nth {
                        0 => anchor_fn(&g.name),
                        n => format!("{}-{}", anchor_fn(&g.name), n + 1),
                    };
                    callable(&mut page, ix, g, &path, bind, &anchor, "###");
                }
            }
            Item::Record(r) => record(&mut page, ix, m, r, binds),
            Item::Interface(x) => interface(&mut page, ix, m, x, binds),
            _ => type_item(&mut page, ix, item),
        }
        while page.ends_with("\n\n") {
            page.pop();
        }
        items.push((file, page));
    }

    let values: Vec<&ValueDecl> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Value(v) => Some(v),
            _ => None,
        })
        .collect();
    if !values.is_empty() {
        out.push_str("## Values\n\nProvided by the TypeScript runtime: no binding symbol.\n\n");
        for v in values {
            let path = value_path(m, &v.name);
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{path}`\n",
                anchor_value(&v.name)
            );
            paragraphs(&mut out, &v.doc);
            let _ = writeln!(
                out,
                "```ts\nconst {}: {}\n```\n",
                v.name,
                ts::type_text(ix, &v.ty, &[])
            );
            quote(&mut out, ix, &block(&ix.api.source, v.line), v.line, None);
        }
    }

    let columns: Vec<&Column> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Column(c) => Some(c),
            _ => None,
        })
        .collect();
    if !columns.is_empty() {
        out.push_str("## Column kinds\n\nEach is a factory, `");
        out.push_str(&columns[0].name);
        out.push_str("(value?, options?)`, that makes one field of a component or resource layout");
        for c in columns.iter().filter(|c| c.capacity) {
            let _ = write!(
                out,
                "; `{}(capacity, value?, options?)` takes its capacity in bytes first",
                c.name
            );
        }
        for c in columns.iter().filter(|c| c.list) {
            let _ = write!(
                out,
                "; `{}(of, capacity, options?)` takes another factory's field (`u32()`, `Asset()`) and a capacity in elements",
                c.name
            );
        }
        out.push_str(".\n\n");
        out.push_str("| Column | Value | Typed array | Stride | Doc |\n|---|---|---|---|---|\n");
        for c in &columns {
            let stride = if c.capacity {
                "the capacity".to_string()
            } else if c.list {
                "the element's × (capacity + 1)".to_string()
            } else {
                c.stride.to_string()
            };
            let (value, array) = if c.list {
                (
                    "an array of the element's".to_string(),
                    "the element's".to_string(),
                )
            } else {
                (
                    code(&ts::type_text(ix, &c.value, &[])),
                    format!("`{}`", c.array),
                )
            };
            let _ = writeln!(
                out,
                "| <a id=\"{}\"></a>`{}` | {} | {} | {} | {} |",
                anchor_column(&c.name),
                c.name,
                value,
                array,
                stride,
                cell(&c.doc.join(" "))
            );
        }
        out.push('\n');
        for c in columns {
            quote(&mut out, ix, &block(&ix.api.source, c.line), c.line, None);
        }
    }

    let consts: Vec<&Const> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Const(c) => Some(c),
            _ => None,
        })
        .collect();
    if !consts.is_empty() {
        out.push_str("## Constants\n\n| Constant | Type | Value | Doc |\n|---|---|---|---|\n");
        for c in &consts {
            let _ = writeln!(
                out,
                "| <a id=\"{}\"></a>`{}` | {} | {} | {} |",
                anchor_const(&c.name),
                c.name,
                code(&ts::type_text(ix, &c.ty, &[])),
                code(&crate::values::show(&c.value)),
                cell(&c.doc.join(" "))
            );
        }
        out.push('\n');
        for c in consts {
            quote(&mut out, ix, &block(&ix.api.source, c.line), c.line, None);
        }
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    (out, items)
}

fn item_doc(item: &Item) -> &[String] {
    match item {
        Item::Fn(x) => &x.doc,
        Item::Const(x) => &x.doc,
        Item::Handle(x) => &x.doc,
        Item::Struct(x) => &x.doc,
        Item::Enum(x) => &x.doc,
        Item::Union(x) => &x.doc,
        Item::Interface(x) => &x.doc,
        Item::Callback(x) => &x.doc,
        Item::Column(x) => &x.doc,
        Item::Record(x) => &x.doc,
        Item::Value(x) => &x.doc,
    }
}

/// How an item is named, what kind it is and its anchor on its page.
fn item_label(m: &Module, item: &Item) -> (String, String, String) {
    match item {
        Item::Fn(f) => {
            let path = match m.place {
                Place::Top => f.name.clone(),
                Place::Root => format!("og.{}", f.name),
                Place::Namespace => format!("og.{}.{}", m.name, f.name),
            };
            (path, "function".into(), anchor_fn(&f.name))
        }
        Item::Const(c) => (c.name.clone(), "constant".into(), anchor_const(&c.name)),
        Item::Column(c) => (
            format!("{}()", c.name),
            "column kind".into(),
            anchor_column(&c.name),
        ),
        Item::Record(r) => (
            format!("og.{}", r.name),
            r.kind.as_str().into(),
            anchor_record(&r.name),
        ),
        Item::Struct(s) => (
            s.name.clone(),
            if s.desc { "desc struct" } else { "struct" }.into(),
            anchor_type(&s.name),
        ),
        Item::Enum(e) => (e.name.clone(), "enum".into(), anchor_type(&e.name)),
        Item::Union(u) => (u.name.clone(), "union".into(), anchor_type(&u.name)),
        Item::Handle(h) => (h.name.clone(), "handle".into(), anchor_type(&h.name)),
        Item::Interface(i) => (i.name.clone(), "interface".into(), anchor_type(&i.name)),
        Item::Callback(c) => (c.name.clone(), "callback".into(), anchor_type(&c.name)),
        Item::Value(v) => (
            value_path(m, &v.name),
            "value".into(),
            anchor_value(&v.name),
        ),
    }
}

/// Where a module-level name appears in TypeScript.
fn value_path(m: &Module, name: &str) -> String {
    match m.place {
        Place::Top => name.to_string(),
        Place::Root => format!("og.{name}"),
        Place::Namespace => format!("og.{}.{name}", m.name),
    }
}

// ── the index ───────────────────────────────────────────────────────────────

fn index(ix: &Index<'_>) -> String {
    let api = ix.api;
    let mut out = String::new();
    let _ = writeln!(out, "<!-- {GENERATED_BY} -->\n");
    let _ = writeln!(out, "# The Origin Engine API\n");
    let _ = writeln!(
        out,
        "`{}`, API version {}: {} modules, generated from `crates/og-api/api.ogt`. Each module has an overview page and every function, component and type a page of its own under `<module>/`; every signature is quoted from the IDL. `og api find <words>` ranks every entry; `og api show <name>` prints one.\n",
        api.package,
        api.version,
        api.modules.len()
    );
    paragraphs(&mut out, &api.doc);

    out.push_str(
        "## Modules\n\n| Module | TypeScript | Publish builds | Summary |\n|---|---|---|---|\n",
    );
    for m in &api.modules {
        let ts_form = match m.place {
            _ if !has_fns(m) => "types and tokens".to_string(),
            Place::Top => "named exports".to_string(),
            Place::Root => "`og.*`".to_string(),
            Place::Namespace => format!("`og.{}.*`", m.name),
        };
        let _ = writeln!(
            out,
            "| [`{}`]({}.md) | {} | {} | {} |",
            m.name,
            m.name,
            ts_form,
            publish_text(m.publish),
            cell(first(&m.doc))
        );
    }
    out.push('\n');

    if !api.guides.is_empty() {
        out.push_str(
            "## Guides\n\nWorkflow, thresholds and pitfalls no signature states, written by hand beside the IDL (`crates/og-api/guides/`). `og api show <guide>` prints one; `og api find` returns their sections.\n\n| Guide | Summary |\n|---|---|\n",
        );
        for g in &api.guides {
            let _ = writeln!(
                out,
                "| [`{}`](guide-{}.md) | {} |",
                g.name,
                g.name,
                cell(&guide_summary(&g.text))
            );
        }
        out.push('\n');
    }

    if !api.units.is_empty() {
        out.push_str(
            "## Units\n\nField units the docs and Tune use.\n\n| Unit | Means |\n|---|---|\n",
        );
        for u in &api.units {
            let _ = writeln!(out, "| `{}` | {} |", u.name, cell(&u.doc.join(" ")));
        }
        out.push('\n');
    }

    while out.ends_with("\n\n") {
        out.pop();
    }
    out
}
