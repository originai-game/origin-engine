//! Generated guides for the engine's TypeScript entry points the IDL does
//! not generate, and for the default UI design tokens:
//! - `crates/og-api/guides/ui-tsx.md`: every export of
//!   `@origingame/engine/ui` (`packages/engine/src/ui/index.ts`, its
//!   re-exports resolved in the modules they come from);
//! - `crates/og-api/guides/theme-tokens.md`: the default style pack's
//!   colour roles, scales, `packClass` slots, sounds, images and icons, with
//!   the utilities that read each scale as og-ui-next's class parser answers;
//! - `crates/og-api/guides/playtest.md`: every export of
//!   `@origingame/engine/playtest` (`packages/engine/src/playtest.ts`).
//!
//! A guide's sections only order and group the exports: an export no
//! section places, or a section naming what the module no longer exports,
//! fails `og api gen` and `og api check`. The text is the sources' JSDoc as
//! written.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ClassElement, Comment, Declaration, Expression, FormalParameters, Statement, TSAccessibility,
    TSSignature, TSType, TSTypeAnnotation, TSTypeParameterDeclaration, VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

pub const UI_TSX_PATH: &str = "crates/og-api/guides/ui-tsx.md";
pub const THEME_TOKENS_PATH: &str = "crates/og-api/guides/theme-tokens.md";
pub const PLAYTEST_PATH: &str = "crates/og-api/guides/playtest.md";

#[derive(Clone)]
struct Field {
    name: String,
    optional: bool,
    ty: String,
    doc: String,
}

#[derive(Clone)]
enum Kind {
    Function {
        sig: String,
    },
    Value {
        ty: String,
    },
    Const {
        value: String,
    },
    Interface {
        params: String,
        extends: String,
        fields: Vec<Field>,
    },
    TypeAlias {
        params: String,
        rhs: String,
    },
    Class {
        extends: String,
        fields: Vec<Field>,
    },
}

#[derive(Clone)]
struct Item {
    name: String,
    doc: String,
    kind: Kind,
}

/// One parsed module's text and comments.
struct Src<'s> {
    rel: &'s str,
    text: &'s str,
    comments: &'s [Comment],
}

impl Src<'_> {
    /// The JSDoc block attached to the node starting at `at` (the last one
    /// when several are).
    fn raw_doc(&self, at: u32) -> Option<&str> {
        self.comments
            .iter()
            .rev()
            .find(|c| c.attached_to == at && c.is_block())
            .map(|c| c.span.source_text(self.text))
            .filter(|t| t.starts_with("/**"))
    }

    fn doc(&self, at: u32) -> String {
        self.raw_doc(at).map(doc_text).unwrap_or_default()
    }

    /// The source of `span` on one line, its comments left out.
    fn text(&self, span: Span) -> String {
        let mut out = String::new();
        let mut at = span.start;
        for c in self
            .comments
            .iter()
            .filter(|c| c.span.start >= span.start && c.span.end <= span.end)
        {
            out.push_str(&self.text[at as usize..c.span.start as usize]);
            out.push(' ');
            at = c.span.end;
        }
        out.push_str(&self.text[at as usize..span.end as usize]);
        squash(&out)
    }

    /// `<T>(a: A): R`, or `<T>(a: A) => R` for a function type.
    fn signature(
        &self,
        type_params: Option<&TSTypeParameterDeclaration>,
        params: &FormalParameters,
        ret: Option<&TSTypeAnnotation>,
        arrow: bool,
        name: &str,
    ) -> Result<String, String> {
        let tp = type_params.map(|t| self.text(t.span)).unwrap_or_default();
        let p = self.text(params.span);
        let Some(r) = ret.map(|r| self.text(r.type_annotation.span())) else {
            return Err(format!(
                "{}: `{name}` has no return type; annotate it so the guide can state it",
                self.rel
            ));
        };
        Ok(if arrow {
            format!("{tp}{p} => {r}")
        } else {
            format!("{tp}{p}: {r}")
        })
    }
}

fn is_internal(raw: Option<&str>) -> bool {
    raw.is_some_and(|t| t.contains("@internal"))
}

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
    paras.retain(|p| !p.is_empty());
    paras.join("\n\n")
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `s` as one Markdown code span, fenced past the backticks it holds.
fn code(s: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for ch in s.chars() {
        run = if ch == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest + 1);
    let pad = if s.starts_with('`') || s.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{s}{pad}{fence}")
}

