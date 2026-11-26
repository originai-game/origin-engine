//! Pure author capabilities and their tracked inputs. Resolve bindings, not
//! spellings: a helper's local `input` is not a cook capability.
use oxc_ast::ast::{
    Argument, CallExpression, Expression, IdentifierReference, ImportDeclarationSpecifier, Program,
    Statement, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::{Scoping, SymbolId};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub const MODULE: &str = "@origingame/engine/author";
pub const SOURCE: &str = include_str!("../../../../../packages/engine/src/author.js");

/// Contextual kinds are bound per project, never in the process-global registry.
pub fn kinds() -> Vec<&'static str> {
    super::super::source_schemas::all()
        .map(|kind| kind.name)
        .chain(["scene", "prefab", "table"])
        .collect()
}

struct Inputs<'s> {
    scoping: &'s Scoping,
    namespaces: BTreeSet<SymbolId>,
    inputs: BTreeSet<SymbolId>,
    /// True when any declaration requires strict UTF-8 text decoding.
    files: BTreeMap<String, bool>,
    error: Option<(u32, &'static str)>,
}

impl Inputs<'_> {
    fn binding(&self, expr: &Expression<'_>, symbols: &BTreeSet<SymbolId>) -> bool {
        let Expression::Identifier(id) = expr else {
            return false;
        };
        self.is_binding(id, symbols)
    }

    fn is_binding(&self, id: &IdentifierReference<'_>, symbols: &BTreeSet<SymbolId>) -> bool {
        id.reference_id
            .get()
            .and_then(|r| self.scoping.get_reference(r).symbol_id())
            .is_some_and(|s| symbols.contains(&s))
    }

    fn input(&self, expr: &Expression<'_>) -> bool {
        self.binding(expr, &self.inputs)
            || matches!(expr, Expression::StaticMemberExpression(m)
                if !m.optional && m.property.name == "input" && self.binding(&m.object, &self.namespaces))
    }
}

impl<'a> Visit<'a> for Inputs<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(m) = &call.callee
            && self.input(&m.object)
        {
            if call.optional || m.optional || !matches!(m.property.name.as_str(), "bytes" | "text")
            {
                self.error
                    .get_or_insert((call.span.start, "call input.bytes or input.text directly"));
            } else if let [Argument::StringLiteral(path)] = call.arguments.as_slice()
                && let Some(path) = path.value.as_str()
            {
                *self.files.entry(path.into()).or_default() |= m.property.name == "text";
            } else {
                self.error.get_or_insert((
                    call.span.start,
                    "author input requires exactly one string literal path",
                ));
            }
            // A recognised declaration has no executable subexpressions.
            // All other uses of these bindings are visited and refused below.
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        if self.binding(&m.object, &self.namespaces) && m.property.name != "input" {
            // Other author exports are ordinary values, not input capabilities.
            return;
        }
        walk::walk_static_member_expression(self, m);
    }

    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if self.is_binding(id, &self.inputs) || self.is_binding(id, &self.namespaces) {
            self.error.get_or_insert((id.span.start, "use a direct author input.bytes/text call with a string literal; do not alias or re-export author inputs"));
        }
    }
}

pub fn collect(
    program: &Program<'_>,
    scoping: &Scoping,
) -> Result<BTreeMap<String, bool>, (u32, &'static str)> {
    let mut inputs = Inputs {
        scoping,
        namespaces: BTreeSet::new(),
        inputs: BTreeSet::new(),
        files: BTreeMap::new(),
        error: None,
    };
    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(d) if d.source.value == MODULE => {
                for specifier in d.specifiers.iter().flatten() {
                    match specifier {
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                            inputs.namespaces.insert(s.local.symbol_id());
                        }
                        ImportDeclarationSpecifier::ImportSpecifier(s)
                            if s.imported.name() == "input" =>
                        {
                            inputs.inputs.insert(s.local.symbol_id());
                        }
                        _ => {}
                    }
                }
            }
            Statement::ExportAllDeclaration(d) if d.source.value == MODULE => {
                return Err((
                    d.span.start,
                    "import author inputs directly in each module; do not re-export them",
                ));
            }
            Statement::ExportFromDeclaration(d)
                if d.source.value == MODULE
                    && d.specifiers.iter().any(|s| s.local.name() == "input") =>
            {
                return Err((
                    d.span.start,
                    "import author inputs directly in each module; do not re-export them",
                ));
            }
            _ => {}
        }
    }
    inputs.visit_program(program);
    match inputs.error {
        Some(error) => Err(error),
        None => Ok(inputs.files),
    }
}

