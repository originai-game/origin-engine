//! The built-in UI component catalogue: every component, control and screen
//! `@origingame/engine/ui/components` exports (its `index.tsx`), as one guide
//! (`crates/og-api/guides/components.md`) that `og api gen` writes from the
//! components' own source and `og api check` holds equal. Each export gets a
//! section with its doc comment, its props (the `<Name>Props` interface: each
//! prop's type and doc comment, and the fields of the row types it names),
//! the pack slots it dresses (`packClass`), the ground it is drawn on, its
//! fixtures and the input actions it answers, so `og api find health bar`
//! answers with the component that draws one.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Comment, Declaration, Expression, ObjectPropertyKind, Program,
    Statement, TSSignature, TSType,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

/// Where the catalogue lives, from the repository root.
pub const PATH: &str = "crates/og-api/guides/components.md";

/// The components' directory and the module a game imports them from.
const DIR: &str = "packages/engine/src/ui/components";
const IMPORT: &str = "@origingame/engine/ui/components";

/// One prop or field: its name, whether it may be left out, its type as
/// written and its doc comment.
#[derive(Clone)]
struct Field {
    name: String,
    optional: bool,
    ty: String,
    doc: String,
}

/// One exported component, read from its file.
struct Component {
    name: String,
    /// Its file, from the repository root.
    file: String,
    /// Where it is filed: the components' kit (`hud`), `controls` or `screens`.
    kit: String,
    doc: String,
    props: Vec<Field>,
    /// The file's other interfaces and type literals the props name.
    rows: Vec<(String, Vec<Field>)>,
    slots: BTreeSet<String>,
    panel: bool,
    fixtures: Vec<String>,
    actions: Vec<(String, String)>,
}

/// A doc comment's text: `/** … */` without its delimiters and leading
/// stars, lines joined, blank lines kept as paragraph breaks.
fn doc_text(raw: &str) -> String {
    let body = raw.trim_start_matches("/**").trim_end_matches("*/");
    let mut paras: Vec<String> = vec![String::new()];
    for line in body.lines() {
        let line = line.trim().trim_start_matches('*').trim();
        if line.is_empty() {
            if !paras.last().is_some_and(String::is_empty) {
                paras.push(String::new());
            }
            continue;
        }
        let p = paras.last_mut().expect("one paragraph");
        if !p.is_empty() {
            p.push(' ');
        }
        p.push_str(line);
    }
    paras.retain(|p| !p.is_empty() && !p.starts_with("Dropped from the .svelte:"));
    paras.join("\n\n")
}

/// The doc comment written right before the node starting at `at`.
fn doc_at(comments: &[Comment], src: &str, at: u32) -> String {
    comments
        .iter()
        .rev()
        .find(|c| c.attached_to == at && c.is_block())
        .map(|c| c.span.source_text(src))
        .filter(|t| t.starts_with("/**"))
        .map(doc_text)
        .unwrap_or_default()
}

/// Source text with its runs of white space as one space.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fields(sigs: &[TSSignature], comments: &[Comment], src: &str) -> Vec<Field> {
    sigs.iter()
        .filter_map(|s| match s {
            TSSignature::TSPropertySignature(p) => Some(Field {
                name: p.key.static_name()?.as_str()?.to_string(),
                optional: p.optional,
                ty: p
                    .type_annotation
                    .as_ref()
                    .map(|t| squash(t.type_annotation.span().source_text(src)))
                    .unwrap_or_default(),
                doc: doc_at(comments, src, p.span.start),
            }),
            TSSignature::TSMethodSignature(m) => Some(Field {
                name: m.key.static_name()?.as_str()?.to_string(),
                optional: m.optional,
                ty: squash(
                    Span::new(m.key.span().end, m.span.end)
                        .source_text(src)
                        .trim_end_matches([';', ',']),
                ),
                doc: doc_at(comments, src, m.span.start),
            }),
            _ => None,
        })
        .collect()
}

/// The `packClass("Slot", "part", "variant")` calls under a file: each as
/// `Slot.part` or `Slot.part.variant` (`…` for a variant chosen at run time).
#[derive(Default)]
struct Slots(BTreeSet<String>);

impl<'a> Visit<'a> for Slots {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(id) = &call.callee
            && id.name == "packClass"
        {
            let words: Vec<String> = call
                .arguments
                .iter()
                .map(|a| match a {
                    Argument::StringLiteral(s) => s.value.display().to_string(),
                    _ => "…".to_string(),
                })
                .collect();
            if words.len() >= 2 {
                self.0.insert(words.join("."));
            }
        }
        walk::walk_call_expression(self, call);
    }
}

/// An exported `const`'s initializer, under `as const` and `satisfies`.
fn exported_const<'p, 'a>(program: &'p Program<'a>, name: &str) -> Option<&'p Expression<'a>> {
    program.body.iter().find_map(|s| {
        let Statement::ExportDeclaration(e) = s else {
            return None;
        };
        let Declaration::VariableDeclaration(v) = &e.declaration else {
            return None;
        };
        v.declarations
            .iter()
            .find(|d| d.id.get_identifier_name().is_some_and(|n| n == name))
            .and_then(|d| d.init.as_ref())
            .map(Expression::get_inner_expression)
    })
}

