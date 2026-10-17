//! The host generators: the Rust traits both hosts implement
//! (`crates/og-api/src/generated/host.rs`), the wasm-bindgen glue of
//! `og-host-web`, the napi glue of `og-host-node`, and the TypeScript
//! interface the loaders hand the runtime (`packages/engine/src/generated/abi.ts`).
//!
//! Every binding crosses the boundary in one wire form, the same on both
//! hosts (`Wire` below): numbers and booleans as themselves, handles as exact
//! JavaScript numbers, strings as strings, every structured value (structs,
//! lists, maps, options, results, unions) as JSON text, interface objects as
//! `u32` handles, game callbacks as `u32` ids into the runtime's callback
//! table, component and resource tokens by schema name. Only what aliases
//! engine memory differs per host, so it is an associated type of the host's
//! `Wire` implementation: `ArchetypeView.entities` (`Array`),
//! `ArchetypeView.column` (`Views`) and the dispatcher (`Function`).

use super::{bindings, default_json, schema_name, snake, Binding, GENERATED_BY};
use crate::index::{Index, TypeDecl};
use crate::model::*;

/// How one IDL type crosses the host boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wire {
    Unit,
    Bool,
    I32,
    U32,
    /// Every JavaScript number that is a float (`f32` and `f64` alike) and
    /// every handle: `f64`, exact for handles below 2^53.
    F64,
    /// A string.
    Text,
    /// Structured data as JSON text.
    Json,
    /// An interface object: a `u32` handle the host hands out.
    Object,
    /// A game callback: a `u32` id into the runtime's callback table.
    Callback,
    /// A component, resource or event token, by schema name.
    Token,
    /// A typed array the host hands out (`Self::Array`).
    Array,
    /// A component's column views (`Self::Views`).
    Views,
}

impl Wire {
    fn rust(self) -> &'static str {
        match self {
            Wire::Unit => "()",
            Wire::Bool => "bool",
            Wire::I32 => "i32",
            Wire::U32 | Wire::Object | Wire::Callback => "u32",
            Wire::F64 => "f64",
            Wire::Text | Wire::Json | Wire::Token => "String",
            Wire::Array => "Self::Array",
            Wire::Views => "Self::Views",
        }
    }

    fn ts(self) -> &'static str {
        match self {
            Wire::Unit => "void",
            Wire::Bool => "boolean",
            Wire::I32 | Wire::U32 | Wire::F64 | Wire::Object | Wire::Callback => "number",
            Wire::Text | Wire::Json | Wire::Token => "string",
            Wire::Array => "A",
            Wire::Views => "V",
        }
    }
}

/// The wire form of `ty` in the signature of `f` (whose generics bind names).
pub fn wire(ix: &Index<'_>, f: Option<&Function>, ty: &TypeRef) -> Wire {
    let name = ty.name();
    if let Some(g) = f.and_then(|f| f.generics.iter().find(|g| g.name == name)) {
        return if g.bound.name() == "Fields" {
            Wire::Json
        } else {
            Wire::Token
        };
    }
    match name {
        "Void" => Wire::Unit,
        "bool" => Wire::Bool,
        "i32" => Wire::I32,
        "u32" => Wire::U32,
        "f32" | "f64" => Wire::F64,
        "String" => Wire::Text,
        "Uint32Array" | "Float32Array" | "Float64Array" | "Int32Array" | "Uint8Array" => {
            Wire::Array
        }
        "View" => Wire::Views,
        _ => match ix.decl(name) {
            Some(TypeDecl::Handle(_)) => Wire::F64,
            Some(TypeDecl::Enum(_)) => Wire::Text,
            Some(TypeDecl::Interface(_)) => Wire::Object,
            Some(TypeDecl::Callback(_)) => Wire::Callback,
            _ => Wire::Json,
        },
    }
}

/// One parameter of a binding as the hosts see it.
pub struct WireParam {
    pub name: String,
    pub wire: Wire,
    pub optional: bool,
}

/// A binding's receiver (for members) and parameters, then its return.
pub fn signature(ix: &Index<'_>, b: &Binding<'_>) -> (Vec<WireParam>, Wire) {
    let mut params = Vec::new();
    if let Some(owner) = b.owner {
        // Interface members take their object's handle; resource methods
        // act on the resource of a world, so they take the world.
        let name = match ix.decl(owner) {
            Some(TypeDecl::Interface(_)) => snake(owner),
            _ => "world".to_string(),
        };
        params.push(WireParam {
            name,
            wire: Wire::Object,
            optional: false,
        });
    }
    let ret = match (b.function, b.prop) {
        (Some(f), _) => {
            let omit = omittable(&f.params, ix);
            for (p, optional) in f.params.iter().zip(omit) {
                params.push(WireParam {
                    name: snake(&p.name),
                    wire: wire(ix, Some(f), &p.ty),
                    optional,
                });
            }
            wire(ix, Some(f), &f.returns)
        }
        (None, Some(p)) => wire(ix, None, &p.ty),
        (None, None) => Wire::Unit,
    };
    (params, ret)
}

