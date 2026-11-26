//! Deterministic TS source generators. This realm has no host I/O bindings;
//! modules and explicit inputs are read and hashed before it is entered.

use super::cache;
use og_core::ogt::{Block, Value};
use oxc_allocator::Allocator;
use oxc_ast::ast::{AwaitExpression, ForOfStatement, ImportExpression, Statement};
use oxc_ast_visit::{Visit, walk};
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use rquickjs::{CatchResultExt, Context, Function, Module, Runtime, context::intrinsic};
use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub(super) mod author;
pub mod source;
pub mod value;

const BUILTIN: &str = "@origingame/engine/generate";
pub(super) const SOURCE_LIMIT: usize = 1024 * 1024;

/// Standalone script source blocks; terrain adds its processing stages.
pub const SCRIPT_LINES: &[crate::grammar::Line] = &[
    crate::grammar::Line::new(
        "script",
        "\"<file.ts>\"",
        "required",
        "Synchronous default export; static relative imports and explicit inputs are tracked.",
        "",
    ),
    crate::grammar::Line::new(
        "seed",
        "<u32>",
        "0",
        "Explicit seed passed to the generator.",
        "",
    ),
    crate::grammar::Line::new(
        "params",
        "{ ... }",
        "{}",
        "Named scalar, array or nested object parameters.",
        "",
    ),
    crate::grammar::Line::new(
        "input",
        "\"<name>\" \"<path>\"",
        "",
        "Repeated input bytes, relative to the authored asset, available as ctx.inputs[name].",
        "",
    ),
];

/// A module's default export receives `{params, seed, inputs}`. Inputs are
/// explicitly named byte arrays; paths are relative to the asset's directory.
#[derive(Clone, Debug)]
pub struct Spec {
    pub script: String,
    pub params: Json,
    pub seed: u32,
    pub inputs: BTreeMap<String, String>,
}