fn cell(s: &str) -> String {
    squash(s).replace('|', "\\|")
}

fn fields(sigs: &[TSSignature], s: &Src) -> Result<Vec<Field>, String> {
    let mut out = Vec::new();
    for sig in sigs {
        out.push(match sig {
            TSSignature::TSPropertySignature(p) => Field {
                name: key_name(
                    p.key
                        .static_name()
                        .and_then(|n| n.as_str().map(str::to_string)),
                    p.span,
                    s,
                )?,
                optional: p.optional,
                ty: p
                    .type_annotation
                    .as_ref()
                    .map(|t| s.text(t.type_annotation.span()))
                    .unwrap_or_else(|| "any".into()),
                doc: s.doc(p.span.start),
            },
            TSSignature::TSMethodSignature(m) => {
                let name = key_name(
                    m.key
                        .static_name()
                        .and_then(|n| n.as_str().map(str::to_string)),
                    m.span,
                    s,
                )?;
                Field {
                    ty: s.signature(
                        m.type_parameters.as_deref(),
                        &m.params,
                        m.return_type.as_deref(),
                        true,
                        &name,
                    )?,
                    name,
                    optional: m.optional,
                    doc: s.doc(m.span.start),
                }
            }
            TSSignature::TSIndexSignature(idx) => {
                let p = &idx.parameter;
                Field {
                    name: format!(
                        "[{}: {}]",
                        p.name,
                        s.text(p.type_annotation.type_annotation.span())
                    ),
                    optional: false,
                    ty: s.text(idx.type_annotation.type_annotation.span()),
                    doc: s.doc(idx.span.start),
                }
            }
            TSSignature::TSCallSignatureDeclaration(c) => Field {
                name: "(call)".into(),
                optional: false,
                ty: s.signature(
                    c.type_parameters.as_deref(),
                    &c.params,
                    c.return_type.as_deref(),
                    true,
                    "a call signature",
                )?,
                doc: s.doc(c.span.start),
            },
            TSSignature::TSConstructSignatureDeclaration(c) => {
                return Err(format!(
                    "{}: a construct signature ({}) is not documented; write a class",
                    s.rel,
                    s.text(c.span)
                ));
            }
        });
    }
    Ok(out)
}

fn key_name(name: Option<String>, span: Span, s: &Src) -> Result<String, String> {
    name.ok_or_else(|| {
        format!(
            "{}: a computed member ({}) has no name to document",
            s.rel,
            s.text(span)
        )
    })
}

/// A `host<P>("tag")` primitive's props type.
fn host_primitive_props(var: &VariableDeclaration, s: &Src) -> Option<String> {
    let init = var.declarations.first()?.init.as_ref()?;
    let Expression::CallExpression(call) = init else {
        return None;
    };
    let Expression::Identifier(id) = &call.callee else {
        return None;
    };
    if id.name != "host" {
        return None;
    }
    let arg = call.type_arguments.as_ref()?.params.first()?;
    Some(s.text(arg.span()))
}

