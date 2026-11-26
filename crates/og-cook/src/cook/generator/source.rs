//! Source-bound edits to default data exports. This is a text patch primitive,
//! not a file writer: its caller must check types, evaluate and validate the
//! patched module before committing it under the project's edit lock.
//!
//! Never follow identifiers or evaluate expressions to find a writable value.
//! A source snapshot and JSON Pointer identify a direct literal, not a runtime
//! entity or a shared definition. Kind-specific provenance supplies that identity.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Comment, Expression, ImportDeclarationSpecifier, ImportOrExportKind, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, Statement,
};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};
use serde_json::Value;
use std::collections::BTreeMap;

/// Original bytes, before TS transformation. Keep this snapshot with the
/// provenance of an evaluated entry, never reconstruct it from current text.
#[derive(Clone, Debug)]
pub struct Source {
    file: String,
    text: String,
}

/// An expected source value (None means absent), and its proposed replacement.
/// Paths are RFC 6901 JSON Pointers; the empty path names the default export.
#[derive(Clone, Debug)]
pub struct Edit {
    pub path: String,
    pub expected: Option<Value>,
    pub value: Value,
}

/// UTF-8 byte offsets and one-based Unicode scalar line/column in original TS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub start: u32,
    pub end: u32,
    pub line: usize,
    pub column: usize,
}

/// Missing properties point to their containing object. Computed or ambiguous
/// values return a diagnostic instead of pretending to have a literal value.
#[derive(Clone, Debug)]
pub struct Field {
    pub location: Location,
    pub value: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub file: String,
    pub path: String,
    pub location: Location,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {} ({}): {}",
            self.file, self.location.line, self.location.column, self.code, self.path, self.message
        )
    }
}

impl std::error::Error for Diagnostic {}

struct Failure {
    code: &'static str,
    span: Span,
    message: String,
    path: Option<String>,
}

fn fail(code: &'static str, span: Span, message: impl Into<String>) -> Failure {
    Failure {
        code,
        span,
        message: message.into(),
        path: None,
    }
}

fn computed(span: Span) -> Failure {
    fail(
        "computed-source",
        span,
        "edit the source expression; only unique direct literals are writable",
    )
}

impl Source {
    pub fn new(file: &str, text: &str) -> Self {
        Self {
            file: file.into(),
            text: text.into(),
        }
    }

    /// Byte identity for a source-bound edit intent. Paths do not enter it.
    pub fn digest(&self) -> String {
        blake3::hash(self.text.as_bytes()).to_hex().to_string()
    }

    fn location(&self, span: Span) -> Location {
        let before = &self.text[..span.start as usize];
        Location {
            start: span.start,
            end: span.end,
            line: before.bytes().filter(|&b| b == b'\n').count() + 1,
            column: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
        }
    }

    fn diagnostic(&self, path: &str, failure: Failure) -> Diagnostic {
        Diagnostic {
            code: failure.code,
            file: self.file.clone(),
            path: failure.path.unwrap_or_else(|| path.into()),
            location: self.location(failure.span),
            message: failure.message,
        }
    }

