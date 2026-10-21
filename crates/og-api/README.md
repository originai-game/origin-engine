# og-api

The engine's public API, declared once in `api.ogt`, the generators that
turn it into everything else (design §3.1), and the host-neutral runtime both
hosts wrap. Three crates, so that an edit rebuilds only what compiles it:

| Crate | Holds | Depends on it |
|---|---|---|
| `og-idl` (`crates/og-idl`) | the IDL's model, validator and generators: `read`, `generate`, `write`, `check` | `og` (`og api gen`, `og api check`) |
| `og-api` (this crate) | `api.ogt` and what the generators write here: `og_api::host`, the traits and record tables; no dependencies | both hosts, `og-runtime`, `og`'s cook |
| `og-runtime` (`crates/og-runtime`) | the host-neutral runtime the traits call (below) | both hosts |

`og api gen` rewrites only a file whose bytes change, and the generated Rust
carries no IDL prose (a trait method names its TypeScript path; the docs
live in `api.ogt`, the `.d.ts` and the wiki). So a doc-only IDL edit
rewrites the TypeScript, schema and wiki outputs and rebuilds no crate; an
edit to a component field's doc rewrites `RECORDS` (the runtime's
reflection serves it) and rebuilds the hosts; a generator edit rebuilds
`og-idl` and `og` only. There is no build script: the outputs are committed
and `og api check` (the gate's drift step) holds them to the IDL.

| Output | Path | Consumer |
|---|---|---|
| TypeScript declarations | `packages/engine/types/index.d.ts` | game code, the shell, the knowledge package |
| runtime binding table | `packages/engine/types/bindings.json` | the TypeScript runtime of both hosts |
| JSON Schema + API catalog | `packages/engine/schema/api.schema.json` | the platform's knowledge package, the wiki generator |
| host traits | `crates/og-api/src/generated/host.rs` (`og_api::host`) | `og-host-web`, `og-host-node` implement them |
| wasm-bindgen glue | `crates/og-host-web/src/generated/glue.rs` | the web host's exports |
| napi glue | `crates/og-host-node/src/generated/glue.rs` | the native host's exports |
| host ABI interface | `packages/engine/src/generated/abi.ts` | the TypeScript runtime (`HostAbi`, `SYMBOLS`) |
| wiki: one page per module and `index.md` | `packages/knowledge/wiki/` | agents, through `@origingame/engine-knowledge` (mounted as the `og-game` skill's `reference/`) |

Every generated file starts with a line saying it is generated from
`api.ogt`. Wiki pages quote each declaration's IDL text with the line it
starts on, its TypeScript form, its docs, its fields' defaults, ranges and
units, and its binding symbol; `index.md` maps every module, type, function
and member to its anchor (`crates/og-idl/tests/wiki.rs` holds the quotes to
the IDL and every link to an anchor). `og api gen` writes them (`--wiki`
writes the wiki alone and removes a page whose module is gone); `og api check`
regenerates into a temporary directory and fails on any difference, including
a file in one of the generated directories (`og_idl::OUTPUT_DIRS`) that the
generator does not write.

```
og api gen     # after editing api.ogt
og api gen --wiki   # the wiki pages alone
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
and `module <name> { ... }`. A module holds `skill og-<name>` (the
knowledge skill that teaches it: the wiki's index groups modules by it, each
page names it, and the knowledge package checks that skill names the page),
`doc`, optionally `place namespace|root|top` and `publish false`, then
declarations.

Most declarations accept a trailing quoted doc on their own line
(`field x f32 "The x component."`) and `doc "..."` lines in their block;
several `doc` lines are paragraphs, joined by newlines.

| Declaration | Form | Means |
|---|---|---|
| `fn` | `fn name { doc; generic N Bound; param ...; returns Type "doc"; publish bool; host bool }` | a function; in an interface or resource, a method; `host false` is TypeScript sugar the runtime implements over other members (`og.ui.panel`): no binding symbol, no trait method |
| `value` | `value name Type "doc"` | a value the TypeScript runtime provides under the module's path (`og.systems.cameras`); no binding symbol |
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
- Text in a component or resource field: a text column (`column str`,
  `capacity true`) followed by its capacity in bytes, `field name str.48`
  (shown `str<48>`; 1 to 4096 bytes of UTF-8). The TypeScript value is a
  `string`, the token's field a `FieldSpec<"str">`; `bindings.json` and
  the record table (`FieldDecl.capacity`) carry the capacity to the
  runtime, which stores the field as og-core's `strN`; the schema gives it
  as `x-og-capacity` and the wiki as `str(48)`. A default is one quoted
  string that fits. Anywhere else (events, structs, parameters) text is
  `String`.
- A component, resource or event name used as a type means its value.

### Validation

`og_idl::parse` refuses an IDL with any of these, each reported with its line:

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
  a text column without its capacity or with one outside 1 to 4096, or
  used outside a component or resource field;
  `tunable`/`readonly` outside them; `optional` outside plain structs (an
  enum may be a column, `column AudioBus { array Uint32Array; stride 1;
  value AudioBus }`: it stores the variant's index, and the generated record
  table carries the variants, which are part of the layout hash);
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
`cargo test -p og-idl` runs that check. The runtime in `packages/engine/src/`
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

## The runtime (`og-runtime`)

`og_runtime` is what the trait methods call: declarations recorded in
call order, engines assembled from them, the tick, the dispatcher and the
view tables. Both hosts depend on it; `og-cli` does not.

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
- **Inside a system**, the world serves the system's views, `Time`, `Rng`,
  `alive`, `find`, `nameOf`, `tick`, `has` (from its own views), and through
  `og_ecs::SystemContext`'s packed-byte calls `get`/`set` (immediate),
  `spawn`/`insert`/`remove`/`despawn` (queued, applied when the system
  returns; the system's views retire as moving), `events` (delivered this
  tick), `emit`, and reading and writing any resource; each is refused by
  the context unless the system declared the type among its reads or writes
  (events too). `instantiate` is still refused inside systems (scenes spawn
  between ticks).
- **Modules** (`runtime/modules.rs`): every world gets physics, the UI and
  audio. The schedule is `og.physics.queries` (records the colliders so
  `og.physics.raycast` answers inside systems), `og.ui.begin`, the game's
  systems, `og.ui.end`, `og.physics`, `og.audio`; the `og.ui` state block
  carries the UI's interaction state. The host's options choose the physics
  profile (`profile: "2d" | "3d"`, from project.ogt) and may defer
  `GameDesc.start` to the first tick boundary (`deferStart`: the web Worker
  attaches its canvas first); the web host reports the CSS viewport
  (`og_web_set_viewport`), native hosts use the render target. The UI, the
  screen-space sprites and text (`screen: true`) go to og-render's overlay,
  drawn over the finished frame whatever the camera; the effect resources
  and components are read by `runtime/fx.rs` during extraction.
- Entity names live in the runtime, not in snapshots or the world hash.
- **E4 feel modules** (`runtime/feel.rs`): `og.tween` (og-tween, writing
  every registered component) and `og.flipbook` (og-anim) run after the
  game's systems and before `og.anim`; `og.tilemap` (adopts og.Tilemap
  entities, keeps their og-tilemap `MapState`, spawns and despawns the
  merged static colliders as edits land) and `og.camera` (applies queued
  `og.camera.shake/punch/kick`, decays trauma, punch and kick by the
  unscaled tick, moves the camera along an og.CameraRail) after it;
  `og.sprite` (afterimage samples, floating text expiry) after physics and
  audio. Each has a state block. A system's `time` carries the time scale
  (`scale`, `hitStop`, `scaledDt`, `scaledElapsed`) in the dispatch payload;
  `og.Time` reads the same through `og.TimeScale`. Extraction draws og.Sprite
  rows (their flipbook frame, flips, nine-slice pieces, y sort keys, blob
  shadows and afterimages), floating texts and tile layers (chunks cached
  per layer and rebuilt when a chunk's version moves), gives flashing
  og.Mesh entities a flash copy of their material (16 levels), and applies
  trauma shake, punch, kick and og.CameraZone clamping to the frame only.
  Presets (`og.tween.squash`, `flash`) hold their impact for the tick they
  start in. Tested through TypeScript in `packages/engine/tests/feel.test.ts`.