fn read(root: &Path, name: &str, local: &str, rel: &str) -> Result<Option<Component>, String> {
    if !name.starts_with(|c: char| c.is_ascii_uppercase()) {
        return Ok(None);
    }
    let file = format!("{DIR}/{rel}.tsx")
        .replace("/./", "/")
        .replace("components/../", "");
    let src = std::fs::read_to_string(root.join(&file)).map_err(|e| format!("{file}: {e}"))?;
    let alloc = Allocator::default();
    let parsed = Parser::new(&alloc, &src, SourceType::tsx()).parse();
    if parsed.diagnostics.has_errors() {
        return Err(format!("{file}: does not parse"));
    }
    let program = &parsed.program;
    let comments = &program.comments[..];
    let mut found_fn = false;
    let mut doc = String::new();
    let mut props_type = String::new();
    let mut types: Vec<(String, Vec<Field>)> = Vec::new();
    for s in &program.body {
        let (decl, at) = match s {
            Statement::ExportDeclaration(e) => (&e.declaration, e.span.start),
            _ => continue,
        };
        match decl {
            Declaration::FunctionDeclaration(f)
                if f.id.as_ref().is_some_and(|i| i.name == local) =>
            {
                found_fn = true;
                doc = doc_at(comments, &src, at);
                if let Some(t) = f
                    .params
                    .items
                    .first()
                    .and_then(|p| p.type_annotation.as_ref())
                {
                    props_type = squash(t.type_annotation.span().source_text(&src));
                }
            }
            Declaration::TSInterfaceDeclaration(i) => {
                types.push((i.id.name.to_string(), fields(&i.body.body, comments, &src)));
            }
            Declaration::TSTypeAliasDeclaration(t) => {
                if let TSType::TSTypeLiteral(l) = &t.type_annotation {
                    types.push((t.id.name.to_string(), fields(&l.members, comments, &src)));
                }
            }
            _ => {}
        }
    }
    if !found_fn {
        return Ok(None);
    }
    // The interfaces are often not exported; read every one in the file.
    for s in &program.body {
        if let Statement::TSInterfaceDeclaration(i) = s
            && !types.iter().any(|(n, _)| n.as_str() == i.id.name.as_str())
        {
            types.push((i.id.name.to_string(), fields(&i.body.body, comments, &src)));
        }
        if let Statement::TSTypeAliasDeclaration(t) = s
            && let TSType::TSTypeLiteral(l) = &t.type_annotation
            && !types.iter().any(|(n, _)| n.as_str() == t.id.name.as_str())
        {
            types.push((t.id.name.to_string(), fields(&l.members, comments, &src)));
        }
    }
    let props = types
        .iter()
        .position(|(n, _)| *n == props_type)
        .map(|i| types.remove(i).1)
        .unwrap_or_default();
    // The row types the props name, in the order they first appear.
    let mut rows = Vec::new();
    for f in &props {
        for (n, fs) in &types {
            let named =
                f.ty.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .any(|w| w == n);
            if named && !rows.iter().any(|(r, _): &(String, Vec<Field>)| r == n) {
                rows.push((n.clone(), fs.iter().map(Field::clone).collect()));
            }
        }
    }
    let mut slots = Slots::default();
    slots.visit_program(program);
    let panel = matches!(exported_const(program, "ground"), Some(Expression::StringLiteral(s)) if s.value == "panel");
    let keys = |name: &str| -> Vec<(String, String)> {
        let Some(Expression::ObjectExpression(o)) = exported_const(program, name) else {
            return Vec::new();
        };
        o.properties
            .iter()
            .filter_map(|p| match p {
                ObjectPropertyKind::ObjectProperty(p) => Some((
                    p.key.static_name()?.as_str()?.to_string(),
                    match p.value.get_inner_expression() {
                        Expression::StringLiteral(s) => s.value.display().to_string(),
                        _ => String::new(),
                    },
                )),
                ObjectPropertyKind::SpreadProperty(_) => None,
            })
            .collect()
    };
    let kit = file.rsplit('/').nth(1).unwrap_or_default().to_string();
    Ok(Some(Component {
        name: name.to_string(),
        file,
        kit,
        doc,
        props,
        rows,
        slots: slots.0,
        panel,
        fixtures: keys("fixtures").into_iter().map(|(k, _)| k).collect(),
        actions: keys("actions"),
    }))
}

