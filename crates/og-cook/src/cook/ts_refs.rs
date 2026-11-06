//! Static asset references and conservative computed-load prefixes in game code.
use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, StringLiteral, TemplateLiteral};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use std::collections::BTreeMap;
use std::ops::Range;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct Literal {
    pub value: String,
    pub range: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Computed {
    pub prefix: String,
    pub line: usize,
}

#[derive(Default)]
pub struct References {
    pub literals: Vec<Literal>,
    pub computed: Vec<Computed>,
}

/// Parse instead of matching source text: comments, escapes, regex literals,
/// multiline calls, TS assertions and template interpolation retain their meaning.
pub fn parse(path: &Path, text: &str) -> Result<References, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).map_err(|e| e.to_string())?;
    let parsed = Parser::new(&allocator, text, source_type).parse();
    if !parsed.diagnostics.is_empty() {
        return Err(format!(
            "{}: cannot analyse asset references in invalid TypeScript",
            path.display()
        ));
    }
    let mut visitor = Visitor {
        text,
        refs: References::default(),
    };
    visitor.visit_program(&parsed.program);
    Ok(visitor.refs)
}

struct Visitor<'s> {
    text: &'s str,
    refs: References,
}

fn literal(expr: &Expression<'_>) -> Option<String> {
    match expr.get_inner_expression() {
        Expression::StringLiteral(s) => s.value.as_str().map(str::to_owned),
        Expression::TemplateLiteral(t) if t.expressions.is_empty() => {
            t.quasis.first()?.value.cooked?.as_str().map(str::to_owned)
        }
        _ => None,
    }
}

fn prefix(expr: &Expression<'_>) -> String {
    if let Some(value) = literal(expr) {
        return value;
    }
    match expr.get_inner_expression() {
        Expression::TemplateLiteral(t) => t
            .quasis
            .first()
            .and_then(|q| q.value.cooked)
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        Expression::BinaryExpression(b) if b.operator.as_str() == "+" => prefix(&b.left),
        _ => String::new(),
    }
}

fn load_path_argument(expr: &Expression<'_>) -> Option<usize> {
    let method = expr.get_inner_expression().as_member_expression()?;
    let namespace = method
        .object()
        .get_inner_expression()
        .as_member_expression()?;
    if !namespace
        .static_property_name()
        .is_some_and(|name| name == "load")
        || !matches!(namespace.object().get_inner_expression(), Expression::Identifier(id) if id.name == "og")
    {
        return None;
    }
    let Some(name) = method.static_property_name() else {
        return Some(1);
    };
    let name = name.as_str()?;
    // bundle's path names an external cooked directory, not a local asset.
    if name == "bundle" {
        return None;
    }
    static PATHS: OnceLock<BTreeMap<String, usize>> = OnceLock::new();
    PATHS
        .get_or_init(|| {
            let text = concat!(
                "type api\nversion 1\n---\n",
                include_str!("../../../og-api/api/load.ogt")
            );
            let doc = og_core::ogt::Document::parse(text).expect("load IDL");
            let module = doc
                .body
                .get("module")
                .and_then(|e| e.block.as_ref())
                .expect("load module");
            module
                .get_all("fn")
                .filter_map(|entry| {
                    let name = entry.values.first()?.as_word()?;
                    let position = entry.block.as_ref()?.get_all("param").position(|param| {
                        param.values.first().and_then(|v| v.as_word()) == Some("path")
                    })?;
                    Some((name.into(), position))
                })
                .collect()
        })
        .get(name)
        .copied()
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_string_literal(&mut self, s: &StringLiteral<'a>) {
        if let Some(value) = s.value.as_str() {
            self.refs.literals.push(Literal {
                value: value.into(),
                range: s.span.start as usize..s.span.end as usize,
            });
        }
    }

    fn visit_template_literal(&mut self, t: &TemplateLiteral<'a>) {
        if t.expressions.is_empty()
            && let Some(value) = t
                .quasis
                .first()
                .and_then(|q| q.value.cooked)
                .and_then(|s| s.as_str())
        {
            self.refs.literals.push(Literal {
                value: value.into(),
                range: t.span.start as usize..t.span.end as usize,
            });
        }
        walk::walk_template_literal(self, t);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(argument) = load_path_argument(&call.callee) {
            let path = call.arguments.get(argument).and_then(|a| a.as_expression());
            if path.is_none_or(|e| literal(e).is_none()) {
                let start = path.map_or(call.span.start, |e| e.span().start) as usize;
                self.refs.computed.push(Computed {
                    prefix: path.map(prefix).unwrap_or_default(),
                    line: self.text[..start].bytes().filter(|b| *b == b'\n').count() + 1,
                });
            }
        }
        walk::walk_call_expression(self, call);
    }
}