/// The items one declaration makes, each with `doc`.
fn decl_items(decl: &Declaration, doc: String, s: &Src) -> Result<Vec<Item>, String> {
    let item = |name: &str, kind| Item {
        name: name.to_string(),
        doc: doc.clone(),
        kind,
    };
    Ok(match decl {
        Declaration::FunctionDeclaration(f) => {
            let Some(id) = &f.id else { return Ok(vec![]) };
            vec![item(
                &id.name,
                Kind::Function {
                    sig: s.signature(
                        f.type_parameters.as_deref(),
                        &f.params,
                        f.return_type.as_deref(),
                        false,
                        &id.name,
                    )?,
                },
            )]
        }
        Declaration::VariableDeclaration(v) => {
            let mut out = Vec::new();
            for d in &v.declarations {
                let Some(name) = d.id.get_identifier_name() else {
                    return Err(format!(
                        "{}: a destructured declaration ({}) has no single name to document",
                        s.rel,
                        s.text(d.span)
                    ));
                };
                let kind = if let Some(props) = host_primitive_props(v, s) {
                    Kind::Value {
                        ty: format!("(props: {props}) => UiNode"),
                    }
                } else if let Some(ann) = &d.type_annotation {
                    Kind::Value {
                        ty: s.text(ann.type_annotation.span()),
                    }
                } else {
                    match &d.init {
                        Some(Expression::ArrowFunctionExpression(a)) => Kind::Value {
                            ty: s.signature(
                                a.type_parameters.as_deref(),
                                &a.params,
                                a.return_type.as_deref(),
                                true,
                                &name,
                            )?,
                        },
                        Some(init) => Kind::Const {
                            value: s.text(init.span()),
                        },
                        None => {
                            return Err(format!(
                                "{}: `{name}` has neither a type nor a value",
                                s.rel
                            ));
                        }
                    }
                };
                out.push(item(&name, kind));
            }
            out
        }
        Declaration::TSInterfaceDeclaration(i) => vec![item(
            &i.id.name,
            Kind::Interface {
                params: i
                    .type_parameters
                    .as_ref()
                    .map(|t| s.text(t.span))
                    .unwrap_or_default(),
                extends: i
                    .extends
                    .iter()
                    .map(|h| s.text(h.span))
                    .collect::<Vec<_>>()
                    .join(", "),
                fields: fields(&i.body.body, s)?,
            },
        )],
        Declaration::TSTypeAliasDeclaration(t) => {
            let params = t
                .type_parameters
                .as_ref()
                .map(|t| s.text(t.span))
                .unwrap_or_default();
            vec![item(
                &t.id.name,
                match &t.type_annotation {
                    TSType::TSTypeLiteral(lit) => Kind::Interface {
                        params,
                        extends: String::new(),
                        fields: fields(&lit.members, s)?,
                    },
                    other => Kind::TypeAlias {
                        params,
                        rhs: s.text(other.span()),
                    },
                },
            )]
        }
        Declaration::ClassDeclaration(c) => {
            let Some(id) = &c.id else { return Ok(vec![]) };
            let mut out = Vec::new();
            for el in &c.body.body {
                match el {
                    ClassElement::PropertyDefinition(p)
                        if !p.key.is_private_identifier()
                            && !matches!(
                                p.accessibility,
                                Some(TSAccessibility::Private | TSAccessibility::Protected)
                            ) =>
                    {
                        out.push(Field {
                            name: key_name(
                                p.key
                                    .static_name()
                                    .and_then(|n| n.as_str().map(str::to_string)),
                                p.span,
                                s,
                            )?,
                            optional: p.optional,
                            ty: p
                                .type_annotation
                                .as_ref()
                                .map(|t| s.text(t.type_annotation.span()))
                                .unwrap_or_else(|| "any".into()),
                            doc: s.doc(p.span.start),
                        });
                    }
                    ClassElement::MethodDefinition(m)
                        if !m.key.is_private_identifier()
                            && !m.kind.is_constructor()
                            && !matches!(
                                m.accessibility,
                                Some(TSAccessibility::Private | TSAccessibility::Protected)
                            ) =>
                    {
                        let name = key_name(
                            m.key
                                .static_name()
                                .and_then(|n| n.as_str().map(str::to_string)),
                            m.span,
                            s,
                        )?;
                        out.push(Field {
                            ty: s.signature(
                                m.value.type_parameters.as_deref(),
                                &m.value.params,
                                m.value.return_type.as_deref(),
                                true,
                                &name,
                            )?,
                            name,
                            optional: m.optional,
                            doc: s.doc(m.span.start),
                        });
                    }
                    _ => {}
                }
            }
            vec![item(
                &id.name,
                Kind::Class {
                    extends: c
                        .heritage
                        .as_ref()
                        .map(|h| {
                            let end = h
                                .type_arguments
                                .as_ref()
                                .map_or(h.expression.span().end, |t| t.span.end);
                            s.text(Span::new(h.expression.span().start, end))
                        })
                        .unwrap_or_default(),
                    fields: out,
                },
            )]
        }
        _ => vec![],
    })
}

/// A relative module specifier `from` names, as a path under the root.
fn resolve(rel: &str, from: &str) -> Result<String, String> {
    if !from.starts_with('.') {
        return Err(format!(
            "{rel}: re-exports from `{from}`, a package; the guide documents this package's own modules"
        ));
    }
    let mut segs: Vec<&str> = rel.split('/').collect();
    segs.pop();
    for seg in from.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                segs.pop();
            }
            x => segs.push(x),
        }
    }
    Ok(segs.join("/"))
}

