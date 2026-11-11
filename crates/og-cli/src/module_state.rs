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
//! The same state one step further in is reported the same way. A variable
//! a function declares (or takes as a parameter) and a closure handed to
//! `defineSystem` or `og.game` writes lives in that closure from tick to
//! tick. An object imported from another of the game's files and written
//! through a member (`shared.count++`) is that file's module state. A
//! closure that is only called while its function runs (a `forEach`
//! callback summing into a local) keeps nothing and is not reported; one
//! that reaches the engine another way (returned, then registered by its
//! caller) and a method called on an imported object are not seen.
//!
//! Parsed and resolved (oxc's semantic model), not scanned: a local that
//! shadows a module-scope name is its own binding, and a write at module
//! scope itself (once, as the module loads) is not state, since every run
//! loads the module the same way. Playtests (`*.og.ts`) and tests drive a
//! run and are not game state.

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Expression, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_semantic::{AstNodes, NodeId, Scoping, SemanticBuilder, SymbolId};
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

/// One variable holding state outside the world: 1-based line and column
/// of the first write, the name, how it was declared (`let`, `var`,
/// `const`, `parameter` or `import`) and, for one a function declares, the
/// call its writing closure is handed to.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub line: u32,
    pub col: u32,
    pub name: String,
    pub kind: &'static str,
    pub kept_by: Option<&'static str>,
}

impl Finding {
    pub fn message(&self) -> String {
        const WHY: &str = "outside the world snapshot and the presentation checkpoint, so a restore, a rollback or a fork does not bring it back; keep it in a component or a resource";
        if let Some(call) = self.kept_by {
            let verb = if self.kind == "const" {
                "mutated"
            } else {
                "written"
            };
            return format!(
                "{} '{}' is {verb} by a function handed to {call}: it lives in that closure from tick to tick, {WHY}",
                self.kind, self.name
            );
        }
        if self.kind == "import" {
            return format!(
                "imported '{}' is written through a member in a function: the object lives in the module it comes from, {WHY}",
                self.name
            );
        }
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
/// (`c.push(v)`); `calls` false leaves the mutators out, for an object whose
/// type is not known to be a container.
fn mutated(nodes: &AstNodes<'_>, id: NodeId, calls: bool) -> bool {
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
                return calls
                    && c.callee.span() == inner
                    && matches!(member, Some(Some(name)) if MUTATORS.contains(&name));
            }
            _ => return false,
        }
        inner = kind.span();
    }
    false
}

