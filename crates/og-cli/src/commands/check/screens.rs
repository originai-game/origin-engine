//! Source facts, not a visual verdict: missing literal title art and named
//! screen candidates that compose host elements without their built-in.

use super::{Project, project_dirs};
use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Expression, IdentifierReference, JSXAttributeItem,
        JSXAttributeName, JSXAttributeValue, JSXElement, JSXElementName, JSXMemberExpressionObject,
        ObjectPropertyKind,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{AstNodes, Scoping, SemanticBuilder};
use oxc_span::{SourceType, Span};
use std::path::Path;

const ART: u32 = 9112;
const SCREEN: u32 = 9113;

fn role(name: &str) -> Option<(&'static str, &'static str)> {
    let lower = name.to_ascii_lowercase();
    let base = lower
        .strip_suffix("screen")
        .or_else(|| lower.strip_suffix("view"))
        .unwrap_or(&lower);
    Some(match base {
        "title" => ("TitleScreen", "title"),
        "mainmenu" => ("MainMenuScreen", "main-menu"),
        "pause" | "pausemenu" => ("PauseScreen", "pause"),
        "settings" | "settingsmenu" | "options" | "optionsmenu" => ("SettingsScreen", "settings"),
        "saveload" | "save" | "load" | "savemenu" | "loadmenu" => ("SaveLoadScreen", "save-load"),
        "results" | "result" | "settlement" => ("ResultsScreen", "results"),
        "gameover" => ("GameOverScreen", "game-over"),
        _ => return None,
    })
}

struct Scan<'s, 'a> {
    nodes: &'s AstNodes<'a>,
    scopes: &'s Scoping,
    // A use of a built-in screen or host element, at its source position.
    uses: Vec<(Span, String)>,
    found: Vec<(u32, u32, String)>,
}

impl Scan<'_, '_> {
    /// Resolve named aliases and namespace imports; a local/shadowed name is
    /// not an engine component. Re-exports and dynamic expressions are unknown.
    fn imported(&self, id: &IdentifierReference<'_>, member: Option<&str>) -> Option<String> {
        let symbol = self
            .scopes
            .get_reference(id.reference_id.get()?)
            .symbol_id()?;
        let decl = self.scopes.symbol_declaration(symbol);
        let module = self.nodes.ancestor_kinds(decl).find_map(|k| match k {
            AstKind::ImportDeclaration(i) => i.source.value.as_str(),
            _ => None,
        })?;
        let name = match (self.nodes.kind(decl), member) {
            (AstKind::ImportSpecifier(s), None) => s.imported.name().to_string(),
            (AstKind::ImportNamespaceSpecifier(_), Some(name)) => name.to_owned(),
            _ => return None,
        };
        if module == "@origingame/engine/ui" && matches!(name.as_str(), "View" | "Text" | "mount") {
            return Some(name);
        }
        let (_, path) = role(&name)?;
        (module == "@origingame/engine/ui/components"
            || module == format!("@origingame/engine/ui/screens/{path}")
            || module == format!("@origingame/engine/ui/screens/{path}.tsx"))
        .then_some(name)
    }

    fn expression(&self, e: &Expression<'_>) -> Option<String> {
        match e.get_inner_expression() {
            Expression::Identifier(id) => self.imported(id, None),
            Expression::StaticMemberExpression(m) => {
                let Expression::Identifier(id) = m.object.get_inner_expression() else {
                    return None;
                };
                self.imported(id, Some(m.property.name.as_str()))
            }
            _ => None,
        }
    }

    fn art(&mut self, at: u32, props: Vec<(Option<String>, bool)>) {
        for key in ["keyArt", "logo"] {
            // None marks a spread/computed key. A later one can override a
            // literal, but an explicit prop after it has a known value.
            let state = props
                .iter()
                .rev()
                .find(|(name, _)| name.as_deref().is_none_or(|n| n == key));
            let why = match state {
                Some((Some(_), true)) => "is literal 0",
                None => "is omitted",
                _ => continue,
            };
            self.found.push((at, ART, format!(
                "TitleScreen `{key}` {why}; no {key} texture is supplied here. Load the game's art with og.load.texture and pass its value; see og api show screens (§Title art)."
            )));
        }
    }
}

fn zero(e: &Expression<'_>) -> bool {
    matches!(e.get_inner_expression(), Expression::NumericLiteral(n) if n.value == 0.0)
}

