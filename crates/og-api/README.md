# og-api

The engine's public API, declared once in `api.ogt`, and the generators that
turn it into everything else (design §3.1). Phase 1 week 2 ships the model,
the validator and the TypeScript side:

| Output | Path | Consumer |
|---|---|---|
| TypeScript declarations | `packages/engine/types/index.d.ts` | game code, the shell, the knowledge package |
| runtime binding table | `packages/engine/types/bindings.json` | the TypeScript glue of both hosts |
| JSON Schema + API catalog | `packages/engine/schema/api.schema.json` | the platform's knowledge package, the wiki generator |

Rust host traits, wasm-bindgen and napi glue and the wiki pages come next
(phase 1 M2). Every generated file starts with a line saying it is generated
from `api.ogt`. `og api gen` writes them; `og api check` regenerates into a
temporary directory and fails on any difference, including a file in
`packages/engine/types/` or `packages/engine/schema/` that the generator does
not write.

```
og api gen     # after editing api.ogt
og api check   # CI: exit 1 and the differing files on drift
```

## The `.ogt` grammar

`api.ogt` is ordinary `.ogt` text. `og-core` owns the engine's reader and
writer; `og-api` keeps a private reader (`src/ogt.rs`) until that lands, and
is then switched to `og-core`'s tree. The two accept the same text, which is
this grammar:

```
document  = header body
header    = "type" SP word NL "version" SP digits NL "---" NL
body      = { line NL }
line      = blank | comment | entry | close
entry     = key { SP value } [ SP "{" ]        ; a trailing "{" opens a block
close     = "}"                                ; alone on its line
key       = word
value     = string | number | word
word      = ( ALPHA | "_" ) { ALNUM | "_" | "." | "-" }
number    = [ "-" ] digits [ "." digits ] [ ( "e" | "E" ) [ "+" | "-" ] digits ]
string    = '"' { char | escape } '"'          ; one line; followed by a space or the line end
escape    = '\"' | '\\' | '\n' | '\r' | '\t' | '\u{' 1*6HEX '}'
comment   = "#" to the end of the line, where "#" starts a token
```

- Tokens are separated by spaces or tabs; indentation is not significant
  (the files use two spaces per level). `\r\n` line ends are accepted.
- A token starting with a digit or `-` is a number; `true` and `false` are
  words read as booleans. Numbers keep their source text, so integers stay
  exact.
- `{` may only end a line; `}` must stand alone; every block is closed.
- The `api` reader additionally requires the header on lines 1 to 3, with
  type `api` and version `1`.

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
`cargo test -p og-api` runs that check.

## Binding symbols

Each callable gets a host symbol: `og_<module>_<fn>` for module functions,
`og_<interface>_<method>` for members and resource methods,
`og_<interface>_get_<prop>` for props, all in snake_case. They are unique;
the validator refuses a declaration that would repeat one.