/// The call that keeps a function it is given and runs it on later ticks:
/// `defineSystem(...)` (a system's body) or `og.game(...)` (`start` and the
/// commands).
fn registrar(callee: &Expression<'_>) -> Option<&'static str> {
    match callee.get_inner_expression() {
        Expression::Identifier(id) if id.name == "defineSystem" => Some("defineSystem"),
        Expression::StaticMemberExpression(m)
            if m.property.name == "game"
                && matches!(m.object.get_inner_expression(), Expression::Identifier(o) if o.name == "og") =>
        {
            Some("og.game")
        }
        _ => None,
    }
}

/// The registrar the expression at `id` is an argument of, directly or
/// inside the object and array literals of one (`og.game({ commands: { .. } })`).
fn handed(nodes: &AstNodes<'_>, id: NodeId) -> Option<&'static str> {
    let mut inner: Span = nodes.get_node(id).kind().span();
    for n in nodes.ancestors(id) {
        let kind = n.kind();
        match kind {
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::ObjectExpression(_)
            | AstKind::ObjectProperty(_)
            | AstKind::ArrayExpression(_)
            | AstKind::SpreadElement(_) => {}
            AstKind::CallExpression(c) => {
                return (c.callee.span() != inner)
                    .then(|| registrar(&c.callee))
                    .flatten();
            }
            _ => return None,
        }
        inner = kind.span();
    }
    None
}

/// The registrar that keeps the function at `id`: the function is handed
/// to it where it is written, or it is named (`function run() {}`,
/// `const run = () => {}`) and the name is.
fn kept(nodes: &AstNodes<'_>, scoping: &Scoping, id: NodeId) -> Option<&'static str> {
    if let Some(call) = handed(nodes, id) {
        return Some(call);
    }
    let symbol = match nodes.kind(id) {
        AstKind::Function(f) if f.id.is_some() => f.id.as_ref()?.symbol_id.get(),
        _ => match nodes.parent_kind(id) {
            AstKind::VariableDeclarator(d) => match &d.id {
                BindingPattern::BindingIdentifier(b) => b.symbol_id.get(),
                _ => None,
            },
            _ => None,
        },
    }?;
    scoping
        .get_resolved_references(symbol)
        .find_map(|r| handed(nodes, r.node_id()))
}

/// The functions enclosing `id`, innermost first.
fn functions(nodes: &AstNodes<'_>, id: NodeId) -> Vec<NodeId> {
    nodes
        .ancestor_ids(id)
        .filter(|&a| {
            matches!(
                nodes.kind(a),
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
            )
        })
        .collect()
}

/// How `symbol` is declared when it can hold state (`let`, `var`, a `const`
/// container, a `parameter`, an `import` from one of the game's own files).
fn declared(nodes: &AstNodes<'_>, scoping: &Scoping, symbol: SymbolId) -> Option<&'static str> {
    let decl = scoping.symbol_declaration(symbol);
    match nodes.kind(decl) {
        AstKind::VariableDeclarator(d) => {
            let AstKind::VariableDeclaration(v) = nodes.parent_kind(decl) else {
                return None;
            };
            match v.kind {
                VariableDeclarationKind::Let => Some("let"),
                VariableDeclarationKind::Var => Some("var"),
                VariableDeclarationKind::Const if container(d.init.as_ref()) => Some("const"),
                _ => None,
            }
        }
        AstKind::FormalParameter(_) => Some("parameter"),
        AstKind::ImportSpecifier(_)
        | AstKind::ImportDefaultSpecifier(_)
        | AstKind::ImportNamespaceSpecifier(_) => nodes
            .ancestor_kinds(decl)
            .any(|k| matches!(k, AstKind::ImportDeclaration(i) if i.source.value.starts_with('.')))
            .then_some("import"),
        _ => None,
    }
}

/// The state outside the world `text` (a game file at `path`) writes.
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
    let root = scoping.root_scope_id();
    for symbol in scoping.symbol_ids() {
        let Some(kind) = declared(nodes, scoping, symbol) else {
            continue;
        };
        let decl = scoping.symbol_declaration(symbol);
        let module = scoping.symbol_scope_id(symbol) == root;
        // A parameter is a value until it is assigned; an import's type is
        // not known here, so only a member written counts.
        let writes = |id: NodeId, write: bool| match kind {
            "const" => mutated(nodes, id, true),
            "parameter" => write,
            "import" => mutated(nodes, id, false),
            _ => write || mutated(nodes, id, true),
        };
        let outer = functions(nodes, decl).len();
        let first = scoping.get_resolved_references(symbol).find_map(|r| {
            let id = r.node_id();
            if !writes(id, r.is_write()) {
                return None;
            }
            if module {
                return in_function(nodes, id).then_some((id, None));
            }
            if kind == "import" {
                return None;
            }
            // The functions between the write and the variable's own.
            let inner = functions(nodes, id);
            let between = &inner[..inner.len().saturating_sub(outer)];
            between
                .iter()
                .find_map(|&f| kept(nodes, scoping, f))
                .map(|call| (id, Some(call)))
        });
        if let Some((id, kept_by)) = first {
            let (line, col) = line_col(nodes.get_node(id).kind().span().start);
            found.push(Finding {
                line,
                col,
                name: scoping.symbol_name(symbol).to_string(),
                kind,
                kept_by,
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
    #[test]
    fn a_closure_handed_to_the_engine_keeps_what_it_writes() {
        // Line 2 `n`, 3 `seen`, 4 `step`: written by the system's body, by a
        // callback inside it, and by a named function handed over by name.
        // `hits` (line 13) is a command's; `total` and `acc` are summed and
        // dropped inside one call, and `local` is the body's own.
        let src = "function make(name: string, step: number) {
  let n = 0;
  const seen = new Set<number>();
  function run() { step += 1; }
  const sum = (xs: number[]) => { let total = 0; xs.forEach((x) => { total += x; }); return total; };
  defineSystem(name + 'b', {}, run);
  return defineSystem(name, {}, (_q, time) => {
    let local = 0; local++;
    n += sum([1, 2]);
    [1].forEach((x) => { seen.add(x + time.tick); });
  });
}
function game() { let hits = 0; const acc: number[] = []; acc.push(1);
  return og.game({ systems: [make('a', 1)], commands: { hit: () => { hits++; } } });
}
export default game();
";
        let found: Vec<_> = scan(Path::new("main.ts"), src)
            .unwrap()
            .into_iter()
            .map(|f| (f.name, f.kind, f.line, f.kept_by))
            .collect();
        assert_eq!(
            found,
            vec![
                ("step".into(), "parameter", 4, Some("defineSystem")),
                ("n".into(), "let", 9, Some("defineSystem")),
                ("seen".into(), "const", 10, Some("defineSystem")),
                ("hits".into(), "let", 14, Some("og.game")),
            ]
        );
    }

    #[test]
    fn an_object_from_another_game_file_is_state_when_a_member_is_written() {
        // `shared` and the namespace `all` are another file's objects; the
        // engine's `og` is not a game file, `Duel` is only read, and a method
        // call says nothing of an object whose type is not known here.
        let src = "import { og } from '@origingame/engine';
import { shared, Duel, ui } from './state';
import * as all from './all.ts';
export function tick() {
  shared.count++;
  all.grid.cells[0] = 1;
  (og as any).x = 1;
  ui.set('score', Duel.fields.length);
}
";
        assert_eq!(
            names(src),
            vec![("shared".into(), "import", 5), ("all".into(), "import", 6)]
        );
    }
}
