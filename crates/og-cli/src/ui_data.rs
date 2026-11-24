//! Facts about a data variant reading the wrong element. Only diagnose
//! an explicit ancestor carrier across known host elements/control flow;
//! custom components, spreads and unresolved class expressions are unknown.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, ImportDeclarationSpecifier, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
    JSXElement, JSXElementName, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::SourceType;
use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

pub const CODE: u32 = 9110;

#[derive(Debug, PartialEq)]
pub struct Finding {
    pub line: u32,
    pub col: u32,
    pub class: String,
    pub message: String,
}

#[derive(Default)]
struct Frame {
    data: Vec<String>,
    group: Option<String>,
    barrier: bool,
}

pub fn scan(path: &Path, text: &str) -> Vec<Finding> {
    let Ok(kind) = SourceType::from_path(path) else {
        return Vec::new();
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, kind).parse();
    let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
    let mut hosts = HashSet::new();
    let mut flow = HashSet::new();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(i) = statement else {
            continue;
        };
        for spec in i.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(s) = spec else {
                continue;
            };
            let Some(id) = s.local.symbol_id.get() else {
                continue;
            };
            let name = s.imported.name();
            if i.source
                .value
                .as_str()
                .is_some_and(|module| nsh_jsx::is_host_export(module, name.as_str()))
            {
                hosts.insert(id);
            }
            if i.source.value == "@origingame/engine/ui" && matches!(name.as_str(), "Show" | "For")
            {
                flow.insert(id);
            }
        }
    }
    let mut scan = Scan {
        text,
        scopes: semantic.scoping(),
        hosts,
        flow,
        frames: Vec::new(),
        found: Vec::new(),
    };
    scan.visit_program(&parsed.program);
    scan.found
}

struct Scan<'s> {
    text: &'s str,
    scopes: &'s Scoping,
    hosts: HashSet<SymbolId>,
    flow: HashSet<SymbolId>,
    frames: Vec<Frame>,
    found: Vec<Finding>,
}

/// Fully static strings only. A spread or unresolved expression can
/// supply a group/class; do not assert it cannot carry a state.
fn strings(e: &Expression<'_>, out: &mut Vec<(String, u32)>) {
    match e {
        Expression::StringLiteral(s) => out.push((s.value.display().to_string(), s.span.start + 1)),
        Expression::TemplateLiteral(t) if t.expressions.is_empty() => {
            for q in &t.quasis {
                out.push((q.value.raw.to_string(), q.span.start));
            }
        }
        Expression::ConditionalExpression(c) => {
            strings(&c.consequent, out);
            strings(&c.alternate, out);
        }
        Expression::LogicalExpression(l) => strings(&l.right, out),
        Expression::ParenthesizedExpression(p) => strings(&p.expression, out),
        _ => {}
    }
}

impl Scan<'_> {
    fn report(&mut self, class: &str, at: u32, name: &str, group: Option<&str>) {
        let prefix = &self.text[..at as usize];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() as u32 + 1;
        let col = prefix.rsplit('\n').next().unwrap_or("").chars().count() as u32 + 1;
        let fix = group.map_or_else(
            || {
                format!(
                    "give that ancestor a named `group/<name>` and use `group-data-{name}/<name>:`"
                )
            },
            |g| format!("use `group-data-{name}/{g}:`"),
        );
        self.found.push(Finding { line, col, class: class.into(), message: format!(
            "`{class}` reads this element's `data-{name}`, but only an ancestor declares it; {fix}") });
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    fn visit_jsx_element(&mut self, el: &JSXElement<'a>) {
        let id = match &el.opening_element.name {
            JSXElementName::IdentifierReference(r) => r
                .reference_id
                .get()
                .and_then(|id| self.scopes.get_reference(id).symbol_id()),
            _ => None,
        };
        let host = id.is_some_and(|id| self.hosts.contains(&id));
        let flow = id.is_some_and(|id| self.flow.contains(&id));
        let mut frame = Frame {
            barrier: !(host || flow),
            ..Default::default()
        };
        let mut classes = Vec::new();
        for a in &el.opening_element.attributes {
            let JSXAttributeItem::Attribute(a) = a else {
                frame.barrier = true;
                continue;
            };
            let JSXAttributeName::Identifier(n) = &a.name else {
                continue;
            };
            if let Some(name) = n.name.strip_prefix("data-") {
                frame.data.push(name.to_string());
            }
            if n.name == "class" {
                match &a.value {
                    Some(JSXAttributeValue::StringLiteral(s)) => {
                        classes.push((s.value.display().to_string(), s.span.start + 1))
                    }
                    Some(JSXAttributeValue::ExpressionContainer(c)) => {
                        if let Some(e) = c.expression.as_expression() {
                            strings(e, &mut classes);
                        }
                    }
                    _ => {}
                }
            }
        }
        for (text, _) in &classes {
            frame.group = text
                .split_whitespace()
                .find_map(|c| c.strip_prefix("group/"))
                .map(str::to_owned)
                .or(frame.group);
        }
        if host && !frame.barrier {
            let mut seen = BTreeSet::new();
            for (text, offset) in &classes {
                for class in text.split_whitespace() {
                    for variant in class.split(':').take(class.matches(':').count()) {
                        let Some(name) = variant.strip_prefix("data-") else {
                            continue;
                        };
                        // Bracket/value variants have a different contract. Engine-set
                        // attributes can exist without a JSX prop on this element.
                        if name.contains('[')
                            || frame.data.iter().any(|n| n == name)
                            || matches!(name, "lit" | "clamped" | "occluded")
                            || name.starts_with("edge-")
                            || name.starts_with("dir-")
                        {
                            continue;
                        }
                        let ancestor = self
                            .frames
                            .iter()
                            .rev()
                            .take_while(|f| !f.barrier)
                            .find(|f| f.data.iter().any(|n| n == name));
                        if let Some(a) = ancestor {
                            let group = a.group.clone();
                            if seen.insert((class, name)) {
                                self.report(class, *offset, name, group.as_deref());
                            }
                        }
                    }
                }
            }
        }
        self.frames.push(frame);
        walk::walk_jsx_element(self, el);
        self.frames.pop();
    }
}

pub fn lint(root: &Path, dirs: &[&Path], json: bool) {
    let mut files = Vec::new();
    for dir in dirs {
        crate::lockstep_lint::sources(dir, &mut files);
    }
    files.sort();
    files.dedup();
    let mut found = Vec::new();
    for path in files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "tsx" || e == "jsx"))
    {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let file = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
        for f in scan(path, &text) {
            if !json {
                println!(
                    "{file}({},{}): warning OG{CODE}: {}",
                    f.line, f.col, f.message
                );
            }
            found.push(serde_json::json!({"file":file, "line":f.line, "col":f.col, "code":CODE, "class":f.class, "message":f.message}));
        }
    }
    if json && !found.is_empty() {
        println!(
            "{}",
            serde_json::json!({"event":"ui-data-carriers", "warnings":found})
        );
    }
}
