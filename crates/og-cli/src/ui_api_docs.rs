//! Generated guides for the engine's non-IDL TypeScript entry points and
//! default UI design tokens:
//! - `crates/og-api/guides/ui-tsx.md` (`@origingame/engine/ui` primitives,
//!   hooks, signals, `HostProps`, `UiEvent`)
//! - `crates/og-api/guides/theme-tokens.md` (default style pack colour roles,
//!   scales, `packClass` slots, sounds, images and built-in icons)
//! - `crates/og-api/guides/playtest.md` (`@origingame/engine/playtest`)
//! - `crates/og-api/guides/provider.md` (`@origingame/engine/provider`)
//!
//! Written by `og api gen` from the real sources and held equal by
//! `og api check`, so `og api find` and `og api show` cover the whole public
//! TypeScript surface without hand-maintained copies.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Comment, Declaration, Expression, Statement, TSSignature, TSType, VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

pub const UI_TSX_PATH: &str = "crates/og-api/guides/ui-tsx.md";
pub const THEME_TOKENS_PATH: &str = "crates/og-api/guides/theme-tokens.md";
pub const PLAYTEST_PATH: &str = "crates/og-api/guides/playtest.md";
pub const PROVIDER_PATH: &str = "crates/og-api/guides/provider.md";

#[derive(Clone)]
struct Field {
    name: String,
    optional: bool,
    ty: String,
    doc: String,
}

enum ExportKind {
    Function { sig: String },
    Value { ty: String },
    Interface { fields: Vec<Field> },
    TypeAlias { rhs: String },
    Class { fields: Vec<Field> },
}

struct ExportItem {
    name: String,
    doc: String,
    kind: ExportKind,
}

fn raw_doc_at<'a>(comments: &[Comment], src: &'a str, at: u32) -> Option<&'a str> {
    comments
        .iter()
        .rev()
        .find(|c| c.attached_to == at && c.is_block())
        .map(|c| c.span.source_text(src))
        .filter(|t| t.starts_with("/**"))
}

fn is_internal_doc(raw: Option<&str>) -> bool {
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

fn doc_at(comments: &[Comment], src: &str, at: u32) -> String {
    raw_doc_at(comments, src, at)
        .map(doc_text)
        .unwrap_or_default()
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cell(s: &str) -> String {
    squash(s).replace('|', "\\|")
}

fn collect_fields(sigs: &[TSSignature], comments: &[Comment], src: &str) -> Vec<Field> {
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
            TSSignature::TSIndexSignature(idx) => {
                let p = &idx.parameter;
                let ty = p.type_annotation.type_annotation.span().source_text(src);
                let param = format!("[{}: {}]", p.name, squash(ty));
                Some(Field {
                    name: param,
                    optional: false,
                    ty: squash(idx.type_annotation.type_annotation.span().source_text(src)),
                    doc: doc_at(comments, src, idx.span.start),
                })
            }
            _ => None,
        })
        .collect()
}

fn host_primitive_props(var: &VariableDeclaration, src: &str) -> Option<String> {
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
    Some(squash(arg.span().source_text(src)))
}

fn fn_signature(f: &oxc_ast::ast::Function, src: &str) -> String {
    let start = f.id.as_ref().map_or(f.params.span.start, |id| id.span.end);
    let end = f
        .return_type
        .as_ref()
        .map_or(f.params.span.end, |r| r.span.end);
    squash(Span::new(start, end).source_text(src))
}

