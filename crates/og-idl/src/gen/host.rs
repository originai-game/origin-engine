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

use super::{Binding, GENERATED_BY, bindings, default_json, schema_name, snake};
use crate::index::{Index, TypeDecl};
use crate::json::Json;
use crate::model::*;
use std::fmt::Write;

/// `crates/og-api/src/generated/consts.rs`: the IDL's module constants as
/// Rust items (`og_api::consts`). Generated, so a constant the IDL gains is
/// published without a hand edit, and a crate that wants only the values
/// depends on og-api instead of restating them (`og-probe`, whose constants
/// the probe, the runtime and the hosts read).
pub fn consts_rs(ix: &Index<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("// {GENERATED_BY}\n"));
    out.push_str(
        "//! The IDL's module constants, generated from the IDL; `og_api::consts`\n\
         //! is them. The TypeScript side's is packages/engine/src/generated/consts.ts.\n",
    );
    for m in &ix.api.modules {
        for item in &m.items {
            let Item::Const(c) = item else { continue };
            let _ = writeln!(out);
            for line in &c.doc {
                let _ = writeln!(out, "/// {line}");
            }
            let (ty, value) = match &c.value {
                og_core::ogt::Value::String(s) => {
                    ("&str", Json::str(s.clone()).render().trim_end().to_string())
                }
                v => {
                    let ty = match c.ty {
                        TypeRef::Named { ref name, args: _ } if name == "u32" || name == "i32" => {
                            name.as_str()
                        }
                        _ => panic!(
                            "const `{}`: the Rust constants hold u32, i32 and String",
                            c.name
                        ),
                    };
                    (ty, crate::values::show(v))
                }
            };
            let _ = writeln!(out, "pub const {}: {ty} = {value};", c.name);
        }
    }
    out
}

/// `(name, Rust type, value, doc)` of a host-ABI constant. Authored here
/// once: the dispatcher's kinds and the shared buffer's layout. `host_rs`
/// emits them as `og_api::host::*` and `abi_ts` as abi.ts's exports, so the
/// engine's dispatcher calls and the TypeScript dispatcher's branches read
/// one table.
type AbiConst = (&'static str, &'static str, u32, &'static str);

const ABI_CONSTS: &[AbiConst] = &[
    (
        "DISPATCH_SYSTEM",
        "u32",
        0,
        "A system call (`Dispatch.kind`): the shared buffer opens with the time scale's four doubles, then the system's archetypes from word `FRAME_QUERY_WORD`.",
    ),
    (
        "DISPATCH_COMMAND",
        "u32",
        1,
        "A command handler's call (`Dispatch.kind`): the command's arguments are the payload's JSON list.",
    ),
    (
        "DISPATCH_FACTS",
        "u32",
        2,
        "The facts function's call (`Dispatch.kind`): its answer is `=` then the facts' JSON.",
    ),
    (
        "DISPATCH_CONDITION",
        "u32",
        3,
        "A system's run condition (`Dispatch.kind`): the answer is `=true` or `=false`.",
    ),
    (
        "DISPATCH_HOOK",
        "u32",
        4,
        "A hook's call (`Dispatch.kind`): the payload is the hooked entity.",
    ),
    (
        "DISPATCH_PLATFORM",
        "u32",
        5,
        "The platform pump's call (`Dispatch.kind`): the host drains what it handed the engine.",
    ),
    (
        "DISPATCH_UI",
        "u32",
        6,
        "A named UI registration pass (`Dispatch.kind`): the world is read only, and a changed world hash refuses it.",
    ),
    (
        "SHARED_BYTES",
        "usize",
        65536,
        "The shared buffer's size in bytes: what a host installs, and what both sides view.",
    ),
    (
        "FRAME_QUERY_WORD",
        "usize",
        8,
        "Where a system call's archetypes start in the shared buffer, in words: after the time scale's four doubles.",
    ),
];

/// The host-ABI constants as Rust items, appended to host.rs.
fn abi_consts_rs(out: &mut String) {
    out.push_str("\n/// The host ABI's constants: the dispatcher's kinds and the shared buffer's layout, one table (og-idl's gen/host.rs) both hosts read.\n");
    for (name, ty, value, doc) in ABI_CONSTS {
        let _ = writeln!(out, "/// {doc}\npub const {name}: {ty} = {value};");
    }
}

/// The host-ABI constants as TypeScript exports, written before the `Dispatcher` type.
fn abi_consts_ts(out: &mut String) {
    for (name, _, value, doc) in ABI_CONSTS {
        out.push_str(&format!("/** {doc} */\nexport const {name} = {value};\n"));
    }
}

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
    /// Bytes handed to the engine (`Bytes`, a `Uint8Array`): copied across
    /// as the host's `Self::Bytes`.
    Bytes,
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
            Wire::Bytes => "Self::Bytes",
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
            Wire::Bytes => "Uint8Array",
        }
    }
}

