//! Deterministic TS source generators. This realm has no host I/O bindings;
//! modules and explicit inputs are read and hashed before it is entered.

use super::cache;
use og_core::ogt::{Block, Value};
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
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

const BUILTIN: &str = "@origingame/engine/generate";

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

fn read(root: &Path, path: &str) -> Result<Vec<u8>, String> {
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
    std::fs::read(at).map_err(|e| format!("{path}: {e}"))
}

/// The complete, closed module graph. No loader reaches the filesystem.
pub struct Program {
    entry: String,
    modules: BTreeMap<String, String>,
    pub key: String,
    context: String,
}

fn module(
    root: &Path,
    path: &str,
    modules: &mut BTreeMap<String, String>,
    hash: &mut blake3::Hasher,
) -> Result<(), String> {
    if modules.contains_key(path) {
        return Ok(());
    }
    if modules.len() >= 128 {
        return Err("a generator imports at most 128 modules".into());
    }
    let bytes = read(root, path)?;
    if bytes.len() > 1024 * 1024
        || modules.values().map(String::len).sum::<usize>() + bytes.len() > 8 * 1024 * 1024
    {
        return Err("generator source budget is 1 MiB per module and 8 MiB in total".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| format!("{path}: {e}"))?;
    hash.update(&(path.len() as u64).to_le_bytes());
    hash.update(path.as_bytes());
    hash.update(blake3::hash(&bytes).as_bytes());
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).map_err(|e| format!("{path}: {e}"))?;
    let parsed = Parser::new(&allocator, text, source_type).parse();
    if !parsed.diagnostics.is_empty() {
        return Err(format!("{path}: {:?}", parsed.diagnostics));
    }
    let mut program = parsed.program;
    let mut imports = Vec::new();
    for s in &program.body {
        let source = match s {
            Statement::ImportDeclaration(d) => d.source.value.as_str(),
            Statement::ExportAllDeclaration(d) => d.source.value.as_str(),
            Statement::ExportFromDeclaration(d) => d.source.value.as_str(),
            _ => None,
        };
        if let Some(name) = source {
            if name == BUILTIN {
                continue;
            }
            if !name.starts_with("./") && !name.starts_with("../") {
                return Err(format!(
                    "{path}: imports must be relative with an explicit suffix, or {BUILTIN}: {name}"
                ));
            }
            imports.push(relative(path, name)?);
        }
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
    modules.insert(path.to_string(), Codegen::new().build(&program).code);
    for name in imports {
        module(root, &name, modules, hash)?;
    }
    Ok(())
}

impl Program {
    pub fn prepare(root: &Path, source: &str, spec: &Spec) -> Result<Self, String> {
        let entry = relative(source, &spec.script)?;
        let mut hash = blake3::Hasher::new();
        hash.update(include_bytes!("generator.rs"));
        hash.update(include_bytes!(
            "../../../../crates/og-generation/src/lib.rs"
        ));
        hash.update(include_bytes!(
            "../../../../crates/og-api/src/generated/wire.rs"
        ));
        hash.update(og_generation::MODULE_SOURCE.as_bytes());
        hash.update(include_bytes!("../../../../Cargo.lock"));
        hash.update(&crate::format::revisions(&crate::format::Area::ALL).to_le_bytes());
        let mut modules = BTreeMap::from([(
            BUILTIN.to_string(),
            og_generation::MODULE_SOURCE.to_string(),
        )]);
        module(root, &entry, &mut modules, &mut hash)?;
        let mut inputs = BTreeMap::new();
        let mut total = 0;
        for (name, path) in &spec.inputs {
            let bytes = read(root, &relative(source, path)?)?;
            total += bytes.len();
            if total > 64 * 1024 * 1024 {
                return Err("generator inputs exceed 64 MiB in total".into());
            }
            inputs.insert(name, bytes);
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
            context,
        })
    }

    pub fn run(&self) -> Result<Json, String> {
        let store = cache::default_path().map(cache::Store::new);
        if let Some((bytes, _)) = store.as_ref().and_then(|s| s.action(&self.key)) {
            return serde_json::from_slice(&bytes).map_err(|e| e.to_string());
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
        let output = ctx.with(|ctx| -> Result<String, String> {
            let run = || -> rquickjs::Result<String> {
                ctx.eval::<(), _>(r#"
                    Math.random = () => { throw new Error('use ctx.seed and an explicit seeded algorithm'); };
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
                ctx.globals().set("__context", ctx.json_parse(self.context.as_bytes())?)?;
                let source = self.modules.get(&self.entry).expect("prepared entry");
                let (m, p) = Module::declare(ctx.clone(), self.entry.as_str(), source.as_bytes())?.eval()?;
                p.finish::<()>()?;
                let f: Function = m.get("default")?;
                ctx.globals().set("__generate", f)?;
                ctx.eval("(() => { const value = __generate(Object.freeze(__context)); if (value && typeof value.then === 'function') throw new Error('a generator must return synchronously'); return JSON.stringify(value, (_, v) => { if (typeof v === 'number' && !Number.isFinite(v)) throw new Error('non-finite generator output'); return ArrayBuffer.isView(v) ? Array.from(v) : v; }); })()")
            };
            run().catch(&ctx).map_err(|e| format!("{}: {e}", self.entry))
        })?;
        let value: Json = serde_json::from_str(&output).map_err(|e| {
            format!("generator must return synchronous JSON-compatible source data: {e}")
        })?;
        if let Some(store) = store {
            let _ = store.put_action(&self.key, output.as_bytes(), &[]);
        }
        Ok(value)
    }
}

/// A heightfield uses metres, not a normalised 0..1 image. Range mapping is
/// exclusively the terrain file's explicit `height` setting.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heightfield {
    pub columns: u32,
    pub rows: u32,
    pub heights: Vec<f32>,
}

impl Heightfield {
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
