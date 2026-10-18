//! The wiki generator (design §3.1): `packages/knowledge/wiki/`, one page per
//! module plus `index.md`, the map from every module, type and function to
//! its page. Every declaration is quoted from `api.ogt` exactly as written,
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
    let mut out = vec![("index.md".to_string(), index(ix, &binds))];
    for m in &ix.api.modules {
        out.push((format!("{}.md", m.name), module_page(ix, m, &binds)));
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
fn quote(out: &mut String, lines: &[&str], line: usize, more: Option<&str>) {
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
    let _ = writeln!(out, "\n<sub>`api.ogt` line {line}</sub>\n");
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

fn paragraphs(out: &mut String, doc: &[String]) {
    for d in doc {
        out.push_str(d);
        out.push_str("\n\n");
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

fn fields_table(out: &mut String, ix: &Index<'_>, fields: &[Field]) {
    if fields.is_empty() {
        return;
    }
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
            cell(&field_notes(f)),
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
    quote(out, &block(&ix.api.source, f.line), f.line, None);
}

fn interface(out: &mut String, ix: &Index<'_>, m: &Module, i: &Interface, binds: &[Binding<'_>]) {
    let _ = writeln!(
        out,
        "<a id=\"{}\"></a>\n\n### `{}` (interface)\n",
        anchor_type(&i.name),
        i.name
    );
    paragraphs(out, &i.doc);
    if let Some(base) = &i.extends {
        let page = ix
            .module_of(base)
            .map_or(String::new(), |bm| format!("{}.md", bm.name));
        let link = if ix.module_of(base).map(|bm| &bm.name) == Some(&m.name) {
            String::new()
        } else {
            page
        };
        let _ = writeln!(
            out,
            "Extends [`{base}`]({link}#{}): every member of it is a member here.\n",
            anchor_type(base)
        );
    }
    quote(
        out,
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
                quote(out, &block(&ix.api.source, p.line), p.line, None);
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
    let _ = writeln!(
        out,
        "Token `og.{}`, registered under the schema name `{}`; its value type is `{}`.\n",
        r.name,
        schema_name(&r.name),
        r.name
    );
    fields_table(out, ix, &r.fields);
    quote(out, &block(&ix.api.source, r.line), r.line, None);
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
            if s.desc {
                out.push_str(
                    "A desc: every field may be omitted and takes its default; `{}` is valid.\n\n",
                );
            }
            fields_table(out, ix, &s.fields);
            quote(out, &block(src, s.line), s.line, None);
        }
        Item::Enum(e) => {
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` (enum)\n",
                anchor_type(&e.name),
                e.name
            );
            paragraphs(out, &e.doc);
            let _ = writeln!(
                out,
                "A string: one of the variants below. The zero value is `\"{}\"`.\n",
                e.variants.first().map_or("", |v| v.name.as_str())
            );
            out.push_str("| Variant | Doc |\n|---|---|\n");
            for v in &e.variants {
                let _ = writeln!(out, "| `\"{}\"` | {} |", v.name, cell(&v.doc.join(" ")));
            }
            out.push('\n');
            quote(out, &block(src, e.line), e.line, None);
        }
        Item::Union(u) => {
            let _ = writeln!(
                out,
                "<a id=\"{}\"></a>\n\n### `{}` (union)\n",
                anchor_type(&u.name),
                u.name
            );
            paragraphs(out, &u.doc);
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
                        |p| format!("[`{p}`](#{})", anchor_type(p))
                    ),
                    cell(&c.doc.join(" "))
                );
            }
            out.push('\n');
            quote(out, &block(src, u.line), u.line, None);
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
            quote(out, &block(src, h.line), h.line, None);
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
            quote(out, &block(src, c.line), c.line, None);
        }
        _ => {}
    }
}