fn object(b: &Block) -> Result<Json, String> {
    let mut out = serde_json::Map::new();
    for e in &b.entries {
        let value = if let Some(b) = &e.block {
            object(b)?
        } else {
            let values = e
                .values
                .iter()
                .map(|v| match v {
                    Value::Number(n) => serde_json::to_value(n.as_f64()).map_err(|e| e.to_string()),
                    _ => Ok(v.as_bool().map_or_else(
                        || json!(v.as_str().or_else(|| v.as_word()).unwrap_or("")),
                        Json::Bool,
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            if values.len() == 1 {
                values[0].clone()
            } else {
                Json::Array(values)
            }
        };
        if out.insert(e.key.clone(), value).is_some() {
            return Err(format!("line {}: duplicate parameter '{}'", e.line, e.key));
        }
    }
    Ok(Json::Object(out))
}

impl Spec {
    /// A script-only generation block, without subsystem processing fields.
    pub fn standalone(b: &Block) -> Result<Self, String> {
        for e in &b.entries {
            if crate::grammar::find(SCRIPT_LINES, &e.key, &[]).is_none() {
                return Err(format!(
                    "line {}: unknown generate field '{}'",
                    e.line, e.key
                ));
            }
            if e.key != "input" && b.get_all(&e.key).count() != 1 {
                return Err(format!(
                    "line {}: duplicate generate field '{}'",
                    e.line, e.key
                ));
            }
        }
        let seed = match b.get("seed") {
            Some(e) if e.values.len() == 1 && e.block.is_none() => e
                .number(0)
                .and_then(og_core::ogt::Number::as_u64)
                .ok()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or("generate.seed is a u32 integer")?,
            Some(_) => return Err("generate.seed takes one u32 integer".into()),
            None => 0,
        };
        Self::parse(b, seed)?.ok_or_else(|| "generate needs script".into())
    }

    pub fn parse(b: &Block, seed: u32) -> Result<Option<Self>, String> {
        let Some(e) = b.get("script") else {
            return Ok(None);
        };
        let script = e
            .values
            .first()
            .and_then(|v| v.as_str())
            .filter(|_| e.values.len() == 1 && e.block.is_none())
            .ok_or_else(|| format!("line {}: script takes one quoted TS path", e.line))?
            .to_string();
        let params = match b.get("params") {
            Some(e) => object(
                e.block
                    .as_ref()
                    .ok_or_else(|| format!("line {}: params is a block", e.line))?,
            )?,
            None => json!({}),
        };
        let mut inputs = BTreeMap::new();
        for e in b.get_all("input") {
            let v: Vec<_> = e.values.iter().filter_map(|v| v.as_str()).collect();
            if v.len() != 2 || e.values.len() != 2 || e.block.is_some() {
                return Err(format!(
                    "line {}: input takes a quoted name and path",
                    e.line
                ));
            }
            if inputs.insert(v[0].to_string(), v[1].to_string()).is_some() {
                return Err(format!("line {}: duplicate input '{}'", e.line, v[0]));
            }
        }
        Ok(Some(Self {
            script,
            params,
            seed,
            inputs,
        }))
    }
}

fn relative(base: &str, name: &str) -> Result<String, String> {
    let path = Path::new(base).parent().unwrap_or(Path::new("")).join(name);
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            Component::ParentDir if out.pop() => {}
            _ => return Err(format!("generator path leaves its input root: {name}")),
        }
    }
    Ok(out.to_string_lossy().replace('\\', "/"))
}

fn read(root: &Path, path: &str, overlay: &BTreeMap<String, String>) -> Result<Vec<u8>, String> {
    let path = relative("", path)?;
    let mut at = root.to_path_buf();
    for part in Path::new(&path).components() {
        at.push(part);
        if std::fs::symlink_metadata(&at)
            .map_err(|e| format!("{path}: {e}"))?
            .file_type()
            .is_symlink()
        {
            return Err(format!("{path}: generator inputs cannot follow symlinks"));
        }
    }
    let meta = std::fs::metadata(&at).map_err(|e| format!("{path}: {e}"))?;
    if !meta.is_file() || meta.len() > 64 * 1024 * 1024 {
        return Err(format!(
            "{path}: generator input must be a file of at most 64 MiB"
        ));
    }
    let bytes = match overlay.get(&path) {
        Some(text) => text.as_bytes().to_vec(),
        None => std::fs::read(at).map_err(|e| format!("{path}: {e}"))?,
    };
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(format!(
            "{path}: generator input must be a file of at most 64 MiB"
        ));
    }
    Ok(bytes)
}

/// The complete, closed module graph. No loader reaches the filesystem.
pub struct Program {
    entry: String,
    modules: BTreeMap<String, String>,
    pub key: String,
    /// Sorted, root-relative source and input paths, excluding built-in modules.
    pub dependencies: Vec<String>,
    /// Original entry bytes from the same read used to prepare this program.
    /// Literal edits never follow imported/shared definitions.
    pub source: source::Source,
    context: String,
    author_inputs: Arc<BTreeMap<String, Vec<u8>>>,
    export: Export,
}

#[derive(Clone, Copy)]
enum Export {
    Call,
    Value,
}

#[derive(Default)]
struct Synchronous {
    forbidden: Option<(u32, &'static str)>,
}

impl<'a> Visit<'a> for Synchronous {
    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        self.forbidden
            .get_or_insert((expr.span.start, "dynamic import"));
    }

    fn visit_await_expression(&mut self, expr: &AwaitExpression<'a>) {
        self.forbidden.get_or_insert((expr.span.start, "await"));
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        if stmt.r#await {
            self.forbidden.get_or_insert((stmt.span.start, "for await"));
        }
        walk::walk_for_of_statement(self, stmt);
    }
}

struct ModuleGraph {
    modules: BTreeMap<String, String>,
    inputs: BTreeMap<String, bool>,
    hash: blake3::Hasher,
    entry_source: Option<source::Source>,
}

