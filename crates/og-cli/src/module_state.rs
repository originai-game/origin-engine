//! State a game keeps outside the world (docs/design.md §2.4): a game
//! file's module-scope `let` or `var` written from a function, or a
//! module-scope `const` container (`[]`, `{}`, `new Map`, `new Set`, a typed
//! array) mutated from one, lives in the JS module, not in the world
//! snapshot or the presentation checkpoint. A restore, a rollback, a fork
//! or a capture-only run does not bring it back, so the run after one
//! differs from the run that was saved. `og check` reports each as an
//! error (OG9101) naming the file, line and variable, so `og run`, `og test`
//! and `og build` stop on it as on a type error.
//!
//! Parsed and resolved (oxc's semantic model), not scanned: a local that
//! shadows a module-scope name is its own binding, and a write at module
//! scope itself (once, as the module loads) is not state, since every run
//! loads the module the same way. Playtests (`*.og.ts`) and tests drive a
//! run and are not game state.

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{Expression, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_semantic::{AstNodes, NodeId, SemanticBuilder};
use oxc_span::{GetSpan, SourceType, Span};
use std::path::Path;

/// The check's code in `og check`'s diagnostics.
pub const CODE: u32 = 9101;

/// Methods that change an array, a map, a set or a typed array in place.
const MUTATORS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "sort",
    "reverse",
    "fill",
    "copyWithin",
    "set",
    "add",
    "delete",
    "clear",
];

/// Constructors whose instance is a container a `const` can hold state in.
const CONTAINERS: &[&str] = &[
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "Array",
    "Float32Array",
    "Float64Array",
    "Int8Array",
    "Int16Array",
    "Int32Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Uint16Array",
    "Uint32Array",
    "BigInt64Array",
    "BigUint64Array",
];

/// One module-scope variable written from a function: 1-based line and
/// column of the first write, the name and how it was declared.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub line: u32,
    pub col: u32,
    pub name: String,
    pub kind: &'static str,
}

impl Finding {
    pub fn message(&self) -> String {
        let what = if self.kind == "const" {
            format!(
                "module-scope const '{}' is mutated in a function",
                self.name
            )
        } else {
            format!(
                "module-scope {} '{}' is written in a function",
                self.kind, self.name
            )
        };
        format!(
            "{what}: it lives outside the world snapshot and the presentation checkpoint, so a restore, a rollback or a fork does not bring it back; keep it in a component or a resource"
        )
    }
}

fn container(init: Option<&Expression<'_>>) -> bool {
    match init.map(Expression::get_inner_expression) {
        Some(Expression::ArrayExpression(_) | Expression::ObjectExpression(_)) => true,
        Some(Expression::NewExpression(n)) => {
            matches!(&n.callee, Expression::Identifier(id) if CONTAINERS.contains(&id.name.as_str()))
        }
        _ => false,
    }
}

/// Whether `id` is inside a function (its body runs after the module loaded).
fn in_function(nodes: &AstNodes<'_>, id: NodeId) -> bool {
    nodes.ancestors(id).any(|n| {
        matches!(
            n.kind(),
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) | AstKind::StaticBlock(_)
        ) || matches!(n.kind(), AstKind::PropertyDefinition(p) if !p.r#static)
    })
}

/// Whether the read at `id` is the object of a member that is written
/// (`c.x = 1`, `c[i]++`, `delete c.x`, `[c.x] = v`) or of a mutator's call
/// (`c.push(v)`).
fn mutated(nodes: &AstNodes<'_>, id: NodeId) -> bool {
    let mut inner: Span = nodes.get_node(id).kind().span();
    let mut member: Option<Option<&str>> = None;
    for n in nodes.ancestors(id) {
        let kind = n.kind();
        match kind {
            AstKind::StaticMemberExpression(m) if m.object.span() == inner => {
                member = Some(Some(m.property.name.as_str()));
            }
            AstKind::ComputedMemberExpression(m) if m.object.span() == inner => {
                member = Some(None);
            }
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::ChainExpression(_) => {}
            AstKind::AssignmentExpression(a) => return member.is_some() && a.left.span() == inner,
            AstKind::UpdateExpression(u) => return member.is_some() && u.argument.span() == inner,
            AstKind::UnaryExpression(u) => {
                return member.is_some()
                    && u.operator == oxc_ast::ast::UnaryOperator::Delete
                    && u.argument.span() == inner;
            }
            AstKind::ArrayAssignmentTarget(_)
            | AstKind::ObjectAssignmentTarget(_)
            | AstKind::AssignmentTargetWithDefault(_)
            | AstKind::AssignmentTargetPropertyProperty(_)
            | AstKind::AssignmentTargetRest(_) => return member.is_some(),
            AstKind::CallExpression(c) => {
                return c.callee.span() == inner
                    && matches!(member, Some(Some(name)) if MUTATORS.contains(&name));
            }
            _ => return false,
        }
        inner = kind.span();
    }
    false
}