fn module_page(ix: &Index<'_>, m: &Module, binds: &[Binding<'_>]) -> String {
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
    out.push_str("- **Index**: [every module and name](index.md).\n\n");
    quote(
        &mut out,
        &header(&ix.api.source, m.line),
        m.line,
        Some("declarations follow, each quoted below"),
    );

    // Contents, in declaration order.
    out.push_str("## Contents\n\n| Name | Kind | Summary |\n|---|---|---|\n");
    for item in &m.items {
        let (label, kind, anchor) = item_label(m, item);
        let doc = item_doc(item);
        let _ = writeln!(
            out,
            "| [`{label}`](#{anchor}) | {kind} | {} |",
            cell(first(doc))
        );
    }
    out.push('\n');

    let fns: Vec<&Function> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f),
            _ => None,
        })
        .collect();
    if !fns.is_empty() {
        out.push_str("## Functions\n\n");
        for f in fns {
            let bind = find(binds, &m.name, None, &f.name);
            let path = bind.map_or_else(|| value_path(m, &f.name), |b| b.path.clone());
            callable(&mut out, ix, f, &path, bind, &anchor_fn(&f.name), "###");
        }
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
            quote(&mut out, &block(&ix.api.source, v.line), v.line, None);
        }
    }

    let records: Vec<&Record> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Record(r) => Some(r),
            _ => None,
        })
        .collect();
    if !records.is_empty() {
        out.push_str("## Components, resources and events\n\n");
        for r in records {
            record(&mut out, ix, m, r, binds);
        }
    }

    let interfaces: Vec<&Interface> = m
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Interface(x) => Some(x),
            _ => None,
        })
        .collect();
    if !interfaces.is_empty() {
        out.push_str("## Interfaces\n\n");
        for i in interfaces {
            interface(&mut out, ix, m, i, binds);
        }
    }

    let types: Vec<&Item> = m
        .items
        .iter()
        .filter(|i| {
            matches!(
                i,
                Item::Struct(_)
                    | Item::Enum(_)
                    | Item::Union(_)
                    | Item::Handle(_)
                    | Item::Callback(_)
            )
        })
        .collect();
    if !types.is_empty() {
        out.push_str("## Types\n\n");
        for t in types {
            type_item(&mut out, ix, t);
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
        out.push_str(
            "(value?, options?)`, that makes one field of a component or resource layout.\n\n",
        );
        out.push_str("| Column | Value | Typed array | Stride | Doc |\n|---|---|---|---|---|\n");
        for c in &columns {
            let _ = writeln!(
                out,
                "| <a id=\"{}\"></a>`{}` | {} | `{}` | {} | {} |",
                anchor_column(&c.name),
                c.name,
                code(&ts::type_text(ix, &c.value, &[])),
                c.array,
                c.stride,
                cell(&c.doc.join(" "))
            );
        }
        out.push('\n');
        for c in columns {
            quote(&mut out, &block(&ix.api.source, c.line), c.line, None);
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
            quote(&mut out, &block(&ix.api.source, c.line), c.line, None);
        }
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    out
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

fn index(ix: &Index<'_>, binds: &[Binding<'_>]) -> String {
    let api = ix.api;
    let mut out = String::new();
    let _ = writeln!(out, "<!-- {GENERATED_BY} -->\n");
    let _ = writeln!(out, "# The Origin Engine API\n");
    let _ = writeln!(
        out,
        "`{}`, API version {}: {} modules, generated from `crates/og-api/api.ogt`. Each module has one page; every signature on it is quoted from the IDL.\n",
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

    if !api.units.is_empty() {
        out.push_str(
            "## Units\n\nField units the docs and Tune use.\n\n| Unit | Means |\n|---|---|\n",
        );
        for u in &api.units {
            let _ = writeln!(out, "| `{}` | {} |", u.name, cell(&u.doc.join(" ")));
        }
        out.push('\n');
    }

    // Every name: items, interface members and resource methods.
    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    for m in &api.modules {
        for item in &m.items {
            let (label, kind, anchor) = item_label(m, item);
            let symbol = match item {
                Item::Fn(f) => find(binds, &m.name, None, &f.name)
                    .map(|b| b.symbol.clone())
                    .unwrap_or_default(),
                _ => String::new(),
            };
            rows.push((label, kind, format!("{}.md#{anchor}", m.name), symbol));
            let members: Vec<(&str, &str, &str)> = match item {
                Item::Interface(i) => i
                    .members
                    .iter()
                    .map(|mem| {
                        (
                            i.name.as_str(),
                            mem.name(),
                            if matches!(mem, Member::Prop(_)) {
                                "property"
                            } else {
                                "method"
                            },
                        )
                    })
                    .collect(),
                Item::Record(r) => r
                    .methods
                    .iter()
                    .map(|f| (r.name.as_str(), f.name.as_str(), "method"))
                    .collect(),
                _ => Vec::new(),
            };
            for (owner, name, kind) in members {
                let symbol = find(binds, &m.name, Some(owner), name)
                    .map(|b| b.symbol.clone())
                    .unwrap_or_default();
                rows.push((
                    format!("{owner}.{name}"),
                    kind.to_string(),
                    format!("{}.md#{}", m.name, anchor_member(owner, name)),
                    symbol,
                ));
            }
        }
    }
    rows.sort_by(|a, b| {
        a.0.to_lowercase()
            .cmp(&b.0.to_lowercase())
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });
    let _ = writeln!(
        out,
        "## Every name\n\n{} entries: modules' functions, types, records, column kinds and constants, and every interface member and resource method.\n",
        rows.len()
    );
    out.push_str("| Name | Kind | Page | Binding symbol |\n|---|---|---|---|\n");
    for (label, kind, link, symbol) in rows {
        let module = link.split('.').next().unwrap_or("");
        let _ = writeln!(
            out,
            "| [`{label}`]({link}) | {kind} | {module} | {} |",
            code(&symbol)
        );
    }
    out
}