/// Every export of the module `rel` (values and types, `@internal` left
/// out) in source order, a re-export resolved to its declaration.
fn module_exports(root: &Path, rel: &str) -> Result<Vec<Item>, String> {
    let text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    let alloc = Allocator::default();
    let parsed = Parser::new(&alloc, &text, SourceType::ts()).parse();
    if let Some(e) = parsed.diagnostics.first() {
        return Err(format!("{rel}: {e}"));
    }
    let s = Src {
        rel,
        text: &text,
        comments: &parsed.program.comments,
    };
    let renamed = |item: &Item, name: &str| Item {
        name: name.to_string(),
        ..item.clone()
    };
    let mut locals: HashMap<String, Vec<Item>> = HashMap::new();
    for stmt in &parsed.program.body {
        if let Some(d) = stmt.as_declaration() {
            for item in decl_items(d, s.doc(d.span().start), &s)? {
                locals.entry(item.name.clone()).or_default().push(item);
            }
        }
    }
    let mut out = Vec::new();
    for stmt in &parsed.program.body {
        match stmt {
            Statement::ExportDeclaration(exp) => {
                let raw = s.raw_doc(exp.span.start);
                if is_internal(raw) {
                    continue;
                }
                let doc = raw.map(doc_text).unwrap_or_default();
                out.extend(decl_items(&exp.declaration, doc, &s)?);
            }
            Statement::ExportNamedDeclaration(exp) => {
                if is_internal(s.raw_doc(exp.span.start)) {
                    continue;
                }
                for sp in &exp.specifiers {
                    let local = sp.local.name();
                    let found = locals.get(local.as_str()).ok_or_else(|| {
                        format!("{rel}: `export {{ {local} }}` names no declaration of the module")
                    })?;
                    out.extend(found.iter().map(|i| renamed(i, &sp.exported.name())));
                }
            }
            Statement::ExportFromDeclaration(exp) => {
                if is_internal(s.raw_doc(exp.span.start)) {
                    continue;
                }
                let from = resolve(rel, &exp.source.value.display().to_string())?;
                let theirs = module_exports(root, &from)?;
                for sp in &exp.specifiers {
                    let local = sp.local.name();
                    let found: Vec<&Item> =
                        theirs.iter().filter(|i| i.name == local.as_str()).collect();
                    if found.is_empty() {
                        return Err(format!(
                            "{rel}: re-exports `{local}`, which {from} does not export"
                        ));
                    }
                    out.extend(found.into_iter().map(|i| renamed(i, &sp.exported.name())));
                }
            }
            Statement::ExportAllDeclaration(exp) => {
                if exp.exported.is_some() {
                    return Err(format!(
                        "{rel}: `export * as` a namespace is not documented; export its names"
                    ));
                }
                out.extend(module_exports(
                    root,
                    &resolve(rel, &exp.source.value.display().to_string())?,
                )?);
            }
            Statement::ExportDefaultDeclaration(_) => {
                return Err(format!(
                    "{rel}: a default export has no name to document; export it by name"
                ));
            }
            _ => {}
        }
    }
    Ok(out)
}

fn write_fields_table(out: &mut String, fields: &[Field]) {
    if fields.is_empty() {
        return;
    }
    let _ = writeln!(out, "| Field | Type | What |");
    let _ = writeln!(out, "|---|---|---|");
    for f in fields {
        let q = if f.optional { "?" } else { "" };
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            cell(&code(&format!("{}{q}", f.name))),
            cell(&code(&f.ty)),
            cell(&f.doc)
        );
    }
    out.push('\n');
}

fn blank_line(out: &mut String) {
    while !out.ends_with("\n\n") {
        out.push('\n');
    }
}