/// The module-scope state `text` (a game file at `path`) writes.
pub fn scan(path: &Path, text: &str) -> Result<Vec<Finding>, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).map_err(|e| e.to_string())?;
    let parsed = Parser::new(&allocator, text, source_type).parse();
    if !parsed.diagnostics.is_empty() {
        return Err(format!("{}: cannot parse", path.display()));
    }
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    let semantic = built.semantic;
    let scoping = semantic.scoping();
    let nodes = semantic.nodes();
    let line_col = |offset: u32| {
        let before = &text[..offset as usize];
        let line = before.matches('\n').count() as u32 + 1;
        let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) as u32 + 1;
        (line, col)
    };
    let mut found = Vec::new();
    for symbol in scoping.iter_bindings_in(scoping.root_scope_id()) {
        let decl = scoping.symbol_declaration(symbol);
        let AstKind::VariableDeclarator(d) = nodes.get_node(decl).kind() else {
            continue;
        };
        let AstKind::VariableDeclaration(v) = nodes.parent_kind(decl) else {
            continue;
        };
        let kind = match v.kind {
            VariableDeclarationKind::Let => "let",
            VariableDeclarationKind::Var => "var",
            VariableDeclarationKind::Const if container(d.init.as_ref()) => "const",
            _ => continue,
        };
        let first = scoping.get_resolved_references(symbol).find(|r| {
            let id = r.node_id();
            in_function(nodes, id)
                && if kind == "const" {
                    mutated(nodes, id)
                } else {
                    r.is_write() || mutated(nodes, id)
                }
        });
        if let Some(r) = first {
            let (line, col) = line_col(nodes.get_node(r.node_id()).kind().span().start);
            found.push(Finding {
                line,
                col,
                name: scoping.symbol_name(symbol).to_string(),
                kind,
            });
        }
    }
    found.sort_by_key(|f| (f.line, f.col));
    Ok(found)
}

/// The findings in the game's sources under `dir` (paths relative to `root`).
pub fn check(root: &Path, dir: &Path) -> Vec<(String, Finding)> {
    let mut files = Vec::new();
    crate::lockstep_lint::sources(dir, &mut files);
    files
        .into_iter()
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            !(name.contains(".og.") || name.contains(".test.") || name.contains(".spec."))
        })
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|t| (p, t)))
        .flat_map(|(p, t)| {
            let rel = crate::project::typescript_path(p.strip_prefix(root).unwrap_or(&p));
            // A file that does not parse is the type checker's to report.
            scan(&p, &t)
                .unwrap_or_default()
                .into_iter()
                .map(move |f| (rel.clone(), f))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(src: &str) -> Vec<(String, &'static str, u32)> {
        scan(Path::new("main.ts"), src)
            .unwrap()
            .into_iter()
            .map(|f| (f.name, f.kind, f.line))
            .collect()
    }

    #[test]
    fn writes_from_functions_are_state() {
        let src = "let score = 0;\nconst seen = new Set<number>();\nconst grid: number[] = [];\nconst cfg = { speed: 1 };\nexport function tick() {\n  score += 1;\n  seen.add(score);\n  grid[score] = 1;\n  (cfg as any).speed++;\n}\n";
        assert_eq!(
            names(src),
            vec![
                ("score".into(), "let", 6),
                ("seen".into(), "const", 7),
                ("grid".into(), "const", 8),
                ("cfg".into(), "const", 9),
            ]
        );
    }

    #[test]
    fn shadows_reads_and_load_time_writes_are_not() {
        // A local named like a module-scope one, reads, a write as the module
        // loads, a const never mutated, a non-mutating call and a member read.
        let src = "let score = 0;\nlet mode = 'a';\nif (Math.PI > 3) mode = 'b';\nconst list = [1, 2];\nconst q = new Map<string, number>();\nfunction f(score: number) { score = 2; let mode = 'c'; mode = 'd'; return list.map((x) => x + score).length + (q.get('k') ?? 0) + list[0]!; }\nconst g = () => { const list = [] as number[]; list.push(1); return list; };\nexport { f, g };\n";
        assert_eq!(names(src), vec![]);
    }

    #[test]
    fn destructuring_and_callbacks_write() {
        let src = "let a = 0, b = 0;\nexport const s = { onTick: () => { [a, b] = [b, a]; } };\n";
        assert_eq!(
            names(src),
            vec![("a".into(), "let", 2), ("b".into(), "let", 2)]
        );
    }
}