impl<'a> Visit<'a> for Scan<'_, 'a> {
    fn visit_jsx_element(&mut self, el: &JSXElement<'a>) {
        let name = match &el.opening_element.name {
            JSXElementName::IdentifierReference(id) => self.imported(id, None),
            JSXElementName::MemberExpression(m) => match &m.object {
                JSXMemberExpressionObject::IdentifierReference(id) => {
                    self.imported(id, Some(m.property.name.as_str()))
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(name) = name {
            self.uses.push((el.span, name.clone()));
            if name == "TitleScreen" {
                let mut props = Vec::new();
                for a in &el.opening_element.attributes {
                    let JSXAttributeItem::Attribute(a) = a else {
                        props.push((None, false));
                        continue;
                    };
                    let JSXAttributeName::Identifier(key) = &a.name else {
                        continue;
                    };
                    let empty = match &a.value {
                        Some(JSXAttributeValue::ExpressionContainer(c)) => {
                            c.expression.as_expression().is_some_and(zero)
                        }
                        _ => false,
                    };
                    props.push((Some(key.name.to_string()), empty));
                }
                self.art(el.span.start, props);
            }
        }
        walk::walk_jsx_element(self, el);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let direct = self.expression(&call.callee);
        if let Some(name) = &direct {
            self.uses.push((call.span, name.clone()));
        }
        // mount(world, TitleScreen, props), including import aliases.
        let mounted = (direct.as_deref() == Some("mount"))
            .then(|| {
                call.arguments
                    .get(1)?
                    .as_expression()
                    .and_then(|e| self.expression(e))
            })
            .flatten();
        if let Some(name) = &mounted {
            self.uses.push((call.span, name.clone()));
        }
        let arg = if direct.as_deref() == Some("TitleScreen") {
            call.arguments.first()
        } else if mounted.as_deref() == Some("TitleScreen") {
            call.arguments.get(2)
        } else {
            walk::walk_call_expression(self, call);
            return;
        };
        match arg
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression)
        {
            Some(Expression::ObjectExpression(obj)) => {
                let mut props = Vec::new();
                for p in &obj.properties {
                    match p {
                        ObjectPropertyKind::ObjectProperty(p) => {
                            props.push((
                                p.key
                                    .static_name()
                                    .and_then(|key| key.as_str().map(str::to_owned)),
                                zero(&p.value),
                            ));
                        }
                        ObjectPropertyKind::SpreadProperty(_) => props.push((None, false)),
                    }
                }
                self.art(call.span.start, props);
            }
            None if arg.is_none() => self.art(call.span.start, Vec::new()),
            _ => {}
        }
        walk::walk_call_expression(self, call);
    }
}

fn scan(path: &Path, text: &str) -> Vec<(u32, u32, String)> {
    let Ok(kind) = SourceType::from_path(path) else {
        return Vec::new();
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, kind).parse();
    if parsed.diagnostics.has_errors() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let mut scan = Scan {
        nodes: semantic.nodes(),
        scopes: semantic.scoping(),
        uses: Vec::new(),
        found: Vec::new(),
    };
    scan.visit_program(&parsed.program);
    for node in semantic.nodes().iter() {
        let (name, span) = match node.kind() {
            AstKind::Function(f) if f.is_function_declaration() => match &f.id {
                Some(id) => (id.name.as_str(), f.span),
                None => continue,
            },
            AstKind::VariableDeclarator(d)
                if d.init.as_ref().is_some_and(|e| {
                    matches!(
                        e.get_inner_expression(),
                        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                    )
                }) =>
            {
                let Some(name) = d.id.get_identifier_name() else {
                    continue;
                };
                (name.as_str(), d.span)
            }
            _ => continue,
        };
        let Some((builtin, path)) = role(name) else {
            continue;
        };
        let inside = |s: &Span| s.start >= span.start && s.end <= span.end;
        let host = scan
            .uses
            .iter()
            .any(|(s, n)| inside(s) && matches!(n.as_str(), "View" | "Text"));
        let delegates = scan.uses.iter().any(|(s, n)| inside(s) && n == builtin);
        if host && !delegates {
            scan.found.push((span.start, SCREEN, format!(
                "`{name}` is a locally defined screen-role candidate (matched by name), composes View/Text, and has no direct {builtin} use. Built-in: @origingame/engine/ui/screens/{path}.tsx; customize with `og ui eject screens/{path}`. If replacing the whole screen, record the reason in a code comment; see og api show screens."
            )));
        }
    }
    scan.found.sort_by_key(|(at, code, _)| (*at, *code));
    scan.found.dedup();
    scan.found
}

pub(super) fn lint(p: &Project, json: bool) {
    let mut files = Vec::new();
    for dir in project_dirs(p) {
        crate::lockstep_lint::sources(&dir, &mut files);
    }
    files.sort();
    files.dedup();
    let mut warnings = Vec::new();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file = path
            .strip_prefix(&p.root)
            .unwrap_or(&path)
            .to_string_lossy();
        for (at, code, message) in scan(&path, &text) {
            let prefix = &text[..at as usize];
            let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
            let col = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
            if !json {
                println!("{file}({line},{col}): warning OG{code}: {message}");
            }
            warnings.push(serde_json::json!({"file":file, "line":line, "col":col, "code":code, "message":message}));
        }
    }
    if json && !warnings.is_empty() {
        println!(
            "{}",
            serde_json::json!({"event":"ui-screens", "warnings":warnings})
        );
    }
}