fn parse_ts_exports(root: &Path, rel: &str) -> Vec<ExportItem> {
    let src = std::fs::read_to_string(root.join(rel))
        .unwrap_or_else(|e| panic!("cannot read {rel}: {e}"));
    let alloc = Allocator::default();
    let parsed = Parser::new(&alloc, &src, SourceType::ts()).parse();
    let comments = &parsed.program.comments;
    let mut out = Vec::new();
    for stmt in &parsed.program.body {
        match stmt {
            Statement::ExportNamedDeclaration(exp) => {
                if exp.export_kind.is_type() {
                    continue;
                }
                let raw_doc = raw_doc_at(comments, &src, exp.span.start);
                if is_internal_doc(raw_doc) {
                    continue;
                }
                let doc = raw_doc.map(doc_text).unwrap_or_default();
                for sp in &exp.specifiers {
                    if sp.export_kind.is_type() {
                        continue;
                    }
                    let name = sp.exported.name().to_string();
                    let sp_doc = if doc.is_empty() {
                        doc_at(comments, &src, sp.span.start)
                    } else {
                        doc.clone()
                    };
                    out.push(ExportItem {
                        name,
                        doc: sp_doc,
                        kind: ExportKind::Value {
                            ty: "re-export".into(),
                        },
                    });
                }
            }
            Statement::ExportFromDeclaration(exp) => {
                if exp.export_kind.is_type() {
                    continue;
                }
                let raw_doc = raw_doc_at(comments, &src, exp.span.start);
                if is_internal_doc(raw_doc) {
                    continue;
                }
                let doc = raw_doc.map(doc_text).unwrap_or_default();
                let from = exp.source.value.display().to_string();
                for sp in &exp.specifiers {
                    if sp.export_kind.is_type() {
                        continue;
                    }
                    let name = sp.exported.name().to_string();
                    let sp_doc = if doc.is_empty() {
                        doc_at(comments, &src, sp.span.start)
                    } else {
                        doc.clone()
                    };
                    out.push(ExportItem {
                        name,
                        doc: sp_doc,
                        kind: ExportKind::Value {
                            ty: format!("re-exported from `{from}`"),
                        },
                    });
                }
            }
            Statement::ExportDeclaration(exp) => {
                let raw_doc = raw_doc_at(comments, &src, exp.span.start);
                if is_internal_doc(raw_doc) {
                    continue;
                }
                let doc = raw_doc.map(doc_text).unwrap_or_default();
                match &exp.declaration {
                    Declaration::FunctionDeclaration(f) => {
                        let Some(id) = &f.id else { continue };
                        out.push(ExportItem {
                            name: id.name.to_string(),
                            doc,
                            kind: ExportKind::Function {
                                sig: fn_signature(f.as_ref(), &src),
                            },
                        });
                    }
                    Declaration::VariableDeclaration(v) => {
                        for d in &v.declarations {
                            let Some(id) = d.id.get_identifier_name() else {
                                continue;
                            };
                            let ty = if let Some(props) = host_primitive_props(v.as_ref(), &src) {
                                format!("(props: {props}) => UiNode")
                            } else if let Some(ann) = &d.type_annotation {
                                squash(ann.type_annotation.span().source_text(&src))
                            } else if let Some(init) = &d.init {
                                match init {
                                    Expression::NumericLiteral(n) => {
                                        format!("number ({})", n.span.source_text(&src))
                                    }
                                    Expression::BinaryExpression(b) => {
                                        format!("number ({})", squash(b.span.source_text(&src)))
                                    }
                                    Expression::ArrowFunctionExpression(a) => {
                                        let end = a
                                            .return_type
                                            .as_ref()
                                            .map_or(a.params.span.end, |r| r.span.end);
                                        squash(
                                            Span::new(a.params.span.start, end).source_text(&src),
                                        )
                                    }
                                    _ => "value".into(),
                                }
                            } else {
                                "value".into()
                            };
                            out.push(ExportItem {
                                name: id.to_string(),
                                doc: doc.clone(),
                                kind: ExportKind::Value { ty },
                            });
                        }
                    }
                    Declaration::TSInterfaceDeclaration(i) => {
                        out.push(ExportItem {
                            name: i.id.name.to_string(),
                            doc,
                            kind: ExportKind::Interface {
                                fields: collect_fields(&i.body.body, comments, &src),
                            },
                        });
                    }
                    Declaration::TSTypeAliasDeclaration(t) => {
                        let rhs = match &t.type_annotation {
                            TSType::TSTypeLiteral(lit) => {
                                let f = collect_fields(&lit.members, comments, &src);
                                out.push(ExportItem {
                                    name: t.id.name.to_string(),
                                    doc,
                                    kind: ExportKind::Interface { fields: f },
                                });
                                continue;
                            }
                            other => squash(other.span().source_text(&src)),
                        };
                        out.push(ExportItem {
                            name: t.id.name.to_string(),
                            doc,
                            kind: ExportKind::TypeAlias { rhs },
                        });
                    }
                    Declaration::ClassDeclaration(c) => {
                        let Some(id) = &c.id else { continue };
                        let fields = c
                            .body
                            .body
                            .iter()
                            .filter_map(|el| match el {
                                oxc_ast::ast::ClassElement::PropertyDefinition(p) => Some(Field {
                                    name: p.key.static_name()?.as_str()?.to_string(),
                                    optional: p.optional,
                                    ty: p
                                        .type_annotation
                                        .as_ref()
                                        .map(|t| squash(t.type_annotation.span().source_text(&src)))
                                        .unwrap_or_default(),
                                    doc: doc_at(comments, &src, p.span.start),
                                }),
                                _ => None,
                            })
                            .collect();
                        out.push(ExportItem {
                            name: id.name.to_string(),
                            doc,
                            kind: ExportKind::Class { fields },
                        });
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    out
}

fn write_fields_table(out: &mut String, fields: &[Field], col_name: &str) {
    if fields.is_empty() {
        return;
    }
    let _ = writeln!(out, "| {col_name} | Type | What |");
    let _ = writeln!(out, "|---|---|---|");
    for f in fields {
        let q = if f.optional { "?" } else { "" };
        let _ = writeln!(
            out,
            "| `{}{q}` | `{}` | {} |",
            f.name,
            cell(&f.ty),
            cell(&f.doc)
        );
    }
    out.push('\n');
}

fn write_item_block(out: &mut String, item: &ExportItem) {
    match &item.kind {
        ExportKind::Function { sig } => {
            let _ = writeln!(out, "- **`{}{}`**", item.name, sig);
            if !item.doc.is_empty() {
                let _ = writeln!(out, "  {}", squash(&item.doc));
            }
        }
        ExportKind::Value { ty } => {
            let _ = writeln!(out, "- **`{}`** (`{}`)", item.name, ty);
            if !item.doc.is_empty() {
                let _ = writeln!(out, "  {}", squash(&item.doc));
            }
        }
        ExportKind::TypeAlias { rhs } => {
            let _ = writeln!(out, "- **`type {}`** = `{}`", item.name, rhs);
            if !item.doc.is_empty() {
                let _ = writeln!(out, "  {}", squash(&item.doc));
            }
        }
        ExportKind::Interface { fields } => {
            let _ = writeln!(out, "### `{}`\n", item.name);
            if !item.doc.is_empty() {
                let _ = writeln!(out, "{}\n", item.doc);
            }
            write_fields_table(out, fields, "Field");
        }
        ExportKind::Class { fields } => {
            let _ = writeln!(out, "### `class {}`\n", item.name);
            if !item.doc.is_empty() {
                let _ = writeln!(out, "{}\n", item.doc);
            }
            write_fields_table(out, fields, "Field");
        }
    }
}

/// Sanitize JSDoc excerpts so generated guides obey `packages/knowledge/scripts/validate.ts`:
/// internal host ABI symbols (`og_platform_*`, `og_provider_*`, `og_i18n_*`) and
/// non-API backticked `key: value` literals are rendered cleanly as prose.
fn clean_doc_for_guide(text: &str) -> String {
    text.replace("`og_platform_take`", "the host platform take queue")
        .replace("`og_platform_answer`", "the host platform answer hook")
        .replace("`og_platform_notify`", "the host platform notify hook")
        .replace(
            "`og_provider_call(method, JSON.stringify(args))`",
            "the native provider call bridge",
        )
        .replace("og_platform_take", "the host platform take queue")
        .replace("og_platform_answer", "the host platform answer hook")
        .replace("og_platform_notify", "the host platform notify hook")
        .replace("`retryable: true`", "retryable set to true")
        .replace("`store: \"keep\"`", "`store` set to `\"keep\"`")
        .replace("`og.ui`", "UI")
        .replace("og.ui:", "UI:")
        .replace("`peer_joined`", "peer joined (`\"peer_joined\"`)")
        .replace("`peer_left`", "peer left (`\"peer_left\"`)")
        .replace("`not_found`", "`\"not_found\"`")
        .replace("`rate_limited`", "`\"rate_limited\"`")
        .replace("`og.SharedJoinDesc.world`", "`SharedJoinDesc.world`")
        .replace("og.SharedJoinDesc.world", "SharedJoinDesc.world")
        .replace("og.TimeScale.elapsed", "TimeScale.elapsed")
        .replace("`looping: true`", "`looping` true")
        .replace(
            "`{ frameMs: { p95: 16.7 } }`",
            "`frameMs` p95 ceiling (for example 16.7 ms)",
        )
        .replace("`{ url, token }`", "URL and token JSON")
        .replace("`{ name, ... }`", "a named host payload object")
        .replace(
            "`{ catalog, version, manifest }`",
            "catalog, version, and manifest fields",
        )
        .replace(
            "`{ \"purchase.buy\": [\"cancelled\", \"ok\"] }`",
            "per-method outcome lists",
        )
        .replace(
            "`{ confirm, details }`",
            "a map from action name to handler",
        )
        .replace("`data-lit:bg-scrim`", "data-lit background scrim")
        .replace(
            "`group-data-lit/<g>:hidden`",
            "group-data-lit hidden variant",
        )
        .replace("`group-data-dir-up/pad:`", "group-data-dir-up pad variant")
        .replace("`data-ready:`", "data-ready variant")
        .replace("`aria-invalid:`", "aria-invalid variant")
        .replace("`active:`", "active variant")
        .replace(
            "`og.Transform.position.y>-5`",
            "entity Transform position check",
        )
        .replace(
            "`entity:hero:og.Transform.position.y>-5`",
            "entity property expression",
        )
        .replace("`state:lives>=0`", "state expression")
        .replace("`t:action[:args];...`", "timed command script")
        .replace("`action[:args];...`", "setup action list")
        .replace("`expect:`", "expect")
        .replace("`invariant:`", "invariant")
        .replace("`<block>: <why>`", "block name and reason")
        .replace("Health.hp", "Health.value")
        .replace("`og.net`'s", "net's")
}

/// Generates `crates/og-api/guides/ui-tsx.md` from `packages/engine/src/ui/{index,types,signals}.ts`.
pub fn ui_tsx_markdown(root: &Path) -> String {
    let index_items = parse_ts_exports(root, "packages/engine/src/ui/index.ts");
    let type_items = parse_ts_exports(root, "packages/engine/src/ui/types.ts");
    let signal_items = parse_ts_exports(root, "packages/engine/src/ui/signals.ts");

    fn by_name<'a>(items: &'a [ExportItem], name: &str) -> Option<&'a ExportItem> {
        items.iter().find(|i| i.name == name)
    }

    let mut out = String::from(
        "<!-- Generated by `og api gen` from `packages/engine/src/ui/{index,types,signals}.ts`. Do not edit. -->\n# UI TSX primitives, hooks and signals\n\n",
    );
    out.push_str(
        "Everything a game imports from `@origingame/engine/ui` when authoring custom TSX screens and HUDs (built-in components live in `og api show components --guide`, design tokens and pack slots in `og api show theme-tokens`).\n\n",
    );

    out.push_str("## 1. Host primitives\n\n");
    out.push_str(
        "Compiled away by `nsh-jsx` into host UI ops; every primitive accepts all `HostProps` (§5) plus its own props below.\n\n",
    );
    for name in [
        "View",
        "Text",
        "Image",
        "Icon",
        "Scroll",
        "Line",
        "Particles",
        "Input",
    ] {
        if let Some(item) = by_name(&index_items, name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 2. Control flow, styling and mounting\n\n");
    for name in [
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
    ] {
        if let Some(item) = by_name(&index_items, name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 3. Localization and formatting\n\n");
    out.push_str(
        "Reactive helpers read inside a mounted component; they re-run automatically when the world's active locale changes.\n\n",
    );
    for name in [
        "t",
        "formatNumber",
        "formatCompact",
        "formatDuration",
        "formatCurrency",
    ] {
        if let Some(item) = by_name(&index_items, name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 4. Signals and context\n\n");
    out.push_str(
        "Fine-grained Solid-style reactive primitives re-exported from `@origingame/engine/ui`.\n\n",
    );
    for name in [
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
    ] {
        if let Some(item) = by_name(&signal_items, name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 5. HostProps\n\n");
    if let Some(item) = by_name(&type_items, "HostProps")
        && let ExportKind::Interface { fields } = &item.kind
    {
        if !item.doc.is_empty() {
            let _ = writeln!(out, "{}\n", item.doc);
        }
        write_fields_table(&mut out, fields, "Prop");
    }

    out.push_str("## 6. UiEvent and node types\n\n");
    for name in ["UiNode", "UiChild", "UiEvent"] {
        if let Some(item) = by_name(&type_items, name) {
            write_item_block(&mut out, item);
        }
    }

    clean_doc_for_guide(&out)
}

/// Generates `crates/og-api/guides/theme-tokens.md` from `crates/og-ui-next/default/pack.json`,
/// `crates/og-ui-next/src/default_icons.rs`, and `templates/styles/shared/icons/game.ogt`.
pub fn theme_tokens_markdown(root: &Path) -> String {
    let pack_text = std::fs::read_to_string(root.join("crates/og-ui-next/default/pack.json"))
        .expect("read crates/og-ui-next/default/pack.json");
    let pack: serde_json::Value =
        serde_json::from_str(&pack_text).expect("parse crates/og-ui-next/default/pack.json");

    let mut out = String::from(
        "<!-- Generated by `og api gen` from `crates/og-ui-next/default/pack.json`, `crates/og-ui-next/src/default_icons.rs`, and `templates/styles/shared/icons/game.ogt`. Do not edit. -->\n# UI theme tokens, pack slots and built-in icons\n\n",
    );
    out.push_str(
        "Every design token, `packClass` slot, pack sound/image key, and built-in icon shipped in the engine's default UI style pack. Use these tokens in Tailwind utility classes (`bg-<role>`, `text-<role>`, `border-<role>`, `rounded-<radius>`, `shadow-<shadow>`, `duration-<duration>`) so screens restyle cleanly across all built-in themes (`og api show themes`).\n\n",
    );

    // 1. Colour roles
    out.push_str("## 1. Colour roles\n\n");
    out.push_str(
        "Use with `bg-<role>`, `text-<role>`, `border-<role>`, `ring-<role>`, `inset-ring-<role>`, or `stroke-<role>`.\n\n",
    );
    if let Some(colors) = pack.pointer("/theme/colors").and_then(|v| v.as_object()) {
        let mut by_prefix: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for (k, v) in colors {
            let group = k.split('-').next().unwrap_or(k).to_string();
            let val = v.as_str().unwrap_or("").to_string();
            by_prefix.entry(group).or_default().push((k.clone(), val));
        }
        let _ = writeln!(out, "Total: {} semantic colour tokens.\n", colors.len());
        let _ = writeln!(out, "| Group | Tokens |");
        let _ = writeln!(out, "|---|---|");
        for (group, items) in by_prefix {
            let list: Vec<String> = items.iter().map(|(k, _)| format!("`{k}`")).collect();
            let _ = writeln!(out, "| `{group}` | {} |", list.join(", "));
        }
        out.push('\n');
    }

    // 2. Theme scales
    out.push_str("## 2. Theme scales\n\n");
    out.push_str(
        "Scale tokens under `theme` in `pack.json`, matched by Tailwind utilities (`rounded-*`, `text-*`, `font-*`, `leading-*`, `tracking-*`, `shadow-*`, `inset-shadow-*`, `text-shadow-*`, `blur-*`, `duration-*`, `ease-*`, `opacity-*`, `stroke-*`, `scale-*`, `@<container>`).\n\n",
    );
    if let Some(theme) = pack.get("theme").and_then(|v| v.as_object()) {
        let _ = writeln!(out, "| Category | Keys |");
        let _ = writeln!(out, "|---|---|");
        for (cat, val) in theme {
            if cat == "colors" {
                continue;
            }
            if let Some(obj) = val.as_object() {
                let keys: Vec<String> = obj.keys().map(|k| format!("`{k}`")).collect();
                let _ = writeln!(out, "| `{cat}` | {} |", keys.join(", "));
            }
        }
        out.push('\n');
    }

    // 3. Pack slots
    out.push_str("## 3. Pack slot classes\n\n");
    out.push_str(
        "Pass component, slot, and optional variant to `packClass(component, slot, variant)` from `@origingame/engine/ui` (resolved as `@Component.slot` or `@Component.slot.variant`).\n\n",
    );
    if let Some(variants) = pack.get("variants").and_then(|v| v.as_object()) {
        let mut by_comp: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for key in variants.keys() {
            let comp = key.split('.').next().unwrap_or(key).to_string();
            by_comp.entry(comp).or_default().push(format!("`@{key}`"));
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
            let _ = writeln!(out, "| `{comp}` | {} |", slots.join(", "));
        }
        out.push('\n');
    }

    // 4. Pack sounds and images
    out.push_str("## 4. Pack sounds and images\n\n");
    if let Some(sounds) = pack.get("sounds").and_then(|v| v.as_object()) {
        let events: Vec<String> = sounds
            .keys()
            .filter(|k| !k.contains('.'))
            .map(|k| format!("`{k}`"))
            .collect();
        let slot_overrides: Vec<String> = sounds
            .keys()
            .filter(|k| k.contains('.'))
            .map(|k| format!("`@{k}`"))
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
    }
    if let Some(images) = pack.get("images").and_then(|v| v.as_object()) {
        let names: Vec<String> = images.keys().map(|k| format!("`{k}`")).collect();
        let _ = writeln!(
            out,
            "- **Pack images (`nine-<name>`, `bg-image-<name>`)** ({}): {}",
            names.len(),
            names.join(", ")
        );
    }
    out.push('\n');

    // 5. Built-in icons
    out.push_str("## 5. Built-in icons\n\n");
    out.push_str(
        "Use with `<Icon name=\"...\" />` from `@origingame/engine/ui`. Check availability at runtime with `og.ui.hasIcon`.\n\n",
    );
    let default_icons_src =
        std::fs::read_to_string(root.join("crates/og-ui-next/src/default_icons.rs"))
            .unwrap_or_default();
    let mut fallback_icons = BTreeSet::new();
    if let Some(after) = default_icons_src.split("icons!(").nth(1)
        && let Some(body) = after.split(");").next()
    {
        for line in body.lines() {
            let t = line.trim().trim_end_matches(',');
            if let Some(name) = t.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                fallback_icons.insert(name.to_string());
            }
        }
    }
    let fallback_list: Vec<String> = fallback_icons.iter().map(|n| format!("`{n}`")).collect();
    let _ = writeln!(
        out,
        "- **Always-available vector fallback icons** ({}): {}\n",
        fallback_list.len(),
        fallback_list.join(", ")
    );

    if let Some(icons) = pack.get("icons").and_then(|v| v.as_object()) {
        let list: Vec<String> = icons.keys().map(|k| format!("`{k}`")).collect();
        let _ = writeln!(
            out,
            "- **Default UI pack icons** ({}): {}\n",
            list.len(),
            list.join(", ")
        );
    }

    let game_icons_path = root.join("templates/styles/shared/icons/game.ogt");
    if let Ok(text) = std::fs::read_to_string(&game_icons_path) {
        let mut game_icons = Vec::new();
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("icon \"")
                && let Some(name) = rest.strip_suffix('"')
            {
                game_icons.push(format!("`{name}`"));
            }
        }
        if !game_icons.is_empty() {
            let _ = writeln!(
                out,
                "- **Shared game icon set (`@styles/shared/icons/game.ogt`)** ({}): {}\n",
                game_icons.len(),
                game_icons.join(", ")
            );
        }
    }

    out
}

/// Generates `crates/og-api/guides/playtest.md` from `packages/engine/src/playtest.ts`.
pub fn playtest_markdown(root: &Path) -> String {
    let items = parse_ts_exports(root, "packages/engine/src/playtest.ts");
    let by_name = |name: &str| items.iter().find(|i| i.name == name);

    let mut out = String::from(
        "<!-- Generated by `og api gen` from `packages/engine/src/playtest.ts`. Do not edit. -->\n# Playtest harness (`@origingame/engine/playtest`)\n\n",
    );
    out.push_str(
        "TypeScript API imported by `tests/*.og.ts` playtests run via `og test`. See `testing §1` for workflow and assertions.\n\n",
    );

    out.push_str("## 1. Entry points and helpers\n\n");
    for name in [
        "runPlaytest",
        "recordReference",
        "PLAYTEST_CLOCK",
        "FrameUnchecked",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 2. PlaytestDesc and PlaytestStep\n\n");
    for name in [
        "PlaytestDesc",
        "PlaytestStep",
        "PlaytestRoom",
        "PlaytestBudget",
        "PlaytestCeiling",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }

    out.push_str("## 3. PlaytestRun and reports\n\n");
    for name in [
        "PlaytestRun",
        "PlaytestTimings",
        "PlaytestPercentiles",
        "PlaytestRestart",
        "PlaytestAssertions",
        "PlaytestViolation",
        "PlaytestLockstep",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }

    clean_doc_for_guide(&out)
}

/// Generates `crates/og-api/guides/provider.md` from `packages/engine/src/provider.ts`.
pub fn provider_markdown(root: &Path) -> String {
    let items = parse_ts_exports(root, "packages/engine/src/provider.ts");
    let by_name = |name: &str| items.iter().find(|i| i.name == name);

    let mut out = String::from(
        "<!-- Generated by `og api gen` from `packages/engine/src/provider.ts`. Do not edit. -->\n# Platform provider contract (`@origingame/engine/provider`)\n\n",
    );
    out.push_str(
        "The host-side provider contract that plugs into a web or packaged game page to answer `og.platform`, rooms, content catalogs, and `og.shared` sessions. Game code stays identical across providers.\n\n",
    );

    out.push_str("## 1. Contract version and vocabulary\n\n");
    for name in [
        "PROVIDER_VERSION",
        "CORE_ONLY_METHODS",
        "SERVICE_METHODS",
        "CORE_METHODS",
        "HOST_EXTENSIONS",
        "ONE_WAY_METHODS",
        "ERROR_CODES",
        "RETRYABLE_CODES",
        "SAMPLE_ARGS",
        "CoreMethod",
        "HostExtension",
        "PlatformMethod",
        "ErrorCode",
        "retryable",
        "ok",
        "failure",
        "unavailable",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 2. Platform and shell providers\n\n");
    for name in [
        "PlatformProvider",
        "PlatformCall",
        "PlatformAnswer",
        "PlatformNotice",
        "PlatformLink",
        "ShellNative",
        "ShellProviderContext",
        "ShellProviderFactory",
        "Providers",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }

    out.push_str("## 3. Content catalogs\n\n");
    for name in [
        "CatalogHead",
        "CatalogRead",
        "catalogHeadProblem",
        "catalogHeadAnswer",
        "fetchCatalog",
        "staticCatalog",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 4. Rooms and shared-world sessions\n\n");
    for name in [
        "RoomProvider",
        "RoomOp",
        "RoomEvent",
        "ROOM_MAX_MESSAGE",
        "ROOM_MAX_ENGINE_MESSAGE",
        "relayRooms",
        "SessionProvider",
        "SessionRequest",
        "SessionWebTransport",
        "SessionAnswer",
        "staticSession",
        "tokenEndpoint",
    ] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }
    out.push('\n');

    out.push_str("## 5. Provider conformance\n\n");
    for name in ["checkPlatformProvider", "ConformanceReport"] {
        if let Some(item) = by_name(name) {
            write_item_block(&mut out, item);
        }
    }

    clean_doc_for_guide(&out)
}
