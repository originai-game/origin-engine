# og-core

The vocabulary every engine crate shares. No global state, no wall clock, no
host floating-point library; every fallible call returns `Result<_, Error>`.

## API map

| Module | Entry points | Notes |
|---|---|---|
| `error` | `Error { kind, reason }`, `ErrorKind`, `Result<T>` | `kind` to branch on, `reason` to read; the probe shows `{ok:false, reason}` |
| `handle` | `RawHandle`, `HandleAllocator`, `Handle<T>`, `Pool<T>`, `RETIRED_GENERATION`, `MAX_GENERATION` | generational; a dead handle is `ErrorKind::DeadHandle` |
| `reflect` | `Reflect` + `#[derive(Reflect)]`, `Schema`, `FieldSchema`, `FieldType`, `FieldValue`, `Layout`, `schemas_to_json` | the one schema (fields, types, docs, labels, units, min/max, default, `tunable`) |
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

Field types are exactly `f32 f64 i32 u32 i64 u64 Vec2 Vec3 Vec4 Quat`
(`FieldValue` is sealed). Structs with named fields, `struct T {}` and unit
structs (tags) derive; generics, tuple structs and enums are refused at compile
time. `///` docs become `doc`. Values move as **packed bytes**: the fields in
schema order, little-endian, no padding (`Layout::encode`/`decode`).
`Schema::layout_hash` covers name, version, field names and types, not
metadata. `Schema::from_json` is how a host declares a layout by name.

Schema names are identifiers, optionally dot-separated. Engine-owned schemas
are named `og.<Type>` (`og.Time`, `og.Rng`), the same string as their
TypeScript token, so a host, the probe and scene text name them identically;
game types use plain names.

## Invariants

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