/// The wire form of `ty` in the signature of `f` (whose generics bind names).
pub fn wire(ix: &Index<'_>, f: Option<&Function>, ty: &TypeRef) -> Wire {
    let name = ty.name();
    if let Some(g) = f.and_then(|f| f.generics.iter().find(|g| g.name == name)) {
        return match g.bound.name() {
            "Fields" => Wire::Json,
            // A document's name, typed against the game's documents.
            "ScreenName" | "ComponentName" => Wire::Text,
            _ => Wire::Token,
        };
    }
    match name {
        "Void" => Wire::Unit,
        "bool" => Wire::Bool,
        "i32" => Wire::I32,
        "u32" => Wire::U32,
        "f32" | "f64" => Wire::F64,
        "String" => Wire::Text,
        "Bytes" => Wire::Bytes,
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
        "Install the function the engine calls for systems, commands and facts: `dispatcher(kind, engine, id, query, tick, dt, elapsed, payload)` (payload: a command's arguments as a JSON list) returns `\"\"` on success, the error otherwise (`kind`: 0 system, 1 command, 2 facts, whose answer is `\"=\"` then JSON). A system's call has an empty payload: its time scale and its query's archetypes are in the shared buffer (og_host_shared) when the dispatcher is entered.",
    ),
    (
        "og_host_shared",
        &[],
        "Self::Shared",
        "The shared buffer: 65536 bytes the engine and the runtime both see, where what crosses on every system call is written as numbers instead of text. Valid until the next engine call. A system's call: four f64 (the time scale, the hit stop's ticks left, the scaled dt and the scaled elapsed time) at bytes 0 to 31, then its query's archetypes from word 8 as og_engine_query_archetypes writes them.",
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
        "One display frame: feed `elapsed_ms` of wall clock to the fixed-tick accumulator (unless paused), run the command steps due, take the ticks it allows up to `GameDesc.maxCatchUpTicks` through the probe, then render when a renderer is attached. JSON `{ ok, ticks, tick, rendered, paused, quit }` or a refusal; `quit` is true once a `quit` command ran.",
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
        "The files the command channel produced since the last call (`screenshot`, `dump_*`, `help`), as JSON `[{ tick, action, file, kind, data }]`: `file` is og-probe's `Output::file_name()`, `kind` is `png`, `json`, `text` or `wav`, `data` is base64 for a PNG or a WAV and the text otherwise. The host writes them (og run) or posts them.",
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
        "Write some fields of a resource as JSON: the runtime's path for a write a publish build refuses (logged once per system). A Result.",
    ),
    (
        "og_engine_resource_layout",
        &[("world", "u32", "number"), ("resource", "String", "string")],
        "String",
        "A resource's packed layout, how the runtime reads and writes its fields as bytes: a Result of `{ epoch, size, fields: [{ name, kind, offset, size, readonly, capacity, variants, element }] }` (`kind`: f32, f64, u32, i32, bool, Vector2, Vector3, Quaternion, Color, Matrix4, handle, str, enum, or list: `element` one of f32, f64, u32, i32, handle and `capacity` its elements, a count lane first, each lane the element's size). Refused for og.Time and og.Rng, whose handles are computed, and for what is not a registered resource.",
    ),
    (
        "og_engine_read_resource",
        &[("world", "u32", "number"), ("resource", "String", "string")],
        "Self::Bytes",
        "A resource's packed bytes, then the layout epoch as a little-endian u32 (it moves when any layout is declared again: a new engine, a reload). Empty when refused (undeclared in the running system, not a resource): the runtime then asks og_world_resource for the reason.",
    ),
    (
        "og_engine_write_resource_bytes",
        &[
            ("world", "u32", "number"),
            ("resource", "String", "string"),
            ("bytes", "Self::Bytes", "Uint8Array"),
        ],
        "String",
        "Overwrite a resource with packed bytes followed by the layout epoch they were made with (as og_engine_read_resource answers): how the runtime flushes a handle's field writes, once before the next engine call and at the end of every system. Read-only fields keep their value. A Result; refused for a stale epoch, a wrong size, an invalid value or a resource the running system did not declare among its writes.",
    ),
    (
        "og_engine_component_id",
        &[
            ("world", "u32", "number"),
            ("component", "String", "string"),
        ],
        "u32",
        "A component's number in this world (its schema id index): what og_engine_query_archetypes lists as an archetype's types and og_engine_view_column takes. 4294967295 for a name this world has not registered as a component. It holds until the layout epoch moves.",
    ),
    (
        "og_engine_component_layout",
        &[
            ("world", "u32", "number"),
            ("component", "String", "string"),
        ],
        "String",
        "A component's packed layout, as og_engine_resource_layout answers a resource's, with its number (`id`, og_engine_component_id's) and its fields in the order world.get answers them (by name). A Result; refused for a name this world has not registered as a component.",
    ),
    (
        "og_engine_read_component",
        &[
            ("world", "u32", "number"),
            ("entity", "f64", "number"),
            ("component", "u32", "number"),
        ],
        "u32",
        "world.get without text: write the packed bytes of a component (og_engine_component_id's number) of an entity to the start of the shared buffer, and answer the layout epoch they were read under. 0 when refused (no such entity or component, undeclared in the running system, a column a view holds): the runtime then asks og_world_get for the reason.",
    ),
    (
        "og_engine_write_component",
        &[
            ("world", "u32", "number"),
            ("entity", "f64", "number"),
            ("component", "u32", "number"),
            ("epoch", "u32", "number"),
        ],
        "u32",
        "world.set without text: overwrite fields of a component the entity has from the shared buffer. The packed record is at the buffer's start; from the next word boundary after it come a count and that many (offset, size) word pairs, the byte ranges to take (the fields the value named); the rest keeps its value. `epoch` is the layout epoch the bytes were made with. The answer is the layout epoch now, and the write happened only when the two are equal. 0 when refused (as og_engine_read_component, or an invalid value): the runtime then calls og_world_set, which says why.",
    ),
    (
        "og_engine_query_archetypes",
        &[("query", "u32", "number"), ("from", "u32", "number")],
        "u32",
        "Write a query's archetypes, from its `from`th, to the shared buffer as u32 words from word 0, and answer how many were written: the query's archetype count, how many follow, their tick and the layout epoch; then per archetype its view handle, its entity count, its type count and its types (og_engine_component_id's numbers). What did not fit is asked for with a later `from`. `from` 0 makes a world query's views (Query.archetypes outside systems).",
    ),
    (
        "og_engine_view_column",
        &[("view", "u32", "number"), ("component", "u32", "number")],
        "Self::Views",
        "ArchetypeView.column: the columns of a component (og_engine_component_id's number) in a view, one per field in field order. A running system's view borrows them when first asked and holds them until the system returns. Empty when refused (a stale view, a component the system did not declare or the archetype lacks, a column borrowed otherwise).",
    ),
    (
        "og_engine_view_add_scaled",
        &[
            ("view", "u32", "number"),
            ("target", "u32", "number"),
            ("target_field", "u32", "number"),
            ("source", "u32", "number"),
            ("source_field", "u32", "number"),
            ("scale", "f64", "number"),
        ],
        "String",
        "ArchetypeView.addScaled: target += source * scale over the view's rows (every row, or those its change filters pass), in place in the columns og_engine_view_column hands out, the fields given by their place in that call's answer. Each f32 element is computed in f64 and rounded once. A Result; refused, with nothing changed, as og_engine_view_column is, for a read-only target or for columns of different sizes.",
    ),
    (
        "og_engine_ui_patch",
        &[
            ("world", "u32", "number"),
            ("bytes", "Self::Bytes", "Uint8Array"),
        ],
        "Self::Bytes",
        "Apply a binary UI source/computation patch batch. Replies are binary results in request order. Source values and the dependency graph belong to this engine; the TypeScript runtime executes requested computation IDs outside the engine lock.",
    ),
    (
        "og_engine_inspector_panel",
        &[("engine", "u32", "number")],
        "String",
        "The development inspector's panel since the last call, for the TSX panel the runtime mounts: its values as JSON while it is open and they changed, `null` once when it closed, empty when nothing changed (always empty in a publish build).",
    ),
    (
        "og_engine_inspector_event",
        &[("engine", "u32", "number"), ("event", "String", "string")],
        "bool",
        "One event of the inspector's TSX panel, JSON `{ kind, name, row, value }`, done at the next frame: `click` and `changed` (a control's prop) as the panel names them, `input` and `commit` of a field part's text, `cancel`, `focus` and `blur` of a text box, `panel` (the panel's node). False for a malformed event or a publish build.",
    ),
    (
        "og_platform_take",
        &[("engine", "u32", "number")],
        "String",
        "The platform calls (IDL `platform`) made since the last call, oldest first, as JSON `[{ request, method, args, tick }]`. The host answers each but `telemetry` (one-way) with og_platform_answer.",
    ),
    (
        "og_platform_answer",
        &[
            ("engine", "u32", "number"),
            ("request", "u32", "number"),
            ("answer", "String", "string"),
        ],
        "bool",
        "Answer one platform call: JSON `{ ok: true, data }` or `{ ok: false, error: { code, message, retryable } }`. It is applied at the start of the next tick (og.platform sends PlatformAnswered). False for an unknown, one-way or answered request, or a malformed answer.",
    ),
    (
        "og_platform_set_host",
        &[("engine", "u32", "number"), ("info", "String", "string")],
        "()",
        "Say what answers this engine's platform calls: JSON `{ host, capabilities }` (`player`, `local` or `none`; the methods it answers), as og.platform.info reports.",
    ),
    (
        "og_platform_notify",
        &[("engine", "u32", "number"), ("notice", "String", "string")],
        "bool",
        "Unasked host input for og.platform (docs/platform-asks.md #19): JSON `{ kind, data, time? }`, kind `purchase` (a purchase settled: `{ purchase, product, quantity, state }`), `entitlements` (`{ entitlements }`, changed without a call) or `invite` (`{ invite, from, payload, time }`). Applied at the start of the next tick (og.PlatformNotice) and logged with the answers. False for a malformed notice or an unknown kind.",
    ),
    (
        "og_room_take",
        &[("engine", "u32", "number")],
        "String",
        "The room ops (IDL `platform` rooms, og.net's lockstep) made since the last call, oldest first, as JSON `[op]`: `{ op: \"create\", name, capacity }`, `{ op: \"join\", room, name }`, `{ op: \"leave\" }`, `{ op: \"send\", to?, channel?, data? | bytes? }` (bytes base64; a channel starting `og.` is the engine's own). The host carries each in order (docs/design.md §2.8).",
    ),
    (
        "og_room_deliver",
        &[("engine", "u32", "number"), ("event", "String", "string")],
        "bool",
        "One room event from the host, JSON: `{ type: \"joined\", room, peer, peers: [{ peer, name }] }`, `{ type: \"peer_joined\", peer, name }`, `{ type: \"peer_left\", peer }`, `{ type: \"message\", from, channel?, data? | bytes? }`, `{ type: \"left\", reason }` or `{ type: \"error\", op, code, message }`. Lockstep's messages act between ticks at once; the game's events are sent at the start of the next tick. False for a malformed event.",
    ),
    (
        "og_room_set_host",
        &[("engine", "u32", "number"), ("info", "String", "string")],
        "()",
        "Say what carries this engine's rooms: JSON `{ host, join? }` (`relay`, `player` or `none`), as og.platform.roomInfo reports; with `none` roomCreate and roomJoin are refused. `join` is the room a Play link named (`?room=<id>`): the engine joins it once, itself, and og.RoomJoined answers.",
    ),
    (
        "og_shared_take",
        &[("engine", "u32", "number")],
        "String",
        "The shared-world ops (IDL `shared`) made since the last call, oldest first, as JSON `[op]`: `{ op: \"connect\", world, reconnect }` (ask the session provider for a connection and open it; `reconnect` after a lost one), `{ op: \"send\", bytes }` (one binary message, base64, on the open connection) or `{ op: \"close\" }`. The host carries each in order (docs/design.md §2.12).",
    ),
    (
        "og_shared_deliver",
        &[("engine", "u32", "number"), ("event", "String", "string")],
        "bool",
        "One shared-world event from the host, JSON: `{ type: \"host\", host }` (what carries sessions here: the provider's name, or `none`), `{ type: \"open\", transport, token }` (connected; the token is the provider's, sent in hello as it is), `{ type: \"message\", bytes }` (base64), `{ type: \"closed\", reason }` or `{ type: \"refused\", code, message }` (the provider refused). Applied at the start of the next tick and journaled (og.platform.journal) without the token. False for a malformed event.",
    ),
    (
        "og_engine_set_content",
        &[("engine", "u32", "number"), ("base", "String", "string")],
        "()",
        "Name the content base og.load.data reads catalogs from (guide loading §14), for an engine created with `contentLater`: a page that learns it from its Player's hello. Empty for none: waiting data loads then fail `unavailable`. Until it is named they stay pending (30 s at most on the web).",
    ),
    (
        "og_engine_trace",
        &[("engine", "u32", "number"), ("on", "bool", "boolean")],
        "()",
        "Turn an engine's trace on or off (`og run --trace`, the shell's `trace()`). Engine 0xffffffff is the loads made before an engine exists: its boot scene's.",
    ),
    (
        "og_engine_trace_take",
        &[("engine", "u32", "number"), ("last", "bool", "boolean")],
        "String",
        "The trace's spans since the last take, as JSON `{ now, events }`: Chrome trace events on the engine's clock. `last` waits for the GPU's frames in flight where the host can (natively; the web's arrive in a later take).",
    ),
    (
        "og_engine_perf",
        &[("engine", "u32", "number"), ("on", "bool", "boolean")],
        "()",
        "Turn an engine's performance record on (a fresh one) or off (`og run --perf`, the shell's `perf()`, guide perf).",
    ),
    (
        "og_engine_perf_report",
        &[("engine", "u32", "number"), ("budgets", "String", "string")],
        "String",
        "The performance record's report so far, as JSON `{ json, text }`: each line's mean, median, p95, p99 and max, flagged against `budgets` (a JSON object of line to milliseconds), and the hitches with their causes. The record is the engine's only one of frame times: `frame interval` (a presenting host's, og_present_frame's) and `frame work` are its lines.",
    ),
    (
        "og_engine_perf_frames",
        &[("engine", "u32", "number")],
        "String",
        "The performance record's frames since the last call, binned (field telemetry's `frames`): JSON `{ count, ms, bins, interval, work, hitches, longHitches, max }`, `bins` the 11 upper edges in ms (8.4 to 250) and `interval` and `work` 12 counts each, the last above 250 ms. A hitch is a frame whose interval is more than twice the median of the 64 before it, a long one also 250 ms or more. `null` while the record is off.",
    ),
];