    fn with_root<T>(
        &self,
        path: &str,
        f: impl FnOnce(&Expression<'_>, &[Comment], Option<&str>) -> Result<T, Failure>,
    ) -> Result<T, Diagnostic> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&self.file).map_err(|e| {
            self.diagnostic(path, fail("invalid-source", Span::default(), e.to_string()))
        })?;
        let parsed = Parser::new(&allocator, &self.text, source_type).parse();
        if !parsed.diagnostics.is_empty() {
            return Err(self.diagnostic(
                path,
                fail(
                    "invalid-source",
                    parsed.diagnostics[0]
                        .labels
                        .first()
                        .map_or(Span::default(), |l| {
                            Span::new(l.offset(), l.offset() + l.len())
                        }),
                    format!("{:?}", parsed.diagnostics),
                ),
            ));
        }
        let semantics = SemanticBuilder::new()
            .with_check_syntax_error(true)
            .build(&parsed.program);
        if !semantics.diagnostics.is_empty() {
            return Err(self.diagnostic(
                path,
                fail(
                    "invalid-source",
                    semantics.diagnostics[0]
                        .labels
                        .first()
                        .map_or(Span::default(), |l| {
                            Span::new(l.offset(), l.offset() + l.len())
                        }),
                    format!("{:?}", semantics.diagnostics),
                ),
            ));
        }
        let defaults: Vec<_> = parsed
            .program
            .body
            .iter()
            .filter_map(|s| {
                if let Statement::ExportDefaultDeclaration(d) = s {
                    Some(d)
                } else {
                    None
                }
            })
            .collect();
        let result = match defaults.as_slice() {
            [d] => d
                .declaration
                .as_expression()
                .ok_or_else(|| computed(d.span))
                .and_then(|root| {
                    let content = factory(root, &parsed.program, semantics.semantic.scoping());
                    let (kind, value) =
                        content.map_or((None, root), |(kind, value)| (Some(kind), value));
                    f(value, &parsed.program.comments, kind)
                }),
            _ => Err(fail(
                "invalid-source",
                Span::default(),
                "expected one default data export",
            )),
        };
        result.map_err(|e| self.diagnostic(path, e))
    }

    /// Discover an entry without executing it. Only a direct imported
    /// author.file.<kind>(value) call identifies content, never an alias,
    /// lookalike local object, helper result or re-exported factory.
    pub fn content_kind(&self) -> Result<String, Diagnostic> {
        self.with_root("", |root, _, kind| {
            kind.map(str::to_string).ok_or_else(|| fail(
                "missing-factory", root.span(),
                "default-export a direct file.<kind>(value) call imported from @origingame/engine/author",
            ))
        })
    }

    pub fn field(&self, path: &str) -> Result<Field, Diagnostic> {
        self.with_root(path, |root, _, _| {
            let target = locate(root, &segments(path)?)?;
            Ok(Field {
                location: self.location(target.span()),
                value: match target {
                    Target::Value(e) => Some(literal(e)?),
                    Target::Missing(_, _) => None,
                },
            })
        })
    }

    /// All edits refer to this snapshot, not to each other's intermediate
    /// results. Any conflict rejects the entire batch. No bytes are written.
    /// Text outside replaced value spans/insertion points is unchanged.
    pub fn patch(&self, current: &str, edits: &[Edit]) -> Result<String, Diagnostic> {
        if current != self.text {
            return Err(self.diagnostic(
                "",
                fail(
                    "source-conflict",
                    Span::default(),
                    "source changed; recook before saving",
                ),
            ));
        }
        self.with_root("", |root, comments, _| {
            let mut paths: Vec<Vec<String>> = Vec::new();
            let mut replacements = Vec::new();
            let mut insertions: BTreeMap<u32, (Option<u32>, Vec<String>)> = BTreeMap::new();
            for edit in edits {
                let result = (|| {
                    let path = segments(&edit.path)?;
                    if paths
                        .iter()
                        .any(|p| p.starts_with(&path) || path.starts_with(p))
                    {
                        return Err(fail(
                            "overlapping-edits",
                            root.span(),
                            "duplicate or ancestor/descendant edit paths",
                        ));
                    }
                    paths.push(path.clone());
                    let target = locate(root, &path)?;
                    let span = target.span();
                    let before = match &target {
                        Target::Value(e) => Some(literal(e)?),
                        Target::Missing(_, _) => None,
                    };
                    if !same_optional(before.as_ref(), edit.expected.as_ref()) {
                        return Err(fail(
                            "value-conflict",
                            span,
                            "expected value does not match the source literal",
                        ));
                    }
                    representable(&edit.value, span)?;
                    if let Some(expected) = &edit.expected {
                        representable(expected, span)?;
                    }
                    let json = serde_json::to_string(&edit.value).expect("JSON value serializes");
                    match target {
                        Target::Value(_) => {
                            if comments.iter().any(|c| c.span.start >= span.start && c.span.end <= span.end) {
                                return Err(fail("commented-source", span, "edit individual literal fields instead of replacing a value containing comments"));
                            }
                            let original = &self.text[span.start as usize..span.end as usize];
                            let text = if original.starts_with('\'') && edit.value.is_string() {
                                single_quoted(&json)
                            } else {
                                json
                            };
                            replacements.push((span.start, span.end, text));
                        }
                        Target::Missing(object, key) => {
                            if key == "__proto__" {
                                return Err(computed(span));
                            }
                            let at = object.span.end - 1;
                            let comma = object.properties.last().and_then(|p| {
                                let end = p.span().end;
                                let mut from = end;
                                let mut comma = false;
                                for comment in comments
                                    .iter()
                                    .filter(|c| c.span.start >= end && c.span.end <= at)
                                {
                                    comma |= self.text[from as usize..comment.span.start as usize]
                                        .contains(',');
                                    from = comment.span.end;
                                }
                                comma |= self.text[from as usize..at as usize].contains(',');
                                (!comma).then_some(end)
                            });
                            insertions
                                .entry(at)
                                .or_insert_with(|| (comma, Vec::new()))
                                .1
                                .push(format!("{}: {json}", serde_json::to_string(&key).unwrap()));
                        }
                    }
                    Ok(())
                })();
                if let Err(mut error) = result {
                    // Keep the requested field in the outer diagnostic.
                    error.path = Some(edit.path.clone());
                    return Err(error);
                }
            }
            for (at, (comma, fields)) in insertions {
                if let Some(comma) = comma {
                    replacements.push((comma, comma, ",".into()));
                }
                let indent = self.text[..at as usize].rsplit('\n').next().unwrap_or("");
                let text = if indent.chars().all(|c| c == ' ' || c == '\t') {
                    let newline = if self.text.contains("\r\n") {
                        "\r\n"
                    } else {
                        "\n"
                    };
                    format!(
                        "  {}{newline}{indent}",
                        fields.join(&format!(",{newline}{indent}  "))
                    )
                } else {
                    format!(" {}", fields.join(", "))
                };
                replacements.push((at, at, text));
            }
            replacements.sort_by_key(|&(start, end, _)| (start, end));
            let mut text = self.text.clone();
            for (start, end, replacement) in replacements.into_iter().rev() {
                text.replace_range(start as usize..end as usize, &replacement);
            }
            Ok(text)
        })
        .and_then(|text| {
            // JSON is valid JS data, but an object replacing a scalar default
            // export, for example, still needs the surrounding TS grammar.
            Self::new(&self.file, &text).with_root("", |_, _, _| Ok(()))?;
            Ok(text)
        })
    }
}