impl ModuleGraph {
    fn read(
        &mut self,
        root: &Path,
        path: &str,
        overlay: &BTreeMap<String, String>,
    ) -> Result<(), String> {
        if self.modules.contains_key(path) {
            return Ok(());
        }
        if self.modules.len() >= 128 {
            return Err("a generator imports at most 128 modules".into());
        }
        let bytes = read(root, path, overlay)?;
        if bytes.len() > SOURCE_LIMIT
            || self.modules.values().map(String::len).sum::<usize>() + bytes.len() > 8 * 1024 * 1024
        {
            return Err("generator source budget is 1 MiB per module and 8 MiB in total".into());
        }
        let text = std::str::from_utf8(&bytes).map_err(|e| format!("{path}: {e}"))?;
        self.entry_source
            .get_or_insert_with(|| source::Source::new(path, text));
        self.hash.update(&(path.len() as u64).to_le_bytes());
        self.hash.update(path.as_bytes());
        self.hash.update(blake3::hash(&bytes).as_bytes());
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).map_err(|e| format!("{path}: {e}"))?;
        let parsed = Parser::new(&allocator, text, source_type).parse();
        if !parsed.diagnostics.is_empty() {
            return Err(format!("{path}: {:?}", parsed.diagnostics));
        }
        let mut program = parsed.program;
        let mut synchronous = Synchronous::default();
        synchronous.visit_program(&program);
        if let Some((offset, feature)) = synchronous.forbidden {
            let before = &text[..offset as usize];
            let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
            let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
            return Err(format!(
                "{path}:{line}:{column}: {feature} is not allowed in synchronous cook modules"
            ));
        }
        let semantics = SemanticBuilder::new().build(&program);
        if !semantics.diagnostics.is_empty() {
            return Err(format!("{path}: {:?}", semantics.diagnostics));
        }
        let mut options = TransformOptions::default();
        options.env.es2016.exponentiation_operator = true;
        let transformed = Transformer::new(&allocator, Path::new(path), &options)
            .build_with_scoping(semantics.semantic.into_scoping(), &mut program);
        if !transformed.diagnostics.is_empty() {
            return Err(format!("{path}: {:?}", transformed.diagnostics));
        }
        for (input, text) in
            author::collect(&program, &transformed.scoping).map_err(|(offset, message)| {
                let before = &text[..offset as usize];
                let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
                let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
                format!("{path}:{line}:{column}: {message}")
            })?
        {
            let input = relative("", &input).map_err(|e| format!("{path}: {e}"))?;
            *self.inputs.entry(input).or_default() |= text;
        }
        // Only executable imports enter the realm. Type-only imports are erased
        // by OXC; their resolution belongs to the project's type checker.
        let mut imports = Vec::new();
        for s in &program.body {
            let source = match s {
                Statement::ImportDeclaration(d) => d.source.value.as_str(),
                Statement::ExportAllDeclaration(d) => d.source.value.as_str(),
                Statement::ExportFromDeclaration(d) => d.source.value.as_str(),
                _ => None,
            };
            if let Some(name) = source {
                if name == BUILTIN || name == author::MODULE {
                    continue;
                }
                if !name.starts_with("./") && !name.starts_with("../") {
                    return Err(format!(
                        "{path}: imports must be relative with an explicit suffix, {BUILTIN} or {}: {name}",
                        author::MODULE
                    ));
                }
                imports.push(relative(path, name)?);
            }
        }
        self.modules
            .insert(path.to_string(), Codegen::new().build(&program).code);
        for name in imports {
            self.read(root, &name, overlay)?;
        }
        Ok(())
    }
}

impl Program {
    pub fn prepare(root: &Path, source: &str, spec: &Spec) -> Result<Self, String> {
        Self::prepare_export(root, source, spec, Export::Call, &BTreeMap::new())
    }

    /// Evaluate an authored module's default data export, without calling it.
    /// Uses the same closed imports, isolated realm, budgets and action cache
    /// as generators. Domain schema validation belongs to the consuming cook.
    pub fn prepare_value(root: &Path, script: &str) -> Result<Self, String> {
        Self::prepare_value_with_overlay(root, script, &BTreeMap::new())
    }