/// The presenting host's members: what a host that shows the engine's
/// frames on a display (the web Worker's canvas, the desktop shell's
/// window) tells the engine and asks of it. The engine keeps the duties
/// themselves (pacing under the frame cap, the display's interval,
/// automatic quality's budget, the frame-time record), so every
/// presenting host keeps them alike.
const PRESENT: &[LifecycleMember] = &[
    (
        "og_present_frame",
        &[("engine", "u32", "number"), ("now_ms", "f64", "number")],
        "String",
        "One display frame of a presenting host at `now_ms`, the host's monotonic clock (requestAnimationFrame's timestamp, the desktop window loop's). The engine keeps the frame cap (og.render.setDisplay, DisplayInfo.maxFps) by drawing only some display frames, measures the display's interval from these timestamps unless og_present_refresh told it, and runs a drawn one as og_engine_frame with the time since the last drawn one, which the performance record takes as its interval. JSON: og_engine_frame's answer with `drawn: true`, or `{ ok: true, drawn: false }`; both with `nextMs`, the host time the next frame to draw is due, which a host without a display callback waits until (winit's ControlFlow::WaitUntil). A host calls it on every display frame, drawn or not, and names its locale at og_engine_create (`locale`).",
    ),
    (
        "og_present_resume",
        &[("engine", "u32", "number")],
        "()",
        "The clock stopped, held or went on (play paused or resumed, a probe capture's wait): the next og_present_frame draws, and is handed no time from before.",
    ),
    (
        "og_present_refresh",
        &[("engine", "u32", "number"), ("hz", "f64", "number")],
        "()",
        "The display's refresh rate as the host reads it from the display (the desktop window's monitor), Hz; 0 or less is unknown. A host that does not tell it calls og_present_frame on every display frame (requestAnimationFrame), whose timestamps measure it.",
    ),
    (
        "og_present_fullscreen",
        &[("engine", "u32", "number"), ("on", "bool", "boolean")],
        "()",
        "The host's window went fullscreen or left it (the page's fullscreenchange, the desktop window): DisplayInfo.fullscreen, and the player's setting follows it, so a player who leaves with Escape finds Fullscreen off. The frame answer's `fullscreen` is what the player asks for, which the host carries out.",
    ),
];