/// Trait name of a module: `world` → `WorldApi`.
fn trait_name(module: &str) -> String {
    let mut c = module.chars();
    match c.next() {
        Some(first) => format!("{}{}Api", first.to_ascii_uppercase(), c.as_str()),
        None => "Api".to_string(),
    }
}

fn rust_param(p: &WireParam) -> String {
    let ty = p.wire.rust();
    if p.optional {
        format!("{}: Option<{ty}>", p.name)
    } else {
        format!("{}: {ty}", p.name)
    }
}

fn first_doc(b: &Binding<'_>) -> String {
    let doc = match (b.function, b.prop) {
        (Some(f), _) => f.doc.first().cloned(),
        (None, Some(p)) => p.doc.first().cloned(),
        _ => None,
    };
    doc.unwrap_or_default()
}

/// The lifecycle members: not game API, but what the TypeScript loaders
/// drive. Both hosts implement them like any module.
/// `(name, Rust type, TypeScript type)` of a lifecycle parameter.
type LifecycleParam = (&'static str, &'static str, &'static str);

/// `(symbol, params, Rust return, doc)` of a lifecycle member.
type LifecycleMember = (
    &'static str,
    &'static [LifecycleParam],
    &'static str,
    &'static str,
);

const LIFECYCLE: &[LifecycleMember] = &[
    (
        "og_host_set_dispatcher",
        &[("dispatcher", "Self::Function", "Dispatcher")],
        "()",
        "Install the function the engine calls for systems, commands and facts: `dispatcher(kind, engine, id, query, tick, dt, elapsed, payload)` (payload: a command's arguments as a JSON list) returns `\"\"` on success, the error otherwise (`kind`: 0 system, 1 command, 2 facts, whose answer is `\"=\"` then JSON).",
    ),
    (
        "og_engine_create",
        &[("game", "u32", "number"), ("options", "String", "string")],
        "String",
        "Assemble a world from a declared game (`og.game`) and the declarations made so far. `options` is JSON `{ seed?, root? }`: a replay seed, and the project directory (native) or URL base (web) whose `cooked/` bundle is opened and whose GameDesc.scene is spawned before the first tick. The answer is a Result of the engine id, which is also the world handle.",
    ),
    (
        "og_engine_destroy",
        &[("engine", "u32", "number")],
        "bool",
        "Drop an engine and everything it owns. False for an unknown engine.",
    ),
    (
        "og_engine_advance",
        &[("engine", "u32", "number"), ("steps", "u32", "number")],
        "String",
        "Take fixed steps regardless of the wall clock: a Result of the tick reached, or the first failing system with its tick.",
    ),
    (
        "og_engine_command",
        &[
            ("engine", "u32", "number"),
            ("action", "String", "string"),
            ("args", "String", "string"),
        ],
        "String",
        "Run a registered command action now, between ticks (`args` is a JSON list of strings). The scheduled channel is `og.commands.schedule`.",
    ),
    (
        "og_engine_frame",
        &[("engine", "u32", "number"), ("elapsed_ms", "f64", "number")],
        "String",
        "One display frame: feed `elapsed_ms` of wall clock to the fixed-tick accumulator (unless paused), run the command steps due, take the ticks it allows up to `GameDesc.maxCatchUp` through the probe, then render when a renderer is attached. JSON `{ ok, ticks, tick, rendered, paused, quit }` or a refusal; `quit` is true once a `quit` command ran.",
    ),
    (
        "og_host_begin_bundle",
        &[],
        "()",
        "A new game bundle is about to be imported (hot reload): its component, resource and command declarations replace the previous bundle's; systems and games keep their handles. The probe's `reload` then assembles the latest `og.game` and restores the world into it.",
    ),
    (
        "og_engine_call",
        &[("engine", "u32", "number"), ("frame", "String", "string")],
        "String",
        "The probe's JSON-RPC 2.0 entry: one request frame in, its answer frame out (og-probe `Probe::call`). The native loopback WebSocket and the web request ring carry exactly these frames. Call it between ticks, never from a system.",
    ),
    (
        "og_engine_take_outputs",
        &[("engine", "u32", "number")],
        "String",
        "The files the command channel produced since the last call (`screenshot`, `dump_*`, `help`), as JSON `[{ tick, action, file, kind, data }]`: `file` is og-probe's `Output::file_name()`, `kind` is `png`, `json` or `text`, `data` is base64 for a PNG and the text otherwise. The host writes them (og run) or posts them.",
    ),
    (
        "og_engine_take_log",
        &[("engine", "u32", "number")],
        "String",
        "Log lines since the last call, as JSON `[{ tick, warning, text }]`: the command channel's and the probe's warnings and the loader's, each naming the fix.",
    ),
    (
        "og_engine_write_resource",
        &[
            ("world", "u32", "number"),
            ("resource", "String", "string"),
            ("value", "String", "string"),
        ],
        "String",
        "Write some fields of a resource: how the runtime makes `world.resource(R).value` a live value. A Result.",
    ),
];

/// `crates/og-api/src/generated/host.rs`.
pub fn host_rs(ix: &Index<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("// {GENERATED_BY}\n"));
    out.push_str(
        "//! The host contract, generated from the IDL: one trait per module with\n\
         //! callables, each method a binding symbol with its wire signature (see\n\
         //! `og_api::gen::host` for the wire forms). `og-host-web` and\n\
         //! `og-host-node` implement every trait, and their generated glue calls\n\
         //! the methods, so a member missing on either host is a build error.\n\n\
         // Field metadata (min, max, step) is written as the IDL states it.\n\
         #![allow(clippy::approx_constant)]\n\n",
    );
    out.push_str(
        "/// What aliases engine memory, chosen per host.\n\
         pub trait Wire {\n\
         \x20   /// A typed array the host hands out (`ArchetypeView.entities`).\n\
         \x20   type Array;\n\
         \x20   /// A component's column views (`ArchetypeView.column`).\n\
         \x20   type Views;\n\
         \x20   /// A JavaScript function the engine calls back.\n\
         \x20   type Function;\n\
         }\n\n",
    );
    out.push_str(
        "/// Engine lifecycle: what the TypeScript loaders drive.\npub trait Lifecycle: Wire {\n",
    );
    for (name, params, ret, doc) in LIFECYCLE {
        out.push_str(&format!("    /// {doc}\n"));
        let ps: Vec<String> = std::iter::once("&self".to_string())
            .chain(params.iter().map(|(n, t, _)| format!("{n}: {t}")))
            .collect();
        let r = if *ret == "()" {
            String::new()
        } else {
            format!(" -> {ret}")
        };
        out.push_str(&format!("    fn {name}({}){r};\n", ps.join(", ")));
    }
    out.push_str("}\n");

    let all = bindings(ix);
    let mut traits = vec!["Lifecycle".to_string()];
    for m in &ix.api.modules {
        let members: Vec<&Binding<'_>> = all.iter().filter(|b| b.module.name == m.name).collect();
        if members.is_empty() {
            continue;
        }
        let tname = trait_name(&m.name);
        out.push_str(&format!(
            "\n/// `{}`: {}\npub trait {tname}: Wire {{\n",
            m.name,
            m.doc.first().cloned().unwrap_or_default()
        ));
        for b in members {
            let (params, ret) = signature(ix, b);
            out.push_str(&format!("    /// `{}`: {}\n", b.path, first_doc(b)));
            let mut ps = vec!["&self".to_string()];
            ps.extend(params.iter().map(rust_param));
            let r = if ret == Wire::Unit {
                String::new()
            } else {
                format!(" -> {}", ret.rust())
            };
            out.push_str(&format!("    fn {}({}){r};\n", b.symbol, ps.join(", ")));
        }
        out.push_str("}\n");
        traits.push(tname);
    }
    out.push_str(&format!(
        "\n/// Every trait: a host is a type that implements all of them.\npub trait Host: {} {{}}\n\nimpl<T: {}> Host for T {{}}\n",
        traits.join(" + "),
        traits.join(" + ")
    ));

    out.push_str("\n/// Every binding symbol, in binding-table order (`bindings.json` ids).\npub const SYMBOLS: &[&str] = &[\n");
    for b in &all {
        out.push_str(&format!("    \"{}\",\n", b.symbol));
    }
    out.push_str("];\n");

    out.push_str(
        "\n/// One field of an engine-owned record.\n\
         #[derive(Clone, Copy, Debug, PartialEq)]\n\
         pub struct FieldDecl {\n\
         \x20   pub name: &'static str,\n\
         \x20   /// The column kind (components, resources) or the type (events).\n\
         \x20   pub column: &'static str,\n\
         \x20   /// The default value as JSON.\n\
         \x20   pub default: &'static str,\n\
         \x20   pub readonly: bool,\n\
         \x20   pub tunable: bool,\n\
         \x20   /// The meaningful range and Tune's increment; `None` when unstated.\n\
         \x20   pub min: Option<f64>,\n\
         \x20   pub max: Option<f64>,\n\
         \x20   pub step: Option<f64>,\n\
         \x20   /// A unit of the API's unit table, or empty.\n\
         \x20   pub unit: &'static str,\n\
         \x20   /// The person-facing name, or empty.\n\
         \x20   pub label: &'static str,\n\
         \x20   /// The field's docs, joined.\n\
         \x20   pub doc: &'static str,\n\
         \x20   /// An enum column's variants in index order; empty otherwise.\n\
         \x20   pub variants: &'static [&'static str],\n\
         }\n\n\
         /// An engine-owned component, resource or event.\n\
         #[derive(Clone, Copy, Debug, PartialEq)]\n\
         pub struct RecordDecl {\n\
         \x20   /// `component`, `resource` or `event`.\n\
         \x20   pub kind: &'static str,\n\
         \x20   /// The IDL name, the TypeScript token's name on `og`.\n\
         \x20   pub name: &'static str,\n\
         \x20   /// The schema name it registers under.\n\
         \x20   pub schema: &'static str,\n\
         \x20   pub fields: &'static [FieldDecl],\n\
         }\n\n\
         /// Every engine-owned record, in declaration order.\n\
         pub const RECORDS: &[RecordDecl] = &[\n",
    );
    for m in &ix.api.modules {
        for item in &m.items {
            let Item::Record(r) = item else { continue };
            out.push_str(&format!(
                "    RecordDecl {{\n        kind: \"{}\",\n        name: \"{}\",\n        schema: \"{}\",\n        fields: &[\n",
                r.kind.as_str(),
                r.name,
                schema_name(&r.name)
            ));
            for f in &r.fields {
                let default = default_json(ix, &f.ty, f.default.as_deref())
                    .map(|j| j.compact())
                    .unwrap_or_else(|| "null".to_string());
                let opt = |v: Option<f64>| match v {
                    Some(x) => format!("Some({x:?})"),
                    None => "None".to_string(),
                };
                let variants = match ix.decl(f.ty.name()) {
                    Some(crate::index::TypeDecl::Enum(e)) => e
                        .variants
                        .iter()
                        .map(|v| format!("{:?}", v.name))
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => String::new(),
                };
                out.push_str(&format!(
                    "            FieldDecl {{\n                name: \"{}\",\n                column: \"{}\",\n                default: {:?},\n                readonly: {},\n                tunable: {},\n                min: {},\n                max: {},\n                step: {},\n                unit: {:?},\n                label: {:?},\n                doc: {:?},\n                variants: &[{}],\n            }},\n",
                    f.name,
                    f.ty,
                    default,
                    f.readonly,
                    f.tunable,
                    opt(f.min),
                    opt(f.max),
                    opt(f.step),
                    f.unit.as_deref().unwrap_or_default(),
                    f.label.as_deref().unwrap_or_default(),
                    f.doc.join(" "),
                    variants
                ));
            }
            out.push_str("        ],\n    },\n");
        }
    }
    out.push_str("];\n");
    out
}