    /// Validate a candidate save without writing it first. Overlay keys are
    /// normalized root-relative TS paths to existing regular source files.
    /// Imports, explicit inputs, source snapshots and action keys all observe
    /// the same candidate bytes. Unused candidates do not enter this action.
    pub fn prepare_value_with_overlay(
        root: &Path,
        script: &str,
        overlay: &BTreeMap<String, String>,
    ) -> Result<Self, String> {
        Self::prepare_export(
            root,
            "",
            &Spec {
                script: script.into(),
                params: json!({}),
                seed: 0,
                inputs: BTreeMap::new(),
            },
            Export::Value,
            overlay,
        )
    }

    fn prepare_export(
        root: &Path,
        source: &str,
        spec: &Spec,
        export: Export,
        overlay: &BTreeMap<String, String>,
    ) -> Result<Self, String> {
        for path in overlay.keys() {
            if relative("", path)? != *path || !path.ends_with(".ts") {
                return Err(format!(
                    "{path}: candidate source must be a normalized root-relative .ts path"
                ));
            }
            if read(root, path, overlay)?.len() > SOURCE_LIMIT {
                return Err(format!(
                    "{path}: candidate source budget is 1 MiB per module"
                ));
            }
        }
        let entry = relative(source, &spec.script)?;
        let mut hash = blake3::Hasher::new();
        hash.update(match export {
            Export::Call => b"call",
            Export::Value => b"data",
        });
        hash.update(include_bytes!("generator.rs"));
        hash.update(include_bytes!("generator_value.js"));
        hash.update(include_bytes!("generator/value.rs"));
        hash.update(include_bytes!("generator/author.rs"));
        hash.update(author::SOURCE.as_bytes());
        hash.update(&serde_json::to_vec(&author::kinds()).expect("kind names serialize"));
        hash.update(include_bytes!("../../../og-core/src/rng.rs"));
        hash.update(include_bytes!(
            "../../../../crates/og-generation/src/lib.rs"
        ));
        hash.update(include_bytes!(
            "../../../../crates/og-api/src/generated/wire.rs"
        ));
        hash.update(og_generation::MODULE_SOURCE.as_bytes());
        hash.update(include_bytes!("../../../../Cargo.lock"));
        hash.update(&crate::format::revisions(&crate::format::Area::ALL).to_le_bytes());
        let modules = BTreeMap::from([
            (
                BUILTIN.to_string(),
                og_generation::MODULE_SOURCE.to_string(),
            ),
            (author::MODULE.to_string(), author::SOURCE.to_string()),
        ]);
        let mut graph = ModuleGraph {
            modules,
            inputs: BTreeMap::new(),
            hash,
            entry_source: None,
        };
        graph.read(root, &entry, overlay)?;
        let ModuleGraph {
            modules,
            inputs: declared,
            mut hash,
            entry_source,
        } = graph;
        let mut dependencies: std::collections::BTreeSet<String> = modules
            .keys()
            .filter(|p| *p != BUILTIN && *p != author::MODULE)
            .cloned()
            .collect();
        let mut inputs = BTreeMap::new();
        let mut total = 0;
        for (name, path) in &spec.inputs {
            let path = relative(source, path)?;
            let bytes = read(root, &path, overlay)?;
            dependencies.insert(path);
            total += bytes.len();
            if total > 64 * 1024 * 1024 {
                return Err("generator inputs exceed 64 MiB in total".into());
            }
            inputs.insert(name, bytes);
        }
        let mut author_inputs = BTreeMap::new();
        for (path, text) in declared {
            let bytes = read(root, &path, overlay)?;
            total += bytes.len();
            if total > 64 * 1024 * 1024 {
                return Err("generator inputs exceed 64 MiB in total".into());
            }
            if text {
                std::str::from_utf8(&bytes)
                    .map_err(|e| format!("{path}: input.text requires UTF-8: {e}"))?;
            }
            hash.update(&(path.len() as u64).to_le_bytes());
            hash.update(path.as_bytes());
            hash.update(blake3::hash(&bytes).as_bytes());
            dependencies.insert(path.clone());
            author_inputs.insert(path, bytes);
        }
        // JS has one numeric parameter type. OGT's 17.0 and CLI JSON's 17
        // must identify the same action, not two spelling-dependent caches.
        fn numbers(value: &mut Json) {
            match value {
                Json::Number(n) => *value = json!(n.as_f64()),
                Json::Array(v) => v.iter_mut().for_each(numbers),
                Json::Object(v) => v.values_mut().for_each(numbers),
                _ => {}
            }
        }
        let mut params = spec.params.clone();
        numbers(&mut params);
        let context = json!({"params": params, "seed": spec.seed, "inputs": inputs}).to_string();
        hash.update(context.as_bytes());
        Ok(Self {
            entry,
            modules,
            key: hash.finalize().to_hex().to_string(),
            dependencies: dependencies.into_iter().collect(),
            source: entry_source
                .ok_or("generator entry must be a source file, not a built-in module")?,
            context,
            author_inputs: Arc::new(author_inputs),
            export,
        })
    }