/// The host traits written beside the modules': name, doc and members.
const HOST_TRAITS: &[(&str, &str, &[LifecycleMember])] = &[
    (
        "Lifecycle",
        "Engine lifecycle: what the TypeScript loaders drive.",
        LIFECYCLE,
    ),
    (
        "Present",
        "What a presenting host tells the engine and asks of it: the frames it shows on a display.",
        PRESENT,
    ),
];

/// `crates/og-api/src/generated/host.rs`.
pub fn host_rs(ix: &Index<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("// {GENERATED_BY}\n"));
    out.push_str(
        "//! The host contract, generated from the IDL: one trait per module with\n\
         //! callables, each method a binding symbol with its wire signature (see\n\
         //! og-idl's `gen/host.rs` for the wire forms). `og-host-web` and\n\
         //! `og-host-node` implement every trait, and their generated glue calls\n\
         //! the methods, so a member missing on either host is a build error.\n\
         //!\n\
         //! The IDL's prose is not repeated here: a method names its TypeScript\n\
         //! path, documented in api.ogt and packages/knowledge/wiki/<module>.md.\n\
         //! An edit to the docs alone then leaves this file, and so the hosts\n\
         //! and the wasm, unchanged; field docs stay in RECORDS, which the\n\
         //! runtime's reflection serves.\n\n",
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
         \x20   /// Bytes copied across, either way (a resource's packed record).\n\
         \x20   type Bytes;\n\
         \x20   /// The shared buffer as the host hands it out (`og_host_shared`).\n\
         \x20   type Shared;\n\
         }\n\n",
    );
    let mut traits = Vec::new();
    for (i, (tname, tdoc, members)) in HOST_TRAITS.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("/// {tdoc}\npub trait {tname}: Wire {{\n"));
        for (name, params, ret, doc) in *members {
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
        traits.push(tname.to_string());
    }

    let all = bindings(ix);
    for m in &ix.api.modules {
        let members: Vec<&Binding<'_>> = all.iter().filter(|b| b.module.name == m.name).collect();
        if members.is_empty() {
            continue;
        }
        let tname = trait_name(&m.name);
        out.push_str(&format!(
            "\n/// `{0}`: packages/knowledge/wiki/{0}.md.\npub trait {tname}: Wire {{\n",
            m.name
        ));
        for b in members {
            let (params, ret) = signature(ix, b);
            out.push_str(&format!("    /// `{}`\n", b.path));
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

    out.push_str("\npub use og_records::{COLUMNS, ColumnDecl, FieldDecl, RECORDS, RecordDecl};\n");
    abi_consts_rs(&mut out);
    out
}

/// `crates/og-records/src/generated/records.rs`: the record and column
/// tables, out of host.rs so that og-cook, which reads only them, does not
/// rebuild when the traits change.
pub fn records_rs(ix: &Index<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("// {GENERATED_BY}\n"));
    out.push_str(
        "//! The engine-owned record and column tables, generated from the IDL;\n\
         //! `og_api::host` re-exports them.\n\n\
         // Field metadata (min, max, step) is written as the IDL states it.\n\
         #![allow(clippy::approx_constant)]\n",
    );
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
         \x20   /// How og.WorldOrigin's shift moves it (og_core::Worldspace), or empty.\n\
         \x20   pub worldspace: &'static str,\n\
         \x20   /// The person-facing name, or empty.\n\
         \x20   pub label: &'static str,\n\
         \x20   /// The field's docs, joined; empty in a publish build (og-api's\n\
         \x20   /// `publish`), where nothing reads them.\n\
         \x20   pub doc: &'static str,\n\
         \x20   /// An enum column's variants in index order; empty otherwise.\n\
         \x20   pub variants: &'static [&'static str],\n\
         \x20   /// A text column's capacity in bytes (`str.48` in the IDL); 0 otherwise.\n\
         \x20   pub capacity: u32,\n\
         \x20   /// A list column's capacity in elements (`List.Post.8` in the IDL),\n\
         \x20   /// whose `column` is the element's kind; 0 otherwise.\n\
         \x20   pub list: u32,\n\
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
                    "            FieldDecl {{\n                name: \"{}\",\n                column: \"{}\",\n                default: {:?},\n                readonly: {},\n                tunable: {},\n                min: {},\n                max: {},\n                step: {},\n                unit: {:?},\n                worldspace: {:?},\n                label: {:?},\n                doc: doc!({:?}),\n                variants: &[{}],\n                capacity: {},\n                list: {},\n            }},\n",
                    f.name,
                    ix.list(&f.ty).map_or(f.ty.name(), |(element, _)| element.name()),
                    default,
                    f.readonly,
                    f.tunable,
                    opt(f.min),
                    opt(f.max),
                    opt(f.step),
                    f.unit.as_deref().unwrap_or_default(),
                    f.worldspace.as_deref().unwrap_or_default(),
                    f.label.as_deref().unwrap_or_default(),
                    f.doc.join(" "),
                    variants,
                    ix.capacity(&f.ty).unwrap_or(0),
                    ix.list(&f.ty).map_or(0, |(_, n)| n)
                ));
            }
            out.push_str("        ],\n    },\n");
        }
    }
    out.push_str("];\n");

    // Every column kind the IDL declares, so the runtime accepts a game's
    // field of a kind no engine record happens to use (an enum column's
    // variants come from here, not from the records that use it).
    out.push_str(
        "\n/// A column kind the IDL declares (`column X`): a kind a game's component or resource field may have.\n\
         #[derive(Clone, Copy, Debug, PartialEq)]\n\
         pub struct ColumnDecl {\n\
         \x20   pub name: &'static str,\n\
         \x20   /// An enum column's variants in index order; empty otherwise.\n\
         \x20   pub variants: &'static [&'static str],\n\
         }\n\n\
         /// Every declared column kind, in declaration order.\n\
         pub const COLUMNS: &[ColumnDecl] = &[\n",
    );
    for m in &ix.api.modules {
        for item in &m.items {
            let Item::Column(c) = item else { continue };
            if c.list {
                // A list's kind is its element's.
                continue;
            }
            let variants = match ix.decl(c.value.name()) {
                Some(crate::index::TypeDecl::Enum(e)) => e
                    .variants
                    .iter()
                    .map(|v| format!("{:?}", v.name))
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            };
            out.push_str(&format!(
                "    ColumnDecl {{ name: \"{}\", variants: &[{}] }},\n",
                c.name, variants
            ));
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
    /// `og-host-c`: one `extern "C"` entry per symbol over `OgValue`
    /// arguments, and the table a C or C++ shell registers them from.
    C,
}

fn glue_type(target: Target, w: &str) -> String {
    match (target, w) {
        (Target::Web, "Self::Array") => "Vec<u32>".into(),
        (Target::Web, "Self::Views") => "Vec<u32>".into(),
        (Target::Web, "Self::Function") => "js_sys::Function".into(),
        (Target::Web, "Self::Bytes") => "Vec<u8>".into(),
        (Target::Web, "Self::Shared") => "u32".into(),
        (Target::Node, "Self::Array") => "crate::NodeArray".into(),
        (Target::Node, "Self::Views") => "crate::NodeViews".into(),
        (Target::Node, "Self::Function") => "crate::NodeFunction".into(),
        (Target::Node, "Self::Bytes") => "crate::NodeBytes".into(),
        (Target::Node, "Self::Shared") => "crate::NodeShared".into(),
        (Target::C, "Self::Array") => "crate::CArray".into(),
        (Target::C, "Self::Views") => "crate::CViews".into(),
        (Target::C, "Self::Function") => "crate::CFunction".into(),
        (Target::C, "Self::Bytes") => "crate::CBytes".into(),
        (Target::C, "Self::Shared") => "crate::CShared".into(),
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
    if target == Target::C {
        // Every argument converts from its `OgValue`, the answer into `out`.
        out.push_str(&format!(
            "unsafe extern \"C\" fn {symbol}(args: *const crate::OgValue, out: *mut crate::OgValue) {{\n"
        ));
        let call: Vec<String> = std::iter::once("&Host".to_string())
            .chain((0..params.len()).map(|i| format!("unsafe {{ crate::arg(args, {i}) }}")))
            .collect();
        out.push_str(&format!(
            "    let r = <Host as {trait_name}>::{symbol}({});\n    unsafe {{ crate::IntoC::into_c(r, out) }}\n}}\n\n",
            call.join(", ")
        ));
        return;
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
        Target::C => unreachable!(),
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
        Target::C => out.push_str(
            "//! The C ABI of every binding symbol: an `extern \"C\"` entry each,\n\
             //! taking its arguments as `OgValue`s and writing its answer to one,\n\
             //! that calls the host's implementation of its `og_api::host` trait;\n\
             //! `SYMBOLS` lists them for the shell that registers them.\n\n\
             #![allow(clippy::too_many_arguments, clippy::let_unit_value, unused_unsafe, unused_variables)]\n\n\
             use crate::CHost as Host;\n\
             use og_api::host::*;\n\n",
        ),
    }
    // (symbol, arity, publish) of each entry, for C's table.
    let mut table: Vec<(String, usize, bool)> = Vec::new();
    for (tname, _, members) in HOST_TRAITS {
        for (name, params, ret, _) in *members {
            table.push((name.to_string(), params.len(), true));
            let ps: Vec<(String, String)> = params
                .iter()
                .map(|(n, t, _)| (n.to_string(), t.to_string()))
                .collect();
            glue_fn(target, &mut out, tname, name, &ps, ret, true);
        }
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
        table.push((b.symbol.clone(), ps.len(), b.publish));
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
    if target == Target::C {
        out.push_str("/// Every entry, in the order of `packages/engine/src/generated/abi.ts`.\npub static SYMBOLS: &[crate::Symbol] = &[\n");
        for (symbol, arity, publish) in &table {
            if !publish {
                out.push_str("    #[cfg(not(feature = \"publish\"))]\n");
            }
            out.push_str(&format!(
                "    crate::Symbol {{ name: c\"{symbol}\", arity: {arity}, call: {symbol} }},\n"
            ));
        }
        out.push_str("];\n");
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
         // and `V` how it hands out a component's column views (the shared buffer\n\
         // is `unknown`: each loader knows its host's form).\n\n\
         /// The engine's call back into the runtime (`og_host_set_dispatcher`).\n\
         export type Dispatcher = (kind: number, engine: number, id: number, query: number, tick: number, dt: number, elapsed: number, payload: string) => string;\n\n",
    );
    abi_consts_ts(&mut out);
    out.push_str("\nexport interface HostAbi<A, V> {\n");
    for (name, params, ret, doc) in HOST_TRAITS.iter().flat_map(|t| t.2.iter()) {
        out.push_str(&format!("  /** {doc} */\n"));
        let ps: Vec<String> = params.iter().map(|(n, _, t)| format!("{n}: {t}")).collect();
        let r = match *ret {
            "()" => "void",
            "bool" => "boolean",
            "Self::Bytes" => "Uint8Array",
            "Self::Views" => "V",
            "Self::Shared" => "unknown",
            "u32" => "number",
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
    out.push_str("}\n\n/** Every binding symbol, in binding-table order, then the lifecycle and present members. */\nexport const SYMBOLS: readonly string[] = [\n");
    for b in &all {
        out.push_str(&format!("  \"{}\",\n", b.symbol));
    }
    for (name, ..) in HOST_TRAITS.iter().flat_map(|t| t.2.iter()) {
        out.push_str(&format!("  \"{name}\",\n"));
    }
    out.push_str("];\n");
    entity_checks(ix, &all, &mut out);
    out
}

/// `checkEntities` of abi.ts: every binding with an `Entity` parameter
/// wrapped in a `typeof` check, so a value that is not a number (null from
/// world.find, undefined, a Result) is refused naming the call and the
/// parameter, alike on both hosts, before napi's conversion error or
/// wasm-bindgen's coercion to 0 or NaN.
fn entity_checks(ix: &Index<'_>, all: &[Binding<'_>], out: &mut String) {
    out.push_str(
        "\n/** Why `value`, passed as `param` of `call` where an entity is expected, is not one. */\n\
         function notEntity(call: string, param: string, value: unknown): never {\n\
         \x20 const pass = \"pass the number world.spawn, world.find or a.entity(i) returned\";\n\
         \x20 const what =\n\
         \x20   value === null ? \"null (world.find returns null when no entity has the name)\"\n\
         \x20   : value === undefined ? `undefined (${pass})`\n\
         \x20   : typeof value === \"object\" && \"ok\" in value ? \"a Result, not an entity (world.spawn answers { ok, value }: pass its value)\"\n\
         \x20   : `${/^[aeiou]/.test(typeof value) ? \"an\" : \"a\"} ${typeof value}, not an entity (${pass})`;\n\
         \x20 throw new Error(`${call}: ${param} is ${what}`);\n\
         }\n\n\
         /** `abi` with every binding that takes an `Entity` checking it is a number first (`notEntity`). */\n\
         export function checkEntities<A, V>(abi: HostAbi<A, V>): HostAbi<A, V> {\n\
         \x20 return {\n\
         \x20   ...abi,\n",
    );
    for b in all {
        let Some(f) = b.function else { continue };
        let (params, _) = signature(ix, b);
        // The receiver (world, an interface object) comes first.
        let offset = params.len() - f.params.len();
        let checked: Vec<(usize, &Param)> = f
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.ty.name() == "Entity" && !f.generics.iter().any(|g| g.name == "Entity")
            })
            .map(|(i, p)| (i + offset, p))
            .collect();
        if checked.is_empty() {
            continue;
        }
        let call = match b.owner {
            Some(owner) => {
                let mut c = owner.chars();
                let first = c.next().map(|c| c.to_ascii_lowercase()).unwrap_or_default();
                format!("{first}{}.{}", c.as_str(), b.name)
            }
            None => b.path.clone(),
        };
        let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
        let tests: Vec<String> = checked
            .iter()
            .map(|(i, p)| {
                let n = names[*i];
                let bad = if params[*i].optional {
                    format!("{n} !== undefined && typeof {n} !== \"number\"")
                } else {
                    format!("typeof {n} !== \"number\"")
                };
                format!(
                    "      if ({bad}) notEntity(\"{call}\", \"{}\", {n});\n",
                    p.name
                )
            })
            .collect();
        out.push_str(&format!(
            "    {}({}) {{\n{}      return abi.{}({});\n    }},\n",
            b.symbol,
            names
                .iter()
                .zip(&params)
                .map(|(n, p)| {
                    let ty = p.wire.ts();
                    if p.optional {
                        format!("{n}: {ty} | undefined")
                    } else {
                        format!("{n}: {ty}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", "),
            tests.concat(),
            b.symbol,
            names.join(", ")
        ));
    }
    out.push_str("  };\n}\n");
}