fn write_item(out: &mut String, item: &Item) {
    let doc_line = |out: &mut String| {
        if !item.doc.is_empty() {
            let _ = writeln!(out, "  {}", squash(&item.doc));
        }
    };
    let heading = |out: &mut String, title: String, fields: &[Field]| {
        blank_line(out);
        let _ = writeln!(out, "### {title}\n");
        if !item.doc.is_empty() {
            let _ = writeln!(out, "{}\n", item.doc);
        }
        write_fields_table(out, fields);
    };
    match &item.kind {
        Kind::Function { sig } => {
            let _ = writeln!(out, "- **{}**", code(&format!("{}{sig}", item.name)));
            doc_line(out);
        }
        Kind::Value { ty } => {
            let _ = writeln!(out, "- **{}**: {}", code(&item.name), code(ty));
            doc_line(out);
        }
        Kind::Const { value } => {
            let _ = writeln!(
                out,
                "- **{}**",
                code(&format!("const {} = {value}", item.name))
            );
            doc_line(out);
        }
        Kind::TypeAlias { params, rhs } => {
            let _ = writeln!(
                out,
                "- **{}** = {}",
                code(&format!("type {}{params}", item.name)),
                code(rhs)
            );
            doc_line(out);
        }
        Kind::Interface {
            params,
            extends,
            fields,
        } => {
            let mut title = code(&format!("{}{params}", item.name));
            if !extends.is_empty() {
                let _ = write!(title, " (extends {})", code(extends));
            }
            heading(out, title, fields);
        }
        Kind::Class { extends, fields } => {
            let mut title = code(&format!("class {}", item.name));
            if !extends.is_empty() {
                let _ = write!(title, " (extends {})", code(extends));
            }
            heading(out, title, fields);
        }
    }
}

/// A guide section: its title, a lead paragraph and the exports it places,
/// in order.
struct Section {
    title: &'static str,
    intro: &'static str,
    names: &'static [&'static str],
}

/// Writes `sections` as `## N.` sections of `items`; every item is placed
/// once and every name placed is exported, or it fails naming the gap.
fn place(
    out: &mut String,
    items: &[Item],
    sections: &[Section],
    source: &str,
    guide: &str,
) -> Result<(), String> {
    let mut placed = vec![false; items.len()];
    for (n, sec) in sections.iter().enumerate() {
        blank_line(out);
        let _ = writeln!(out, "## {}. {}\n", n + 1, sec.title);
        if !sec.intro.is_empty() {
            let _ = writeln!(out, "{}\n", sec.intro);
        }
        for name in sec.names {
            let mut any = false;
            for (i, item) in items.iter().enumerate() {
                if item.name != *name {
                    continue;
                }
                if placed[i] {
                    return Err(format!(
                        "{guide}: `{name}` is placed by two sections (crates/og-cli/src/ui_api_docs.rs)"
                    ));
                }
                placed[i] = true;
                any = true;
                write_item(out, item);
            }
            if !any {
                return Err(format!(
                    "{guide}: section \"{}\" places `{name}`, which {source} does not export; drop it from the section (crates/og-cli/src/ui_api_docs.rs)",
                    sec.title
                ));
            }
        }
    }
    let missing: Vec<&str> = items
        .iter()
        .zip(&placed)
        .filter(|(_, p)| !**p)
        .map(|(i, _)| i.name.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{guide}: {source} exports {} that no section places; add each to a section in crates/og-cli/src/ui_api_docs.rs (or mark it @internal)",
            missing.join(", ")
        ));
    }
    blank_line(out);
    out.pop();
    Ok(())
}

const UI_SECTIONS: &[Section] = &[
    Section {
        title: "Host primitives",
        intro: "Compiled away by `nsh-jsx` into host UI ops; every primitive accepts all `HostProps` (§5) plus its own props below.",
        names: &[
            "View",
            "Text",
            "Image",
            "Icon",
            "Scroll",
            "Line",
            "Particles",
            "Input",
        ],
    },
    Section {
        title: "Control flow, styling and mounting",
        intro: "",
        names: &[
            "Show",
            "For",
            "Switch",
            "Match",
            "ErrorBoundary",
            "packClass",
            "UiLayer",
            "mount",
            "useWorld",
            "useClock",
            "useStore",
        ],
    },
    Section {
        title: "Localization and formatting",
        intro: "Reactive helpers read inside a mounted component; they re-run automatically when the world's active locale changes.",
        names: &[
            "t",
            "formatNumber",
            "formatCompact",
            "formatDuration",
            "formatCurrency",
        ],
    },
    Section {
        title: "Signals and context",
        intro: "Fine-grained Solid-style reactive primitives.",
        names: &[
            "createSignal",
            "createMemo",
            "createEffect",
            "createRenderEffect",
            "batch",
            "untrack",
            "createRoot",
            "onCleanup",
            "createContext",
            "useContext",
            "provide",
            "Accessor",
            "Signal",
            "Context",
        ],
    },
    Section {
        title: "HostProps",
        intro: "",
        names: &["HostProps"],
    },
    Section {
        title: "UiEvent and node types",
        intro: "",
        names: &["UiNode", "UiChild", "UiEvent"],
    },
];

