# og-api

The engine's public API, declared once in `api.ogt`, the generators that
turn it into everything else (design §3.1), and the host-neutral runtime both
hosts wrap.

| Output | Path | Consumer |
|---|---|---|
| TypeScript declarations | `packages/engine/types/index.d.ts` | game code, the shell, the knowledge package |
| runtime binding table | `packages/engine/types/bindings.json` | the TypeScript runtime of both hosts |
| JSON Schema + API catalog | `packages/engine/schema/api.schema.json` | the platform's knowledge package, the wiki generator |
| host traits | `crates/og-api/src/generated/host.rs` (`og_api::host`) | `og-host-web`, `og-host-node` implement them |
| wasm-bindgen glue | `crates/og-host-web/src/generated/glue.rs` | the web host's exports |
| napi glue | `crates/og-host-node/src/generated/glue.rs` | the native host's exports |
| host ABI interface | `packages/engine/src/generated/abi.ts` | the TypeScript runtime (`HostAbi`, `SYMBOLS`) |

The wiki pages come next. Every generated file starts with a line saying it
is generated from `api.ogt`. `og api gen` writes them; `og api check`
regenerates into a temporary directory and fails on any difference, including
a file in one of the generated directories (`og_api::OUTPUT_DIRS`) that the
generator does not write.

```
og api gen     # after editing api.ogt
og api check   # CI: exit 1 and the differing files on drift
```

## The `.ogt` grammar

`api.ogt` is ordinary `.ogt` text read by `og_core::ogt`, the engine's one
reader (its module docs are the authority). The grammar, one construct per
line:

```
document  = { blank | comment } header body
header    = "type" SP word NL { blank | comment } "version" SP digits NL { blank | comment } "---" NL
body      = { line NL }
line      = blank | comment | entry | close
entry     = key { SP value } [ SP "{" ]        ; a trailing "{" opens a block
close     = "}"                                ; alone on its line
key       = word                               ; never `true` or `false`
value     = string | number | word
word      = ( ALPHA | "_" ) { ALNUM | "_" | "." | "-" }
number    = [ "-" ] digits [ "." digits ] [ ( "e" | "E" ) [ "+" | "-" ] digits ]
string    = '"' { char | escape } '"'          ; one line; followed by a space or the line end
escape    = '\"' | '\\' | '\n' | '\r' | '\t' | '\u{' 1*6HEX '}'
comment   = "#" to the end of the line, where "#" starts a token
```

- Tokens are separated by spaces or tabs; indentation is not significant
  (the files use two spaces per level). `\r\n` line ends are accepted.
- A token starting with a digit or `-` is a number; numbers keep their source
  text, so integers stay exact. `true` and `false` are words, read with
  `Value::as_bool()`.
- `{` may only end a line; `}` must stand alone; every block is closed.
- Comments and blank lines may precede the header. The IDL requires type
  `api` and version `1`; syntax errors keep og-core's line numbers.

## The IDL

Top level: `package "<npm name>"`, `doc "..."` lines, `unit <name> "<doc>"`
and `module <name> { ... }`. A module holds `doc`, optionally
`place namespace|root|top` and `publish false`, then declarations.

Most declarations accept a trailing quoted doc on their own line
(`field x f32 "The x component."`) and `doc "..."` lines in their block;
several `doc` lines are paragraphs, joined by newlines.

| Declaration | Form | Means |
|---|---|---|
| `fn` | `fn name { doc; generic N Bound; param ...; returns Type "doc"; publish bool }` | a function; in an interface or resource, a method |
| `param` | `param name Type "doc" { default ...; optional true; flatten true }` | a parameter; `flatten` spreads a desc's fields into the wire params |
| `const` | `const NAME Type value "doc"` | a constant (u32, i32, f32, f64, String or bool) |
| `handle` | `handle Name "doc"` | a generational handle |
| `struct` | `struct Name { doc; desc true; field ... }` | plain data; `desc true` makes every field optional with a default |
| `field` | `field name Type "doc" { default ...; min; max; step; unit; label; tunable; readonly; optional }` | a field |
| `enum` | `enum Name { variant name "doc" }` | string variants; the first is the zero value |
| `union` | `union Name { tag key; case name [Payload] "doc" }` | a tagged union; the payload is a struct |
| `interface` | `interface Name { extends Base; publish bool; prop name Type "doc"; fn ... }` | an object with members |
| `callback` | `callback Name { param ...; returns Type }` | a function game code supplies |
| `column` | `column Name { array TypedArray; stride N; value Type }` | a column kind: what a component or resource field stores |
| `component` / `resource` / `event` | `component Name { field ... }`, resources also `fn ...` | reflected records registered by name |