fn factory<'a, 's>(
    root: &'s Expression<'a>,
    program: &oxc_ast::ast::Program<'a>,
    scoping: &Scoping,
) -> Option<(&'s str, &'s Expression<'a>)> {
    fn member<'a, 's>(expression: &'s Expression<'a>) -> Option<(&'s Expression<'a>, &'s str)> {
        match expression.get_inner_expression() {
            Expression::StaticMemberExpression(m) if !m.optional => {
                Some((&m.object, m.property.name.as_str()))
            }
            Expression::ComputedMemberExpression(m) if !m.optional => {
                let Expression::StringLiteral(key) = m.expression.get_inner_expression() else {
                    return None;
                };
                Some((&m.object, key.value.as_str()?))
            }
            _ => None,
        }
    }
    let Expression::CallExpression(call) = root.get_inner_expression() else {
        return None;
    };
    if call.optional || call.arguments.len() != 1 {
        return None;
    }
    let (object, kind) = member(&call.callee)?;
    let binding = |expression: &Expression<'_>, symbol: SymbolId| {
        matches!(expression.get_inner_expression(), Expression::Identifier(id)
            if id.reference_id.get().and_then(|r| scoping.get_reference(r).symbol_id()) == Some(symbol))
    };
    let imported = program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        import.source.value == super::author::MODULE
            && import.import_kind == ImportOrExportKind::Value
            && import
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        s.import_kind == ImportOrExportKind::Value
                            && s.imported.name() == "file"
                            && binding(object, s.local.symbol_id())
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => member(object)
                        .is_some_and(|(namespace, name)| {
                            name == "file" && binding(namespace, s.local.symbol_id())
                        }),
                    _ => false,
                })
    });
    if !imported {
        return None;
    }
    Some((kind, call.arguments[0].as_expression()?))
}

enum Target<'a, 's> {
    Value(&'s Expression<'a>),
    Missing(&'s ObjectExpression<'a>, String),
}

impl Target<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::Value(e) => e.get_inner_expression().span(),
            Self::Missing(o, _) => o.span,
        }
    }
}

fn segments(path: &str) -> Result<Vec<String>, Failure> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    let Some(path) = path.strip_prefix('/') else {
        return Err(fail(
            "invalid-pointer",
            Span::default(),
            "expected an RFC 6901 JSON Pointer",
        ));
    };
    path.split('/')
        .map(|s| {
            let mut chars = s.chars();
            let mut out = String::new();
            while let Some(c) = chars.next() {
                out.push(if c == '~' {
                    match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => {
                            return Err(fail(
                                "invalid-pointer",
                                Span::default(),
                                "invalid JSON Pointer escape",
                            ));
                        }
                    }
                } else {
                    c
                });
            }
            Ok(out)
        })
        .collect()
}

fn properties<'a, 's>(
    object: &'s ObjectExpression<'a>,
) -> Result<BTreeMap<String, &'s ObjectProperty<'a>>, Failure> {
    let mut properties = BTreeMap::new();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(p) = property else {
            return Err(computed(property.span()));
        };
        // Numeric and other JS coercible keys are not source field identities.
        if p.computed || p.method || p.kind != oxc_ast::ast::PropertyKind::Init {
            return Err(computed(p.span));
        }
        let key = match &p.key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(i) => i.name.to_string(),
            oxc_ast::ast::PropertyKey::StringLiteral(s) => {
                s.value.as_str().ok_or_else(|| computed(s.span))?.to_owned()
            }
            _ => return Err(computed(p.key.span())),
        };
        if key == "__proto__" || properties.insert(key, p.as_ref()).is_some() {
            return Err(computed(p.span));
        }
    }
    Ok(properties)
}