    /// JSON is an explicit projection for existing consumers, not the
    /// evaluator's transfer or cache format. There is only one evaluator.
    pub fn run(&self) -> Result<Json, String> {
        self.run_ir().map(|data| data.root().into_json())
    }

    /// Evaluate into owned canonical author data. Numeric buffers retain their
    /// element type and little-endian bytes across the VM and action cache.
    pub fn run_ir(&self) -> Result<value::Data, String> {
        let store = cache::default_path().map(cache::Store::new);
        if let Some((bytes, _)) = store.as_ref().and_then(|s| s.action(&self.key)) {
            return value::Data::from_bytes(bytes).map_err(|e| format!("{}: {e}", self.entry));
        }
        let rt = Runtime::new().map_err(|e| e.to_string())?;
        rt.set_memory_limit(256 * 1024 * 1024);
        rt.set_max_stack_size(1024 * 1024);
        let fuel = Arc::new(AtomicUsize::new(0));
        rt.set_interrupt_handler(Some(Box::new(move || {
            fuel.fetch_add(1, Ordering::Relaxed) >= 100_000
        })));
        let mut resolver = rquickjs::loader::BuiltinResolver::default();
        let mut loader = rquickjs::loader::BuiltinLoader::default();
        for (name, code) in &self.modules {
            resolver.add_module(name);
            loader.add_module(name, code.as_bytes().to_vec());
        }
        rt.set_loader(resolver, loader);
        // No Date, Performance, OS, filesystem, network, timers or process.
        let ctx = Context::custom::<(
            intrinsic::Eval,
            intrinsic::Json,
            intrinsic::TypedArrays,
            intrinsic::MapSet,
            intrinsic::Promise,
        )>(&rt)
        .map_err(|e| e.to_string())?;
        let output = ctx.with(|ctx| -> Result<Vec<u8>, String> {
            let run = || -> rquickjs::Result<Vec<u8>> {
                ctx.eval::<(), _>(r#"
                    Math.random = () => { throw new Error('use rng(seed) from @origingame/engine/author'); };
                    delete globalThis.SharedArrayBuffer;
                    delete globalThis.Atomics;
                    // All executable source must pass the tracked TS transform,
                    // including exponentiation's software Math.pow lowering.
                    for (const f of [function(){}, async function(){}, function*(){}, async function*(){}]) {
                        Object.defineProperty(Object.getPrototypeOf(f), 'constructor', {value: undefined});
                    }
                    delete globalThis.eval;
                    delete globalThis.Function;
                "#)?;
                // QuickJS's default transcendental functions use the host C
                // library. Use the same software libm on every cook host.
                let math: rquickjs::Object = ctx.globals().get("Math")?;
                for (name, f) in [
                    ("sin", libm::sin as fn(f64) -> f64), ("cos", libm::cos),
                    ("tan", libm::tan), ("asin", libm::asin), ("acos", libm::acos),
                    ("atan", libm::atan), ("sinh", libm::sinh), ("cosh", libm::cosh),
                    ("tanh", libm::tanh), ("asinh", libm::asinh), ("acosh", libm::acosh),
                    ("atanh", libm::atanh), ("exp", libm::exp), ("expm1", libm::expm1),
                    ("log", libm::log), ("log1p", libm::log1p), ("log2", libm::log2),
                    ("log10", libm::log10), ("sqrt", libm::sqrt), ("cbrt", libm::cbrt),
                ] { math.set(name, rquickjs::function::Func::from(f))?; }
                math.set("pow", rquickjs::function::Func::from(libm::pow as fn(f64, f64) -> f64))?;
                math.set("atan2", rquickjs::function::Func::from(libm::atan2 as fn(f64, f64) -> f64))?;
                math.set("hypot", rquickjs::function::Func::from(|v: rquickjs::function::Rest<f64>| {
                    v.0.into_iter().fold(0.0, libm::hypot)
                }))?;
                ctx.globals().set("__procgen", Function::new(ctx.clone(), |ctx: rquickjs::Ctx<'_>, name: String, args: String| {
                    let call = || -> Result<String, String> {
                        let args: Vec<Json> = serde_json::from_str(&args).map_err(|e| e.to_string())?;
                        Ok(og_generation::invoke(&name, &args)?.to_string())
                    };
                    call().map_err(|e| rquickjs::Exception::throw_message(&ctx, &e))
                })?)?;
                author::install(&ctx, Arc::clone(&self.author_inputs))?;
                ctx.globals().set("__context", ctx.json_parse(self.context.as_bytes())?)?;
                let encode: Function = ctx.eval(include_str!("generator_value.js"))?;
                let source = self.modules.get(&self.entry).expect("prepared entry");
                let (m, p) = Module::declare(ctx.clone(), self.entry.as_str(), source.as_bytes())?.eval()?;
                p.finish::<()>()?;
                let value: rquickjs::Value = match self.export {
                    Export::Value => m.get("default")?,
                    Export::Call => {
                        let f: Function = m.get("default")?;
                        let context: rquickjs::Value = ctx.eval("Object.freeze(__context)")?;
                        f.call((context,))?
                    }
                };
                let bytes: rquickjs::TypedArray<u8> = encode.call((value,))?;
                // SAFETY: copy immediately; no JS runs while this slice exists.
                Ok(unsafe { bytes.as_bytes() }.ok_or(rquickjs::Error::Unknown)?.to_vec())
            };
            run().catch(&ctx).map_err(|e| format!("{}: {e}", self.entry))
        })?;
        let value = value::Data::from_bytes(output).map_err(|e| format!("{}: {e}", self.entry))?;
        if let Some(store) = store {
            let _ = store.put_action(&self.key, value.as_bytes(), &[]);
        }
        Ok(value)
    }
}

/// A heightfield uses metres, not a normalised 0..1 image. Range mapping is
/// exclusively the terrain file's explicit `height` setting.
#[derive(Serialize, Deserialize, og_core::reflect::source::schemars::JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Heightfield {
    /// Sample columns; 2..4097. Together with rows, fixes the height count.
    pub columns: u32,
    /// Sample rows; 2..4097, ordered from -z to +z.
    pub rows: u32,
    /// Finite row-major heights in metres; never normalised or clipped.
    #[schemars(extend("x-og-unit" = "m"))]
    pub heights: Vec<f32>,
}

impl Heightfield {
    pub fn schema() -> &'static og_core::reflect::source::InputSchema<Self> {
        static SCHEMA: std::sync::OnceLock<og_core::reflect::source::InputSchema<Heightfield>> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            og_core::reflect::source::InputSchema::new().expect("heightfield input schema")
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(2..=4097).contains(&self.columns)
            || !(2..=4097).contains(&self.rows)
            || self.heights.len() as u64 != u64::from(self.columns) * u64::from(self.rows)
            || self.heights.iter().any(|v| !v.is_finite())
        {
            return Err("a heightfield needs columns/rows in 2..4097 and columns × rows finite heights in metres".into());
        }
        Ok(())
    }
}
