//! Check statically known string capacities without executing game code.
//! Resolve factory imports and const bindings so aliases work and unrelated
//! or shadowed `str` functions are not mistaken for the engine's factory.
//! Dynamic capacities still go through the runtime's declaration validation.

use super::{Diagnostic, Project, Report};
use og_core::reflect::{FieldType, STR_MAX_CAPACITY};
use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, VariableDeclarationKind},
};
use oxc_parser::Parser;
use oxc_semantic::{AstNodes, Scoping, SemanticBuilder};
use oxc_span::SourceType;
use std::path::Path;

fn declaration<'a>(
    id: &IdentifierReference<'a>,
    nodes: &AstNodes<'a>,
    scoping: &Scoping,
) -> Option<oxc_semantic::NodeId> {
    let symbol = scoping.get_reference(id.reference_id.get()?).symbol_id()?;
    let decl = scoping.symbol_declaration(symbol);
    // Only engine imports are factories, not local names that happen to match.
    nodes
        .ancestor_kinds(decl)
        .any(|k| {
            matches!(k,
                AstKind::ImportDeclaration(i) if i.source.value == "@origingame/engine"
            )
        })
        .then_some(decl)
}

fn factory(callee: &Expression<'_>, nodes: &AstNodes<'_>, scoping: &Scoping) -> bool {
    match callee.get_inner_expression() {
        Expression::Identifier(id) => declaration(id, nodes, scoping).is_some_and(
            |d| matches!(nodes.kind(d), AstKind::ImportSpecifier(s) if s.imported.name() == "str"),
        ),
        Expression::StaticMemberExpression(m) if m.property.name == "str" => {
            let Expression::Identifier(id) = m.object.get_inner_expression() else {
                return false;
            };
            declaration(id, nodes, scoping)
                .is_some_and(|d| matches!(nodes.kind(d), AstKind::ImportNamespaceSpecifier(_)))
        }
        _ => false,
    }
}

/// Literal arithmetic and const references, never calls or mutable values.
fn number(
    expr: &Expression<'_>,
    nodes: &AstNodes<'_>,
    scoping: &Scoping,
    depth: u32,
) -> Option<f64> {
    if depth == 32 {
        return None;
    }
    let value = |e: &Expression<'_>| number(e, nodes, scoping, depth + 1);
    match expr.get_inner_expression() {
        Expression::NumericLiteral(n) => Some(n.value),
        Expression::Identifier(id) => {
            let symbol = scoping.get_reference(id.reference_id.get()?).symbol_id()?;
            let decl = scoping.symbol_declaration(symbol);
            let AstKind::VariableDeclaration(v) = nodes.parent_kind(decl) else {
                return None;
            };
            if v.kind != VariableDeclarationKind::Const {
                return None;
            }
            let AstKind::VariableDeclarator(d) = nodes.kind(decl) else {
                return None;
            };
            value(d.init.as_ref()?)
        }
        Expression::UnaryExpression(u) => match u.operator.as_str() {
            "+" => value(&u.argument),
            "-" => value(&u.argument).map(|n| -n),
            _ => None,
        },
        Expression::BinaryExpression(b) => {
            let (l, r) = (value(&b.left)?, value(&b.right)?);
            match b.operator.as_str() {
                "+" => Some(l + r),
                "-" => Some(l - r),
                "*" => Some(l * r),
                "/" => Some(l / r),
                "%" => Some(l % r),
                "**" => Some(l.powf(r)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn scan(path: &Path, text: &str) -> Vec<Diagnostic> {
    let allocator = Allocator::default();
    let Ok(source_type) = SourceType::from_path(path) else {
        return Vec::new();
    };
    let parsed = Parser::new(&allocator, text, source_type).parse();
    // Syntax errors belong to the TypeScript checker.
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let (nodes, scoping) = (semantic.nodes(), semantic.scoping());
    let mut out = Vec::new();
    for node in nodes.iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        if !factory(&call.callee, nodes, scoping) {
            continue;
        }
        let Some(capacity) = call
            .arguments
            .first()
            .and_then(|a| a.as_expression())
            .and_then(|e| number(e, nodes, scoping, 0))
        else {
            continue;
        };
        let reason = if capacity.is_finite()
            && capacity.fract() == 0.0
            && capacity >= 0.0
            && capacity <= usize::MAX as f64
        {
            match FieldType::string(capacity as usize) {
                Ok(_) => continue,
                Err(e) => e.reason().to_string(),
            }
        } else {
            format!(
                "a string capacity is a whole number of bytes from 1 to {STR_MAX_CAPACITY}, got {capacity}"
            )
        };
        let field = nodes
            .ancestor_kinds(node.id())
            .find_map(|k| match k {
                AstKind::ObjectProperty(p) => p.key.static_name().map(|n| n.display().to_string()),
                _ => None,
            })
            .unwrap_or_else(|| "str".into());
        let before = &text[..call.span.start as usize];
        out.push(Diagnostic {
            file: None,
            line: Some(before.matches('\n').count() as u32 + 1),
            col: Some(before.rsplit('\n').next().unwrap_or("").chars().count() as u32 + 1),
            code: 9104,
            message: format!("field '{field}': {reason}"),
        });
    }
    out
}

pub(super) fn check(project: &Project, report: &mut Report) {
    let mut files = Vec::new();
    crate::lockstep_lint::sources(project.entry.parent().unwrap_or(&project.root), &mut files);
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file =
            crate::project::typescript_path(path.strip_prefix(&project.root).unwrap_or(&path));
        for mut d in scan(&path, &text) {
            report.raw.push_str(&format!(
                "{file}({},{}): error OG{}: {}\n",
                d.line.unwrap(),
                d.col.unwrap(),
                d.code,
                d.message
            ));
            d.file = Some(file.clone());
            report.diagnostics.push(d);
        }
    }
}