/// Generates `crates/og-api/guides/ui-tsx.md` from `packages/engine/src/ui/index.ts`.
pub fn ui_tsx_markdown(root: &Path) -> Result<String, String> {
    const SOURCE: &str = "packages/engine/src/ui/index.ts";
    let items = module_exports(root, SOURCE)?;
    let mut out = format!(
        "<!-- Generated by `og api gen` from `{SOURCE}` and the modules it re-exports. Do not edit. -->\n# UI TSX primitives, hooks and signals\n\n"
    );
    out.push_str(
        "Everything a game imports from `@origingame/engine/ui` when authoring custom TSX screens and HUDs (built-in components live in `og api show components --guide`, design tokens and pack slots in `og api show theme-tokens`).\n\n",
    );
    place(&mut out, &items, UI_SECTIONS, SOURCE, UI_TSX_PATH)?;
    Ok(out)
}

const PLAYTEST_SECTIONS: &[Section] = &[
    Section {
        title: "Entry points and helpers",
        intro: "",
        names: &[
            "runPlaytest",
            "recordReference",
            "PLAYTEST_CLOCK",
            "FrameUnchecked",
        ],
    },
    Section {
        title: "PlaytestDesc and PlaytestStep",
        intro: "",
        names: &[
            "PlaytestDesc",
            "PlaytestStep",
            "PlaytestRoom",
            "PlaytestBudget",
            "PlaytestCeiling",
        ],
    },
    Section {
        title: "PlaytestRun and reports",
        intro: "",
        names: &[
            "PlaytestRun",
            "PlaytestTimings",
            "PlaytestPercentiles",
            "PlaytestRestart",
            "PlaytestAssertions",
            "PlaytestViolation",
            "PlaytestLockstep",
        ],
    },
];

/// Generates `crates/og-api/guides/playtest.md` from `packages/engine/src/playtest.ts`.
pub fn playtest_markdown(root: &Path) -> Result<String, String> {
    const SOURCE: &str = "packages/engine/src/playtest.ts";
    let items = module_exports(root, SOURCE)?;
    let mut out = format!(
        "<!-- Generated by `og api gen` from `{SOURCE}`. Do not edit. -->\n# Playtest harness (`@origingame/engine/playtest`)\n\n"
    );
    out.push_str(
        "TypeScript API imported by `tests/*.og.ts` playtests run via `og test`. See `testing §1` for workflow and assertions.\n\n",
    );
    place(&mut out, &items, PLAYTEST_SECTIONS, SOURCE, PLAYTEST_PATH)?;
    Ok(out)
}

/// Utility classes that may read a theme scale, `{}` standing for a key,
/// each checked against og-ui-next's class
/// parser: the guide lists one under a scale only when the parser resolves
/// it through that scale.
const UTILITY_CANDIDATES: &[&str] = &[
    "bg-{}",
    "text-{}",
    "border-{}",
    "outline-{}",
    "ring-{}",
    "inset-ring-{}",
    "fill-{}",
    "stroke-{}",
    "decoration-{}",
    "caret-{}",
    "accent-{}",
    "from-{}",
    "via-{}",
    "to-{}",
    "shadow-{}",
    "inset-shadow-{}",
    "text-shadow-{}",
    "rounded-{}",
    "font-{}",
    "leading-{}",
    "tracking-{}",
    "blur-{}",
    "duration-{}",
    "ease-{}",
    "opacity-{}",
    "scale-{}",
    "max-w-{}",
];

