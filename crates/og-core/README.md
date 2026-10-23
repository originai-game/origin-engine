# og-core

The vocabulary every engine crate shares. No global state, no wall clock, no
host floating-point library; every fallible call returns `Result<_, Error>`.

## API map

| Module | Entry points | Notes |
|---|---|---|
| `error` | `Error { kind, reason }`, `ErrorKind`, `Result<T>` | `kind` to branch on, `reason` to read; the probe shows `{ok:false, reason}` |
| `handle` | `RawHandle` (`#[repr(C)]`: index, generation), `HandleAllocator`, `Handle<T>`, `Pool<T>`, `RETIRED_GENERATION`, `MAX_GENERATION` | generational; a dead handle is `ErrorKind::DeadHandle` |
| `reflect` | `Reflect` + `#[derive(Reflect)]`, `Schema` (`check_packed`, `default_bytes`), `FieldSchema` (`variants`, `enumeration`, `default_bytes`, `check_value`), `FieldType`, `FieldValue` (column views), `ReflectField` (derive fields), `Bool8`, `Str<N>` (`Str64`, `Str256`), `pack_str`/`unpack_str`, `Layout`, `schemas_to_json` | the one schema (fields, types, docs, labels, units, min/max, default, `tunable`, enum variants) |
| `ogt` | `parse`, `Document { ty, version, body }`, `Block`, `Entry`, `Value`, `Number`, `Document::to_text` | the `.ogt` text format, grammar in the module docs |
| `rng` | `Rng::new(seed)`, `next_u32`, `next_f64`, `range`, `range_f32`, `integer`, `fork(label)`, `save`/`restore` | bit-identical to the platform's `rng.ts`/`rng.rs`; schema `og.Rng` |
| `math` | `glam::*` (built with `scalar-math`), `sin`, `cos`, `sin_cos`, `atan`, `atan2`, `exp`, `ln`, `pow`, `quat_from_axis_angle` | deterministic f32 on every host |
| `time` | `Time { tick, dt, elapsed }`, `Time::from_rate(hz)`, `advance()` | `elapsed = tick · dt`, never accumulated; schema `og.Time` |
| `hash` | `StableHasher`, `hash_bytes` | FNV-1a 64, for world and layout hashes |

## Reflection

```rust
/// Hit points of anything that can be destroyed.
#[derive(Clone, Copy, Reflect)]
#[reflect(name = "Health", version = 1)]          // both optional
struct Health {
    /// Current hit points.
    #[reflect(min = 0, max = 1000, label = "Hit points", unit = "hp", tunable, default = 100)]
    hp: f32,
}
```

Structs with named fields, `struct T {}` and unit structs (tags) derive;
generics, tuple structs and enums are refused at compile time. `///` docs
become `doc`. Two more field attributes: `name = "camelCase"` sets the schema
field name (the Rust ident otherwise), `variants = ["a", "b"]` on a `u32`
makes an enum field. Values move as **packed bytes**: the fields in schema
order, little-endian, no padding (`Layout::encode`/`decode`).
`Schema::layout_hash` covers name, version, field names and types, and an
enum's variant names in order, not metadata. `Schema::from_json` is how a
host declares a layout by name.

| type (JSON) | bytes | Rust field | column view | notes |
|---|---|---|---|---|
| `f32 f64 i32 u32 i64 u64` | 4 / 8 | same | same | |
| `vec2 vec3 vec4 quat` | 8 / 12 / 16 / 16 | `Vec2 Vec3 Vec4 Quat` | same | `Float32Array` stride 2/3/4/4 |
| `mat4` | 64 | `Mat4` | `Mat4` | column-major, `Float32Array` stride 16 |
| `bool` | 1 | `bool` or `Bool8` | `Bool8` | `Uint8Array`; byte entry points take 0 or 1 only |
| `enum` | 4 | `u32` + `variants = [..]` | `u32` | the variant index; names in `FieldSchema::variants` |
| `strN` (`str32`, `str64` ...) | N, 1 to 4096 | `Str<N>` (`Str64`, `Str256`) | raw bytes (`Uint8Array` stride N) | UTF-8, zero-padded, no NUL; no range, default or `tunable` |
| `S[N]` (`u64[8]`, `f32[4]`) | (N + 1) × the element's, N 1 to 1024 | `List<T, N>` | the element's array, stride N + 1 lanes | a count lane, then N elements of `f32 f64 i32 u32 i64 u64`, zero past the length; no range, default or `tunable` |