### Types

Type expressions are words in a dotted prefix form, because `.ogt` words
cannot hold `<`: built-in generics have fixed arities, so `Map.List.Entity`
is `Map<List<Entity>>` and `Array.f32.16` is `Array<f32,16>`. Generated output
and diagnostics show the angle-bracket form.

- Scalars: `Void bool i32 u32 f32 f64 String Json Bytes`, and the typed
  arrays `Float32Array Float64Array Uint32Array Int32Array Uint8Array`.
- Generics: `List.T`, `Map.T` (string keys), `Option.T` (null when absent),
  `Result.T` (`{ok, value} | {ok: false, reason}`), `Partial.T`,
  `OneOf.A.B`, `Array.T.N` (a fixed-length tuple of plain numeric data).
- Reflection: `Fields` (a layout), `AnyComponent`, `AnyResource`, `AnyEvent`,
  `AnyReflected` (component or resource), `ComponentType.F` and
  `ResourceType.F` (tokens of a layout `F`), `Value.T` (what a token stands
  for), `View.C` (a component's typed column views).
- A function may declare `generic C AnyComponent` (or another reflection
  bound, or `Fields`) and use `C` in its signature.
- A component, resource or event name used as a type means its value.

### Validation

`og_api::parse` refuses an IDL with any of these, each reported with its line:

- grammar errors, unknown keys, malformed type expressions, extra values on a
  declaration line;
- **unknown types** (and wrong arities, `Value`/`View` of something that is
  not a token, `ComponentType` of something that is not a `Fields` generic);
- **duplicate names**: type names across all modules (and against built-ins),
  modules, columns, units, consts, functions in a module, members of an
  interface including inherited ones, fields, params, variants, cases, and
  names that would collide on `og` or among the package's named exports;
- **missing docs** on the API, units, modules, declarations, params, fields,
  props, variants and cases;
- naming conventions: modules lowercase, types and generics PascalCase,
  functions, params, fields, props and units lowerCamelCase, consts
  SCREAMING_SNAKE_CASE, variants and cases lowercase-first;
- component and resource fields whose type is not a declared column kind;
  `tunable`/`readonly` outside them; `optional` outside plain structs;
- desc fields (and record fields) without a zero value or a `default`;
  defaults of the wrong type or outside `min`..`max`; a range on a
  non-numeric field; an unknown unit;
- a required parameter after an optional one;
- columns whose name is not their value type, whose array cannot hold the
  value's scalars, or whose stride is not the value's scalar count;
- unions whose payload is not a struct or has a field named like the tag;
  `extends` of something that is not an interface;
- a missing `FieldOptions` struct, which the TypeScript generator needs.

`tests/fixtures/` holds one broken IDL per rule family with the expected
diagnostic in its first comment.

## TypeScript mapping

- `place top` modules export their functions by name
  (`import { defineComponent } from "@origingame/engine"`); `place root`
  functions are members of `og` (`og.game`); other modules become
  `og.<module>` namespaces (`og.load.gltf`).
- Every component, resource and event is a token on `og` (`og.Transform`,
  `og.Time`, `og.KeyDown`) and an exported interface of its value
  (`Transform`). Column kinds become factories (`f32()`, `Vector3()`) and the
  `ColumnKinds` table, from which `View<T>` gives each field its typed array:
  `a.column(og.Transform).position` is a `Float32Array` of stride 3.
- Desc structs have every field optional; a desc parameter is optional when
  everything after it is. Other structs are `readonly`, except the math
  values a column stores (`Vector3`, `Quaternion`, ...), which are plain
  mutable data.
- Handles are branded numbers; enums are string-literal unions; unions are
  discriminated on their tag.

`packages/engine/examples/hello.ts` is the design's §3.2 example and
`packages/engine/tests/contract.ts` asserts the declarations' contract with
`@ts-expect-error`; `bun run typecheck` in `packages/engine` checks both, and
`cargo test -p og-api` runs that check. The runtime in `packages/engine/src/`
types its exports by the generated declarations, so a runtime member that
drifts from the IDL fails the same check.

## Binding symbols

Each callable gets a host symbol: `og_<module>_<fn>` for module functions,
`og_<interface>_<method>` for members and resource methods,
`og_<interface>_get_<prop>` for props, all in snake_case. They are unique;
the validator refuses a declaration that would repeat one.

## Hosts: traits, wire forms, glue

`og_api::host` (generated) has one trait per module with callables
(`WorldApi`, `ProbeApi`, ...; `math` and `time` declare data only) plus
`Lifecycle` (create, destroy, advance, command, frame, write a resource, set
the dispatcher), each method named after its binding symbol. `Host` is all of
them. Each host implements every trait in its `src/host.rs` and its generated
glue calls each method, so a member the IDL gains is a build error on a host
that lacks it.

Every binding crosses in one wire form, identical on both hosts:

| IDL type | Wire (Rust / TypeScript) |
|---|---|
| `bool`, `i32`, `u32` | themselves |
| `f32`, `f64`, handles (`Entity`, `Material`, ...) | `f64` / `number` (handles are exact below 2^53) |
| `String`, enums | `String` / `string` |
| interface objects (`World`, `Query`, `ArchetypeView`, `System`, probes) | `u32` handle (a world and its probes are the engine id) |
| callbacks (`SystemRun`, `CommandHandler`, `FactsFn`) | `u32` id into the runtime's callback table |
| component, resource and event tokens | the schema name (`og.Transform`, `Health`) |
| `Fields` | JSON list of `{ name, kind, default, options }`, in declaration order |
| everything else (structs, lists, maps, options, results, unions, `Value`) | JSON text |
| `Uint32Array` (`ArchetypeView.entities`) | `Wire::Array`: a copied `Uint32Array` |
| `View<C>` (`ArchetypeView.column`) | `Wire::Views`: web, byte offsets into `memory.buffer`; node, one ArrayBuffer per field over the column |

Optional parameters are `Option<..>` / `| undefined`. Exports outside
`publish` builds (the commands module, the probe's non-player members) carry
`#[cfg(not(feature = "publish"))]` in the glue; the traits keep them.

Engine-owned records register under dotted schema names (`og.Transform`,
`og.Time`, `og.Rng`; `bindings.json` gives each record's `schema`), so the
TypeScript token and the schema name coincide and never collide with a game's
own PascalCase names.

## The runtime (feature `runtime`)

`og_api::runtime` is what the trait methods call: declarations recorded in
call order, engines assembled from them, the tick, the dispatcher and the
view tables. Both hosts depend on it with the feature; `og-cli` does not.

- **Declarations** (`defineComponent`, `defineResource`, `defineSystem`,
  `og.game`, `og.commands.register`) are recorded per thread before any
  world exists. `og_engine_create(game)` builds an `og_ecs::World`: the
  engine-owned records in IDL order, then the game's components and
  resources in declaration order, then its systems in declaration order with
  `after`/`before` applied (a stable topological order), then its commands.
  The order is a function of the bundle, so both hosts build the same world.
- **Systems**: each is one `og_ecs` system whose closure (`Fn + Send`,
  capturing ids only) calls the host's dispatcher with the system's callback
  id. While it runs, its declared columns are borrowed and published as view
  records; the JavaScript body's calls back into the host are served from the
  running system's `SystemContext`. The TypeScript dispatcher refuses an id it
  has no system for, which fails the tick with the system's name.
- **Views** are stamped. A view handle is valid for the system call it was
  made in (outside systems, until the next structural change) and refused
  after; the TypeScript `ArchetypeView` throws on stale use with its tick and
  system. `Hooks::invalidate_views(moving)` tells the host when views retire
  and whether memory may move next, so a host may keep a column's buffer
  while the column stays put and must detach it before it moves. Every
  structural change outside systems goes through the runtime, and a running
  system's only structural command in M2 is `despawn`, which the runtime
  records; a new structural path must call `queued_structural`.
- **Inside a system**, the world serves the system's views, `Time`, `Rng`
  (declared), `alive`, `despawn`, `find`, `nameOf`, `tick` and (from its own
  views) `has`. `spawn`, `insert`, `remove`, `get`, `set`, `emit` and other
  resources are refused with a reason: `og_ecs::SystemContext` has no
  packed-byte access for host-declared layouts yet.
- **Not yet**: column kinds `bool` and `Matrix4` for components (og-core has
  no one-byte or 4x4 field type), so `og.Mesh` and `og.MatrixWorld` are not
  registered and a game component with a `bool` field is refused at
  declaration; resources store a `bool` as a `u32` 0 or 1. Events with
  string or enum fields are not registered; `world.events` answers `[]`.
  `og.load.*` and `world.instantiate` refuse until the cook (M4); scheduling
  commands and the probe's `sweep`, `reload`, `pick` and `commands` answer
  their refusal until og-probe (M5). Entity names live in the runtime, not in
  snapshots or the world hash.