/// Which of `UTILITY_CANDIDATES` read each scale of the default pack's
/// theme, by scale (`colors`, `radius`, ...), as og-ui-next's class parser
/// answers: each scale gets a key no other has, and a candidate reads the
/// scale when it is a known class with that key.
fn utilities_by_scale(theme_json: &str) -> Result<BTreeMap<String, Vec<&'static str>>, String> {
    use og_ui_next::classes::Classes;
    const PROBE: &str = "ogprobe";
    let theme: serde_json::Map<String, serde_json::Value> = serde_json::from_str(theme_json)
        .map_err(|e| format!("the default pack's theme JSON: {e}"))?;
    let known = |theme: &serde_json::Map<String, serde_json::Value>, class: &str| {
        let json = serde_json::Value::Object(theme.clone()).to_string();
        Classes::new(Some(&json)).map(|(mut c, _)| c.unknown(class).is_empty())
    };
    let class = |pattern: &str, key: &str| pattern.replace("{}", key);
    if known(&theme, &class("bg-{}", "not-a-token-of-any-scale"))? {
        return Err("og-ui-next's class parser accepts an unknown token; the utilities by scale cannot be derived".into());
    }
    let mut out: BTreeMap<String, Vec<&'static str>> = BTreeMap::new();
    for (scale, values) in &theme {
        let Some(first) = values.as_object().and_then(|o| o.values().next()) else {
            continue;
        };
        let mut probe = theme.clone();
        probe
            .get_mut(scale)
            .and_then(serde_json::Value::as_object_mut)
            .expect("an object scale")
            .insert(PROBE.into(), first.clone());
        let mut reads = Vec::new();
        for &p in UTILITY_CANDIDATES {
            if known(&probe, &class(p, PROBE))? {
                reads.push(p);
            }
        }
        out.insert(scale.clone(), reads);
    }
    let unused: Vec<&str> = UTILITY_CANDIDATES
        .iter()
        .copied()
        .filter(|p| !out.values().any(|r| r.contains(p)))
        .collect();
    if !unused.is_empty() {
        return Err(format!(
            "og-ui-next's class parser reads no scale of the default pack through {}; drop them from UTILITY_CANDIDATES (crates/og-cli/src/ui_api_docs.rs)",
            unused.join(", ")
        ));
    }
    Ok(out)
}

