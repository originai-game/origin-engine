//! `packages/engine/src/generated/calls.ts`: every host member as the game
//! reaches it. A module's functions are one frozen object per namespace
//! (`og.render`), an interface is a class over its handle (`World`), and a
//! resource's methods are made per world (`resourceMethods`). A member is
//! its symbol's call with the wire conversions (JSON text both ways, a token
//! by its name, a world by its id), so nothing between the ABI and the
//! `.d.ts` is written by hand. A member the IDL marks `custom` (or every
//! member of an interface marked `custom`, which the runtime builds itself)
//! calls the `Custom` table instead, which `src/runtime/custom.ts`
//! implements: a missing implementation is a type error, never a
//! `not a function` at run time.

use super::host::{Wire, signature, wire};
use super::{Binding, GENERATED_BY, bindings, default_json, schema_name};
use crate::index::{Index, TypeDecl};
use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Words a parameter may not be called in the generated code: the
/// language's and the file's own names.
const RESERVED: &[&str] = &[
    "abi",
    "bin",
    "binDecode",
    "binOpt",
    "custom",
    "json",
    "self",
    "r",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "let",
    "static",
    "implements",
    "interface",
    "package",
    "private",
    "protected",
    "public",
    "await",
];

fn ident(name: &str) -> String {
    if RESERVED.contains(&name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

/// A binding the TypeScript runtime implements by hand.
fn is_custom(ix: &Index<'_>, b: &Binding<'_>) -> bool {
    b.function.is_some_and(|f| f.custom)
        || b.prop.is_some_and(|p| p.custom)
        || b.owner.is_some_and(|o| custom_interface(ix, o))
}

fn custom_interface(ix: &Index<'_>, name: &str) -> bool {
    matches!(ix.decl(name), Some(TypeDecl::Interface(i)) if i.custom)
}

/// The interfaces some binding takes as a parameter: they cross by their
/// `id`, so their class keeps the handle there instead of privately.
fn crossing(ix: &Index<'_>, all: &[Binding<'_>]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for b in all {
        for p in b.function.map(|f| f.params.as_slice()).unwrap_or_default() {
            if matches!(ix.decl(p.ty.name()), Some(TypeDecl::Interface(_))) {
                out.insert(p.ty.name().to_string());
            }
        }
    }
    out
}

/// One parameter as the generated member declares and converts it.
struct Arg {
    name: String,
    ts: &'static str,
    optional: bool,
    convert: String,
}

fn param_ts(w: Wire) -> &'static str {
    match w {
        Wire::Unit => "void",
        Wire::Bool => "boolean",
        Wire::I32 | Wire::U32 | Wire::F64 => "number",
        Wire::Text => "string",
        Wire::Json | Wire::Binary | Wire::Views => "unknown",
        Wire::Token => "{ readonly name: string }",
        Wire::Object => "{ readonly id: number }",
        Wire::Callback => "Function",
        Wire::Array => "Uint32Array",
        Wire::Bytes => "Uint8Array",
    }
}

/// The parameters of `f` as the generated member declares and converts them.
fn args(ix: &Index<'_>, f: Option<&Function>) -> Vec<Arg> {
    let Some(f) = f else {
        return Vec::new();
    };
    let omit = omittable(&f.params, ix);
    f.params
        .iter()
        .zip(omit)
        .map(|(p, optional)| {
            let name = ident(&p.name);
            let w = wire(ix, Some(f), &p.ty);
            let convert = match (w, optional) {
                (Wire::Json, true) => format!("json({name})"),
                (Wire::Json, false) => {
                    // An omitted value is the type's empty one (null, {} or
                    // []) when it has one, so the host's reader names the
                    // field instead of the binding failing on undefined.
                    let empty = default_json(ix, &p.ty, None)
                        .map(|d| d.compact())
                        .filter(|d| matches!(d.as_str(), "null" | "{}" | "[]"));
                    match empty {
                        Some(d) => format!("JSON.stringify({name} ?? {d})"),
                        None => format!("JSON.stringify({name})"),
                    }
                }
                (Wire::Binary, true) => format!("binOpt({name})"),
                (Wire::Binary, false) => {
                    let empty = default_json(ix, &p.ty, None)
                        .map(|d| d.compact())
                        .filter(|d| matches!(d.as_str(), "null" | "{}" | "[]"));
                    match empty {
                        Some(d) => format!("bin({name} ?? {d})"),
                        None => format!("bin({name})"),
                    }
                }
                (Wire::Token, true) => format!("{name}?.name"),
                (Wire::Token, false) => format!("{name}.name"),
                (Wire::Object, true) => format!("{name}?.id"),
                (Wire::Object, false) => format!("{name}.id"),
                _ => name.clone(),
            };
            Arg {
                name,
                ts: param_ts(w),
                optional,
                convert,
            }
        })
        .collect()
}