/// A stream owns its draws; forks depend only on the seed, never the draw count.
/// Keep the engine algorithm in Rust, not a second implementation in JS.
fn stream<'js>(
    ctx: &rquickjs::Ctx<'js>,
    rng: og_core::Rng,
) -> rquickjs::Result<rquickjs::Object<'js>> {
    use rquickjs::{Function, function::MutFn, object::Property};
    let object = rquickjs::Object::new(ctx.clone())?;
    // rquickjs 0.14's u32 conversion round-trips through i32 with wrapping;
    // use f64 so seeds above i32::MAX retain their unsigned numeric value.
    object.prop("seed", Property::from(f64::from(rng.seed())).enumerable())?;
    let mut draws = rng;
    object.prop(
        "next",
        Property::from(Function::new(
            ctx.clone(),
            MutFn::new(move || draws.next_f64()),
        )?)
        .enumerable(),
    )?;
    object.prop(
        "fork",
        Property::from(Function::new(
            ctx.clone(),
            move |ctx: rquickjs::Ctx<'js>, label: String| stream(&ctx, rng.fork(&label)),
        )?)
        .enumerable(),
    )?;
    Ok(object)
}

/// Only pre-read inputs and explicit seeded streams cross the host boundary.
/// Each bytes call copies into VM-owned memory, never shares a mutable input.
pub fn install<'js>(
    ctx: &rquickjs::Ctx<'js>,
    inputs: Arc<BTreeMap<String, Vec<u8>>>,
) -> rquickjs::Result<()> {
    ctx.globals().set(
        "__authorKinds",
        ctx.json_parse(serde_json::to_vec(&kinds()).expect("kind names serialize"))?,
    )?;
    ctx.globals().set(
        "__authorRng",
        rquickjs::Function::new(
            ctx.clone(),
            |ctx: rquickjs::Ctx<'js>, seed: rquickjs::Value<'js>| {
                let seed = seed
                    .as_number()
                    .filter(|n| n.fract() == 0.0 && *n >= 0.0 && *n <= f64::from(u32::MAX))
                    .ok_or_else(|| {
                        rquickjs::Exception::throw_message(&ctx, "rng seed must be a u32 integer")
                    })?;
                stream(&ctx, og_core::Rng::new(seed as u32))
            },
        )?,
    )?;
    ctx.globals().set(
        "__authorInput",
        rquickjs::Function::new(
            ctx.clone(),
            move |ctx: rquickjs::Ctx<'js>, path: String, text: bool| {
                let path = super::relative("", &path)
                    .map_err(|e| rquickjs::Exception::throw_message(&ctx, &e))?;
                let bytes = inputs.get(&path).ok_or_else(|| {
                    rquickjs::Exception::throw_message(
                        &ctx,
                        "author input was not declared before evaluation",
                    )
                })?;
                if text {
                    let text = std::str::from_utf8(bytes)
                        .map_err(|e| rquickjs::Exception::throw_message(&ctx, &e.to_string()))?;
                    Ok::<_, rquickjs::Error>(rquickjs::String::from_str(ctx, text)?.into_value())
                } else {
                    Ok(rquickjs::TypedArray::<u8>::new_copy(ctx, bytes)?.into_value())
                }
            },
        )?,
    )
}