fn utility_list(utilities: &[&str], what: &str) -> String {
    utilities
        .iter()
        .map(|u| code(&u.replace("{}", &format!("<{what}>"))))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generates `crates/og-api/guides/theme-tokens.md` from
/// `crates/og-ui-next/default/pack.json`, og-ui-next's fallback icons and
/// class parser, and `templates/styles/shared/icons/game.ogt`.
pub fn theme_tokens_markdown(root: &Path) -> Result<String, String> {
    use og_ui_next::pack::{Layer, Pack, Scheme};
    const PACK: &str = "crates/og-ui-next/default/pack.json";
    const GAME_ICONS: &str = "templates/styles/shared/icons/game.ogt";
    let pack_text = std::fs::read_to_string(root.join(PACK)).map_err(|e| format!("{PACK}: {e}"))?;
    let pack: serde_json::Value =
        serde_json::from_str(&pack_text).map_err(|e| format!("{PACK}: {e}"))?;
    let section = |key: &str| {
        pack.get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("{PACK}: no `{key}` object"))
    };
    let utilities =
        utilities_by_scale(&Pack::new(&[Layer::default_pack()]).theme_json(Scheme::Dark))?;
    let reads = |scale: &str| utilities.get(scale).map(Vec::as_slice).unwrap_or_default();

    let mut out = format!(
        "<!-- Generated by `og api gen` from `{PACK}`, og-ui-next's fallback icons and class parser, and `{GAME_ICONS}`. Do not edit. -->\n# UI theme tokens, pack slots and built-in icons\n\n"
    );
    out.push_str(
        "Every design token, `packClass` slot, pack sound/image key, and built-in icon shipped in the engine's default UI style pack. Use these tokens in Tailwind utility classes so screens restyle cleanly across all built-in themes (`og api show themes`).\n\n",
    );

    // 1. Colour roles
    let colors = section("theme")?
        .get("colors")
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("{PACK}: no `theme.colors` object"))?;
    out.push_str("## 1. Colour roles\n\n");
    let _ = writeln!(out, "Use with {}.\n", utility_list(reads("colors"), "role"));
    let mut by_prefix: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for k in colors.keys() {
        let group = k.split('-').next().unwrap_or(k);
        by_prefix.entry(group).or_default().push(k);
    }
    let _ = writeln!(out, "Total: {} semantic colour tokens.\n", colors.len());
    let _ = writeln!(out, "| Group | Tokens |");
    let _ = writeln!(out, "|---|---|");
    for (group, items) in by_prefix {
        let list: Vec<String> = items.iter().map(|k| code(k)).collect();
        let _ = writeln!(out, "| {} | {} |", code(group), list.join(", "));
    }
    out.push('\n');

    // 2. Theme scales
    out.push_str("## 2. Theme scales\n\n");
    out.push_str(
        "Scale tokens under `theme` in `pack.json`, with the utility classes that read each.\n\n",
    );
    let _ = writeln!(out, "| Scale | Utilities | Keys |");
    let _ = writeln!(out, "|---|---|---|");
    for (cat, val) in section("theme")? {
        if cat == "colors" {
            continue;
        }
        if let Some(obj) = val.as_object() {
            let keys: Vec<String> = obj.keys().map(|k| code(k)).collect();
            let used = utility_list(reads(cat), "key");
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                code(cat),
                if used.is_empty() { "—".into() } else { used },
                keys.join(", ")
            );
        }
    }
    out.push('\n');

    // 3. Pack slots
    out.push_str("## 3. Pack slot classes\n\n");
    out.push_str(
        "Pass component, slot, and optional variant to `packClass(component, slot, variant)` from `@origingame/engine/ui` (resolved as `@Component.slot` or `@Component.slot.variant`).\n\n",
    );
    let variants = section("variants")?;
    let mut by_comp: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for key in variants.keys() {
        let comp = key.split('.').next().unwrap_or(key);
        by_comp
            .entry(comp)
            .or_default()
            .push(code(&format!("@{key}")));
    }
    let _ = writeln!(
        out,
        "Total: {} pack slots across {} component groups.\n",
        variants.len(),
        by_comp.len()
    );
    let _ = writeln!(out, "| Component | Slots |");
    let _ = writeln!(out, "|---|---|");
    for (comp, slots) in by_comp {
        let _ = writeln!(out, "| {} | {} |", code(comp), slots.join(", "));
    }
    out.push('\n');

    // 4. Pack sounds and images
    out.push_str("## 4. Pack sounds and images\n\n");
    let sounds = section("sounds")?;
    let events: Vec<String> = sounds
        .keys()
        .filter(|k| !k.contains('.'))
        .map(|k| code(k))
        .collect();
    let slot_overrides: Vec<String> = sounds
        .keys()
        .filter(|k| k.contains('.'))
        .map(|k| code(&format!("@{k}")))
        .collect();
    let _ = writeln!(
        out,
        "- **Sound events** ({}): {}",
        events.len(),
        events.join(", ")
    );
    if !slot_overrides.is_empty() {
        let _ = writeln!(
            out,
            "- **Slot sound overrides** ({}): {}",
            slot_overrides.len(),
            slot_overrides.join(", ")
        );
    }
    let names: Vec<String> = section("images")?.keys().map(|k| code(k)).collect();
    let _ = writeln!(
        out,
        "- **Pack images (`nine-<name>`, `bg-image-<name>`)** ({}): {}",
        names.len(),
        names.join(", ")
    );
    out.push('\n');

    // 5. Built-in icons
    out.push_str("## 5. Built-in icons\n\n");
    out.push_str(
        "Use with `<Icon name=\"...\" />` from `@origingame/engine/ui`. Check availability at runtime with `og.ui.hasIcon`.\n\n",
    );
    let fallback: Vec<String> = og_ui_next::default_icons::DEFAULT_ICON_NAMES
        .iter()
        .map(|n| code(n))
        .collect();
    let _ = writeln!(
        out,
        "- **Always-available vector fallback icons** ({}): {}\n",
        fallback.len(),
        fallback.join(", ")
    );
    let list: Vec<String> = section("icons")?.keys().map(|k| code(k)).collect();
    let _ = writeln!(
        out,
        "- **Default UI pack icons** ({}): {}\n",
        list.len(),
        list.join(", ")
    );
    let text =
        std::fs::read_to_string(root.join(GAME_ICONS)).map_err(|e| format!("{GAME_ICONS}: {e}"))?;
    let doc = og_core::ogt::Document::parse(&text).map_err(|e| format!("{GAME_ICONS}: {e}"))?;
    let game_icons: Vec<String> = doc
        .body
        .entries
        .iter()
        .filter(|e| e.key == "icon")
        .filter_map(|e| e.values.first().and_then(|v| v.as_str()))
        .map(code)
        .collect();
    if game_icons.is_empty() {
        return Err(format!("{GAME_ICONS}: no `icon \"<name>\"` entries"));
    }
    let _ = writeln!(
        out,
        "- **Shared game icon set (`@styles/shared/icons/game.ogt`)** ({}): {}",
        game_icons.len(),
        game_icons.join(", ")
    );
    Ok(out)
}