/// Which host a glue file is for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Web,
    Node,
}

fn glue_type(target: Target, w: &str) -> String {
    match (target, w) {
        (Target::Web, "Self::Array") => "Vec<u32>".into(),
        (Target::Web, "Self::Views") => "Vec<u32>".into(),
        (Target::Web, "Self::Function") => "js_sys::Function".into(),
        (Target::Node, "Self::Array") => "crate::NodeArray".into(),
        (Target::Node, "Self::Views") => "crate::NodeViews".into(),
        (Target::Node, "Self::Function") => "crate::NodeFunction".into(),
        (_, other) => other.into(),
    }
}

/// The export of one symbol for one host.
fn glue_fn(
    target: Target,
    out: &mut String,
    trait_name: &str,
    symbol: &str,
    params: &[(String, String)],
    ret: &str,
    publish: bool,
) {
    if !publish {
        out.push_str("#[cfg(not(feature = \"publish\"))]\n");
    }
    let mut ps: Vec<String> = params
        .iter()
        .map(|(n, t)| format!("{n}: {}", glue_type(target, t)))
        .collect();
    let args: Vec<&str> = params.iter().map(|(n, _)| n.as_str()).collect();
    let r = if ret == "()" {
        String::new()
    } else {
        format!(" -> {}", glue_type(target, ret))
    };
    match target {
        Target::Web => out.push_str("#[wasm_bindgen]\n"),
        Target::Node => {
            out.push_str(&format!("#[napi(js_name = \"{symbol}\")]\n"));
            ps.insert(0, "env: Env".to_string());
        }
    }
    out.push_str(&format!("pub fn {symbol}({}){r} {{\n", ps.join(", ")));
    if target == Target::Node {
        out.push_str("    let _env = crate::EnvScope::enter(&env);\n");
    }
    let call: Vec<&str> = std::iter::once("&Host").chain(args).collect();
    out.push_str(&format!(
        "    <Host as {trait_name}>::{symbol}({})\n}}\n\n",
        call.join(", ")
    ));
}