A string is fixed-capacity and inline, so it is plain data in a component,
a resource or an event alike: its bytes are the text then zeros, so the
world's hash and snapshots depend on the text alone. The capacity is part of
the type name (`str32`) and of the layout hash; a migrating restore keeps a
text that fits a new capacity and starts one that does not empty. Text over
the capacity is refused by `pack_str` with the fix named (`declare the field
str(40) or larger`), never truncated. `Schema::check_packed` is the value check the
world's byte entry points run (bool 0/1, enum index in range, strings
zero-padded UTF-8), naming the field; every other type accepts every bit
pattern. `FieldSchema::default_bytes` is a field's declared default (or
zeros), which a migrating restore fills new fields with.

A list is fixed-capacity and inline in the same way: lane 0 is the count,
held as a value of the element type (`2` in a `u64[8]`, `1.0` in an
`f32[3]`), so `[]` and `[0]` are different bytes, and the lanes past the
length are zero, so the bytes depend on the list alone. `pack_list`
refuses a list over the capacity with the fix named (`declare the field
u64[9] or larger`); `list_len` and `list_elements` read one; a migrating
restore keeps a list that fits a new capacity and starts one that does not
empty, as it does a string.

Schema names are identifiers, optionally dot-separated. Engine-owned schemas
are named `og.<Type>` (`og.Time`, `og.Rng`), the same string as their
TypeScript token, so a host, the probe and scene text name them identically;
game types use plain names.

## Invariants

- `RawHandle` is `#[repr(C)]` (index, then generation): in memory it is the
  little-endian `u64` of `to_bits()`, so hosts can view handle arrays in place.
- Generations start at 1 (the packed handle `0` is never live); freeing bumps
  the generation; a slot whose generation reaches `RETIRED_GENERATION` (2^21)
  is retired and never handed out again, so a live generation is at most
  `MAX_GENERATION` (2^21 - 1) and `to_bits()` (`generation << 32 | index`) is
  at most 2^53 - 1: every handle and entity id is an exact JavaScript number.
  Freed slots are reused last-freed-first.
- `.ogt`: `parse(d.to_text()) == d` for every document; `to_text(parse(t)) == t`
  for canonical text (two spaces per level, single spaces, `\n`, final
  newline). Numbers keep their source text. Comments and blank lines are not
  data and are not written back. `Entry::line` is the source line, ignored by
  `==`.
- Deterministic math uses only `+ - * /`, comparisons, `floor`, integer
  operations and bit casts (no `libm`, no FMA), evaluates in f64 and rounds
  once: within 1 ulp, almost always correctly rounded; argument reduction is
  exact for every finite f32 (Cody–Waite below 2^24, Payne–Hanek above).
- glam is built with `scalar-math` for the whole workspace (root
  `Cargo.toml`), so `Vec3A`, `Vec4`, `Quat`, `Mat3A`, `Mat4` and `Affine3A`
  arithmetic runs one scalar implementation, in one operation order, on SSE2,
  NEON and wasm (with or without `simd128`): bit-identical everywhere. As a
  result `Vec4`, `Quat` and `Mat4` are 4-byte aligned (`Vec3A`/`Mat3A` stay
  16); GPU structs declare their own alignment.
- Simulation transcendentals go only through `og_core::math` (`sin`, `cos`,
  `sin_cos`, `atan`, `atan2`, `exp`, `ln`, `pow`, `quat_from_axis_angle`).
  glam's own trigonometry and exponentials (`Quat::from_axis_angle`,
  `from_euler`, `slerp`, `Mat4::from_rotation_*`, `powf`, ...) and `f32::sin`
  and friends call the host `libm` and are for rendering only.
- Little-endian only (a big-endian build is a compile error).

## Tests

`cargo test -p og-core`: `tests/math.rs` (fixed-input vectors from an
independent oracle, C99 special values, a 1-ulp sweep), `tests/glam_scalar.rs`
(glam `Vec3A`/`Vec4`/`Quat`/`Mat3A`/`Mat4`/`Affine3A` results equal an
explicit scalar reference bit for bit; fails without `scalar-math`),
`tests/rng.rs` (the
platform vectors), `tests/ogt.rs` (nested round trips, refusals with line
numbers, every `.ogt` file in the repository), `tests/reflect.rs`,
`tests/handle.rs` (including a slot cycled to retirement).

Across targets: `tools/wasm-determinism/run.sh` builds a digest of 1.4M
`og_core::math` and glam results and the bouncing-boxes world hash for the
host (debug, release) and wasm32 (scalar, `simd128`, under node) and fails
unless all agree; the latest numbers are in
`tests/evidence/M1/cross-target-determinism.md`.