fn locate<'a, 's>(expr: &'s Expression<'a>, path: &[String]) -> Result<Target<'a, 's>, Failure> {
    let expr = expr.get_inner_expression();
    let Some((key, rest)) = path.split_first() else {
        return Ok(Target::Value(expr));
    };
    match expr {
        Expression::ObjectExpression(o) => {
            let properties = properties(o)?;
            match properties.get(key) {
                Some(p) => locate(&p.value, rest),
                None if rest.is_empty() => Ok(Target::Missing(o, key.clone())),
                None => Err(fail(
                    "missing-source",
                    o.span,
                    "parent object does not exist",
                )),
            }
        }
        Expression::ArrayExpression(a) => {
            if a.elements.iter().any(|e| e.as_expression().is_none()) {
                return Err(computed(a.span));
            }
            let index = key.parse::<usize>().ok().filter(|i| i.to_string() == *key);
            match index
                .and_then(|i| a.elements.get(i))
                .and_then(|e| e.as_expression())
            {
                Some(e) => locate(e, rest),
                None => Err(fail(
                    "missing-source",
                    a.span,
                    "expected an existing canonical array index",
                )),
            }
        }
        _ => Err(computed(expr.span())),
    }
}

fn literal(expr: &Expression<'_>) -> Result<Value, Failure> {
    let expr = expr.get_inner_expression();
    Ok(match expr {
        Expression::NullLiteral(_) => Value::Null,
        Expression::BooleanLiteral(b) => Value::Bool(b.value),
        Expression::StringLiteral(s) => {
            Value::String(s.value.as_str().ok_or_else(|| computed(s.span))?.into())
        }
        Expression::NumericLiteral(n) => serde_json::Number::from_f64(n.value)
            .map(Value::Number)
            .ok_or_else(|| computed(n.span))?,
        Expression::UnaryExpression(u)
            if u.operator == oxc_ast::ast::UnaryOperator::UnaryNegation =>
        {
            let Expression::NumericLiteral(n) = u.argument.get_inner_expression() else {
                return Err(computed(u.span));
            };
            serde_json::Number::from_f64(-n.value)
                .map(Value::Number)
                .ok_or_else(|| computed(u.span))?
        }
        Expression::ArrayExpression(a) => Value::Array(
            a.elements
                .iter()
                .map(|e| literal(e.as_expression().ok_or_else(|| computed(e.span()))?))
                .collect::<Result<_, _>>()?,
        ),
        Expression::ObjectExpression(o) => Value::Object(
            properties(o)?
                .into_iter()
                .map(|(k, p)| Ok((k, literal(&p.value)?)))
                .collect::<Result<_, _>>()?,
        ),
        _ => return Err(computed(expr.span())),
    })
}

fn same_optional(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(Value::Number(a)), Some(Value::Number(b))) => a.as_f64() == b.as_f64(),
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| same_optional(Some(a), Some(b)))
        }
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            a.len() == b.len() && a.iter().all(|(k, a)| same_optional(Some(a), b.get(k)))
        }
        (a, b) => a == b,
    }
}

fn representable(value: &Value, span: Span) -> Result<(), Failure> {
    match value {
        Value::Number(n) => {
            let f = n.as_f64().expect("JSON numbers are finite");
            if n.as_u64().is_some_and(|u| f as u128 != u as u128)
                || n.as_i64().is_some_and(|i| f as i128 != i as i128)
            {
                return Err(fail(
                    "invalid-value",
                    span,
                    "integer cannot be represented exactly by a TS number",
                ));
            }
        }
        Value::Array(a) => {
            for v in a {
                representable(v, span)?;
            }
        }
        Value::Object(o) => {
            if o.contains_key("__proto__") {
                return Err(fail(
                    "invalid-value",
                    span,
                    "__proto__ is not a data property in a TS object literal",
                ));
            }
            for v in o.values() {
                representable(v, span)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn single_quoted(json: &str) -> String {
    let mut out = "'".to_owned();
    let mut chars = json[1..json.len() - 1].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let next = chars.next().expect("JSON escape has a character");
                if next != '"' {
                    out.push('\\');
                }
                out.push(next);
            }
            '\'' => out.push_str("\\'"),
            _ => out.push(c),
        }
    }
    out.push('\'');
    out
}