/// `crates/og-host-web/src/generated/glue.rs` or `crates/og-host-node/src/generated/glue.rs`.
pub fn glue(ix: &Index<'_>, target: Target) -> String {
    let mut out = String::new();
    out.push_str(&format!("// {GENERATED_BY}\n"));
    match target {
        Target::Web => out.push_str(
            "//! The wasm-bindgen exports of every binding symbol: each calls the\n\
             //! host's implementation of its `og_api::host` trait.\n\n\
             #![allow(clippy::too_many_arguments)]\n\n\
             use crate::WebHost as Host;\n\
             use og_api::host::*;\n\
             use wasm_bindgen::prelude::*;\n\n",
        ),
        Target::Node => out.push_str(
            "//! The napi exports of every binding symbol, under their snake_case\n\
             //! names: each calls the host's implementation of its `og_api::host` trait.\n\n\
             #![allow(clippy::too_many_arguments)]\n\n\
             use crate::NodeHost as Host;\n\
             use napi::Env;\n\
             use napi_derive::napi;\n\
             use og_api::host::*;\n\n",
        ),
    }
    for (name, params, ret, _) in LIFECYCLE {
        let ps: Vec<(String, String)> = params
            .iter()
            .map(|(n, t, _)| (n.to_string(), t.to_string()))
            .collect();
        glue_fn(target, &mut out, "Lifecycle", name, &ps, ret, true);
    }
    for b in bindings(ix) {
        let (params, ret) = signature(ix, &b);
        let ps: Vec<(String, String)> = params
            .iter()
            .map(|p| {
                let t = p.wire.rust().to_string();
                let t = if p.optional {
                    format!("Option<{t}>")
                } else {
                    t
                };
                (p.name.clone(), t)
            })
            .collect();
        glue_fn(
            target,
            &mut out,
            &trait_name(&b.module.name),
            &b.symbol,
            &ps,
            ret.rust(),
            b.publish,
        );
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    out
}

/// `packages/engine/src/generated/abi.ts`: the host ABI as the runtime sees it.
pub fn abi_ts(ix: &Index<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("// {GENERATED_BY}\n"));
    out.push_str(
        "// The host ABI: every binding symbol with its wire signature. Both hosts\n\
         // export exactly these functions; `A` is how a host hands out typed arrays\n\
         // and `V` how it hands out a component's column views.\n\n\
         /// The engine's call back into the runtime (`og_host_set_dispatcher`).\n\
         export type Dispatcher = (kind: number, engine: number, id: number, query: number, tick: number, dt: number, elapsed: number, payload: string) => string;\n\n\
         export interface HostAbi<A, V> {\n",
    );
    for (name, params, ret, doc) in LIFECYCLE {
        out.push_str(&format!("  /** {doc} */\n"));
        let ps: Vec<String> = params.iter().map(|(n, _, t)| format!("{n}: {t}")).collect();
        let r = match *ret {
            "()" => "void",
            "bool" => "boolean",
            _ => "string",
        };
        out.push_str(&format!("  {name}({}): {r};\n", ps.join(", ")));
    }
    let all = bindings(ix);
    for b in &all {
        let (params, ret) = signature(ix, b);
        let ps: Vec<String> = params
            .iter()
            .map(|p| {
                if p.optional {
                    format!("{}: {} | undefined", p.name, p.wire.ts())
                } else {
                    format!("{}: {}", p.name, p.wire.ts())
                }
            })
            .collect();
        out.push_str(&format!("  /** `{}` */\n", b.path));
        out.push_str(&format!(
            "  {}({}): {};\n",
            b.symbol,
            ps.join(", "),
            ret.ts()
        ));
    }
    out.push_str("}\n\n/** Every binding symbol, in binding-table order, then the lifecycle members. */\nexport const SYMBOLS: readonly string[] = [\n");
    for b in &all {
        out.push_str(&format!("  \"{}\",\n", b.symbol));
    }
    for (name, ..) in LIFECYCLE {
        out.push_str(&format!("  \"{name}\",\n"));
    }
    out.push_str("];\n");
    out
}