/// `name?: T`, or `name: T | undefined` when a required one follows.
fn declare(args: &[Arg]) -> String {
    let last_required = args.iter().rposition(|a| !a.optional);
    args.iter()
        .enumerate()
        .map(|(i, a)| match (a.optional, last_required) {
            (true, Some(r)) if r > i => format!("{}: {} | undefined", a.name, a.ts),
            (true, _) => format!("{}?: {}", a.name, a.ts),
            (false, _) => format!("{}: {}", a.name, a.ts),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The member's TypeScript return type and the expression that returns it
/// from the symbol's call `call`, where `receiver` is the owner's handle.
fn answer(ix: &Index<'_>, b: &Binding<'_>, call: &str, receiver: Option<&str>) -> (String, String) {
    let (_, ret) = signature(ix, b);
    match ret {
        Wire::Json => ("unknown".into(), format!("JSON.parse({call}) as unknown")),
        Wire::Binary => ("unknown".into(), format!("binDecode({call})")),
        Wire::Object => {
            let ty = b
                .function
                .map(|f| f.returns.name().to_string())
                .or(b.prop.map(|p| p.ty.name().to_string()))
                .unwrap_or_default();
            // A builder answering its own handle answers itself, so calls chain on one object.
            let wrap = match receiver {
                Some(h) if b.owner == Some(ty.as_str()) => {
                    format!("{{ const r = {call}; return r === {h} ? this : new {ty}(r); }}")
                }
                _ => format!("new {ty}({call})"),
            };
            (ty, wrap)
        }
        Wire::Array => ("Uint32Array".into(), call.into()),
        w => (param_ts(w).into(), call.into()),
    }
}

/// `custom.<key>.<name>(<receiver>, <args>)`, the call a custom member makes.
fn custom_call(key: &str, name: &str, receiver: Option<&str>, args: &[Arg]) -> String {
    let all: Vec<&str> = receiver
        .into_iter()
        .chain(args.iter().map(|a| a.name.as_str()))
        .collect();
    format!("custom.{key}.{name}({})", all.join(", "))
}

/// The symbol's call.
fn host_call(b: &Binding<'_>, receiver: Option<&str>, args: &[Arg]) -> String {
    let all: Vec<&str> = receiver
        .into_iter()
        .chain(args.iter().map(|a| a.convert.as_str()))
        .collect();
    format!("abi().{}({})", b.symbol, all.join(", "))
}

/// An arrow function: a namespace's member, or a resource method over `world`.
fn arrow(
    ix: &Index<'_>,
    b: &Binding<'_>,
    key: &str,
    receiver: Option<&str>,
    custom: &mut Table,
) -> String {
    let a = args(ix, b.function);
    if is_custom(ix, b) {
        custom.add(key, b.name, receiver.map(|_| "world: number"), &a);
        return format!(
            "({}): unknown => {}",
            declare(&a),
            custom_call(key, b.name, receiver, &a)
        );
    }
    let call = host_call(b, receiver, &a);
    let (ty, body) = answer(ix, b, &call, None);
    format!("({}): {ty} => {body}", declare(&a))
}

/// The `Custom` interface's entries, by module or owner.
#[derive(Default)]
struct Table(BTreeMap<String, Vec<String>>);

impl Table {
    fn add(&mut self, key: &str, name: &str, receiver: Option<&str>, args: &[Arg]) {
        let params: Vec<String> = receiver
            .map(str::to_string)
            .into_iter()
            .chain(std::iter::once(declare(args)).filter(|d| !d.is_empty()))
            .collect();
        self.0
            .entry(key.to_string())
            .or_default()
            .push(format!("{name}({}): unknown;", params.join(", ")));
    }
}

/// One generated class: an interface's members over its handle, every
/// one it declares: a host member calls its symbol, a `custom` or a
/// `host false` one the table.
fn class(
    ix: &Index<'_>,
    i: &Interface,
    bound: &[&Binding<'_>],
    crosses: bool,
    custom: &mut Table,
    out: &mut String,
) {
    let name = &i.name;
    let h = if crosses { "this.id" } else { "this.#handle" };
    let _ = writeln!(
        out,
        "\nexport class {name}{} {{",
        i.extends
            .as_ref()
            .map(|e| format!(" extends {e}"))
            .unwrap_or_default()
    );
    if crosses {
        out.push_str("  /** Its handle, which the host functions taking one read. */\n  readonly id: number;\n");
    } else {
        out.push_str("  readonly #handle: number;\n");
    }
    let _ = writeln!(
        out,
        "  constructor(handle: number) {{\n{}    {h} = handle;\n  }}",
        if i.extends.is_some() {
            "    super(handle);\n"
        } else {
            ""
        }
    );
    let _ = writeln!(
        out,
        "  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */\n  static {}handleOf(o: {name}): number {{\n    return o.{};\n  }}",
        if i.extends.is_some() { "override " } else { "" },
        if crosses { "id" } else { "#handle" }
    );
    let self_ty = format!("self: {name}");
    let mut seen = BTreeSet::new();
    for m in &i.members {
        // Overloads (`host false` only) are one member.
        if !seen.insert(m.name()) {
            continue;
        }
        let (f, is_prop) = match m {
            Member::Fn(f) => (Some(f), false),
            Member::Prop(_) => (None, true),
        };
        let a = args(ix, f);
        let head = if is_prop {
            format!("get {}()", m.name())
        } else {
            format!("{}({})", m.name(), declare(&a))
        };
        let b = bound.iter().find(|b| b.name == m.name());
        let Some(b) = b.filter(|b| !is_custom(ix, b)) else {
            custom.add(name, m.name(), Some(&self_ty), &a);
            let call = custom_call(name, m.name(), Some("this"), &a);
            let _ = writeln!(out, "  {head}: unknown {{\n    return {call};\n  }}");
            continue;
        };
        let call = host_call(b, Some(h), &a);
        let (ty, body) = answer(ix, b, &call, Some(h));
        let body = match body.strip_prefix("{ ").and_then(|b| b.strip_suffix(" }")) {
            Some(block) => block.replace("; ", ";\n    "),
            None => format!("return {body};"),
        };
        let _ = writeln!(out, "  {head}: {ty} {{\n    {body}\n  }}");
    }
    out.push_str("}\n");
}

pub fn calls_ts(ix: &Index<'_>) -> String {
    let all = bindings(ix);
    let crosses = crossing(ix, &all);
    let mut custom = Table::default();
    let mut out = format!("// {GENERATED_BY}\n");
    out.push_str(
        "// Every host member as a game reaches it: og's namespaces, the interfaces'\n\
         // classes over their handles, and the resources' methods. Each is its\n\
         // binding symbol's call (src/generated/abi.ts) with the wire's conversions;\n\
         // a member the IDL marks `custom` calls src/runtime/custom.ts instead.\n\n\
         import { abi } from \"../runtime/host.ts\";\n\
         import { bin, binDecode, binOpt } from \"../runtime/binary.ts\";\n\
         import { custom } from \"../runtime/custom.ts\";\n\
         import { json } from \"../runtime/result.ts\";\n",
    );

    // ── interfaces ──
    let mut by_owner: BTreeMap<&str, Vec<&Binding<'_>>> = BTreeMap::new();
    for b in &all {
        if let Some(o) = b.owner {
            by_owner.entry(o).or_default().push(b);
        }
    }
    let mut done = BTreeSet::new();
    fn emit<'a>(
        ix: &Index<'a>,
        name: &str,
        by_owner: &BTreeMap<&str, Vec<&Binding<'a>>>,
        crosses: &BTreeSet<String>,
        done: &mut BTreeSet<String>,
        custom: &mut Table,
        out: &mut String,
    ) {
        let Some(TypeDecl::Interface(i)) = ix.decl(name) else {
            return;
        };
        if i.custom || !done.insert(name.to_string()) {
            return;
        }
        if let Some(base) = &i.extends {
            emit(ix, base, by_owner, crosses, done, custom, out);
        }
        let members = by_owner.get(name).cloned().unwrap_or_default();
        class(ix, i, &members, crosses.contains(name), custom, out);
    }
    for module in &ix.api.modules {
        for item in &module.items {
            if let Item::Interface(i) = item
                && by_owner.contains_key(i.name.as_str())
            {
                emit(
                    ix,
                    &i.name,
                    &by_owner,
                    &crosses,
                    &mut done,
                    &mut custom,
                    &mut out,
                );
            }
        }
    }

    // ── resources ──
    out.push_str("\n/** The methods each engine resource's object carries beside its fields, made for one world. */\nexport const resourceMethods: Readonly<Record<string, (world: number) => object>> = Object.freeze({\n");
    for module in &ix.api.modules {
        for item in &module.items {
            let Item::Record(r) = item else { continue };
            let members: Vec<&Binding<'_>> = all
                .iter()
                .filter(|b| b.owner == Some(r.name.as_str()))
                .collect();
            if members.is_empty() {
                continue;
            }
            let _ = writeln!(
                out,
                "  \"{}\": (world: number) => ({{",
                schema_name(&r.name)
            );
            for b in members {
                let _ = writeln!(
                    out,
                    "    {}: {},",
                    b.name,
                    arrow(ix, b, &r.name, Some("world"), &mut custom)
                );
            }
            out.push_str("  }),\n");
        }
    }
    out.push_str("});\n");

    // ── og and the package's functions ──
    let mut top = String::new();
    let mut og = String::new();
    for module in &ix.api.modules {
        let fns: Vec<&Binding<'_>> = all
            .iter()
            .filter(|b| b.owner.is_none() && std::ptr::eq(b.module, module))
            .collect();
        if fns.is_empty() {
            continue;
        }
        match module.place {
            Place::Top => {
                for b in fns {
                    let _ = writeln!(
                        top,
                        "  {}: {},",
                        b.name,
                        arrow(ix, b, &module.name, None, &mut custom)
                    );
                }
            }
            Place::Root => {
                for b in fns {
                    let _ = writeln!(
                        og,
                        "  {}: {},",
                        b.name,
                        arrow(ix, b, &module.name, None, &mut custom)
                    );
                }
            }
            Place::Namespace => {
                let _ = writeln!(og, "  {}: Object.freeze({{", module.name);
                for b in fns {
                    let _ = writeln!(
                        og,
                        "    {}: {},",
                        b.name,
                        arrow(ix, b, &module.name, None, &mut custom)
                    );
                }
                og.push_str("  }),\n");
            }
        }
    }
    let _ = write!(
        out,
        "\n/** The package's own functions (`defineComponent`). */\nexport const top = Object.freeze({{\n{top}}});\n\n/** og's functions and namespaces. */\nexport const og = Object.freeze({{\n{og}}});\n"
    );

    out.push_str("\n/** The members the IDL marks `custom`, by module or owner: src/runtime/custom.ts implements each. */\nexport interface Custom {\n");
    for (key, entries) in &custom.0 {
        let _ = writeln!(out, "  readonly {key}: {{");
        for e in entries {
            let _ = writeln!(out, "    {e}");
        }
        out.push_str("  };\n");
    }
    out.push_str("}\n");
    out
}