/// The value exports of `index.tsx`, as (exported name, local name, path from
/// the components' directory); type-only re-exports are skipped.
fn exports(root: &Path) -> Result<Vec<(String, String, String)>, String> {
    let index = format!("{DIR}/index.tsx");
    let src = std::fs::read_to_string(root.join(&index)).map_err(|e| format!("{index}: {e}"))?;
    let alloc = Allocator::default();
    let parsed = Parser::new(&alloc, &src, SourceType::tsx()).parse();
    let mut out = Vec::new();
    for s in &parsed.program.body {
        if let Statement::ExportFromDeclaration(e) = s {
            if e.export_kind.is_type() {
                continue;
            }
            for sp in &e.specifiers {
                if sp.export_kind.is_type() {
                    continue;
                }
                out.push((
                    sp.exported.name().to_string(),
                    sp.local.name().to_string(),
                    e.source.value.display().to_string(),
                ));
            }
        }
    }
    Ok(out)
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// The catalogue's text, from the components under `root`.
pub fn markdown(root: &Path) -> String {
    let mut out = String::from(
        "<!-- Generated from the built-in components' source (packages/engine/src/ui/components/index.tsx and the files it exports) by `og api gen`. Do not edit: `og api check` fails on any difference. -->\n\
# The built-in UI components\n\n\
Every component, control and screen the engine ships as TSX, one section each: what it draws, its props (from its `<Name>Props` interface, each with its type and what it means; a `?` prop may be left out and has its default in the component), the row types those props name, the pack slots it dresses, its fixtures and the input actions it answers.\n\n\
Import one by name from `@origingame/engine/ui/components`, or by its path (`@origingame/engine/ui/components/hud/stat-bar`), and mount it or render it inside your own component. Each draws what the game hands it and reports intents through its `on*` props; it never decides costs, rules or rewards. A style pack restyles a component through the slots it names (`packClass(\"Plate\", \"frame\")` in its class) and the theme's roles; to change one beyond that, copy its file (and its string table) into the game and import the copy. The `screens` guide holds the rules a screen keeps (layout, contrast, grounds, input).\n\n\
A component marked as a piece of a panel is drawn on a panel's ground (inside a `Panel.frame`), never straight over the scene.\n",
    );
    let list = match exports(root) {
        Ok(l) => l,
        Err(e) => {
            let _ = writeln!(out, "\nThe catalogue could not be read: {e}");
            return out;
        }
    };
    let mut comps: Vec<Component> = Vec::new();
    for (name, local, rel) in &list {
        match read(root, name, local, rel) {
            Ok(Some(c)) => comps.push(c),
            Ok(None) => {}
            Err(e) => {
                let _ = writeln!(out, "\n{name}: {e}");
            }
        }
    }
    // The index: each kit's components with their first sentence.
    let mut kits: Vec<&str> = comps.iter().map(|c| c.kit.as_str()).collect();
    kits.dedup();
    let mut seen = BTreeSet::new();
    kits.retain(|k| seen.insert(*k));
    out.push_str("\nBy kit (the directory a component is filed in):\n\n");
    for kit in &kits {
        let names: Vec<String> = comps
            .iter()
            .filter(|c| c.kit == *kit)
            .map(|c| format!("`{}`", c.name))
            .collect();
        let _ = writeln!(out, "- `{kit}`: {}", names.join(", "));
    }
    for (n, c) in comps.iter().enumerate() {
        let _ = write!(out, "\n## {}. {}\n\n", n + 1, c.name);
        if c.doc.is_empty() {
            out.push_str("(No doc comment.)\n");
        } else {
            let _ = writeln!(out, "{}", c.doc);
        }
        let _ = writeln!(
            out,
            "\n`import {{ {} }} from \"{IMPORT}\"` (kit `{}`, `{}`).",
            c.name, c.kit, c.file
        );
        if !c.props.is_empty() {
            out.push_str("\n| Prop | Type | What |\n|---|---|---|\n");
            for f in &c.props {
                let _ = writeln!(
                    out,
                    "| `{}{}` | `{}` | {} |",
                    f.name,
                    if f.optional { "?" } else { "" },
                    cell(&f.ty),
                    cell(&f.doc)
                );
            }
        }
        for (row, fs) in &c.rows {
            let _ = write!(
                out,
                "\nEach `{row}`:\n\n| Field | Type | What |\n|---|---|---|\n"
            );
            for f in fs {
                let _ = writeln!(
                    out,
                    "| `{}{}` | `{}` | {} |",
                    f.name,
                    if f.optional { "?" } else { "" },
                    cell(&f.ty),
                    cell(&f.doc)
                );
            }
        }
        if !c.slots.is_empty() {
            let s: Vec<String> = c.slots.iter().map(|s| format!("`{s}`")).collect();
            let _ = writeln!(out, "\nPack slots: {}.", s.join(", "));
        }
        if c.panel {
            out.push_str("\nA piece of a panel: drawn on a panel's ground.\n");
        }
        if !c.fixtures.is_empty() {
            let s: Vec<String> = c.fixtures.iter().map(|s| format!("`{s}`")).collect();
            let _ = writeln!(out, "\nFixtures: {}.", s.join(", "));
        }
        if !c.actions.is_empty() {
            let s: Vec<String> = c
                .actions
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        format!("`{k}`")
                    } else {
                        format!("`{k}` ({v})")
                    }
                })
                .collect();
            let _ = writeln!(out, "\nActions: {}.", s.join(", "));
        }
    }
    out
}
