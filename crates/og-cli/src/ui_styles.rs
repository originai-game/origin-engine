//! Inline style properties the UI never draws. A host element's `style`
//! declaration whose property NSH does not know (`background`, a CSS
//! shorthand it has no key for) is skipped where it is applied, and one it
//! reads into a field the tree does not draw (`mixBlendMode`) is kept and
//! ignored: either way nothing shows and nothing says so.
//! `og check` reports each as warning OG9111 at the property, with the
//! theme class that draws what it meant (`og_ui_next::classes::style_property`
//! decides, so the check and the tree read one table).
//!
//! It reads the static names only: an object literal's keys and the
//! declarations of a string or template literal. A spread, a computed key
//! or a style that is not a literal is left alone.

use og_ui_next::classes::{StyleProperty, style_property};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, ImportDeclarationSpecifier, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
    JSXElement, JSXElementName, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::SourceType;
use std::{collections::HashSet, path::Path};

pub const CODE: u32 = 9111;

#[derive(Debug, PartialEq)]
pub struct Finding {
    pub line: u32,
    pub col: u32,
    pub property: String,
    pub message: String,
}

/// The class that draws what an ignored property meant, by its name.
fn instead(property: &str) -> &'static str {
    match property
        .trim_start_matches('-')
        .to_ascii_lowercase()
        .replace('-', "")
        .as_str()
    {
        "background" | "backgroundcolor" => {
            "`bg-<role>` for a theme colour (`bg-ally-ink`) or `bg-linear-*` with `from-<role>` for a theme gradient; `backgroundColor` or `backgroundImage` for one that comes from data"
        }
        "inset" => {
            "`inset-<step>`, or `top`, `right`, `bottom` and `left` for a length that comes from data"
        }
        "color" => "`text-<role>`",
        "border" | "bordercolor" => "`border` with `border-<role>`",
        "outline" => "`outline` with `outline-<role>`",
        "font" => "`font-<family>`, `text-<step>` and `font-<weight>`",
        "boxshadow" | "shadow" => "`shadow-<name>`",
        "filter" | "backdropfilter" => {
            "`blur-<step>`, `drop-shadow-<name>` or `backdrop-blur-<step>`"
        }
        "mixblendmode" => "none yet: blend modes are not drawn; leave it out",
        "accentcolor" => "`bg-<role>` on the part that shows the accent",
        _ => "the theme class for it, or leave it out",
    }
}

fn message(property: &str, kind: StyleProperty) -> Option<String> {
    let why = match kind {
        StyleProperty::Drawn => return None,
        StyleProperty::Unknown => format!(
            "`{property}` is not a style property the UI reads, so the declaration is dropped"
        ),
        StyleProperty::NotDrawn(why) => format!("`{property}` is read but never drawn ({why})"),
    };
    Some(format!("{why}: use {}", instead(property)))
}

pub fn scan(path: &Path, text: &str) -> Vec<Finding> {
    let Ok(kind) = SourceType::from_path(path) else {
        return Vec::new();
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, kind).parse();
    let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
    let mut hosts = HashSet::new();
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
            if i.source
                .value
                .as_str()
                .is_some_and(|module| nsh_jsx::is_host_export(module, s.imported.name().as_str()))
            {
                hosts.insert(id);
            }
        }
    }
    let mut scan = Scan {
        text,
        scopes: semantic.scoping(),
        hosts,
        found: Vec::new(),
    };
    scan.visit_program(&parsed.program);
    scan.found
}

struct Scan<'s> {
    text: &'s str,
    scopes: &'s Scoping,
    hosts: HashSet<SymbolId>,
    found: Vec<Finding>,
}

impl Scan<'_> {
    fn check(&mut self, property: &str, at: u32) {
        // A custom property (`--x`) feeds `var()`: it is no style property.
        if property.starts_with("--") {
            return;
        }
        let Some(message) = message(property, style_property(property)) else {
            return;
        };
        let prefix = &self.text[..at as usize];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() as u32 + 1;
        let col = prefix.rsplit('\n').next().unwrap_or("").chars().count() as u32 + 1;
        self.found.push(Finding {
            line,
            col,
            property: property.into(),
            message,
        });
    }

    /// The declarations of CSS text at byte `at` (`a: 1; b: 2`).
    fn css(&mut self, css: &str, at: u32) {
        let mut offset = 0;
        for decl in css.split(';') {
            if let Some((name, _)) = decl.split_once(':') {
                let lead = name.len() - name.trim_start().len();
                let name = name.trim();
                if !name.is_empty() && !name.contains(char::is_whitespace) {
                    self.check(name, at + (offset + lead) as u32);
                }
            }
            offset += decl.len() + 1;
        }
    }

    fn style(&mut self, e: &Expression<'_>) {
        match e {
            Expression::ObjectExpression(o) => {
                for p in &o.properties {
                    let ObjectPropertyKind::ObjectProperty(p) = p else {
                        continue;
                    };
                    match &p.key {
                        PropertyKey::StaticIdentifier(i) => self.check(&i.name, i.span.start),
                        PropertyKey::StringLiteral(s) => {
                            self.check(&s.value.display().to_string(), s.span.start + 1)
                        }
                        _ => {}
                    }
                }
            }
            Expression::StringLiteral(s) => {
                self.css(&s.value.display().to_string(), s.span.start + 1)
            }
            Expression::TemplateLiteral(t) => {
                for q in &t.quasis {
                    self.css(&q.value.raw, q.span.start);
                }
            }
            Expression::ConditionalExpression(c) => {
                self.style(&c.consequent);
                self.style(&c.alternate);
            }
            Expression::ParenthesizedExpression(p) => self.style(&p.expression),
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    fn visit_jsx_element(&mut self, el: &JSXElement<'a>) {
        let host = match &el.opening_element.name {
            JSXElementName::IdentifierReference(r) => r
                .reference_id
                .get()
                .and_then(|id| self.scopes.get_reference(id).symbol_id())
                .is_some_and(|s| self.hosts.contains(&s)),
            _ => false,
        };
        if host {
            for attr in &el.opening_element.attributes {
                let JSXAttributeItem::Attribute(a) = attr else {
                    continue;
                };
                let JSXAttributeName::Identifier(n) = &a.name else {
                    continue;
                };
                if n.name != "style" {
                    continue;
                }
                match &a.value {
                    Some(JSXAttributeValue::StringLiteral(s)) => {
                        self.css(&s.value.display().to_string(), s.span.start + 1)
                    }
                    Some(JSXAttributeValue::ExpressionContainer(c)) => {
                        if let Some(e) = c.expression.as_expression() {
                            self.style(e);
                        }
                    }
                    _ => {}
                }
            }
        }
        walk::walk_jsx_element(self, el);
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
            found.push(serde_json::json!({"file":file, "line":f.line, "col":f.col, "code":CODE, "property":f.property, "message":f.message}));
        }
    }
    if json && !found.is_empty() {
        println!(
            "{}",
            serde_json::json!({"event":"ui-inline-styles", "warnings":found})
        );
    }
}
